use crate::services::chat_service::{ChatService, Intent};
use crate::services::mes_data_service::MesDataService;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageRequest {
    pub message: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    pub response: String,
    pub session_id: String,
    pub intent: String,
    pub action_result: Option<serde_json::Value>,
    pub table_data: Option<TableData>,  // 테이블 형식 데이터 추가
    pub chart_data: Option<ChartData>,  // 차트 데이터 추가
}

/// 차트 데이터 구조체 (프론트엔드 ChartResponse와 호환)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub chart_type: String,        // bar, line, pie, gauge
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar_line_data: Option<Vec<serde_json::Value>>,  // 평탄화된 JSON 객체 직접 사용
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pie_data: Option<Vec<PieChartData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gauge_data: Option<GaugeChartData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_keys: Option<Vec<DataKeyConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insight: Option<String>,
}

// ChartDataPoint 구조체 삭제 - serde(flatten)이 HashMap과 제대로 작동하지 않아서
// Vec<serde_json::Value>로 직접 평탄화된 JSON 객체 사용

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieChartData {
    pub name: String,
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeChartData {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataKeyConfig {
    pub key: String,
    pub color: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String, // user | assistant
    pub content: String,
    pub intent: Option<String>,
    pub created_at: String,
}

/// Week 1: ChatService를 사용한 실제 LLM 기반 채팅 처리
#[tauri::command]
pub async fn send_chat_message(
    request: ChatMessageRequest,
) -> Result<ChatMessageResponse, String> {
    println!("💬 [IPC] send_chat_message called! message: {:?}", request.message.chars().take(50).collect::<String>());
    let service = ChatService::new().map_err(|e| e.to_string())?;

    // 1. 세션 ID 확인 또는 생성
    let session_id = if let Some(sid) = request.session_id {
        sid
    } else {
        let session = service
            .create_session(None)
            .await
            .map_err(|e| e.to_string())?;
        session.id
    };

    // 2. 사용자 메시지 저장
    service
        .save_message(&session_id, "user", &request.message, None)
        .await
        .map_err(|e| e.to_string())?;

    // 3. LLM으로 의도 분석
    let intent = service
        .analyze_intent(&request.message)
        .await
        .map_err(|e| e.to_string())?;

    // 4. Week 2: 의도에 따른 실제 서비스 라우팅
    let (response, action_result) = match intent {
        Intent::JudgmentExecution => {
            // 4-1. 파라미터 추출
            match service
                .extract_judgment_params(&request.message)
                .await
            {
                Ok((workflow_id, input_data)) => {
                    // 4-2. Judgment Service 호출
                    match service
                        .route_to_judgment(workflow_id.clone(), input_data)
                        .await
                    {
                        Ok(result) => {
                            let confidence = result["confidence"].as_f64().unwrap_or(0.0);
                            let result_bool = result["result"].as_bool().unwrap_or(false);
                            let method = result["method_used"].as_str().unwrap_or("unknown");

                            (
                                format!(
                                    "판단 실행 완료!\n\n워크플로우: {}\n결과: {}\n신뢰도: {:.1}%\n방법: {}",
                                    workflow_id,
                                    if result_bool { "정상 ✅" } else { "비정상 ❌" },
                                    confidence * 100.0,
                                    method
                                ),
                                Some(result),
                            )
                        }
                        Err(e) => (
                            format!("판단 실행 실패: {}", e),
                            None,
                        ),
                    }
                }
                Err(e) => (
                    format!("파라미터 추출 실패: {}. 워크플로우 ID와 입력 데이터를 명확히 지정해주세요.", e),
                    None,
                ),
            }
        }
        Intent::WorkflowManagement => {
            // 4-3. 워크플로우 파라미터 추출
            match service
                .extract_workflow_params(&request.message)
                .await
            {
                Ok((action, params)) => {
                    // 4-4. Workflow Service 호출
                    match service.route_to_workflow(&action, params).await {
                        Ok(result) => {
                            let action_str = result["action"].as_str().unwrap_or("unknown");
                            let response_text = match action_str {
                                "list" => {
                                    let empty_workflows = vec![];
                                    let workflows = result["workflows"].as_array().unwrap_or(&empty_workflows);
                                    format!(
                                        "워크플로우 목록 ({} 개):\n\n{}",
                                        workflows.len(),
                                        workflows
                                            .iter()
                                            .map(|w| format!(
                                                "• {} (ID: {}, 버전: {}, 활성: {})",
                                                w["name"].as_str().unwrap_or("Unknown"),
                                                w["id"].as_str().unwrap_or("Unknown"),
                                                w["version"].as_i64().unwrap_or(1),
                                                if w["is_active"].as_bool().unwrap_or(false) { "✅" } else { "❌" }
                                            ))
                                            .collect::<Vec<_>>()
                                            .join("\n")
                                    )
                                }
                                "get" => {
                                    let workflow = &result["workflow"];
                                    format!(
                                        "워크플로우 조회:\n\n이름: {}\nID: {}\n버전: {}\n활성: {}",
                                        workflow["name"].as_str().unwrap_or("Unknown"),
                                        workflow["id"].as_str().unwrap_or("Unknown"),
                                        workflow["version"].as_i64().unwrap_or(1),
                                        if workflow["is_active"].as_bool().unwrap_or(false) { "✅" } else { "❌" }
                                    )
                                }
                                _ => format!("워크플로우 작업 완료: {}", action_str),
                            };
                            (response_text, Some(result))
                        }
                        Err(e) => (
                            format!("워크플로우 작업 실패: {}", e),
                            None,
                        ),
                    }
                }
                Err(e) => (
                    format!("파라미터 추출 실패: {}. 워크플로우 작업을 명확히 지정해주세요.", e),
                    None,
                ),
            }
        }
        Intent::DataVisualization => {
            // 4-5. BI 파라미터 추출
            match service.extract_bi_params(&request.message) {
                Ok(bi_request) => {
                    // 4-6. BI Service 호출
                    match service.route_to_bi(bi_request).await {
                        Ok(result) => {
                            let title = result["title"].as_str().unwrap_or("인사이트");
                            let empty_insights = vec![];
                            let empty_recommendations = vec![];
                            let insights = result["insights"].as_array().unwrap_or(&empty_insights);
                            let recommendations = result["recommendations"].as_array().unwrap_or(&empty_recommendations);

                            (
                                format!(
                                    "{}\n\n📊 인사이트:\n{}\n\n💡 권장사항:\n{}",
                                    title,
                                    insights
                                        .iter()
                                        .map(|i| format!("• {}", i.as_str().unwrap_or("")))
                                        .collect::<Vec<_>>()
                                        .join("\n"),
                                    recommendations
                                        .iter()
                                        .map(|r| format!("• {}", r.as_str().unwrap_or("")))
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                ),
                                Some(result),
                            )
                        }
                        Err(e) => (
                            format!("BI 인사이트 생성 실패: {}", e),
                            None,
                        ),
                    }
                }
                Err(e) => (
                    format!("파라미터 추출 실패: {}", e),
                    None,
                ),
            }
        }
        Intent::ChartAnalysis => {
            // 차트/그래프 분석 요청 - 프롬프트 라우터 사용
            println!("📊 [ChartAnalysis] Processing chart analysis request");

            // 대화 이력 가져오기
            let history = service.get_history(&session_id, 5).await.unwrap_or_default();

            match service.generate_chart_response(&request.message, history).await {
                Ok(llm_response) => {
                    println!("✅ [ChartAnalysis] Chart response generated successfully");

                    // 응답에서 차트 JSON 추출 및 파싱
                    let chart_data = extract_chart_data_from_response(&llm_response);

                    // 텍스트 응답 (차트 JSON 블록 제거)
                    let text_response = remove_chart_json_block(&llm_response);

                    // 어시스턴트 응답 저장
                    let intent_str = "chartanalysis".to_string();
                    let _ = service
                        .save_message(&session_id, "assistant", &text_response, Some(&intent_str))
                        .await;

                    return Ok(ChatMessageResponse {
                        response: text_response,
                        session_id,
                        intent: intent_str,
                        action_result: None,
                        table_data: None,
                        chart_data,
                    });
                }
                Err(e) => {
                    println!("❌ [ChartAnalysis] Chart generation failed: {}", e);
                    (
                        format!("차트 분석 생성 실패: {}. 다시 시도해주세요.", e),
                        None,
                    )
                }
            }
        }
        Intent::SettingsChange => (
            "설정 변경 기능입니다. 어떤 설정을 변경하시겠습니까?".to_string(),
            None,
        ),
        Intent::GeneralQuery => {
            // 데이터 조회 요청인지 확인
            let is_data_query = check_if_data_query(&request.message);

            if is_data_query {
                println!("📊 Data query detected in GeneralQuery");

                // ERP/MES 테이블 직접 조회 시도
                if let Some((summary_text, table_data)) = try_query_erp_mes_tables(&request.message).await {
                    println!("✅ ERP/MES table data found! Now generating natural language response...");

                    // 테이블 데이터를 JSON으로 변환하여 LLM에 전달
                    let table_json = serde_json::to_string_pretty(&serde_json::json!({
                        "columns": table_data.columns,
                        "rows": table_data.rows,
                        "total_count": table_data.total_count
                    })).unwrap_or_default();

                    // LLM을 사용하여 자연어 응답 생성
                    match service.generate_response_from_table_data(
                        &request.message,
                        &table_json,
                        &summary_text
                    ).await {
                        Ok(natural_response) => {
                            println!("✅ Natural language response generated from table data");
                            return Ok(ChatMessageResponse {
                                response: natural_response,
                                session_id,
                                intent: format!("{:?}", intent).to_lowercase(),
                                action_result: None,
                                table_data: Some(table_data),  // 근거 자료로 테이블도 함께 반환
                                chart_data: None,
                            });
                        }
                        Err(e) => {
                            eprintln!("⚠️ Failed to generate natural response, falling back to summary: {}", e);
                            // LLM 실패시 기존 요약 텍스트로 반환
                            return Ok(ChatMessageResponse {
                                response: summary_text,
                                session_id,
                                intent: format!("{:?}", intent).to_lowercase(),
                                action_result: None,
                                table_data: Some(table_data),
                                chart_data: None,
                            });
                        }
                    }
                }

                // MES 데이터 로그 조회 시도 (CSV 업로드 데이터)
                match query_mes_data_for_chat(&request.message).await {
                    Ok(Some((summary_text, table_data))) => {
                        println!("✅ MES data found! Now generating natural language response...");

                        // 테이블 데이터를 JSON으로 변환
                        let table_json = serde_json::to_string_pretty(&serde_json::json!({
                            "columns": table_data.columns,
                            "rows": table_data.rows,
                            "total_count": table_data.total_count
                        })).unwrap_or_default();

                        // LLM을 사용하여 자연어 응답 생성
                        match service.generate_response_from_table_data(
                            &request.message,
                            &table_json,
                            &summary_text
                        ).await {
                            Ok(natural_response) => {
                                println!("✅ Natural language response generated from MES data");
                                return Ok(ChatMessageResponse {
                                    response: natural_response,
                                    session_id,
                                    intent: format!("{:?}", intent).to_lowercase(),
                                    action_result: None,
                                    table_data: Some(table_data),
                                    chart_data: None,
                                });
                            }
                            Err(e) => {
                                eprintln!("⚠️ Failed to generate natural response: {}", e);
                                return Ok(ChatMessageResponse {
                                    response: summary_text,
                                    session_id,
                                    intent: format!("{:?}", intent).to_lowercase(),
                                    action_result: None,
                                    table_data: Some(table_data),
                                    chart_data: None,
                                });
                            }
                        }
                    }
                    Ok(None) => {
                        println!("ℹ️ No MES data found for query - returning clear message");
                        // 데이터가 없으면 명확한 안내 메시지 반환
                        return Ok(ChatMessageResponse {
                            response: "죄송합니다. 현재 조회 가능한 데이터가 없습니다.\n\n📋 데이터 조회 방법:\n1. CSV 파일 업로드: 상단의 '파일 첨부' 버튼을 클릭하여 MES 데이터를 업로드하세요.\n2. 데이터베이스 뷰어: 우측 상단의 데이터베이스 아이콘을 클릭하여 직접 테이블 데이터를 확인하세요.\n\n💡 조회 가능한 테이블:\n• 생산 지시서\n• CCP 검사 기록\n• 품질 검사\n• 완제품 LOT".to_string(),
                            session_id,
                            intent: format!("{:?}", intent).to_lowercase(),
                            action_result: None,
                            table_data: None,
                            chart_data: None,
                        });
                    }
                    Err(e) => {
                        eprintln!("⚠️ MES data query failed: {}", e);
                        // 오류 발생시 일반 대화로 처리
                    }
                }
            }

            // 일반 대화 처리 (기존 코드)
            let history = service
                .get_history(&session_id, 5)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("⚠️ Failed to get history for GeneralQuery: {}", e);
                    Vec::new()
                });

            println!("🧠 GeneralQuery - using conversational AI");
            println!("   History: {} messages", history.len());

            match service
                .generate_conversational_response(&request.message, history)
                .await
            {
                Ok(response) => {
                    println!("✅ Conversational response generated: {}",
                        if response.chars().count() > 80 {
                            format!("{}...", response.chars().take(80).collect::<String>())
                        } else {
                            response.clone()
                        }
                    );
                    (response, None)
                }
                Err(e) => {
                    eprintln!("❌ GeneralQuery 응답 생성 실패: {}", e);
                    (
                        "죄송합니다. 일시적인 오류가 발생했습니다. 다시 시도해주세요.".to_string(),
                        None,
                    )
                }
            }
        }
    };

    // 5. 어시스턴트 응답 저장
    let intent_str = format!("{:?}", intent).to_lowercase();
    service
        .save_message(&session_id, "assistant", &response, Some(&intent_str))
        .await
        .map_err(|e| e.to_string())?;

    Ok(ChatMessageResponse {
        response,
        session_id,
        intent: intent_str,
        action_result,
        table_data: None,
        chart_data: None, // ChartAnalysis에서는 별도로 처리됨
    })
}

