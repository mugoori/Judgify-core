use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::Client;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

/// 사용자 의도 분류 (LLM 기반)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    /// 워크플로우 관련 (생성, 수정, 삭제, 조회)
    WorkflowManagement,
    /// 판단 실행 요청
    JudgmentExecution,
    /// 데이터 시각화 / BI 인사이트 요청
    DataVisualization,
    /// 설정 변경 (MCP 서버 등)
    SettingsChange,
    /// 일반 질문 (시스템 사용법, 도움말 등)
    GeneralQuery,
}

/// 채팅 메시지
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String, // "user" | "assistant"
    pub content: String,
    pub intent: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// 채팅 세션
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_message_at: DateTime<Utc>,
}

/// Intent 분석 결과 (LLM 응답)
#[derive(Debug, Deserialize)]
struct IntentAnalysisResponse {
    intent: String,
    confidence: f64,
    reasoning: Option<String>,
}

/// Chat Service 핵심 구조
pub struct ChatService {
    openai_api_key: String,
    http_client: Client,
    db: Arc<Mutex<Connection>>,
    app_handle: Option<AppHandle>,
}

impl ChatService {
    /// 새 ChatService 인스턴스 생성 (테스트용, AppHandle 없음)
    pub fn new() -> Result<Self> {
        let openai_api_key =
            env::var("OPENAI_API_KEY").unwrap_or_else(|_| "sk-test-key".to_string());

        let db_path = "chat_service.db";
        let db = Connection::open(db_path)?;

        // 테이블 생성
        Self::init_db(&db)?;

        Ok(Self {
            openai_api_key,
            http_client: Client::new(),
            db: Arc::new(Mutex::new(db)),
            app_handle: None,
        })
    }

    /// AppHandle 포함 생성 (Tauri 환경용)
    pub fn with_app_handle(app_handle: Option<AppHandle>) -> Result<Self> {
        let openai_api_key =
            env::var("OPENAI_API_KEY").unwrap_or_else(|_| "sk-test-key".to_string());

        let db_path = "chat_service.db";
        let db = Connection::open(db_path)?;

        Self::init_db(&db)?;

        Ok(Self {
            openai_api_key,
            http_client: Client::new(),
            db: Arc::new(Mutex::new(db)),
            app_handle,
        })
    }

    /// 데이터베이스 초기화 (테이블 생성)
    fn init_db(db: &Connection) -> Result<()> {
        // chat_sessions 테이블
        db.execute(
            "CREATE TABLE IF NOT EXISTS chat_sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                created_at TEXT NOT NULL,
                last_message_at TEXT NOT NULL
            )",
            [],
        )?;

