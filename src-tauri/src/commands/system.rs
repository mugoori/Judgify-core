use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub database_connected: bool,
    pub database_path: String,
    pub claude_configured: bool,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_judgments: u32,
    pub total_workflows: u32,
    pub total_training_samples: u32,
    pub average_confidence: f64,
}

#[tauri::command]
pub async fn get_system_status() -> Result<SystemStatus, String> {
    println!("ℹ️ [IPC] get_system_status called!");
    use crate::database::Database;

    let db_connected = Database::new().is_ok();
    let claude_configured = std::env::var("ANTHROPIC_API_KEY").is_ok();

    let db_path = if let Some(data_dir) = dirs::data_local_dir() {
        data_dir
            .join("Judgify")
            .join("judgify.db")
            .to_string_lossy()
            .to_string()
    } else {
        "Unknown".to_string()
    };

    Ok(SystemStatus {
        database_connected: db_connected,
        database_path: db_path,
        claude_configured,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // Simplified - would need global state to track
    })
}

#[tauri::command]
pub async fn get_system_stats() -> Result<SystemStats, String> {
    println!("📊 [IPC] get_system_stats called!");
    use crate::database::Database;

    let db = Database::new().map_err(|e| e.to_string())?;

    // Simplified stats calculation
    let total_judgments = db
        .get_judgment_history(None, 10000)
        .map(|j| j.len() as u32)
        .unwrap_or(0);

    let total_workflows = db.get_all_workflows().map(|w| w.len() as u32).unwrap_or(0);

    let judgments = db.get_judgment_history(None, 10000).unwrap_or_default();
    let average_confidence = if !judgments.is_empty() {
        judgments.iter().map(|j| j.confidence).sum::<f64>() / judgments.len() as f64
    } else {
        0.0
    };

    Ok(SystemStats {
        total_judgments,
        total_workflows,
        total_training_samples: 0, // Would need separate query
        average_confidence,
    })
}

#[tauri::command]
pub async fn get_data_directory() -> Result<String, String> {
    println!("📁 [IPC] get_data_directory called!");
    if let Some(data_dir) = dirs::data_local_dir() {
        Ok(data_dir.join("Judgify").to_string_lossy().to_string())
    } else {
        Err("Could not determine data directory".to_string())
    }
}

#[tauri::command]
pub async fn export_database(export_path: String) -> Result<(), String> {
    println!("💾 [IPC] export_database called! export_path: {:?}", export_path);
    use std::fs;
    use std::path::Path;

    let db_path = if let Some(data_dir) = dirs::data_local_dir() {
        data_dir.join("Judgify").join("judgify.db")
    } else {
        return Err("Could not determine database path".to_string());
    };

    // 원본 데이터베이스 파일 존재 여부 확인
    if !db_path.exists() {
        return Err(format!(
            "데이터베이스 파일이 아직 생성되지 않았습니다. 앱을 먼저 사용하여 데이터를 생성한 후 백업하세요.\n경로: {:?}",
            db_path
        ));
    }

    // 대상 디렉토리가 존재하지 않으면 생성
    if let Some(parent_dir) = Path::new(&export_path).parent() {
        if !parent_dir.exists() {
            println!("📁 [IPC] Creating directory: {:?}", parent_dir);
            fs::create_dir_all(parent_dir).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
    }

    fs::copy(&db_path, &export_path).map_err(|e| e.to_string())?;
    println!("✅ [IPC] Database exported successfully to: {:?}", export_path);

    Ok(())
}

#[tauri::command]
pub async fn get_token_metrics() -> Result<crate::database::sqlite::TokenMetrics, String> {
    println!("📊 [IPC] get_token_metrics called!");
    use crate::database::Database;

    let db = Database::new().map_err(|e| e.to_string())?;
    db.get_token_metrics().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_api_key(api_key: String) -> Result<(), String> {
    println!("🔑 [IPC] save_api_key called!");

    // API 키 형식 검증
    if !api_key.starts_with("sk-ant-") {
        return Err("Claude API 키 형식이 올바르지 않습니다. 'sk-ant-'로 시작해야 합니다.".to_string());
    }

    // 1. Windows Credential Manager / macOS Keychain / Linux Secret Service에 영구 저장
    match keyring::Entry::new("Judgify", "claude_api_key") {
        Ok(entry) => {
            match entry.set_password(&api_key) {
                Ok(_) => {
                    println!("✅ [IPC] API 키가 시스템 keychain에 저장되었습니다.");
                }
                Err(e) => {
                    eprintln!("⚠️  [IPC] Keychain 저장 실패: {}", e);
                    return Err(format!("시스템 저장소에 API 키 저장 실패: {}", e));
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  [IPC] Keychain 초기화 실패: {}", e);
            return Err(format!("시스템 저장소 초기화 실패: {}", e));
        }
    }

    // 2. 런타임 환경 변수에도 설정 (현재 세션용)
    std::env::set_var("ANTHROPIC_API_KEY", &api_key);

    println!("✅ [IPC] API 키가 성공적으로 설정되었습니다 (keychain + 환경 변수).");
    Ok(())
}

#[tauri::command]
pub async fn load_api_key() -> Result<String, String> {
    println!("🔑 [IPC] load_api_key called!");

    match keyring::Entry::new("Judgify", "claude_api_key") {
        Ok(entry) => {
            match entry.get_password() {
                Ok(api_key) => {
                    println!("✅ [IPC] API 키를 시스템 keychain에서 로드했습니다.");

                    // 환경 변수에도 설정
                    std::env::set_var("ANTHROPIC_API_KEY", &api_key);

                    Ok(api_key)
                }
                Err(e) => {
                    eprintln!("⚠️  [IPC] API 키 로드 실패: {}", e);
                    Err("저장된 API 키가 없습니다. Settings 페이지에서 API 키를 설정해주세요.".to_string())
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  [IPC] Keychain 초기화 실패: {}", e);
            Err(format!("시스템 저장소 초기화 실패: {}", e))
        }
    }
}

#[tauri::command]
pub async fn delete_api_key() -> Result<(), String> {
    println!("🔑 [IPC] delete_api_key called!");

    match keyring::Entry::new("Judgify", "claude_api_key") {
        Ok(entry) => {
            match entry.delete_password() {
                Ok(_) => {
                    println!("✅ [IPC] API 키가 시스템 keychain에서 삭제되었습니다.");

                    // 환경 변수에서도 제거
                    std::env::remove_var("ANTHROPIC_API_KEY");

                    Ok(())
                }
                Err(e) => {
                    eprintln!("⚠️  [IPC] API 키 삭제 실패: {}", e);
                    Err(format!("API 키 삭제 실패: {}", e))
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️  [IPC] Keychain 초기화 실패: {}", e);
            Err(format!("시스템 저장소 초기화 실패: {}", e))
        }
    }
}
