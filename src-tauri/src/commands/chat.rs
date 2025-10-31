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

    // 4. 의도에 따른 응답 생성 (Week 2에서 서비스 라우팅으로 확장 예정)
    let (response, action_result) = match intent {
        Intent::JudgmentExecution => (
            "판단을 실행하겠습니다. 워크플로우를 선택해주세요.".to_string(),
            None, // Week 2: Judgment Service 호출 결과
        ),
        Intent::WorkflowManagement => (
            "워크플로우 관리 기능입니다. 새로운 워크플로우를 만들거나 기존 워크플로우를 편집할 수 있습니다.".to_string(),
            None, // Week 2: Workflow Service 호출 결과
        ),
        Intent::DataVisualization => (
            "데이터 시각화 기능입니다. 어떤 데이터를 분석하고 싶으신가요?".to_string(),
            None, // Week 2: BI Service 호출 결과
        ),
        Intent::SettingsChange => (
            "설정 변경 기능입니다. 어떤 설정을 변경하시겠습니까?".to_string(),
            None,
        ),
        Intent::GeneralQuery => (
            "Judgify AI 어시스턴트입니다. 판단 실행, 워크플로우 관리, 데이터 분석 등을 도와드릴 수 있습니다.".to_string(),
            None,
        ),
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
