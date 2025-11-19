use crate::services::ccp_service::CcpService;
use crate::database::{CcpDocWithScore, CcpJudgmentRequest, CcpJudgmentResponse};

/// Tauri command: CCP 문서 검색 (FTS5 BM25)
///
/// Frontend 사용 예시:
/// ```typescript
/// const docs = await invoke('search_ccp_docs', {
///   companyId: 'COMP_A',
///   ccpId: 'CCP-01',
///   query: '열처리 기준',
///   topK: 5
/// });
/// ```
#[tauri::command]
pub async fn search_ccp_docs(
    company_id: String,
    ccp_id: Option<String>,
    query: String,
    top_k: usize,
) -> Result<Vec<CcpDocWithScore>, String> {
    let service = CcpService::new()
        .map_err(|e| format!("Service 초기화 실패: {}", e))?;

    service.search_ccp_docs(
        &company_id,
        ccp_id.as_deref(),
        &query,
        top_k,
    )
    .map_err(|e| format!("검색 실패: {}", e))
}

/// Tauri command: CCP 상태 판단 (하이브리드)
///
/// Frontend 사용 예시:
/// ```typescript
/// const result = await invoke('judge_ccp_status', {
///   request: {
///     company_id: 'COMP_A',
///     ccp_id: 'CCP-01',
///     period_from: '2025-11-01',
///     period_to: '2025-11-14'
///   }
/// });
/// console.log('위험도:', result.risk_level);
/// console.log('AI 요약:', result.llm_summary);
/// console.log('증거 문서:', result.evidence_docs);
/// ```
#[tauri::command]
pub async fn judge_ccp_status(
    request: CcpJudgmentRequest,
) -> Result<CcpJudgmentResponse, String> {
    let service = CcpService::new()
        .map_err(|e| format!("Service 초기화 실패: {}", e))?;

    service.judge_ccp_status(request)
        .await
        .map_err(|e| format!("판단 실패: {}", e))
}
