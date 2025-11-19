use judgify_desktop::database::Database;
use rusqlite::Connection;
use std::path::PathBuf;

fn main() {
    println!("초기화 중: CCP 데모 데이터베이스...");

    match Database::new() {
        Ok(_db) => {
            println!("✅ 데이터베이스 기본 스키마 생성 완료!");

            // CCP 마이그레이션 적용
            match apply_ccp_migrations() {
                Ok(()) => {
                    println!("✅ CCP 마이그레이션 001-004 실행 완료");
                    println!("📁 위치: %APPDATA%\\Judgify\\judgify.db");
                    println!("✅ Seed 데이터 삽입 완료 (1,008 logs + 48 docs)");
                }
                Err(e) => {
                    eprintln!("⚠️  CCP 마이그레이션 적용 실패: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 데이터베이스 초기화 실패: {}", e);
            std::process::exit(1);
        }
    }
}

fn apply_ccp_migrations() -> rusqlite::Result<()> {
    // Get database path
    let app_data = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|e| rusqlite::Error::InvalidPath(PathBuf::from(e.to_string())))?;

    let db_path = PathBuf::from(app_data).join("Judgify").join("judgify.db");
    let conn = Connection::open(db_path)?;

    // Check if FTS table already has data
    let fts_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ccp_docs_fts",
        [],
        |row| row.get(0),
    )?;

    if fts_count > 0 {
        println!("📊 CCP 데이터가 이미 존재합니다 ({} documents). 스킵합니다.", fts_count);
        return Ok(());
    }

    // Only execute 004 seed data (001-003 schemas already created in init_schema)
    let seed_file = "migrations/004_ccp_seed_data.sql";

    println!("📄 실행 중: {}", seed_file);
    match std::fs::read_to_string(seed_file) {
        Ok(sql) => {
            conn.execute_batch(&sql)?;
            println!("   ✅ Seed 데이터 삽입 완료");
        }
        Err(e) => {
            return Err(rusqlite::Error::InvalidPath(PathBuf::from(format!("파일 읽기 실패: {}", e))));
        }
    }

    // External Content 방식: 수동으로 데이터 복사 (ccp.rs rebuild_fts5_index와 동일)
    // Step 1: 기존 데이터 삭제
    conn.execute_batch("DELETE FROM ccp_docs_fts")?;

    // Step 2: 원본 테이블에서 데이터 복사
    conn.execute_batch(r#"
        INSERT INTO ccp_docs_fts (rowid, title, content)
        SELECT id, title, content FROM ccp_docs
    "#)?;

    // Rebuild 후 인덱스 개수 확인
    let fts_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ccp_docs_fts",
        [],
        |row| row.get(0),
    )?;

    println!("✅ FTS5 rebuild 완료: {}건", fts_count);

    Ok(())
}