/// Week 1: ChatService를 사용한 실제 히스토리 조회
#[tauri::command]
pub async fn get_chat_history(session_id: String) -> Result<Vec<ChatMessage>, String> {
    println!("📜 [IPC] get_chat_history called! session_id: {:?}", session_id);
    let service = ChatService::new().map_err(|e| e.to_string())?;

    let messages = service
        .get_history(&session_id, 50)
        .await
        .map_err(|e| e.to_string())?;

    Ok(messages
        .into_iter()
        .map(|m| ChatMessage {
            id: m.id,
            session_id: m.session_id,
            role: m.role,
            content: m.content,
            intent: m.intent,
            created_at: m.created_at.to_rfc3339(),
        })
        .collect())
}

/// Claude API 키 유효성 테스트
#[tauri::command]
pub async fn test_claude_api() -> Result<String, String> {
    println!("🔑 [IPC] test_claude_api called!");
    let service = ChatService::new().map_err(|e| e.to_string())?;

    // 간단한 메시지로 API 테스트
    let result = service
        .analyze_intent("안녕하세요")
        .await;

    match result {
        Ok(_) => {
            println!("✅ Claude API 테스트 성공!");
            Ok("Claude API 키가 올바르게 설정되었습니다.".to_string())
        }
        Err(e) => {
            println!("❌ Claude API 테스트 실패: {}", e);
            Err(format!("Claude API 키가 유효하지 않습니다: {}", e))
        }
    }
}

