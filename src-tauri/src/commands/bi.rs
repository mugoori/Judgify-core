use crate::services::bi_service::BiService;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

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
    println!("🔍 [IPC] generate_bi_insight called! user_request: {:?}", request.user_request);
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

/// Phase 5: 실시간 이벤트 스트리밍을 지원하는 인사이트 생성
#[tauri::command]
pub async fn generate_bi_insight_stream(
    app_handle: AppHandle,
    request: GenerateInsightRequest,
) -> Result<BiInsightResponse, String> {
    // AppHandle을 포함한 BiService 생성
    let service = BiService::with_app_handle(Some(app_handle))
        .map_err(|e| e.to_string())?;

    // 스트리밍 모드로 인사이트 생성 (6개 이벤트 발생)
    let insight = service
        .generate_insight_stream(request.user_request)
        .await
        .map_err(|e| e.to_string())?;

    Ok(BiInsightResponse {
        title: insight.title,
        insights: insight.insights,
        component_code: insight.component_code,
        recommendations: insight.recommendations,
    })
}
