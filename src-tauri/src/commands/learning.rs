use crate::services::learning_service::LearningService;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveFeedbackRequest {
    pub judgment_id: String,
    pub feedback_type: String,
    pub value: i32,
    pub comment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FewShotSamplesRequest {
    pub workflow_id: String,
    pub limit: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrainingSample {
    pub id: String,
    pub workflow_id: String,
    pub input_data: serde_json::Value,
    pub expected_result: bool,
    pub actual_result: Option<bool>,
    pub accuracy: Option<f64>,
    pub created_at: String,
}

#[tauri::command]
pub async fn save_feedback(request: SaveFeedbackRequest) -> Result<(), String> {
    let service = LearningService::new().map_err(|e| e.to_string())?;

    service
        .save_feedback(
            request.judgment_id,
            request.feedback_type,
            request.value,
            request.comment,
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_few_shot_samples(
    request: FewShotSamplesRequest,
) -> Result<Vec<TrainingSample>, String> {
    let service = LearningService::new().map_err(|e| e.to_string())?;

    let samples = service
        .get_few_shot_samples(request.workflow_id, request.limit)
        .map_err(|e| e.to_string())?;

    Ok(samples
        .into_iter()
        .map(|s| TrainingSample {
            id: s.id,
            workflow_id: s.workflow_id,
            input_data: serde_json::from_str(&s.input_data).unwrap_or(serde_json::json!({})),
            expected_result: s.expected_result,
            actual_result: s.actual_result,
            accuracy: s.accuracy,
            created_at: s.created_at.to_rfc3339(),
        })
        .collect())
}

#[tauri::command]
pub async fn extract_rules(workflow_id: String) -> Result<Vec<String>, String> {
    let service = LearningService::new().map_err(|e| e.to_string())?;

    let rules = service
        .extract_rules(workflow_id)
        .map_err(|e| e.to_string())?;

    Ok(rules)
}