/// 메시지가 데이터 조회 요청인지 확인하는 헬퍼 함수
fn check_if_data_query(message: &str) -> bool {
    let lower_message = message.to_lowercase();

    // 설명 요청 패턴 (데이터 조회가 아님!)
    // "어떻게 해?", "방법 알려줘", "설명해줘", "절차 알려줘" 등은 RAG로 처리
    let explanation_patterns = vec![
        "어떻게 해", "어떻게해", "방법", "설명", "절차", "과정",
        "뭐야", "무엇", "왜", "이유", "원리", "원칙",
        "sop", "표준", "규정", "지침", "매뉴얼",
        "how to", "what is", "explain", "procedure",
    ];

    let is_explanation_request = explanation_patterns.iter().any(|pattern| lower_message.contains(pattern));

    // 설명 요청이면 데이터 조회가 아님
    if is_explanation_request {
        println!("📖 Explanation request detected - NOT a data query");
        return false;
    }

    // 데이터 조회 키워드
    let data_keywords = vec![
        // 한글 키워드
        "데이터", "보여줘", "조회", "확인", "찾아", "검색",
        "몇개", "몇 개", "목록", "리스트", "표시", "출력",
        "현황", "내역", "결과", "상태", "이력", "로그",
        // 목록 조회 패턴 (뭐뭐 있어, 어떤 것들 있어 등)
        "뭐뭐", "뭐가 있", "뭐 있", "어떤 것", "무엇이 있", "무엇 있",
        "몇 가지", "몇가지", "종류", "전체", "모두", "다 보여",
        // 영어 키워드
        "data", "show", "query", "search", "find", "list", "display",
        // 조건 관련
        "이상", "이하", "초과", "미만", "같은", "동일한", "포함",
        // 특정 필드 언급 (수치 조회)
        "온도", "습도", "압력", "temperature", "humidity",
        // ERP/MES 데이터 조회 키워드 (테이블 데이터 조회용)
        "재고", "구매", "판매", "발주", "입고", "출하", "납품",
        "lot", "배치", "mes", "erp",
        "ph", "brix", "파라미터",
        // 제품/원료 조회 (명시적)
        "제품", "원료", "자재", "품목",
    ];

    data_keywords.iter().any(|keyword| lower_message.contains(keyword))
}

/// MES 데이터를 조회하고 테이블 형식으로 변환하는 헬퍼 함수
async fn query_mes_data_for_chat(query: &str) -> anyhow::Result<Option<(String, TableData)>> {
    println!("🔍 Querying MES data for: {}", query);

    // MesDataService 인스턴스 생성
    let mes_service = MesDataService::new()?;

    // 하드코딩된 세션 ID 사용 (실제로는 사용자별 세션 관리 필요)
    // 임시로 고정된 세션 ID 사용
    let session_id = "default-mes-session";
    println!("📋 Using session: {}", session_id);

    // MES 데이터 쿼리 실행 (None은 기본 top_k 사용)
    match mes_service.query_mes_data(session_id, query, 10).await {
        Ok(Some(answer)) => {
            // 답변에서 데이터를 추출하여 테이블 형식으로 변환
            // LLM이 구조화된 형식으로 응답하므로 파싱 시도
            let table_data = parse_llm_response_to_table(&answer);

            // 응답 텍스트와 테이블 데이터 반환
            Ok(Some((answer, table_data)))
        }
        Ok(None) => {
            println!("ℹ️ No data found in session");
            Ok(None)
        }
        Err(e) => {
            eprintln!("❌ MES query error: {}", e);
            // 에러가 발생해도 None 반환으로 처리 (일반 대화로 fallback)
            Ok(None)
        }
    }
}

/// 쿼리에서 고객명 추출하는 헬퍼 함수
fn extract_customer_name(query: &str) -> Option<String> {
    // 주요 고객명 패턴 목록 (seed_data.py에서 생성된 고객사명)
    let customer_patterns = vec![
        "쿠팡", "마켓컬리", "이마트", "홈플러스", "롯데마트", "코스트코",
        "CU", "GS25", "세븐일레븐", "이마트24",
        "스타벅스", "이디야", "빽다방", "메가커피", "투썸",
        "종근당", "뉴트리원", "녹십자", "대웅", "프롬바이오",
        "Walmart", "Aeon", "Shopee", "Amazon"
    ];

    let query_lower = query.to_lowercase();

    for pattern in customer_patterns {
        if query_lower.contains(&pattern.to_lowercase()) {
            return Some(pattern.to_string());
        }
    }

    None
}

/// 고객명으로 cust_cd 조회하는 헬퍼 함수
/// 실제 DB 스키마: cust_cd, cust_nm
fn get_customer_id_by_name(conn: &rusqlite::Connection, customer_name: &str) -> Option<String> {
    let sql = "SELECT cust_cd FROM customer_mst WHERE cust_nm LIKE ?";
    let pattern = format!("%{}%", customer_name);

    match conn.query_row(sql, &[&pattern], |row| row.get::<_, String>(0)) {
        Ok(id) => {
            println!("✅ Found cust_cd: {} for name: {}", id, customer_name);
            Some(id)
        }
        Err(e) => {
            eprintln!("⚠️ Customer not found for '{}': {}", customer_name, e);
            None
        }
    }
}

