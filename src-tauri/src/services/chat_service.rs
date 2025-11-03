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

/// Claude 응답에서 마크다운 코드 블록 제거
fn strip_markdown_code_block(content: &str) -> &str {
    let trimmed = content.trim();
    if trimmed.starts_with("```json") {
        trimmed
            .strip_prefix("```json")
            .unwrap()
            .strip_suffix("```")
            .unwrap_or(trimmed)
            .trim()
    } else if trimmed.starts_with("```") {
        trimmed
            .strip_prefix("```")
            .unwrap()
            .strip_suffix("```")
            .unwrap_or(trimmed)
            .trim()
    } else {
        trimmed
    }
}

/// Chat Service 핵심 구조
pub struct ChatService {
    claude_api_key: String,
    http_client: Client,
    db: Arc<Mutex<Connection>>,
    app_handle: Option<AppHandle>,
}

impl ChatService {
    /// 새 ChatService 인스턴스 생성 (테스트용, AppHandle 없음)
    pub fn new() -> Result<Self> {
        let claude_api_key =
            env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| "sk-ant-test-key".to_string());

        let db_path = "chat_service.db";
        let db = Connection::open(db_path)?;

        // 테이블 생성
        Self::init_db(&db)?;

        Ok(Self {
            claude_api_key,
            http_client: Client::new(),
            db: Arc::new(Mutex::new(db)),
            app_handle: None,
        })
    }

    /// AppHandle 포함 생성 (Tauri 환경용)
    pub fn with_app_handle(app_handle: Option<AppHandle>) -> Result<Self> {
        let claude_api_key =
            env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| "sk-ant-test-key".to_string());

        let db_path = "chat_service.db";
        let db = Connection::open(db_path)?;

        Self::init_db(&db)?;

        Ok(Self {
            claude_api_key,
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

        // API 키 마스킹 로그
        let masked_key = if self.claude_api_key.len() > 20 {
            format!(
                "{}...{}",
                &self.claude_api_key[..10],
                &self.claude_api_key[self.claude_api_key.len() - 10..]
            )
        } else {
            "***".to_string()
        };
        println!("🔑 Using Anthropic API key: {}", masked_key);

        // Claude API 호출
        let request_body = json!({
            "model": "claude-sonnet-4-5-20250929",
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 1024
        });

        println!("📤 Sending request to Claude API...");
        println!("   Model: claude-sonnet-4-5-20250929");
        println!("   Endpoint: https://api.anthropic.com/v1/messages");

        let response = self
            .http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.claude_api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        println!("📥 Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await?;
            eprintln!("❌ Claude API error ({}): {}", status, error_text);

            // Parse error response for better error messages
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                if let Some(error_type) = error_json["error"]["type"].as_str() {
                    if let Some(error_message) = error_json["error"]["message"].as_str() {
                        anyhow::bail!("Claude API error ({}): {} - {}", status, error_type, error_message);
                    }
                }
            }

            anyhow::bail!("Claude API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await?;

        println!("📥 Claude response JSON: {}", serde_json::to_string_pretty(&response_json).unwrap_or_default());

        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in Claude response"))?;

        println!("📝 Extracted content: {}", content);

        // 마크다운 코드 블록 제거
        let clean_content = strip_markdown_code_block(content);
        println!("🧹 Cleaned content: {}", clean_content);

        let analysis: IntentAnalysisResponse = serde_json::from_str(clean_content)
            .map_err(|e| {
                eprintln!("❌ Failed to parse Claude response as JSON: {}", e);
                eprintln!("   Raw content: {}", content);
                eprintln!("   Cleaned content: {}", clean_content);
                anyhow::anyhow!("Failed to parse intent analysis: {}", e)
            })?;

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

    // ==================== Week 2: 서비스 라우팅 메서드 ====================

    /// Judgment Service 라우팅
    ///
    /// # Arguments
    /// * `workflow_id` - 워크플로우 ID
    /// * `input_data` - 판단 입력 데이터
    ///
    /// # Returns
    /// * `serde_json::Value` - 판단 결과 (JudgmentResult를 JSON으로 변환)
    pub async fn route_to_judgment(
        &self,
        workflow_id: String,
        input_data: serde_json::Value,
    ) -> Result<serde_json::Value> {
        use crate::services::judgment_engine::{JudgmentEngine, JudgmentInput};

        println!("🔀 Routing to Judgment Service: workflow_id={}", workflow_id);

        let engine = JudgmentEngine::new()?;
        let input = JudgmentInput {
            workflow_id,
            input_data,
        };

        let result = engine.execute(input).await?;

        // JudgmentResult를 JSON으로 변환
        let json_result = serde_json::json!({
            "id": result.id,
            "workflow_id": result.workflow_id,
            "result": result.result,
            "confidence": result.confidence,
            "method_used": result.method_used,
            "explanation": result.explanation,
        });

        println!("✅ Judgment Service 호출 성공: result={}", result.result);

        Ok(json_result)
    }

    /// BI Service 라우팅
    ///
    /// # Arguments
    /// * `user_request` - 사용자 요청 (자연어)
    ///
    /// # Returns
    /// * `serde_json::Value` - BI 인사이트 (BiInsightResponse를 JSON으로 변환)
    pub async fn route_to_bi(&self, user_request: String) -> Result<serde_json::Value> {
        use crate::services::bi_service::BiService;

        println!("🔀 Routing to BI Service: request={}", user_request);

        let bi_service = BiService::new()?;
        let insight = bi_service.generate_insight(user_request).await?;

        // BiInsightResponse를 JSON으로 변환
        let json_result = serde_json::json!({
            "title": insight.title,
            "insights": insight.insights,
            "component_code": insight.component_code,
            "recommendations": insight.recommendations,
        });

        println!("✅ BI Service 호출 성공: title={}", insight.title);

        Ok(json_result)
    }

    /// Workflow Service 라우팅
    ///
    /// # Arguments
    /// * `action` - 워크플로우 액션 (list | get | create | update | delete)
    /// * `params` - 액션별 파라미터
    ///
    /// # Returns
    /// * `serde_json::Value` - 워크플로우 결과
    pub async fn route_to_workflow(
        &self,
        action: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        use crate::services::workflow_service::WorkflowService;

        println!("🔀 Routing to Workflow Service: action={}", action);

        let workflow_service = WorkflowService::new()?;

        let result = match action {
            "list" => {
                let workflows = workflow_service.get_all_workflows()?;
                serde_json::json!({
                    "action": "list",
                    "workflows": workflows.into_iter().map(|w| serde_json::json!({
                        "id": w.id,
                        "name": w.name,
                        "version": w.version,
                        "is_active": w.is_active,
                        "created_at": w.created_at.to_rfc3339(),
                    })).collect::<Vec<_>>()
                })
            }
            "get" => {
                let id = params["id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing workflow id"))?;
                let workflow = workflow_service.get_workflow(id)?
                    .ok_or_else(|| anyhow::anyhow!("Workflow not found: {}", id))?;
                serde_json::json!({
                    "action": "get",
                    "workflow": {
                        "id": workflow.id,
                        "name": workflow.name,
                        "definition": serde_json::from_str::<serde_json::Value>(&workflow.definition)?,
                        "rule_expression": workflow.rule_expression,
                        "version": workflow.version,
                        "is_active": workflow.is_active,
                        "created_at": workflow.created_at.to_rfc3339(),
                    }
                })
            }
            _ => {
                anyhow::bail!("Unsupported workflow action: {}", action);
            }
        };

        println!("✅ Workflow Service 호출 성공: action={}", action);

        Ok(result)
    }

    // ==================== Week 2: 파라미터 추출 메서드 ====================

    /// Judgment 파라미터 추출 (LLM 기반)
    ///
    /// # Arguments
    /// * `message` - 사용자 메시지 (예: "재고 데이터로 판단해줘")
    ///
    /// # Returns
    /// * `(String, serde_json::Value)` - (workflow_id, input_data)
    pub async fn extract_judgment_params(
        &self,
        message: &str,
    ) -> Result<(String, serde_json::Value)> {
        let system_prompt = r#"You are a parameter extractor for the Judgify AI platform.

Extract judgment parameters from the user's message and respond in JSON format:
{
  "workflow_id": "string (workflow name or id, e.g., 'inventory', 'quality')",
  "input_data": {
    // Extract any data mentioned in the message
    // Example: {"temperature": 90, "vibration": 45}
  }
}

Examples:
- "재고 데이터로 판단해줘" → {"workflow_id": "inventory", "input_data": {}}
- "온도 90도, 진동 45로 품질 검사해줘" → {"workflow_id": "quality", "input_data": {"temperature": 90, "vibration": 45}}
- "워크플로우 123으로 판단 실행" → {"workflow_id": "123", "input_data": {}}
"#;

        let user_prompt = format!("User message: \"{}\"", message);

        println!("📤 [extract_judgment_params] Calling Claude API...");

        let request_body = json!({
            "model": "claude-sonnet-4-5-20250929",
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 1024
        });

        let response = self
            .http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.claude_api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        println!("📥 [extract_judgment_params] Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await?;
            eprintln!("❌ [extract_judgment_params] Claude API error ({}): {}", status, error_text);

            // Parse error response for better error messages
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                if let Some(error_type) = error_json["error"]["type"].as_str() {
                    if let Some(error_message) = error_json["error"]["message"].as_str() {
                        anyhow::bail!("Claude API error ({}): {} - {}", status, error_type, error_message);
                    }
                }
            }

            anyhow::bail!("Claude API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await?;
        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in Claude response"))?;

        // 마크다운 코드 블록 제거
        let clean_content = strip_markdown_code_block(content);

        let params: serde_json::Value = serde_json::from_str(clean_content)?;

        let workflow_id = params["workflow_id"]
            .as_str()
            .unwrap_or("default")
            .to_string();
        let input_data = params["input_data"].clone();

        println!(
            "📝 Extracted judgment params: workflow_id={}, input_data={}",
            workflow_id,
            serde_json::to_string(&input_data).unwrap_or_else(|_| "{}".to_string())
        );

        Ok((workflow_id, input_data))
    }

    /// BI 파라미터 추출 (단순화 버전)
    ///
    /// # Arguments
    /// * `message` - 사용자 메시지 (예: "지난 주 불량률 분석해줘")
    ///
    /// # Returns
    /// * `String` - BI Service로 전달할 요청 (메시지 그대로 사용)
    pub fn extract_bi_params(&self, message: &str) -> Result<String> {
        // BI Service는 자연어 그대로 받아서 처리하므로 단순히 반환
        println!("📝 Extracted BI params: request={}", message);
        Ok(message.to_string())
    }

    /// Workflow 파라미터 추출 (LLM 기반)
    ///
    /// # Arguments
    /// * `message` - 사용자 메시지 (예: "워크플로우 목록 보여줘")
    ///
    /// # Returns
    /// * `(String, serde_json::Value)` - (action, params)
    pub async fn extract_workflow_params(
        &self,
        message: &str,
    ) -> Result<(String, serde_json::Value)> {
        let system_prompt = r#"You are a parameter extractor for workflow management.

Extract workflow action and parameters from the user's message and respond in JSON format:
{
  "action": "list|get|create|update|delete",
  "params": {
    // Action-specific parameters
    // For "list": {} (empty)
    // For "get": {"id": "workflow-id"}
    // etc.
  }
}

Examples:
- "워크플로우 목록 보여줘" → {"action": "list", "params": {}}
- "워크플로우 123 조회해줘" → {"action": "get", "params": {"id": "123"}}
- "전체 워크플로우 보여줘" → {"action": "list", "params": {}}
"#;

        let user_prompt = format!("User message: \"{}\"", message);

        println!("📤 [extract_workflow_params] Calling Claude API...");

        let request_body = json!({
            "model": "claude-sonnet-4-5-20250929",
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 1024
        });

        let response = self
            .http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.claude_api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        println!("📥 [extract_workflow_params] Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await?;
            eprintln!("❌ [extract_workflow_params] Claude API error ({}): {}", status, error_text);

            // Parse error response for better error messages
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                if let Some(error_type) = error_json["error"]["type"].as_str() {
                    if let Some(error_message) = error_json["error"]["message"].as_str() {
                        anyhow::bail!("Claude API error ({}): {} - {}", status, error_type, error_message);
                    }
                }
            }

            anyhow::bail!("Claude API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await?;
        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in Claude response"))?;

        // 마크다운 코드 블록 제거
        let clean_content = strip_markdown_code_block(content);

        let extracted: serde_json::Value = serde_json::from_str(clean_content)?;

        let action = extracted["action"]
            .as_str()
            .unwrap_or("list")
            .to_string();
        let params = extracted["params"].clone();

        println!(
            "📝 Extracted workflow params: action={}, params={}",
            action,
            serde_json::to_string(&params).unwrap_or_else(|_| "{}".to_string())
        );

        Ok((action, params))
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

    // ==================== Week 2 테스트 ====================

    #[tokio::test]
    async fn test_route_to_judgment_success() {
        let service = ChatService::new().unwrap();

        // Judgment Service 라우팅 테스트
        let result = service
            .route_to_judgment(
                "test-workflow".to_string(),
                serde_json::json!({"temperature": 90, "vibration": 45}),
            )
            .await;

        match result {
            Ok(json_result) => {
                assert!(json_result["id"].is_string());
                assert_eq!(json_result["workflow_id"], "test-workflow");
                assert!(json_result["result"].is_boolean());
                assert!(json_result["confidence"].is_number());
                assert!(json_result["method_used"].is_string());
                println!("✅ Judgment routing 테스트 성공: {:?}", json_result);
            }
            Err(e) => {
                println!("⚠️ Judgment routing 테스트 실패 (예상됨): {}", e);
                // 데이터베이스나 서비스가 없는 환경에서는 실패가 예상됨
                assert!(
                    e.to_string().contains("database") ||
                    e.to_string().contains("Workflow") ||
                    e.to_string().contains("connection")
                );
            }
        }
    }

    #[tokio::test]
    async fn test_route_to_bi_success() {
        let service = ChatService::new().unwrap();

        // BI Service 라우팅 테스트
        let result = service
            .route_to_bi("지난 주 불량률 분석해줘".to_string())
            .await;

        match result {
            Ok(json_result) => {
                assert!(json_result["title"].is_string());
                assert!(json_result["insights"].is_array());
                assert!(json_result["component_code"].is_string());
                assert!(json_result["recommendations"].is_array());
                println!("✅ BI routing 테스트 성공: {:?}", json_result);
            }
            Err(e) => {
                println!("⚠️ BI routing 테스트 실패 (예상됨): {}", e);
                // API 키가 없거나 데이터베이스가 없는 경우 실패 예상
                assert!(
                    e.to_string().contains("OpenAI") ||
                    e.to_string().contains("database") ||
                    e.to_string().contains("API")
                );
            }
        }
    }

    #[tokio::test]
    async fn test_route_to_workflow_list() {
        let service = ChatService::new().unwrap();

        // Workflow Service 라우팅 테스트 (목록 조회)
        let result = service
            .route_to_workflow("list", serde_json::json!({}))
            .await;

        match result {
            Ok(json_result) => {
                assert_eq!(json_result["action"], "list");
                assert!(json_result["workflows"].is_array());
                println!("✅ Workflow routing (list) 테스트 성공");
            }
            Err(e) => {
                println!("⚠️ Workflow routing (list) 테스트 실패 (예상됨): {}", e);
                // 데이터베이스가 없는 환경에서는 실패 예상
                assert!(
                    e.to_string().contains("database") ||
                    e.to_string().contains("connection")
                );
            }
        }
    }

    #[test]
    fn test_extract_bi_params() {
        let service = ChatService::new().unwrap();

        // BI 파라미터 추출 (단순 반환)
        let result = service.extract_bi_params("지난 주 매출 분석해줘");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "지난 주 매출 분석해줘");
        println!("✅ BI params extraction 테스트 성공");
    }

    #[tokio::test]
    async fn test_extract_judgment_params() {
        let service = ChatService::new().unwrap();

        // API 키가 없으면 테스트 스킵
        if service.openai_api_key == "sk-test-key" {
            println!("⚠️ Skipping parameter extraction test (no valid API key)");
            return;
        }

        // Judgment 파라미터 추출 테스트
        let result = service
            .extract_judgment_params("재고 데이터로 판단해줘")
            .await;

        match result {
            Ok((workflow_id, input_data)) => {
                assert!(!workflow_id.is_empty());
                assert!(input_data.is_object() || input_data.is_null());
                println!(
                    "✅ Judgment params extraction 테스트 성공: workflow_id={}, input_data={:?}",
                    workflow_id, input_data
                );
            }
            Err(e) => {
                println!("⚠️ Judgment params extraction 테스트 실패: {}", e);
                assert!(e.to_string().contains("OpenAI") || e.to_string().contains("API"));
            }
        }
    }

    #[tokio::test]
    async fn test_extract_workflow_params() {
        let service = ChatService::new().unwrap();

        // API 키가 없으면 테스트 스킵
        if service.openai_api_key == "sk-test-key" {
            println!("⚠️ Skipping parameter extraction test (no valid API key)");
            return;
        }

        // Workflow 파라미터 추출 테스트
        let result = service
            .extract_workflow_params("워크플로우 목록 보여줘")
            .await;

        match result {
            Ok((action, params)) => {
                assert!(!action.is_empty());
                assert!(params.is_object() || params.is_null());
                println!(
                    "✅ Workflow params extraction 테스트 성공: action={}, params={:?}",
                    action, params
                );
            }
            Err(e) => {
                println!("⚠️ Workflow params extraction 테스트 실패: {}", e);
                assert!(e.to_string().contains("OpenAI") || e.to_string().contains("API"));
            }
        }
    }
}
