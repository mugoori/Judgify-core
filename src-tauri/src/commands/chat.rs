use crate::services::chat_service::{ChatService, Intent};
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
            // Week 3: 대화형 응답 생성 (하드코딩 제거)
            // 1. 대화 이력 조회 (최근 5개)
            let history = service
                .get_history(&session_id, 5)
                .await
                .unwrap_or_else(|e| {
                    eprintln!("⚠️ Failed to get history for GeneralQuery: {}", e);
                    Vec::new()
                });

            println!("🧠 GeneralQuery detected - using conversational AI");
            println!("   History: {} messages", history.len());

            // 2. Claude API로 대화형 응답 생성
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
                    // Fallback: 간단한 안내 메시지
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