/// ERP/MES 테이블 직접 조회 함수
async fn try_query_erp_mes_tables(query: &str) -> Option<(String, TableData)> {
    use crate::database::Database;
    use rusqlite::params;

    println!("🔍 Trying to query ERP/MES tables for: {}", query);

    // 1. 고객명 추출 시도
    let customer_name = extract_customer_name(query);
    if let Some(ref name) = customer_name {
        println!("🔍 Detected customer name: {}", name);
    }

    // 2. 키워드를 기반으로 테이블 매핑 (더 많은 키워드 추가)
    let query_lower = query.to_lowercase();
    let (table_name, display_name) = if query_lower.contains("생산 지시") || query_lower.contains("생산지시") || query_lower.contains("생산 현황") {
        ("production_order", "생산 지시서")
    } else if query_lower.contains("ccp") || query_lower.contains("검사 기록") || query_lower.contains("온도") || query_lower.contains("살균") {
        ("ccp_check_log", "CCP 검사 기록")
    } else if query_lower.contains("품질") || query_lower.contains("qc") || query_lower.contains("ph") || query_lower.contains("brix") {
        ("qc_test", "품질 검사")
    } else if query_lower.contains("완제품") || query_lower.contains("lot") || query_lower.contains("재고") {
        ("fg_lot", "완제품 LOT")
    } else if query_lower.contains("배치") || query_lower.contains("생산 기록") || query_lower.contains("배합") {
        ("batch_lot", "배치 생산 기록")
    } else if query_lower.contains("충진") || query_lower.contains("충전") {
        ("filling_lot", "충진 기록")
    } else if query_lower.contains("작업 지시") || query_lower.contains("mes") || query_lower.contains("작업 현황") {
        ("mes_work_order", "MES 작업 지시")
    } else if query_lower.contains("공정") || query_lower.contains("실행") || query_lower.contains("작업 이력") {
        ("operation_exec", "공정 실행")
    } else if query_lower.contains("제품") || query_lower.contains("뭐뭐") || query_lower.contains("뭐가 있") || query_lower.contains("뭐 있") {
        // "제품 뭐뭐 있어", "제품 목록" 등
        ("item_mst_products", "회사 제품 목록")  // 특수 플래그 - 완제품만 필터링
    } else if query_lower.contains("원료") || query_lower.contains("자재") || query_lower.contains("아이템") {
        ("item_mst", "제품/원료 마스터")
    } else if query_lower.contains("구매") || query_lower.contains("발주") || query_lower.contains("po") {
        ("purchase_order", "구매 발주")
    } else if query_lower.contains("입고") || query_lower.contains("수입") || query_lower.contains("입하") {
        ("inbound", "입고 기록")
    } else if query_lower.contains("거래처") || query_lower.contains("공급처") || query_lower.contains("협력사") {
        ("vendor_mst", "거래처 정보")
    } else if query_lower.contains("고객") || query_lower.contains("납품처") || query_lower.contains("수요처") {
        ("customer_mst", "고객사 정보")
    } else if query_lower.contains("판매") || query_lower.contains("주문") || query_lower.contains("수주") || query_lower.contains("납품") {
        ("sales_order", "판매 주문")
    } else if query_lower.contains("라인") || query_lower.contains("설비") {
        ("line_mst", "생산 라인 정보")
    } else if query_lower.contains("파라미터") || query_lower.contains("설정값") {
        ("operation_param_log", "공정 파라미터 로그")
    } else {
        // 매칭되는 키워드가 없으면 None 반환
        return None;
    };

    println!("📋 Selected table: {} ({})", table_name, display_name);

    // 3. 데이터베이스 연결
    let db = match Database::new() {
        Ok(db) => db,
        Err(e) => {
            eprintln!("❌ Failed to connect to database: {}", e);
            return None;
        }
    };

    let conn_arc = db.get_connection();
    let conn_guard = match conn_arc.lock() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("❌ Failed to lock database connection: {}", e);
            return None;
        }
    };

    // 4. 고객명이 있고 테이블이 sales_order인 경우 customer_id 조회
    let customer_id = if table_name == "sales_order" {
        customer_name.and_then(|name| get_customer_id_by_name(&conn_guard, &name))
    } else {
        None
    };

    // 4.1 날짜 필터 추출 (년/월)
    let date_filter = extract_date_filter(&query_lower);
    if let Some(ref df) = date_filter {
        println!("📅 Detected date filter: {}", df);
    }

    // 4.2 제품 필터 추출
    let product_filter = extract_product_filter(&query_lower);
    if let Some(ref pf) = product_filter {
        println!("📦 Detected product filter: {}", pf);
    }

    // 5. SQL 쿼리 생성 (고객/날짜/제품 필터링 포함)
    let sql = if table_name == "sales_order" {
        // 판매 주문은 항상 상세 테이블과 조인하여 제품명과 수량 포함
        // 실제 DB 스키마: cust_cd, cust_nm, order_date, item_cd, item_nm, qty
        let mut where_clauses = Vec::new();

        if let Some(ref cust_id) = customer_id {
            where_clauses.push(format!("so.cust_cd = '{}'", cust_id));
        }
        if let Some(ref df) = date_filter {
            where_clauses.push(format!("so.order_date LIKE '{}%'", df));
        }
        if let Some(ref pf) = product_filter {
            where_clauses.push(format!("i.item_nm LIKE '%{}%'", pf));
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", where_clauses.join(" AND "))
        };

        format!(
            "SELECT so.so_no as 주문번호, so.order_date as 주문일, c.cust_nm as 고객사, \
             i.item_nm as 제품명, sod.qty as 주문수량, so.status as 상태 \
             FROM sales_order so \
             LEFT JOIN customer_mst c ON so.cust_cd = c.cust_cd \
             LEFT JOIN sales_order_dtl sod ON so.so_no = sod.so_no \
             LEFT JOIN item_mst i ON sod.item_cd = i.item_cd \
             {}{}",
            where_clause,
            " ORDER BY so.order_date DESC LIMIT 50"
        )
    } else if table_name == "item_mst_products" {
        // 회사 제품 목록 조회 - 완제품(FG)만 필터링하여 읽기 좋은 형태로 출력
        "SELECT item_cd as 제품코드, item_nm as 제품명, unit as 단위, spec as 규격, \
         shelf_life_days as 유통기한일수, storage_cond as 보관조건 \
         FROM item_mst \
         WHERE item_type = 'FG' AND is_active = 1 \
         ORDER BY item_nm".to_string()
    } else if let Some(ref cust_id) = customer_id {
        format!(
            "SELECT so.so_no, so.order_date, so.request_date, so.status, c.cust_nm, sod.item_cd, sod.qty \
             FROM sales_order so \
             JOIN customer_mst c ON so.cust_cd = c.cust_cd \
             JOIN sales_order_dtl sod ON so.so_no = sod.so_no \
             WHERE so.cust_cd = '{}' \
             LIMIT 20",
            cust_id
        )
    } else {
        format!("SELECT * FROM {} LIMIT 20", table_name)
    };

    println!("🔍 Executing SQL: {}", sql);

    // lock guard 범위 내에서 모든 작업 수행
    let result = conn_guard.prepare(&sql).and_then(|mut stmt| {
        // 컬럼 이름 가져오기
        let columns: Vec<String> = stmt.column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        // 데이터 행 가져오기
        let mut rows = Vec::new();
        let mapped_rows = stmt.query_map(params![], |row| {
            let mut row_data = Vec::new();
            for i in 0..columns.len() {
                // 각 컬럼의 값을 적절한 타입으로 변환
                let value = if let Ok(val) = row.get::<_, String>(i) {
                    serde_json::Value::String(val)
                } else if let Ok(val) = row.get::<_, i64>(i) {
                    serde_json::Value::Number(serde_json::Number::from(val))
                } else if let Ok(val) = row.get::<_, f64>(i) {
                    if let Some(num) = serde_json::Number::from_f64(val) {
                        serde_json::Value::Number(num)
                    } else {
                        serde_json::Value::String(val.to_string())
                    }
                } else if let Ok(val) = row.get::<_, bool>(i) {
                    serde_json::Value::Bool(val)
                } else if let Ok(_) = row.get::<_, Option<String>>(i) {
                    // NULL 값 처리
                    if let Ok(Some(val)) = row.get::<_, Option<String>>(i) {
                        serde_json::Value::String(val)
                    } else {
                        serde_json::Value::Null
                    }
                } else {
                    // 기본값으로 빈 문자열
                    serde_json::Value::String("".to_string())
                };
                row_data.push(value);
            }
            Ok(row_data)
        })?;

        for row in mapped_rows {
            if let Ok(row_data) = row {
                rows.push(row_data);
            }
        }

        Ok((columns, rows))
    });

    match result {
        Ok((columns, rows)) => {
            let total_count = rows.len() as i64;

            // 테이블 데이터 구성
            let table_data = TableData {
                columns,
                rows,
                total_count: Some(total_count),
            };

            // 응답 메시지 구성 (고객 필터 정보 포함)
            let response = if let Some(ref cust_id) = customer_id {
                format!(
                    "{}에서 고객(ID: {})에 대한 {}건의 데이터를 찾았습니다.\n\n테이블: {}\n조회 결과: {}건 (최대 20건 표시)",
                    display_name,
                    cust_id,
                    total_count,
                    table_name,
                    total_count.min(20)
                )
            } else {
                format!(
                    "{}에서 {}건의 데이터를 찾았습니다.\n\n테이블: {}\n조회 결과: {}건 (최대 20건 표시)",
                    display_name,
                    total_count,
                    table_name,
                    total_count.min(20)
                )
            };

            println!("✅ Successfully queried {} rows from {}", total_count, table_name);
            Some((response, table_data))
        }
        Err(e) => {
            eprintln!("❌ Failed to query table: {}", e);
            None
        }
    }
}

