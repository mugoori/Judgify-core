use crate::services::judgment_engine::{JudgmentEngine, JudgmentInput, JudgmentResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteJudgmentRequest {
    pub workflow_id: String,
    pub input_data: serde_json::Value,
}

#[tauri::command]
pub async fn execute_judgment(
    request: ExecuteJudgmentRequest,
) -> Result<JudgmentResult, String> {
    println!("⚖️ [IPC] execute_judgment called! workflow_id: {:?}", request.workflow_id);
    let engine = JudgmentEngine::new().map_err(|e| e.to_string())?;

    let input = JudgmentInput {
        workflow_id: request.workflow_id,
        input_data: request.input_data,
    };

    engine.execute(input).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_judgment_history(
    workflow_id: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<JudgmentResult>, String> {
    println!("📊 [IPC] get_judgment_history called! workflow_id: {:?}, limit: {:?}", workflow_id, limit);
    let engine = JudgmentEngine::new().map_err(|e| e.to_string())?;
    engine.get_history(workflow_id, limit.unwrap_or(50))
        .await
        .map_err(|e| e.to_string())
}
