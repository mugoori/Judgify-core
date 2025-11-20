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
                if let Some((response_text, table_data)) = try_query_erp_mes_tables(&request.message).await {
                    println!("✅ ERP/MES table data found!");
                    return Ok(ChatMessageResponse {
                        response: response_text,
                        session_id,
                        intent: format!("{:?}", intent).to_lowercase(),
                        action_result: None,
                        table_data: Some(table_data),
                    });
                }

                // MES 데이터 로그 조회 시도 (CSV 업로드 데이터)
                match query_mes_data_for_chat(&request.message).await {
                    Ok(Some((response_text, table_data))) => {
                        println!("✅ MES data found and formatted");
                        // table_data를 별도로 반환
                        return Ok(ChatMessageResponse {
                            response: response_text,
                            session_id,
                            intent: format!("{:?}", intent).to_lowercase(),
                            action_result: None,
                            table_data: Some(table_data),
                        });
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
        table_data: None, // 일단 None으로 설정, 추후 GeneralQuery에서 채울 예정
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
    let data_keywords = vec![
        // 한글 키워드
        "데이터", "보여줘", "조회", "확인", "찾아", "검색", "알려줘",
        "어떤", "몇개", "몇 개", "목록", "리스트", "표시", "출력",
        "현황", "내역", "결과", "정보", "상태", "이력", "로그",
        // 영어 키워드
        "data", "show", "query", "search", "find", "list", "display",
        // 조건 관련
        "이상", "이하", "초과", "미만", "같은", "동일한", "포함",
        // 특정 필드 언급
        "온도", "습도", "압력", "시간", "날짜", "temperature", "humidity",
        // ERP/MES 관련 키워드
        "생산", "품질", "검사", "재고", "구매", "판매",
        "발주", "입고", "출하", "납품", "고객", "거래처",
        "ccp", "qc", "lot", "배치", "공정", "작업",
        "mes", "erp", "제품", "원료", "자재", "라인",
        "설비", "ph", "brix", "파라미터", "충진", "살균",
    ];

    let lower_message = message.to_lowercase();
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

/// 고객명으로 customer_id 조회하는 헬퍼 함수
fn get_customer_id_by_name(conn: &rusqlite::Connection, customer_name: &str) -> Option<String> {
    let sql = "SELECT customer_id FROM customer_mst WHERE customer_name LIKE ?";
    let pattern = format!("%{}%", customer_name);

    match conn.query_row(sql, &[&pattern], |row| row.get::<_, String>(0)) {
        Ok(id) => {
            println!("✅ Found customer_id: {} for name: {}", id, customer_name);
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
    } else if query_lower.contains("제품") || query_lower.contains("원료") || query_lower.contains("자재") || query_lower.contains("아이템") {
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

    // 5. SQL 쿼리 생성 (고객 필터링 포함)
    let sql = if let Some(ref cust_id) = customer_id {
        format!(
            "SELECT so.so_no, so.so_date, so.due_date, so.status, c.customer_name, sod.item_id, sod.order_qty \
             FROM sales_order so \
             JOIN customer_mst c ON so.customer_id = c.customer_id \
             JOIN sales_order_dtl sod ON so.so_no = sod.so_no \
             WHERE so.customer_id = '{}' \
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