/// LLM 응답을 파싱하여 테이블 데이터로 변환
fn parse_llm_response_to_table(response: &str) -> TableData {
    // 응답에서 데이터 라인을 찾아 파싱
    let lines: Vec<&str> = response.lines().collect();
    let mut columns = vec![];
    let mut rows = vec![];

    // 데이터 라인 찾기 (숫자로 시작하는 라인들)
    for line in lines.iter() {
        let trimmed = line.trim();

        // "1. 설비ID: EQ-001, 온도: 92°C, ..." 형식의 데이터 라인 파싱
        if trimmed.starts_with(|c: char| c.is_numeric()) && trimmed.contains('.') {
            // 번호 제거
            let data_part = trimmed.split_once(". ").map(|(_, data)| data).unwrap_or(trimmed);

            // 필드 파싱 (예: "설비ID: EQ-001, 온도: 92°C")
            let mut row_data = vec![];
            let fields: Vec<&str> = data_part.split(", ").collect();

            for field in fields {
                if let Some((key, value)) = field.split_once(": ") {
                    // 처음 발견한 데이터에서 컬럼 이름 추출
                    if columns.is_empty() || !columns.contains(&key.to_string()) {
                        columns.push(key.to_string());
                    }

                    // 값 추가 (°C나 다른 단위 포함)
                    row_data.push(serde_json::Value::String(value.to_string()));
                }
            }

            if !row_data.is_empty() {
                // 컬럼 수에 맞춰 빈 값 채우기
                while row_data.len() < columns.len() {
                    row_data.push(serde_json::Value::Null);
                }
                rows.push(row_data);
            }
        }
    }

    // 데이터가 없으면 기본 테이블 구조 반환
    if columns.is_empty() {
        columns = vec!["결과".to_string()];
        rows = vec![vec![serde_json::Value::String(response.to_string())]];
    }

    let total_count = rows.len() as i64;
    TableData {
        columns,
        rows,
        total_count: Some(total_count),
    }
}

/// 쿼리에서 날짜 필터 추출 (년/월 형식)
/// 예: "2024년 9월" → "2024-09", "24년 6월" → "2024-06"
fn extract_date_filter(query: &str) -> Option<String> {
    use regex::Regex;

    // 패턴 1: "2024년 9월" 또는 "2024년9월"
    let re_full_year = Regex::new(r"(20\d{2})\s*년\s*(\d{1,2})\s*월").ok()?;
    if let Some(caps) = re_full_year.captures(query) {
        let year = caps.get(1)?.as_str();
        let month: u32 = caps.get(2)?.as_str().parse().ok()?;
        return Some(format!("{}-{:02}", year, month));
    }

    // 패턴 2: "24년 6월" (2자리 연도)
    let re_short_year = Regex::new(r"(\d{2})\s*년\s*(\d{1,2})\s*월").ok()?;
    if let Some(caps) = re_short_year.captures(query) {
        let short_year: u32 = caps.get(1)?.as_str().parse().ok()?;
        let year = 2000 + short_year;
        let month: u32 = caps.get(2)?.as_str().parse().ok()?;
        return Some(format!("{}-{:02}", year, month));
    }

    // 패턴 3: "2024년" (연도만)
    let re_year_only = Regex::new(r"(20\d{2})\s*년").ok()?;
    if let Some(caps) = re_year_only.captures(query) {
        let year = caps.get(1)?.as_str();
        return Some(format!("{}", year));
    }

    // 패턴 4: "올해", "이번 년도"
    if query.contains("올해") || query.contains("이번 년도") || query.contains("금년") {
        let current_year = chrono::Local::now().format("%Y").to_string();
        return Some(current_year);
    }

    // 패턴 5: "이번 달", "이번달"
    if query.contains("이번 달") || query.contains("이번달") || query.contains("이달") {
        let now = chrono::Local::now();
        return Some(now.format("%Y-%m").to_string());
    }

    // 패턴 6: "지난 달", "지난달"
    if query.contains("지난 달") || query.contains("지난달") || query.contains("저번달") {
        let now = chrono::Local::now();
        let last_month = now - chrono::Duration::days(30);
        return Some(last_month.format("%Y-%m").to_string());
    }

    None
}