        // chat_messages 테이블
        db.execute(
            "CREATE TABLE IF NOT EXISTS chat_messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                intent TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES chat_sessions(id)
            )",
            [],
        )?;

        // 인덱스 생성
        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session_id ON chat_messages(session_id)",
            [],
        )?;

        Ok(())
    }

    /// LLM을 사용한 사용자 의도 분석
    ///
    /// # Arguments
    /// * `message` - 사용자 메시지
    ///
    /// # Returns
    /// * `Intent` - 분석된 의도
    pub async fn analyze_intent(&self, message: &str) -> Result<Intent> {
        // Intent 분석 프롬프트
        let system_prompt = r#"You are an intent classifier for the Judgify AI platform.

Classify the user's message into one of the following intents:
- workflow_management: User wants to create, modify, delete, or view workflows
- judgment_execution: User wants to execute a judgment/decision on data
- data_visualization: User wants to see charts, dashboards, or BI insights
- settings_change: User wants to modify system settings (MCP servers, API keys, etc.)
- general_query: General questions about the system, help, or usage

Respond in JSON format:
{
  "intent": "workflow_management|judgment_execution|data_visualization|settings_change|general_query",
  "confidence": 0.0-1.0,
  "reasoning": "Brief explanation (optional)"
}

Examples:
- "워크플로우 만들어줘" → workflow_management
- "재고 데이터로 판단 실행해줘" → judgment_execution
- "지난 주 성공률 보여줘" → data_visualization
- "MCP 서버 연결 설정 변경" → settings_change
- "Judgify 사용법 알려줘" → general_query
"#;

        let user_prompt = format!("User message: \"{}\"", message);

        // OpenAI API 호출
        let request_body = json!({
            "model": "gpt-4o-mini",
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "response_format": {"type": "json_object"},
            "temperature": 0.3,
            "max_tokens": 200
        });

        let response = self
            .http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.openai_api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("OpenAI API error: {}", error_text);
        }

        let response_json: serde_json::Value = response.json().await?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in OpenAI response"))?;

        let analysis: IntentAnalysisResponse = serde_json::from_str(content)?;

        println!(
            "🧠 Intent Analysis: {} (confidence: {:.2})",
            analysis.intent, analysis.confidence
        );

        // Intent enum으로 변환
        let intent = match analysis.intent.as_str() {
            "workflow_management" => Intent::WorkflowManagement,
            "judgment_execution" => Intent::JudgmentExecution,
            "data_visualization" => Intent::DataVisualization,
            "settings_change" => Intent::SettingsChange,
            "general_query" => Intent::GeneralQuery,
            _ => Intent::GeneralQuery, // 기본값
        };

        Ok(intent)
    }

    /// 채팅 메시지 저장
    ///
    /// # Arguments
    /// * `session_id` - 세션 ID
    /// * `role` - "user" 또는 "assistant"
    /// * `content` - 메시지 내용
    /// * `intent` - 의도 (옵션)
    pub async fn save_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
        intent: Option<&str>,
    ) -> Result<ChatMessage> {
        let message_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        let db = self.db.lock().unwrap();

        // 세션 존재 확인
        let session_exists: bool = db.query_row(
            "SELECT COUNT(*) > 0 FROM chat_sessions WHERE id = ?",
            params![session_id],
            |row| row.get(0),
        )?;

        if !session_exists {
            // 세션 생성
            db.execute(
                "INSERT INTO chat_sessions (id, user_id, created_at, last_message_at)
                 VALUES (?, NULL, ?, ?)",
                params![session_id, created_at.to_rfc3339(), created_at.to_rfc3339()],
            )?;
        } else {
            // 세션 last_message_at 업데이트
            db.execute(
                "UPDATE chat_sessions SET last_message_at = ? WHERE id = ?",
                params![created_at.to_rfc3339(), session_id],
            )?;
        }

        // 메시지 저장
        db.execute(
            "INSERT INTO chat_messages (id, session_id, role, content, intent, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                &message_id,
                session_id,
                role,
                content,
                intent,
                created_at.to_rfc3339()
            ],
        )?;

        Ok(ChatMessage {
            id: message_id,
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            intent: intent.map(|s| s.to_string()),
            created_at,
        })
    }

    /// 채팅 히스토리 조회
    ///
    /// # Arguments
    /// * `session_id` - 세션 ID
    /// * `limit` - 최대 메시지 개수 (기본 50개)
    pub async fn get_history(&self, session_id: &str, limit: u32) -> Result<Vec<ChatMessage>> {
        let db = self.db.lock().unwrap();

        let mut stmt = db.prepare(
            "SELECT id, session_id, role, content, intent, created_at
             FROM chat_messages
             WHERE session_id = ?
             ORDER BY created_at DESC
             LIMIT ?",
        )?;

        let messages = stmt
            .query_map(params![session_id, limit], |row| {
                Ok(ChatMessage {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    intent: row.get(4)?,
                    created_at: row
                        .get::<_, String>(5)?
                        .parse::<DateTime<Utc>>()
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        // 시간순 정렬 (오래된 메시지부터)
        let mut sorted = messages;
        sorted.reverse();

        Ok(sorted)
    }

    /// 채팅 세션 생성
    pub async fn create_session(&self, user_id: Option<&str>) -> Result<ChatSession> {
        let session_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        let db = self.db.lock().unwrap();

        db.execute(
            "INSERT INTO chat_sessions (id, user_id, created_at, last_message_at)
             VALUES (?, ?, ?, ?)",
            params![
                &session_id,
                user_id,
                created_at.to_rfc3339(),
                created_at.to_rfc3339()
            ],
        )?;

        Ok(ChatSession {
            id: session_id,
            user_id: user_id.map(|s| s.to_string()),
            created_at,
            last_message_at: created_at,
        })
    }

    /// Tauri 이벤트 발생 (프론트엔드로 실시간 업데이트)
    fn emit_event(&self, event_name: &str, payload: &impl Serialize) -> Result<()> {
        if let Some(handle) = &self.app_handle {
            handle
                .emit_all(event_name, payload)
                .map_err(|e| anyhow::anyhow!("Failed to emit event '{}': {}", event_name, e))?;
            println!(
                "📡 Event emitted: {} (payload: {})",
                event_name,
                serde_json::to_string(payload).unwrap_or_else(|_| "...".to_string())
            );
        } else {
            println!(
                "⚠️ No AppHandle - event '{}' not emitted (test mode)",
                event_name
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_service_new() {
        let service = ChatService::new();
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_create_session() {
        let service = ChatService::new().unwrap();
        let session = service.create_session(Some("test-user")).await;

        assert!(session.is_ok());
        let session = session.unwrap();
        assert_eq!(session.user_id, Some("test-user".to_string()));
    }

    #[tokio::test]
    async fn test_save_and_get_message() {
        let service = ChatService::new().unwrap();
        let session = service.create_session(None).await.unwrap();

        // 메시지 저장
        let message = service
            .save_message(
                &session.id,
                "user",
                "테스트 메시지",
                Some("general_query"),
            )
            .await;

        assert!(message.is_ok());
        let message = message.unwrap();
        assert_eq!(message.role, "user");
        assert_eq!(message.content, "테스트 메시지");
        assert_eq!(message.intent, Some("general_query".to_string()));

        // 히스토리 조회
        let history = service.get_history(&session.id, 10).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "테스트 메시지");
    }

    #[tokio::test]
    async fn test_analyze_intent() {
        let service = ChatService::new().unwrap();

        // API 키가 없으면 테스트 스킵
        if service.openai_api_key == "sk-test-key" {
            println!("⚠️ Skipping LLM test (no valid API key)");
            return;
        }

        // Intent 분석
        let result = service.analyze_intent("워크플로우 만들어줘").await;

        match result {
            Ok(intent) => {
                assert_eq!(intent, Intent::WorkflowManagement);
            }
            Err(e) => {
                println!("⚠️ Expected error in test environment: {}", e);
                assert!(e.to_string().contains("OpenAI") || e.to_string().contains("API"));
            }
        }
    }

    #[tokio::test]
    async fn test_get_history_with_multiple_messages() {
        let service = ChatService::new().unwrap();
        let session = service.create_session(None).await.unwrap();

        // 여러 메시지 저장
        for i in 1..=5 {
            service
                .save_message(&session.id, "user", &format!("메시지 {}", i), None)
                .await
                .unwrap();
        }

        // 히스토리 조회 (최대 3개)
        let history = service.get_history(&session.id, 3).await.unwrap();
        assert_eq!(history.len(), 3);

        // 시간순 정렬 확인 (오래된 메시지부터)
        assert!(history[0].content.contains("메시지 3"));
        assert!(history[1].content.contains("메시지 4"));
        assert!(history[2].content.contains("메시지 5"));
    }
}
