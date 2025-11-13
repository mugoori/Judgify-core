use crate::database::BackupManager;
use std::path::PathBuf;

#[tauri::command]
pub async fn create_backup(app_handle: tauri::AppHandle) -> Result<String, String> {
    let db_path = app_handle
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| "Failed to get app data directory".to_string())?
        .join("judgify.db");

    let manager = BackupManager::new(db_path)
        .map_err(|e| format!("Failed to initialize backup manager: {}", e))?;

    let backup_path = manager
        .create_backup()
        .map_err(|e| format!("Failed to create backup: {}", e))?;

    // 오래된 백업 자동 정리 (최근 10개만 유지)
    manager
        .cleanup_old_backups(10)
        .map_err(|e| format!("Failed to cleanup old backups: {}", e))?;

    Ok(backup_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn restore_backup(
    app_handle: tauri::AppHandle,
    backup_path: String,
) -> Result<String, String> {
    let db_path = app_handle
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| "Failed to get app data directory".to_string())?
        .join("judgify.db");

    let manager = BackupManager::new(db_path)
        .map_err(|e| format!("Failed to initialize backup manager: {}", e))?;

    let backup_path_buf = PathBuf::from(backup_path);
    manager
        .restore_from_backup(&backup_path_buf)
        .map_err(|e| format!("Failed to restore backup: {}", e))?;

    Ok("Backup restored successfully".to_string())
}

#[tauri::command]
pub async fn list_backups(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
    let db_path = app_handle
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| "Failed to get app data directory".to_string())?
        .join("judgify.db");

    let manager = BackupManager::new(db_path)
        .map_err(|e| format!("Failed to initialize backup manager: {}", e))?;

    let backups = manager
        .list_backups()
        .map_err(|e| format!("Failed to list backups: {}", e))?;

    let backup_paths: Vec<String> = backups
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    Ok(backup_paths)
}

#[tauri::command]
pub async fn get_backup_info(app_handle: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let db_path = app_handle
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| "Failed to get app data directory".to_string())?
        .join("judgify.db");

    let manager = BackupManager::new(db_path)
        .map_err(|e| format!("Failed to initialize backup manager: {}", e))?;

    let backups = manager
        .list_backups()
        .map_err(|e| format!("Failed to list backups: {}", e))?;

    let total_size = manager
        .get_total_backup_size()
        .map_err(|e| format!("Failed to get backup size: {}", e))?;

    Ok(serde_json::json!({
        "count": backups.len(),
        "total_size_bytes": total_size,
        "total_size_mb": (total_size as f64) / (1024.0 * 1024.0),
    }))
}