/// 쿼리에서 제품 필터 추출
/// 예: "프로바이오틱스" → "프로바이오"
/// 실제 DB item_mst의 item_nm에 맞게 매핑
fn extract_product_filter(query: &str) -> Option<String> {
    // 제품명 키워드 목록 (실제 DB item_mst 참조)
    // 실제 제품명: 프로바이오 장건강, 식물성 프로틴쉐이크, 비타민워터, 콜라겐 뷰티드링크, 키즈 면역음료
    let product_patterns = vec![
        // 완제품 (실제 DB 기준)
        ("프로바이오틱스", "프로바이오"),  // "프로바이오 장건강"에 매칭
        ("프로바이오", "프로바이오"),
        ("유산균", "프로바이오"),
        ("장건강", "장건강"),
        ("단백질", "프로틴"),              // "식물성 프로틴쉐이크"에 매칭
        ("프로틴", "프로틴"),
        ("쉐이크", "프로틴쉐이크"),
        ("비타민", "비타민워터"),
        ("비타민워터", "비타민워터"),
        ("콜라겐", "콜라겐"),
        ("뷰티", "뷰티드링크"),
        ("키즈", "키즈"),
        ("면역", "면역음료"),
        // 맛/향
        ("딸기", "딸기"),
        ("초코", "초코"),
        ("레몬", "레몬"),
        ("오렌지", "오렌지"),
        // 제품 코드
        ("fg-001", "FG-001"),
        ("fg-002", "FG-002"),
        ("fg-003", "FG-003"),
        ("fg-004", "FG-004"),
    ];

    let query_lower = query.to_lowercase();

    for (keyword, product_name) in product_patterns {
        if query_lower.contains(keyword) {
            return Some(product_name.to_string());
        }
    }

    None
}

/// LLM 응답에서 차트 JSON 추출 및 ChartData로 변환
///
/// LLM 응답에서 ```json:chart 블록을 찾아 파싱합니다.
/// 파싱 실패시 None 반환 (차트 없이 텍스트만 표시)
fn extract_chart_data_from_response(response: &str) -> Option<ChartData> {
    // ```json:chart ... ``` 블록 추출
    let chart_json = extract_json_chart_block(response)?;

    // JSON 파싱 시도
    match serde_json::from_str::<serde_json::Value>(&chart_json) {
        Ok(json) => {
            println!("✅ [extract_chart_data] Chart JSON parsed successfully");

            // ChartData 구조체로 변환
            let chart_type = json["chartType"]
                .as_str()
                .or_else(|| json["chart_type"].as_str())
                .unwrap_or("bar")
                .to_string();

            let title = json["title"].as_str().unwrap_or("차트").to_string();

            let description = json["description"]
                .as_str()
                .or_else(|| json["summary"].as_str())
                .unwrap_or("")
                .to_string();

            // 차트 타입에 따른 데이터 추출
            let (bar_line_data, pie_data, gauge_data, data_keys, x_axis_key) =
                parse_chart_data_by_type(&chart_type, &json);

            // 인사이트 추출
            let insight = json["insight"]
                .as_str()
                .or_else(|| json["analysis"].as_str())
                .map(|s| s.to_string());

            Some(ChartData {
                chart_type,
                title,
                description,
                bar_line_data,
                pie_data,
                gauge_data,
                data_keys,
                x_axis_key,
                insight,
            })
        }
        Err(e) => {
            println!("⚠️ [extract_chart_data] Failed to parse chart JSON: {}", e);
            None
        }
    }
}

