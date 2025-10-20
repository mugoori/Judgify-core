use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub database_connected: bool,
    pub database_path: String,
    pub openai_configured: bool,
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
    use crate::database::Database;

    let db_connected = Database::new().is_ok();
    let openai_configured = std::env::var("OPENAI_API_KEY").is_ok();

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
        openai_configured,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // Simplified - would need global state to track
    })
}

#[tauri::command]
pub async fn get_system_stats() -> Result<SystemStats, String> {
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
    if let Some(data_dir) = dirs::data_local_dir() {
        Ok(data_dir.join("Judgify").to_string_lossy().to_string())
    } else {
        Err("Could not determine data directory".to_string())
    }
}

#[tauri::command]
pub async fn export_database(export_path: String) -> Result<(), String> {
    use std::fs;

    let db_path = if let Some(data_dir) = dirs::data_local_dir() {
        data_dir.join("Judgify").join("judgify.db")
    } else {
        return Err("Could not determine database path".to_string());
    };

    fs::copy(&db_path, &export_path).map_err(|e| e.to_string())?;

    Ok(())
}
