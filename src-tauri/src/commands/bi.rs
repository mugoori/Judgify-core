use crate::services::bi_service::{BiService, BiInsight};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateInsightRequest {
    pub user_request: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiInsightResponse {
    pub title: String,
    pub insights: Vec<String>,
    pub component_code: String,
    pub recommendations: Vec<String>,
}

#[tauri::command]
pub async fn generate_bi_insight(
    request: GenerateInsightRequest,
) -> Result<BiInsightResponse, String> {
    let service = BiService::new().map_err(|e| e.to_string())?;

    let insight = service
        .generate_insight(request.user_request)
        .await
        .map_err(|e| e.to_string())?;

    Ok(BiInsightResponse {
        title: insight.title,
        insights: insight.insights,
        component_code: insight.component_code,
        recommendations: insight.recommendations,
    })
}