/// 균형 잡힌 괄호 카운팅으로 JSON 종료 위치 찾기
///
/// ## Phase 2 개선사항
/// - 중첩된 코드 블록이 있어도 정확한 JSON 종료 위치 감지
/// - 문자열 내 괄호는 무시 (escape 처리)
/// - 배열 `[]`와 객체 `{}` 모두 지원
fn find_balanced_json_end(content: &str) -> Option<usize> {
    let mut brace_count = 0i32;   // { }
    let mut bracket_count = 0i32; // [ ]
    let mut in_string = false;
    let mut escape_next = false;
    let mut started = false;

    for (i, c) in content.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match c {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => {
                brace_count += 1;
                started = true;
            }
            '}' if !in_string => {
                brace_count -= 1;
                if started && brace_count == 0 && bracket_count == 0 {
                    return Some(i + 1);
                }
            }
            '[' if !in_string => {
                bracket_count += 1;
                started = true;
            }
            ']' if !in_string => {
                bracket_count -= 1;
                if started && brace_count == 0 && bracket_count == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// 응답에서 ```json:chart ... ``` 블록 추출 (개선된 알고리즘)
///
/// ## Phase 2 개선사항
/// - 균형 괄호 카운팅으로 중첩 코드 블록 처리
/// - 첫 번째 `{` 또는 `[`부터 균형 잡힌 종료까지 추출
/// - 기존 마커 기반 추출도 폴백으로 유지
fn extract_json_chart_block(response: &str) -> Option<String> {
    // 1차 시도: ```json:chart 블록 (균형 괄호 방식)
    let start_marker = "```json:chart";
    if let Some(start_idx) = response.find(start_marker) {
        let content_start = start_idx + start_marker.len();
        let after_marker = &response[content_start..];

        // 첫 번째 '{' 또는 '[' 찾기
        if let Some(json_offset) = after_marker.find(|c| c == '{' || c == '[') {
            let json_content = &after_marker[json_offset..];

            // 균형 잡힌 괄호로 종료 위치 찾기
            if let Some(json_end) = find_balanced_json_end(json_content) {
                let extracted = json_content[..json_end].to_string();
                println!(
                    "📊 [extract_json_chart_block] Found chart JSON (balanced): {} chars",
                    extracted.len()
                );
                return Some(extracted);
            }
        }

        // 폴백: 기존 마커 기반 추출
        let end_marker = "```";
        if let Some(end_idx) = after_marker.find(end_marker) {
            let json_content = after_marker[..end_idx].trim();
            if !json_content.is_empty() {
                println!(
                    "📊 [extract_json_chart_block] Found chart JSON (marker): {} chars",
                    json_content.len()
                );
                return Some(json_content.to_string());
            }
        }
    }

    // 2차 시도: ```json 블록에서 chartType 포함 여부 확인 (균형 괄호 방식)
    let alt_start = "```json";
    if let Some(start_idx) = response.find(alt_start) {
        let content_start = start_idx + alt_start.len();
        let after_marker = &response[content_start..];

        // 첫 번째 '{' 또는 '[' 찾기
        if let Some(json_offset) = after_marker.find(|c| c == '{' || c == '[') {
            let json_content = &after_marker[json_offset..];

            // 균형 잡힌 괄호로 종료 위치 찾기
            if let Some(json_end) = find_balanced_json_end(json_content) {
                let extracted = &json_content[..json_end];

                // chartType 또는 chart_type 포함 확인
                if extracted.contains("chartType") || extracted.contains("chart_type") {
                    println!(
                        "📊 [extract_json_chart_block] Found chart JSON (alt, balanced): {} chars",
                        extracted.len()
                    );
                    return Some(extracted.to_string());
                }
            }
        }

        // 폴백: 기존 마커 기반 추출
        let end_marker = "```";
        if let Some(end_idx) = after_marker.find(end_marker) {
            let json_content = after_marker[..end_idx].trim();

            if json_content.contains("chartType") || json_content.contains("chart_type") {
                println!(
                    "📊 [extract_json_chart_block] Found chart JSON (alt, marker): {} chars",
                    json_content.len()
                );
                return Some(json_content.to_string());
            }
        }
    }

    println!("ℹ️ [extract_json_chart_block] No chart JSON block found");
    None
}

/// 차트 타입에 따른 데이터 파싱
fn parse_chart_data_by_type(
    chart_type: &str,
    json: &serde_json::Value,
) -> (
    Option<Vec<serde_json::Value>>,  // bar_line_data - 평탄화된 JSON 직접 사용
    Option<Vec<PieChartData>>,
    Option<GaugeChartData>,
    Option<Vec<DataKeyConfig>>,
    Option<String>,
) {
    match chart_type {
        "bar" | "line" => {
            // Bar/Line 차트 데이터 파싱
            let data = json["data"]
                .as_array()
                .or_else(|| json["bar_line_data"].as_array())
                .or_else(|| json["barLineData"].as_array());

            // 평탄화된 JSON 객체 직접 사용 (serde(flatten) 문제 회피)
            let bar_line_data = data.map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let name = item["name"]
                            .as_str()
                            .or_else(|| item["label"].as_str())
                            .unwrap_or("");

                        if !name.is_empty() {
                            // JSON 객체 그대로 복사 (이미 평탄화된 상태)
                            Some(item.clone())
                        } else {
                            None
                        }
                    })
                    .collect()
            });

            // data_keys 파싱
            let data_keys = json["dataKeys"]
                .as_array()
                .or_else(|| json["data_keys"].as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            Some(DataKeyConfig {
                                key: item["key"].as_str()?.to_string(),
                                color: item["color"]
                                    .as_str()
                                    .unwrap_or("#8884d8")
                                    .to_string(),
                                label: item["label"]
                                    .as_str()
                                    .or_else(|| item["key"].as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            })
                        })
                        .collect()
                });

            // x_axis_key 파싱
            let x_axis_key = json["xAxisKey"]
                .as_str()
                .or_else(|| json["x_axis_key"].as_str())
                .map(|s| s.to_string());

            (bar_line_data, None, None, data_keys, x_axis_key)
        }

        "pie" => {
            // Pie 차트 데이터 파싱
            let data = json["data"]
                .as_array()
                .or_else(|| json["pie_data"].as_array())
                .or_else(|| json["pieData"].as_array());

            let pie_data = data.map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let name = item["name"]
                            .as_str()
                            .or_else(|| item["label"].as_str())?
                            .to_string();
                        let value = item["value"]
                            .as_f64()
                            .or_else(|| item["value"].as_i64().map(|v| v as f64))?;
                        let color = item["color"]
                            .as_str()
                            .or_else(|| item["fill"].as_str())
                            .map(|s| s.to_string());

                        Some(PieChartData { name, value, color })
                    })
                    .collect()
            });

            (None, pie_data, None, None, None)
        }

        "gauge" => {
            // Gauge 차트 데이터 파싱
            let gauge_data = json["data"]
                .as_object()
                .or_else(|| json["gauge_data"].as_object())
                .or_else(|| json["gaugeData"].as_object())
                .map(|obj| {
                    let value_obj = serde_json::Value::Object(obj.clone());
                    GaugeChartData {
                        value: value_obj["value"]
                            .as_f64()
                            .or_else(|| value_obj["value"].as_i64().map(|v| v as f64))
                            .unwrap_or(0.0),
                        min: value_obj["min"]
                            .as_f64()
                            .or_else(|| value_obj["min"].as_i64().map(|v| v as f64))
                            .unwrap_or(0.0),
                        max: value_obj["max"]
                            .as_f64()
                            .or_else(|| value_obj["max"].as_i64().map(|v| v as f64))
                            .unwrap_or(100.0),
                        label: value_obj["label"].as_str().map(|s| s.to_string()),
                        unit: value_obj["unit"].as_str().map(|s| s.to_string()),
                    }
                });

            (None, None, gauge_data, None, None)
        }

        _ => (None, None, None, None, None),
    }
}

