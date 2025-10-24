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

#[tauri::command]
pub async fn send_chat_message(
    request: ChatMessageRequest,
) -> Result<ChatMessageResponse, String> {
    // Simplified chat handler
    // In real implementation, this would use LLM for intent classification
    // and route to appropriate services

    let session_id = request
        .session_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Simple intent detection
    let intent = if request.message.contains("판단") || request.message.contains("실행") {
        "judgment_execution"
    } else if request.message.contains("워크플로우") || request.message.contains("workflow") {
        "workflow_management"
    } else if request.message.contains("분석") || request.message.contains("대시보드") {
        "data_visualization"
    } else {
        "general_query"
    };

    let response = match intent {
        "judgment_execution" => {
            "판단을 실행하려면 워크플로우를 선택해주세요. 사용 가능한 워크플로우를 보여드릴까요?"
                .to_string()
        }
        "workflow_management" => {
            "워크플로우 관리 메뉴로 안내해드리겠습니다. 새로운 워크플로우를 만들거나 기존 워크플로우를 편집할 수 있습니다."
                .to_string()
        }
        "data_visualization" => {
            "데이터 분석 및 시각화 기능을 제공합니다. 어떤 데이터를 분석하고 싶으신가요?".to_string()
        }
        _ => "어떤 도움이 필요하신가요? 판단 실행, 워크플로우 관리, 데이터 분석 등을 도와드릴 수 있습니다."
            .to_string(),
    };

    Ok(ChatMessageResponse {
        response,
        session_id,
        intent: intent.to_string(),
        action_result: None,
    })
}

#[tauri::command]
pub async fn get_chat_history(_session_id: String) -> Result<Vec<ChatMessage>, String> {
    // Simplified - in real implementation, this would retrieve from database
    Ok(vec![])
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String, // user | assistant
    pub content: String,
    pub created_at: String,
}
