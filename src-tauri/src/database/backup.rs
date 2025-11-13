use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use chrono::Utc;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::io::Write;

/// 데이터베이스 백업 관리자
pub struct BackupManager {
    db_path: PathBuf,
    backup_dir: PathBuf,
}

impl BackupManager {
    /// 새 BackupManager 인스턴스 생성
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let backup_dir = db_path
            .parent()
            .context("DB 경로의 부모 디렉토리를 찾을 수 없습니다")?
            .join("backups");

        // 백업 디렉토리 생성 (존재하지 않으면)
        fs::create_dir_all(&backup_dir)
            .context("백업 디렉토리 생성 실패")?;

        Ok(Self { db_path, backup_dir })
    }

    /// 데이터베이스 백업 수행 (gzip 압축)
    /// 
    /// 반환값: 백업 파일 경로
    pub fn create_backup(&self) -> Result<PathBuf> {
        // 백업 파일명 생성 (timestamp 포함)
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("judgify_backup_{}.db.gz", timestamp);
        let backup_path = self.backup_dir.join(&backup_filename);

        // DB 파일 읽기
        let db_data = fs::read(&self.db_path)
            .context("데이터베이스 파일 읽기 실패")?;

        // gzip 압축
        let backup_file = fs::File::create(&backup_path)
            .context("백업 파일 생성 실패")?;
        let mut encoder = GzEncoder::new(backup_file, Compression::default());
        encoder.write_all(&db_data)
            .context("백업 파일 압축 중 오류")?;
        encoder.finish()
            .context("백업 파일 압축 완료 실패")?;

        println!("✅ 백업 완료: {}", backup_path.display());
        Ok(backup_path)
    }

    /// 백업 파일에서 데이터베이스 복구
    /// 
    /// 주의: 기존 데이터베이스를 덮어씁니다!
    pub fn restore_from_backup(&self, backup_path: &Path) -> Result<()> {
        // 백업 파일 존재 확인
        if !backup_path.exists() {
            return Err(anyhow::anyhow!("백업 파일을 찾을 수 없습니다: {}", backup_path.display()));
        }

        // 기존 DB 백업 (safety)
        let safety_backup = self.db_path.with_extension("db.before_restore");
        if self.db_path.exists() {
            fs::copy(&self.db_path, &safety_backup)
                .context("안전 백업 생성 실패")?;
            println!("🔒 기존 DB 안전 백업: {}", safety_backup.display());
        }

        // gzip 압축 해제
        let backup_file = fs::File::open(backup_path)
            .context("백업 파일 열기 실패")?;
        let mut decoder = flate2::read::GzDecoder::new(backup_file);
        let mut restored_data = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut restored_data)
            .context("백업 파일 압축 해제 실패")?;

        // 복구된 데이터를 DB 파일에 쓰기
        fs::write(&self.db_path, restored_data)
            .context("데이터베이스 파일 복구 실패")?;

        println!("✅ 복구 완료: {} ← {}", self.db_path.display(), backup_path.display());
        Ok(())
    }

    /// 모든 백업 파일 목록 조회 (최신순)
    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        let mut backups: Vec<PathBuf> = fs::read_dir(&self.backup_dir)
            .context("백업 디렉토리 읽기 실패")?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()?.to_str()? == "gz" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();

        // 최신순 정렬 (파일명에 timestamp 포함)
        backups.sort_by(|a, b| b.cmp(a));
        Ok(backups)
    }

    /// 오래된 백업 파일 자동 정리 (최근 N개만 유지)
    pub fn cleanup_old_backups(&self, keep_count: usize) -> Result<usize> {
        let backups = self.list_backups()?;
        let to_delete = backups.iter().skip(keep_count);

        let mut deleted_count = 0;
        for backup_path in to_delete {
            fs::remove_file(backup_path)
                .context(format!("백업 파일 삭제 실패: {}", backup_path.display()))?;
            println!("🗑️  삭제: {}", backup_path.display());
            deleted_count += 1;
        }

        if deleted_count > 0 {
            println!("✅ {}개의 오래된 백업 파일 삭제 완료", deleted_count);
        }

        Ok(deleted_count)
    }

    /// 백업 파일 크기 합계 (압축된 크기)
    pub fn get_total_backup_size(&self) -> Result<u64> {
        let backups = self.list_backups()?;
        let total_size: u64 = backups
            .iter()
            .filter_map(|path| fs::metadata(path).ok())
            .map(|metadata| metadata.len())
            .sum();

        Ok(total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_restore_backup() -> Result<()> {
        // 임시 디렉토리 생성
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");

        // 테스트 DB 파일 생성
        fs::write(&db_path, b"test database content")?;

        // BackupManager 초기화
        let manager = BackupManager::new(db_path.clone())?;

        // 백업 생성
        let backup_path = manager.create_backup()?;
        assert!(backup_path.exists());

        // DB 파일 수정
        fs::write(&db_path, b"modified content")?;

        // 백업에서 복구
        manager.restore_from_backup(&backup_path)?;

        // 복구된 내용 검증
        let restored_content = fs::read(&db_path)?;
        assert_eq!(restored_content, b"test database content");

        Ok(())
    }

    #[test]
    fn test_list_backups() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        fs::write(&db_path, b"test")?;

        let manager = BackupManager::new(db_path)?;

        // 3개 백업 생성
        manager.create_backup()?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.create_backup()?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.create_backup()?;

        // 백업 목록 확인
        let backups = manager.list_backups()?;
        assert_eq!(backups.len(), 3);

        Ok(())
    }

    #[test]
    fn test_cleanup_old_backups() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        fs::write(&db_path, b"test")?;

        let manager = BackupManager::new(db_path)?;

        // 5개 백업 생성
        for _ in 0..5 {
            manager.create_backup()?;
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        // 최근 2개만 유지
        let deleted = manager.cleanup_old_backups(2)?;
        assert_eq!(deleted, 3);

        // 백업 개수 검증
        let backups = manager.list_backups()?;
        assert_eq!(backups.len(), 2);

        Ok(())
    }
}