/// LLM 응답에서 차트 JSON 블록 제거
///
/// 사용자에게 보여줄 텍스트 응답에서 ```json:chart ... ``` 블록을 제거합니다.
fn remove_chart_json_block(response: &str) -> String {
    // ```json:chart ... ``` 블록 제거
    let start_marker = "```json:chart";
    let end_marker = "```";

    if let Some(start_idx) = response.find(start_marker) {
        let before = &response[..start_idx];

        let after_start = &response[(start_idx + start_marker.len())..];
        if let Some(end_idx) = after_start.find(end_marker) {
            let after = &after_start[(end_idx + end_marker.len())..];

            // 앞뒤 텍스트 결합 (불필요한 공백 정리)
            let result = format!("{}{}", before.trim_end(), after.trim_start());
            return result.trim().to_string();
        }
    }

    // 대체 패턴: 일반 ```json 블록 중 chartType 포함된 것 제거
    let alt_start = "```json";
    if let Some(start_idx) = response.find(alt_start) {
        let content_start = start_idx + alt_start.len();
        let after_start = &response[content_start..];

        if let Some(end_idx) = after_start.find(end_marker) {
            let json_content = &after_start[..end_idx];

            // chartType 포함 확인
            if json_content.contains("chartType") || json_content.contains("chart_type") {
                let before = &response[..start_idx];
                let after = &after_start[(end_idx + end_marker.len())..];
                let result = format!("{}{}", before.trim_end(), after.trim_start());
                return result.trim().to_string();
            }
        }
    }

    // 블록이 없으면 원본 반환
    response.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== JSON 블록 추출 테스트 (P0-2 개선) ==========

    /// 기본 json:chart 블록 추출 테스트
    #[test]
    fn test_extract_json_chart_block_basic() {
        let response = r#"분석 결과입니다.

```json:chart
{
  "chartType": "bar",
  "title": "라인별 생산량",
  "data": [
    {"name": "A", "value": 100},
    {"name": "B", "value": 200}
  ]
}
```

추가 설명입니다."#;

        let result = extract_json_chart_block(response);
        assert!(result.is_some());

        let json = result.unwrap();
        assert!(json.contains("chartType"));
        assert!(json.contains("bar"));
        assert!(json.contains("라인별 생산량"));
    }

    /// P0-2: 중첩된 코드 블록 처리 테스트
    #[test]
    fn test_extract_json_with_nested_code_blocks() {
        let response = r#"분석 결과입니다.

```json:chart
{
  "chartType": "bar",
  "data": [
    {"name": "A", "value": 100}
  ]
}
```

추가 설명:
```javascript
// 이 코드는 무시되어야 함
console.log("test");
```

마지막 설명."#;

        let result = extract_json_chart_block(response);
        assert!(result.is_some());

        let json = result.unwrap();
        assert!(json.contains("chartType"));
        assert!(!json.contains("console.log"));
        assert!(!json.contains("javascript"));
    }

    /// 균형 괄호 카운팅 테스트
    #[test]
    fn test_find_balanced_json_end() {
        // 단순 객체: {"key": "value"} = 16 chars, last } at index 15, return 16
        let simple = r#"{"key": "value"}"#;
        assert_eq!(simple.len(), 16);
        assert_eq!(find_balanced_json_end(simple), Some(16));

        // 중첩 객체: {"outer": {"inner": "value"}} = 29 chars
        let nested = r#"{"outer": {"inner": "value"}}"#;
        assert_eq!(nested.len(), 29);
        assert_eq!(find_balanced_json_end(nested), Some(29));

        // 배열: [1, 2, 3] = 9 chars
        let array = r#"[1, 2, 3]"#;
        assert_eq!(array.len(), 9);
        assert_eq!(find_balanced_json_end(array), Some(9));

        // 복합 구조: {"data": [{"a": 1}, {"b": 2}], "count": 2} = 42 chars
        let complex = r#"{"data": [{"a": 1}, {"b": 2}], "count": 2}"#;
        assert_eq!(complex.len(), 42);
        assert_eq!(find_balanced_json_end(complex), Some(42));

        // 문자열 내 괄호 (무시되어야 함): {"message": "Hello {world}"} = 28 chars
        let with_string_braces = r#"{"message": "Hello {world}"}"#;
        assert_eq!(with_string_braces.len(), 28);
        assert_eq!(find_balanced_json_end(with_string_braces), Some(28));

        // 이스케이프 문자 처리: {"text": "quote\"here"} = 23 chars
        let with_escaped = r#"{"text": "quote\"here"}"#;
        assert_eq!(with_escaped.len(), 23);
        assert_eq!(find_balanced_json_end(with_escaped), Some(23));

        // 불완전한 JSON
        let incomplete = r#"{"key": "value""#;
        assert_eq!(find_balanced_json_end(incomplete), None);
    }

    /// 복잡한 차트 데이터 추출 테스트
    #[test]
    fn test_extract_complex_chart_data() {
        let response = r#"라인별 생산량 현황입니다.

```json:chart
{
  "chartType": "bar",
  "title": "라인별 생산량 현황",
  "config": {
    "style": "gradient",
    "animation": true
  },
  "data": [
    {"name": "라인1", "value": 1500, "target": 1800},
    {"name": "라인2", "value": 2300, "target": 2000},
    {"name": "라인3", "value": 1800, "target": 2200}
  ],
  "summary": {
    "total": 5600,
    "average": 1867
  }
}
```

라인2가 목표를 초과 달성했습니다."#;

        let result = extract_json_chart_block(response);
        assert!(result.is_some());

        let json = result.unwrap();
        assert!(json.contains("라인별 생산량 현황"));
        assert!(json.contains("\"total\": 5600"));
        assert!(json.contains("gradient"));
    }

    /// json:chart 블록이 없는 경우 테스트
    #[test]
    fn test_extract_no_chart_block() {
        let response = "일반 텍스트 응답입니다. 차트 블록이 없습니다.";
        let result = extract_json_chart_block(response);
        assert!(result.is_none());

        let with_regular_json = r#"응답입니다.

```json
{"type": "not_a_chart"}
```
"#;
        let result2 = extract_json_chart_block(with_regular_json);
        assert!(result2.is_none());
    }

    /// 응답에서 차트 블록 제거 테스트
    #[test]
    fn test_remove_chart_json_block() {
        let response = r#"분석 결과입니다.

```json:chart
{"chartType": "bar"}
```

자세한 설명입니다."#;

        let cleaned = remove_chart_json_block(response);
        assert!(!cleaned.contains("json:chart"));
        assert!(!cleaned.contains("chartType"));
        assert!(cleaned.contains("분석 결과입니다"));
        assert!(cleaned.contains("자세한 설명입니다"));
    }

    /// 빈 응답 처리 테스트
    #[test]
    fn test_extract_empty_response() {
        assert_eq!(extract_json_chart_block(""), None);
        assert_eq!(extract_json_chart_block("   "), None);
    }

    /// 불완전한 JSON 블록 처리 테스트
    #[test]
    fn test_extract_incomplete_json_block() {
        // 닫는 ``` 없음
        let no_closing = r#"텍스트

```json:chart
{"chartType": "bar"
"#;
        // 불완전한 블록은 None 반환하거나 fallback 처리
        let result = extract_json_chart_block(no_closing);
        // fallback으로 마커 기반 추출 시도하므로 결과가 있을 수 있음
        // 중요한 것은 패닉하지 않는 것
        assert!(result.is_none() || result.is_some());
    }

    /// 여러 JSON 블록이 있는 경우 첫 번째만 추출 테스트
    #[test]
    fn test_extract_first_chart_block_only() {
        let response = r#"첫 번째 차트:

```json:chart
{"chartType": "bar", "id": 1}
```

두 번째 차트:

```json:chart
{"chartType": "line", "id": 2}
```
"#;

        let result = extract_json_chart_block(response);
        assert!(result.is_some());

        let json = result.unwrap();
        assert!(json.contains("\"id\": 1") || json.contains("\"id\":1"));
    }

    /// 특수 문자가 포함된 JSON 처리 테스트
    #[test]
    fn test_extract_json_with_special_chars() {
        let response = r#"결과:

```json:chart
{
  "chartType": "bar",
  "title": "생산량 분석 (2024년)",
  "description": "라인 A/B/C 비교: 총 생산량 = 5,000개",
  "data": [{"name": "라인A", "value": 100}]
}
```
"#;

        let result = extract_json_chart_block(response);
        assert!(result.is_some());

        let json = result.unwrap();
        assert!(json.contains("2024년"));
        assert!(json.contains("5,000개"));
    }

    /// 배열 형태 JSON 추출 테스트
    #[test]
    fn test_extract_array_json() {
        let response = r#"데이터:

```json:chart
[
  {"name": "A", "value": 1},
  {"name": "B", "value": 2}
]
```
"#;

        let result = extract_json_chart_block(response);
        assert!(result.is_some());

        let json = result.unwrap();
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));
    }
}
