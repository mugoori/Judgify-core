//! 차트 생성 Tauri 커맨드
//!
//! 자연어 요청 → 차트 데이터 생성

use crate::database::Database;
use crate::services::chart_service::{ChartResponse, ChartService, LLMChartPlan};
use serde::{Deserialize, Serialize};

/// 차트 생성 요청
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateChartRequest {
    pub request: String,
}

/// 차트 생성 응답 (Frontend 전달용)
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateChartResponse {
    pub success: bool,
    pub chart: Option<ChartResponse>,
    pub error: Option<String>,
}

/// Tauri command: 자연어로 차트 생성
///
/// Frontend 사용 예시:
/// ```typescript
/// const result = await invoke<GenerateChartResponse>('generate_chart', {
///   request: '라인별 생산량 비교해줘'
/// });
///
/// if (result.success && result.chart) {
///   // 차트 렌더링
///   renderChart(result.chart);
/// }
/// ```
#[tauri::command]
pub async fn generate_chart(request: String) -> Result<GenerateChartResponse, String> {
    println!("📊 [IPC] generate_chart called!");
    println!("   request: {}", request);

    // 1. ChartService 초기화
    let chart_service = match ChartService::new() {
        Ok(service) => service,
        Err(e) => {
            return Ok(GenerateChartResponse {
                success: false,
                chart: None,
                error: Some(format!("ChartService 초기화 실패: {}", e)),
            });
        }
    };

    // 2. LLM으로 SQL + 차트 설정 생성
    let plan = match chart_service.generate_chart_plan(&request).await {
        Ok(plan) => plan,
        Err(e) => {
            return Ok(GenerateChartResponse {
                success: false,
                chart: None,
                error: Some(format!("차트 계획 생성 실패: {}", e)),
            });
        }
    };

    // 3. Database 연결 + SQL 실행 (별도 blocking 스레드에서 실행)
    // MutexGuard는 Send가 아니므로 spawn_blocking 사용
    let plan_clone = plan.clone();
    let chart_response_result = tokio::task::spawn_blocking(move || {
        execute_chart_query(&plan_clone)
    }).await;

    let mut chart_response = match chart_response_result {
        Ok(Ok(response)) => response,
        Ok(Err(e)) => {
            return Ok(GenerateChartResponse {
                success: false,
                chart: None,
                error: Some(e),
            });
        }
        Err(e) => {
            return Ok(GenerateChartResponse {
                success: false,
                chart: None,
                error: Some(format!("Task 실행 실패: {}", e)),
            });
        }
    };

    // 💡 AI 인사이트 생성 (차트 데이터 기반, conn이 이미 해제됨)
    match chart_service.generate_insight(&chart_response, &request).await {
        Ok(insight) => {
            chart_response.insight = Some(insight);
        }
        Err(e) => {
            println!("⚠️ [IPC] Insight generation failed: {}", e);
            // 인사이트 생성 실패해도 차트는 정상 반환
        }
    }

    println!("✅ [IPC] Chart generated: {} ({:?})", chart_response.title, chart_response.chart_type);

    Ok(GenerateChartResponse {
        success: true,
        chart: Some(chart_response),
        error: None,
    })
}

/// Tauri command: 지원 가능한 차트 시나리오 목록 반환
#[tauri::command]
pub fn get_chart_examples() -> Vec<String> {
    vec![
        "지난 주 살균 온도 추이 보여줘".to_string(),
        "라인별 생산량 비교해줘".to_string(),
        "오늘 불량률 현황".to_string(),
        "설비별 가동률".to_string(),
        "CCP 검사 결과 분포".to_string(),
    ]
}

/// 동기 헬퍼 함수: DB 쿼리 실행 (spawn_blocking용)
fn execute_chart_query(plan: &LLMChartPlan) -> Result<ChartResponse, String> {
    let chart_service = ChartService::new()
        .map_err(|e| format!("ChartService 초기화 실패: {}", e))?;

    let db = Database::new()
        .map_err(|e| format!("Database 연결 실패: {}", e))?;

    let conn_arc = db.get_connection();
    let conn = conn_arc.lock()
        .map_err(|e| format!("Database lock 실패: {}", e))?;

    chart_service.execute_and_transform(&conn, plan)
        .map_err(|e| format!("SQL 실행 실패: {}", e))
}
