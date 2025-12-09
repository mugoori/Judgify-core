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
use crate::utils::security::{sanitize_for_xml, detect_injection_attempt};
use crate::services::cache_service::{CacheService, ChatMessage as CachedMessage};
use crate::services::prompt_router::PromptRouter;

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
    /// 차트/그래프 분석 요청 (확장 프롬프트 사용)
    ChartAnalysis,
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

/// 예측 결과 구조체 (수요/재고 예측용)
#[derive(Debug, Clone)]
struct ForecastResult {
    forecast_type: String,     // "demand" | "inventory"
    item_id: Option<String>,   // 특정 품목 (None이면 전체)
    item_name: Option<String>, // 품목명
    forecast_period: String,   // "next_month" | "next_quarter"
    // 통계 계산 결과
    recent_avg: f64,           // 최근 3개월 평균
    moving_avg_6m: f64,        // 6개월 이동평균
    last_year_same_month: f64, // 전년 동월
    growth_rate: f64,          // 성장률 (%)
    forecast_qty: f64,         // 예측 수량
    safety_stock: f64,         // 안전재고 (재고예측시)
    current_stock: f64,        // 현재재고 (재고예측시)
    // 월별 추세 데이터
    monthly_trend: Vec<(String, f64)>, // (월, 수량)
}

/// 숫자를 천 단위 구분자와 함께 포맷
fn format_number(n: f64) -> String {
    let abs_n = n.abs();
    let formatted = if abs_n >= 100_000_000.0 {
        format!("{:.1}억", n / 100_000_000.0)
    } else if abs_n >= 10_000.0 {
        format!("{:.1}만", n / 10_000.0)
    } else {
        // 천 단위 구분자 추가
        let rounded = n.round() as i64;
        let s = rounded.to_string();
        let chars: Vec<char> = s.chars().collect();
        let mut result = String::new();
        let start = if rounded < 0 { 1 } else { 0 };
        let len = chars.len() - start;
        for (i, c) in chars[start..].iter().enumerate() {
            if i > 0 && (len - i) % 3 == 0 {
                result.push(',');
            }
            result.push(*c);
        }
        if rounded < 0 {
            result = format!("-{}", result);
        }
        result
    };
    formatted
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
    cache: CacheService, // ✅ Memory-First Hybrid Cache 추가
}

/// RAG 검색 결과 구조체
#[derive(Debug, Clone)]
struct RagSearchResult {
    title: String,
    content: String,
    category: String,
    #[allow(dead_code)]
    tags: Option<String>,
}

/// ERP 데이터 조회 결과 구조체
#[derive(Debug, Clone)]
struct ErpQueryResult {
    query_type: String,  // "sales", "purchase", "inventory", "production"
    summary: String,     // 요약 텍스트
    data: serde_json::Value,  // 상세 데이터 (JSON)
}

impl ChatService {
    /// 새 ChatService 인스턴스 생성 (테스트용, AppHandle 없음)
    pub fn new() -> Result<Self> {
        // 🔧 Phase 1 Security Fix: keychain fallback 안전장치
        let claude_api_key = env::var("ANTHROPIC_API_KEY")
            .or_else(|_| {
                eprintln!("⚠️  ANTHROPIC_API_KEY not found in env, retrying from keychain...");
                keyring::Entry::new("Judgify", "claude_api_key")
                    .and_then(|e| e.get_password())
                    .map_err(|e| anyhow::anyhow!("Keychain 로드 실패: {}", e))
            })
            .map_err(|_| anyhow::anyhow!("Claude API 키가 설정되지 않았습니다. Settings 페이지에서 API 키를 설정해주세요."))?;

        // API 키 로그 (마스킹)
        let masked = if claude_api_key.len() > 20 {
            format!("{}...{}", &claude_api_key[..10], &claude_api_key[claude_api_key.len()-10..])
        } else {
            "***".to_string()
        };
        eprintln!("✅ ChatService initialized with API key: {}", masked);

        let db_path = "chat_service.db";
        let db = Connection::open(db_path)?;

        // 테이블 생성
        Self::init_db(&db)?;

        Ok(Self {
            claude_api_key,
            http_client: Client::new(),
            db: Arc::new(Mutex::new(db)),
            app_handle: None,
            cache: CacheService::new(5, 20), // ✅ 5 세션, 20 메시지
        })
    }

    /// AppHandle 포함 생성 (Tauri 환경용)
    pub fn with_app_handle(app_handle: Option<AppHandle>) -> Result<Self> {
        // 🔧 Phase 1 Security Fix: keychain fallback 안전장치
        let claude_api_key = env::var("ANTHROPIC_API_KEY")
            .or_else(|_| {
                eprintln!("⚠️  ANTHROPIC_API_KEY not found in env, retrying from keychain...");
                keyring::Entry::new("Judgify", "claude_api_key")
                    .and_then(|e| e.get_password())
                    .map_err(|e| anyhow::anyhow!("Keychain 로드 실패: {}", e))
            })
            .map_err(|_| anyhow::anyhow!("Claude API 키가 설정되지 않았습니다. Settings 페이지에서 API 키를 설정해주세요."))?;

        // API 키 로그 (마스킹)
        let masked = if claude_api_key.len() > 20 {
            format!("{}...{}", &claude_api_key[..10], &claude_api_key[claude_api_key.len()-10..])
        } else {
            "***".to_string()
        };
        eprintln!("✅ ChatService (with AppHandle) initialized with API key: {}", masked);

        let db_path = "chat_service.db";
        let db = Connection::open(db_path)?;

        Self::init_db(&db)?;

        Ok(Self {
            claude_api_key,
            http_client: Client::new(),
            db: Arc::new(Mutex::new(db)),
            app_handle,
            cache: CacheService::new(5, 20), // ✅ 5 세션, 20 메시지
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
        let system_prompt = r#"You are an intent classifier for the TriFlow AI platform.

Classify the user's message into one of the following intents:
- workflow_management: User wants to create, modify, delete, or view workflows
- judgment_execution: User wants to execute a judgment/decision on data
- chart_analysis: User explicitly wants to SEE CHARTS/GRAPHS with specific operational data (라인별 생산량, 월별 매출, 가동률 게이지, CCP 합격률, 창고별 재고 등)
- data_visualization: User wants general BI insights or trend analysis (without specific chart type request)
- settings_change: User wants to modify system settings (MCP servers, API keys, etc.)
- general_query: General questions, data queries, help, or usage

IMPORTANT:
- If the user is asking to "see data", "show data", "데이터 보여줘", "데이터 조회" → classify as general_query
- chart_analysis vs data_visualization:
  * chart_analysis: User requests SPECIFIC CHART TYPES with keywords like: 라인별, 월별, 설비별, 창고별, 품목별, 공급업체별, 교대별, 작업자별, CCP, 가동률, OEE, 합격률, 불량률, 온도, 재고, 생산량, 매출, 비가동 - EVEN IF combined with "분석", "현황", "추이", "트렌드"
  * data_visualization: ONLY for general analysis/insights request WITHOUT any specific category keywords (e.g., "전반적인 현황", "종합 분석", "전체 트렌드")
- Raw data queries should be general_query, not data_visualization or chart_analysis
- KEY RULE: If the query contains ANY specific category keyword (라인별, 월별, 설비별, 창고별, 품목별, CCP, 가동률, OEE, 온도, 재고, 생산량, 매출 등) → ALWAYS classify as chart_analysis, regardless of whether "분석" or "현황" is also present
- Questions about the company itself (회사, 기업, 조직, 퓨어웰, 우리 회사, 회사 소개, 회사 정보) → ALWAYS classify as general_query (these are company information queries, NOT data analysis)
- Questions about company strategy, DX, digital transformation, business planning → classify as general_query (these need company knowledge, not chart analysis)
- Questions asking for EXPLANATIONS or METHODS (설명해줘, 방법, 어떻게, 절차, 알려줘, 뭐야) → ALWAYS classify as general_query (these need knowledge base, not charts)
- "CCP 체크 방법", "살균 공정 어떻게", "품질 검사 절차" → general_query (asking for SOP/procedure explanation)
- FORECAST/PREDICTION queries (예측, 전망, 다음달, 미래, forecast) → ALWAYS classify as general_query (these need statistical calculation, not BI charts)
- "수요 예측", "재고 예측", "다음달 예측", "생산 전망" → general_query (forecast queries require calculation, not visualization)
- Only classify as data_visualization when user wants to SEE CHARTS/GRAPHS about PAST/CURRENT numerical operational metrics (NOT future predictions)

Respond in JSON format:
{
  "intent": "workflow_management|judgment_execution|chart_analysis|data_visualization|settings_change|general_query",
  "confidence": 0.0-1.0,
  "reasoning": "Brief explanation (optional)"
}

Examples:
- "워크플로우 만들어줘" → workflow_management
- "재고 데이터로 판단 실행해줘" → judgment_execution
- "라인별 생산량 보여줘" → chart_analysis (specific chart: production by line)
- "라인별 생산량 분석" → chart_analysis (has "라인별" specific keyword!)
- "월별 매출 차트" → chart_analysis (specific chart: monthly sales)
- "월별 매출 분석해줘" → chart_analysis (has "월별" specific keyword!)
- "가동률 게이지" → chart_analysis (specific chart: OEE gauge)
- "CCP 합격률 현황" → chart_analysis (specific chart: CCP pass rate)
- "CCP 현황 분석" → chart_analysis (has "CCP" specific keyword!)
- "창고별 재고 비율" → chart_analysis (specific chart: inventory by warehouse)
- "재고 현황 분석" → chart_analysis (has "재고" specific keyword!)
- "온도 변화 추이" → chart_analysis (specific chart: temperature trend)
- "온도 분석해줘" → chart_analysis (has "온도" specific keyword!)
- "생산량 현황" → chart_analysis (has "생산량" specific keyword!)
- "설비별 비가동 분석" → chart_analysis (has "설비별", "비가동" specific keywords!)
- "전반적인 현황 분석해줘" → data_visualization (no specific category keyword)
- "종합 분석" → data_visualization (general analysis, no specific chart)
- "전체적인 품질 현황 알려줘" → data_visualization (general BI insight)
- "온도가 90도 이상인 데이터 보여줘" → general_query (asking for raw data)
- "불량률 트렌드 보여줘" → chart_analysis (specific trend chart request)
- "MCP 서버 연결 설정 변경" → settings_change
- "TriFlow 사용법 알려줘" → general_query
- "데이터 조회해줘" → general_query (raw data query)
- "우리 회사가 뭐하는 회사야?" → general_query (company information)
- "회사 소개해줘" → general_query (company information)
- "퓨어웰 음료 정보 알려줘" → general_query (company information)
- "우리 회사 DX 전략 짜줘" → general_query (needs company knowledge, not chart analysis)
- "회사 시스템 분석해서 전략 세워줘" → general_query (company strategy, not operational data)
- "CCP 체크 방법 설명해줘" → general_query (asking for SOP/procedure explanation)
- "살균 공정 어떻게 해?" → general_query (asking for process explanation)
- "품질 검사 절차 알려줘" → general_query (asking for procedure)
- "인증 뭐 있어?" → general_query (company info question)
- "다음달 수요 예측해줘" → general_query (forecast query - needs calculation, not BI charts)
- "재고 예측" → general_query (forecast query)
- "수요 전망 알려줘" → general_query (forecast query)
- "생산량 예측해줘" → general_query (forecast query)
"#;

        // 프롬프트 인젝션 탐지 (로깅용)
        if detect_injection_attempt(message) {
            eprintln!("⚠️ Intent 분석에서 의심스러운 패턴 감지됨");
        }

        // XML 태그로 안전하게 구조화
        let user_prompt = format!(
            r#"<user_message trust_level="medium">
{}
</user_message>"#,
            sanitize_for_xml(message)
        );

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
            "max_tokens": 8192
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
        // 📌 2024-12-08: 모든 데이터/분석 관련 질문을 ChartAnalysis로 라우팅
        // - data_visualization, general_query도 ChartAnalysis로 처리
        // - prompt_router.rs의 템플릿이 적용되도록 통합
        let intent = match analysis.intent.as_str() {
            "workflow_management" => Intent::WorkflowManagement,
            "judgment_execution" => Intent::JudgmentExecution,
            "chart_analysis" => Intent::ChartAnalysis,
            "data_visualization" => Intent::ChartAnalysis, // 📌 ChartAnalysis로 통합!
            "settings_change" => Intent::SettingsChange,
            "general_query" => Intent::ChartAnalysis, // 📌 ChartAnalysis로 통합!
            _ => Intent::ChartAnalysis, // 📌 기본값도 ChartAnalysis
        };

        println!("📌 Intent 강제 변환: {} → {:?}", analysis.intent, intent);

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

        // 🧹 캐시 무효화 (새 메시지 추가시 기존 캐시 삭제)
        println!("🧹 [Cache] Invalidating cache for session: {}", session_id);
        self.cache.invalidate(session_id);

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
    /// 캐시 우선 히스토리 조회 (Memory-First Hybrid Cache)
    ///
    /// 흐름: 1. 메모리 캐시 → 2. SQLite DB → 3. 캐시 업데이트
    pub async fn get_history(&self, session_id: &str, limit: u32) -> Result<Vec<ChatMessage>> {
        println!("📦 [ChatService] get_history called - session: {}, limit: {}", session_id, limit);

        // 1️⃣ 메모리 캐시 조회
        if let Some(cached) = self.cache.get(session_id) {
            println!("✅ [Cache] HIT - returning {} cached messages", cached.len());
            return Ok(self.convert_cached_to_service_messages(cached));
        }

        println!("❌ [Cache] MISS - querying database");

        // 2️⃣ SQLite 직접 쿼리
        let messages = self.query_database(session_id, limit)?;

        // 3️⃣ 캐시 업데이트
        let cached_messages = self.convert_service_to_cached_messages(&messages);
        self.cache.put(session_id.to_string(), cached_messages);

        println!("💾 [Cache] Stored {} messages in cache", messages.len());

        Ok(messages)
    }

    /// SQLite 직접 쿼리 (private 헬퍼)
    fn query_database(&self, session_id: &str, limit: u32) -> Result<Vec<ChatMessage>> {
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

    /// CachedMessage → ChatMessage 변환
    fn convert_cached_to_service_messages(&self, cached: Vec<CachedMessage>) -> Vec<ChatMessage> {
        cached.into_iter().map(|m| ChatMessage {
            id: m.id,
            session_id: m.session_id,
            role: m.role,
            content: m.content,
            intent: m.intent,
            created_at: m.created_at.parse::<DateTime<Utc>>()
                .unwrap_or_else(|_| Utc::now()),
        }).collect()
    }

    /// ChatMessage → CachedMessage 변환
    fn convert_service_to_cached_messages(&self, messages: &[ChatMessage]) -> Vec<CachedMessage> {
        messages.iter().map(|m| CachedMessage {
            id: m.id.clone(),
            session_id: m.session_id.clone(),
            role: m.role.clone(),
            content: m.content.clone(),
            intent: m.intent.clone(),
            created_at: m.created_at.to_rfc3339(),
        }).collect()
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

    /// MES 데이터 관련 요청인지 확인
    fn is_mes_data_request(request: &str) -> bool {
        // MES 관련 키워드 (생산, 불량, 라인, LOT, 공정, 품질, 센서, 설비, 재고 등)
        let mes_keywords = [
            // 생산/작업 관련
            "불량률", "불량", "양품", "생산량", "생산", "라인별", "라인",
            "LOT", "lot", "배치", "충진", "완제품", "작업지시", "작업자",
            // 공정 관련
            "공정", "살균", "균질", "발효", "냉각", "혼합", "배합",
            "파라미터", "parameter", "목표값", "실적", "공정실행",
            // 품질/검사 관련
            "품질", "검사", "QC", "qc", "품질검사", "미생물", "이화학",
            "금속검출", "metal", "테스트피스", "감도",
            // CCP/센서 관련
            "CCP", "ccp", "살균온도", "냉각온도", "센서", "sensor",
            "온도", "압력", "유속", "농도", "brix", "ph",
            // 설비/이벤트 관련
            "설비", "설비별", "알람", "alarm", "비가동", "downtime",
            "고장", "fault", "정비",
            // 시프트/작업자 관련
            "시프트", "시프트별", "교대", "shift", "작업조",
            // 재고/창고 관련
            "창고", "warehouse", "재고", "inventory", "재고이동",
            "자재투입", "자재출고", "material", "입고", "출고",
            // 체크리스트 관련
            "체크리스트", "checklist", "점검", "일상점검", "가동전점검",
            // 사유코드 관련
            "사유코드", "사유", "reason",
            // 제품/분석 관련
            "제품별", "reject", "good_qty", "reject_qty",
            "추이", "분석", "통계", "현황", "OEE", "oee",
        ];

        let lower_request = request.to_lowercase();
        mes_keywords.iter().any(|keyword| lower_request.contains(&keyword.to_lowercase()))
    }

    /// BI Service 또는 Chart Service 라우팅
    ///
    /// # Arguments
    /// * `user_request` - 사용자 요청 (자연어)
    ///
    /// # Returns
    /// * `serde_json::Value` - BI 인사이트 또는 차트 데이터
    pub async fn route_to_bi(&self, user_request: String) -> Result<serde_json::Value> {
        // MES 데이터 요청인 경우 Chart Service로 라우팅
        if Self::is_mes_data_request(&user_request) {
            println!("🔀 MES 키워드 감지! Chart Service로 라우팅: request={}", user_request);
            return self.route_to_chart(user_request).await;
        }

        // 그 외는 BI Service로 라우팅 (워크플로우 성공률 등)
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

    /// Chart Service 라우팅 (MES 데이터 시각화)
    ///
    /// # Arguments
    /// * `user_request` - 사용자 요청 (자연어)
    ///
    /// # Returns
    /// * `serde_json::Value` - 차트 데이터 + 인사이트
    pub async fn route_to_chart(&self, user_request: String) -> Result<serde_json::Value> {
        use crate::services::chart_service::ChartService;

        println!("📊 Routing to Chart Service (MES): request={}", user_request);

        let chart_service = ChartService::new()?;

        // 1. LLM으로 차트 계획 생성 (SQL 포함)
        let plan = chart_service.generate_chart_plan(&user_request).await?;
        println!("📋 Chart plan generated: {} (SQL: {})", plan.title, plan.sql);

        // 2. DB 연결
        let db_path = std::env::var("APPDATA")
            .map(|p| std::path::PathBuf::from(p).join("Judgify").join("judgify.db"))
            .unwrap_or_else(|_| std::path::PathBuf::from("judgify.db"));

        let conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| anyhow::anyhow!("DB 연결 실패: {}", e))?;

        // 3. SQL 실행 및 차트 데이터 생성
        let mut chart_response = chart_service.execute_and_transform(&conn, &plan)?;

        // 4. 인사이트 생성
        let insight = chart_service.generate_insight(&chart_response, &user_request).await?;
        chart_response.insight = Some(insight.clone());

        // 5. JSON으로 변환 (bar_line_data 또는 pie_data를 적절히 처리)
        let data_value = if let Some(bar_line_data) = &chart_response.bar_line_data {
            serde_json::to_value(bar_line_data).unwrap_or(serde_json::Value::Null)
        } else if let Some(pie_data) = &chart_response.pie_data {
            serde_json::json!(pie_data.iter().map(|d| serde_json::json!({
                "name": d.name,
                "value": d.value,
                "color": d.color
            })).collect::<Vec<_>>())
        } else {
            serde_json::Value::Null
        };

        // 차트 타입에 따라 적절한 데이터 키 사용
        let json_result = if chart_response.bar_line_data.is_some() {
            serde_json::json!({
                "title": chart_response.title,
                "chart_type": format!("{:?}", chart_response.chart_type).to_lowercase(),
                "description": chart_response.description,
                "bar_line_data": data_value,
                "data_keys": chart_response.data_keys,
                "x_axis_key": chart_response.x_axis_key,
                "insight": insight,
                "insights": [insight.clone()],
                "component_code": serde_json::Value::Null,
                "recommendations": ["MES 데이터 기반 분석 결과입니다."],
            })
        } else if chart_response.pie_data.is_some() {
            serde_json::json!({
                "title": chart_response.title,
                "chart_type": format!("{:?}", chart_response.chart_type).to_lowercase(),
                "description": chart_response.description,
                "pie_data": data_value,
                "insight": insight,
                "insights": [insight.clone()],
                "component_code": serde_json::Value::Null,
                "recommendations": ["MES 데이터 기반 분석 결과입니다."],
            })
        } else if chart_response.gauge_data.is_some() {
            serde_json::json!({
                "title": chart_response.title,
                "chart_type": format!("{:?}", chart_response.chart_type).to_lowercase(),
                "description": chart_response.description,
                "gauge_data": chart_response.gauge_data,
                "insight": insight,
                "insights": [insight.clone()],
                "component_code": serde_json::Value::Null,
                "recommendations": ["MES 데이터 기반 분석 결과입니다."],
            })
        } else {
            serde_json::json!({
                "title": chart_response.title,
                "chart_type": format!("{:?}", chart_response.chart_type).to_lowercase(),
                "description": chart_response.description,
                "data": data_value,
                "insight": insight,
                "insights": [insight.clone()],
                "component_code": serde_json::Value::Null,
                "recommendations": ["MES 데이터 기반 분석 결과입니다."],
            })
        };

        println!("✅ Chart Service 호출 성공: title={}", chart_response.title);

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
        let system_prompt = r#"You are a parameter extractor for the TriFlow AI platform.

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
            "max_tokens": 8192
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

    // ==================== RAG: Knowledge Base 검색 ====================

    /// Knowledge Base에서 FTS5 전문검색 수행
    ///
    /// # Arguments
    /// * `query` - 검색어 (한글 지원)
    /// * `limit` - 최대 결과 수 (기본 5)
    ///
    /// # Returns
    /// * `Vec<RagSearchResult>` - 검색 결과 목록
    fn search_knowledge_base(&self, query: &str, limit: usize) -> Vec<RagSearchResult> {
        // Judgify 메인 DB 경로
        let db_path = std::env::var("APPDATA")
            .or_else(|_| std::env::var("HOME"))
            .map(|app_data| {
                std::path::PathBuf::from(app_data)
                    .join("Judgify")
                    .join("judgify.db")
            })
            .unwrap_or_else(|_| std::path::PathBuf::from("judgify.db"));

        let conn = match rusqlite::Connection::open(&db_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("⚠️ RAG DB 연결 실패: {}", e);
                return vec![];
            }
        };

        // FTS5 검색 쿼리 (BM25 랭킹)
        let sql = r#"
            SELECT kb.title, kb.content, kb.category, kb.tags
            FROM knowledge_base kb
            JOIN knowledge_base_fts fts ON kb.rowid = fts.rowid
            WHERE knowledge_base_fts MATCH ?1
            ORDER BY bm25(knowledge_base_fts)
            LIMIT ?2
        "#;

        let mut results = Vec::new();

        // FTS5 검색어 처리 (공백으로 분리된 단어들을 OR 검색)
        let search_terms: Vec<&str> = query.split_whitespace().collect();
        let fts_query = if search_terms.len() > 1 {
            // 여러 단어: OR 검색
            search_terms.join(" OR ")
        } else {
            // 단일 단어: 와일드카드 검색
            format!("{}*", query)
        };

        match conn.prepare(sql) {
            Ok(mut stmt) => {
                match stmt.query_map(params![fts_query, limit as i64], |row| {
                    Ok(RagSearchResult {
                        title: row.get(0)?,
                        content: row.get(1)?,
                        category: row.get(2)?,
                        tags: row.get(3)?,
                    })
                }) {
                    Ok(rows) => {
                        for row in rows.flatten() {
                            results.push(row);
                        }
                    }
                    Err(e) => eprintln!("⚠️ RAG 검색 오류: {}", e),
                }
            }
            Err(e) => eprintln!("⚠️ RAG SQL 준비 오류: {}", e),
        }

        println!("🔍 RAG 검색: '{}' → {} 결과", query, results.len());
        results
    }

    /// ERP 데이터 조회 (매출, 구매, 재고, 생산)
    ///
    /// # Arguments
    /// * `query_type` - 조회 유형 ("sales", "purchase", "inventory", "production")
    /// * `time_filter` - 시간 필터 ("today", "this_week", "this_month", "this_year", "last_year")
    ///
    /// # Returns
    /// * `Option<ErpQueryResult>` - ERP 조회 결과
    fn query_erp_data(&self, query_type: &str, time_filter: &str) -> Option<ErpQueryResult> {
        // Judgify 메인 DB 경로
        let db_path = std::env::var("APPDATA")
            .or_else(|_| std::env::var("HOME"))
            .map(|app_data| {
                std::path::PathBuf::from(app_data)
                    .join("Judgify")
                    .join("judgify.db")
            })
            .unwrap_or_else(|_| std::path::PathBuf::from("judgify.db"));

        let conn = match rusqlite::Connection::open(&db_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("⚠️ ERP DB 연결 실패: {}", e);
                return None;
            }
        };

        // 시간 필터 조건 생성
        // 현재 연도 동적 계산
        let current_year = chrono::Local::now().format("%Y").to_string();
        let last_year_num: i32 = current_year.parse().unwrap_or(2025) - 1;

        let date_condition = match time_filter {
            "today" => "date(order_date) = date('now')".to_string(),
            "yesterday" => "date(order_date) = date('now', '-1 day')".to_string(),
            "this_week" => "date(order_date) >= date('now', '-7 days')".to_string(),
            "this_month" => "strftime('%Y-%m', order_date) = strftime('%Y-%m', 'now')".to_string(),
            "last_month" => "strftime('%Y-%m', order_date) = strftime('%Y-%m', 'now', '-1 month')".to_string(),
            "this_year" => "strftime('%Y', order_date) = strftime('%Y', 'now')".to_string(),
            "last_year" => "strftime('%Y', order_date) = strftime('%Y', 'now', '-1 year')".to_string(),
            // 특정 연도 지원 (동적)
            year if year.parse::<i32>().is_ok() => format!("strftime('%Y', order_date) = '{}'", year),
            _ => "1=1".to_string(), // 전체
        };

        println!("📅 ERP 쿼리: 현재 연도={}, 작년={}, 조건={}", current_year, last_year_num, date_condition);

        match query_type {
            "sales" => {
                // 매출 조회
                let sql = format!(r#"
                    SELECT
                        COUNT(*) as order_count,
                        COALESCE(SUM(total_amount), 0) as total_sales,
                        MIN(order_date) as min_date,
                        MAX(order_date) as max_date
                    FROM sales_order
                    WHERE {}
                "#, date_condition);

                match conn.query_row(&sql, [], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, f64>(1)?,
                        row.get::<_, String>(2).unwrap_or_default(),
                        row.get::<_, String>(3).unwrap_or_default(),
                    ))
                }) {
                    Ok((count, total, min_date, max_date)) => {
                        // 억원 단위 변환
                        let total_억 = total / 100_000_000.0;

                        // 상위 고객별 매출 조회
                        let customer_sql = format!(r#"
                            SELECT
                                c.cust_nm,
                                COALESCE(SUM(so.total_amount), 0) as cust_total
                            FROM sales_order so
                            JOIN customer_mst c ON so.cust_cd = c.cust_cd
                            WHERE {}
                            GROUP BY c.cust_cd, c.cust_nm
                            ORDER BY cust_total DESC
                            LIMIT 5
                        "#, date_condition);

                        let mut top_customers = Vec::new();
                        if let Ok(mut stmt) = conn.prepare(&customer_sql) {
                            if let Ok(rows) = stmt.query_map([], |row| {
                                Ok((
                                    row.get::<_, String>(0)?,
                                    row.get::<_, f64>(1)?
                                ))
                            }) {
                                for row in rows.flatten() {
                                    top_customers.push(serde_json::json!({
                                        "customer": row.0,
                                        "amount": row.1 / 100_000_000.0, // 억원
                                    }));
                                }
                            }
                        }

                        let summary = format!(
                            "총 매출: {:.1}억원 (주문 {}건, 기간: {} ~ {})",
                            total_억, count,
                            if min_date.is_empty() { "N/A" } else { &min_date[..10.min(min_date.len())] },
                            if max_date.is_empty() { "N/A" } else { &max_date[..10.min(max_date.len())] }
                        );

                        println!("📊 ERP 매출 조회: {}", summary);

                        Some(ErpQueryResult {
                            query_type: "sales".to_string(),
                            summary,
                            data: serde_json::json!({
                                "total_sales": total,
                                "total_sales_억원": format!("{:.1}", total_억),
                                "order_count": count,
                                "period": {
                                    "start": min_date,
                                    "end": max_date
                                },
                                "top_customers": top_customers,
                            }),
                        })
                    }
                    Err(e) => {
                        eprintln!("⚠️ ERP 매출 조회 오류: {}", e);
                        None
                    }
                }
            }
            "purchase" => {
                // 구매 조회
                let sql = format!(r#"
                    SELECT
                        COUNT(*) as order_count,
                        COALESCE(SUM(total_amount), 0) as total_purchase
                    FROM purchase_order
                    WHERE {}
                "#, date_condition.replace("order_date", "order_date"));

                match conn.query_row(&sql, [], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, f64>(1)?
                    ))
                }) {
                    Ok((count, total)) => {
                        let total_억 = total / 100_000_000.0;
                        let summary = format!("총 구매: {:.1}억원 (발주 {}건)", total_억, count);

                        println!("📊 ERP 구매 조회: {}", summary);

                        Some(ErpQueryResult {
                            query_type: "purchase".to_string(),
                            summary,
                            data: serde_json::json!({
                                "total_purchase": total,
                                "total_purchase_억원": format!("{:.1}", total_억),
                                "order_count": count,
                            }),
                        })
                    }
                    Err(e) => {
                        eprintln!("⚠️ ERP 구매 조회 오류: {}", e);
                        None
                    }
                }
            }
            "inventory" => {
                // 재고 조회
                let sql = r#"
                    SELECT
                        i.item_cd,
                        im.item_nm,
                        SUM(i.qty) as total_qty,
                        im.item_type
                    FROM inventory i
                    JOIN item_mst im ON i.item_cd = im.item_cd
                    GROUP BY i.item_cd, im.item_nm, im.item_type
                    ORDER BY total_qty DESC
                    LIMIT 10
                "#;

                let mut items = Vec::new();
                if let Ok(mut stmt) = conn.prepare(sql) {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, f64>(2)?,
                            row.get::<_, String>(3)?
                        ))
                    }) {
                        for row in rows.flatten() {
                            items.push(serde_json::json!({
                                "item_cd": row.0,
                                "item_nm": row.1,
                                "qty": row.2,
                                "item_type": row.3
                            }));
                        }
                    }
                }

                let summary = format!("재고 현황: 상위 {}개 품목", items.len());
                println!("📊 ERP 재고 조회: {}", summary);

                Some(ErpQueryResult {
                    query_type: "inventory".to_string(),
                    summary,
                    data: serde_json::json!({
                        "items": items,
                    }),
                })
            }
            "production" => {
                // 생산 조회
                let sql = format!(r#"
                    SELECT
                        COUNT(*) as order_count,
                        COALESCE(SUM(actual_qty), 0) as total_production,
                        status
                    FROM production_order
                    WHERE {}
                    GROUP BY status
                "#, date_condition.replace("order_date", "plan_date"));

                let mut status_data = Vec::new();
                let mut total_count = 0i64;
                let mut total_qty = 0.0f64;

                if let Ok(mut stmt) = conn.prepare(&sql) {
                    if let Ok(rows) = stmt.query_map([], |row| {
                        Ok((
                            row.get::<_, i64>(0)?,
                            row.get::<_, f64>(1)?,
                            row.get::<_, String>(2)?
                        ))
                    }) {
                        for row in rows.flatten() {
                            total_count += row.0;
                            total_qty += row.1;
                            status_data.push(serde_json::json!({
                                "status": row.2,
                                "count": row.0,
                                "qty": row.1
                            }));
                        }
                    }
                }

                let summary = format!("생산 현황: {}건, 생산량 {:.0}", total_count, total_qty);
                println!("📊 ERP 생산 조회: {}", summary);

                Some(ErpQueryResult {
                    query_type: "production".to_string(),
                    summary,
                    data: serde_json::json!({
                        "total_orders": total_count,
                        "total_production": total_qty,
                        "by_status": status_data,
                    }),
                })
            }
            _ => None
        }
    }

    // ==================== 수요/재고 예측 함수 (하이브리드 방식) ====================

    /// 메시지에서 예측 요청 추출
    ///
    /// # Returns
    /// * `Option<(String, Option<String>)>` - (예측 타입, 품목 ID 옵션)
    fn extract_forecast_query(&self, message: &str) -> Option<(String, Option<String>)> {
        let msg_lower = message.to_lowercase();

        // 예측 키워드 확인
        let is_forecast = msg_lower.contains("예측")
            || msg_lower.contains("forecast")
            || msg_lower.contains("예상")
            || msg_lower.contains("전망")
            || msg_lower.contains("다음 달")
            || msg_lower.contains("다음달")
            || msg_lower.contains("내년")
            || msg_lower.contains("앞으로");

        if !is_forecast {
            return None;
        }

        // 예측 타입 결정
        let forecast_type = if msg_lower.contains("수요") || msg_lower.contains("판매") || msg_lower.contains("주문") {
            "demand"
        } else if msg_lower.contains("재고") || msg_lower.contains("stock") {
            "inventory"
        } else if msg_lower.contains("생산") {
            "production"
        } else {
            "demand" // 기본값
        };

        // 품목 추출 시도 - 실제 item_mst 테이블 데이터 기준 (퓨어웰 브랜드 제품)
        // FG-001~008: 완제품, RM-001~015: 원료, PKG-001~007: 포장재
        let item_patterns = [
            // 완제품 (FG: Finished Goods) - 퓨어웰 브랜드
            ("프로바이오 플러스 500", "FG-001"),
            ("프로바이오 플러스", "FG-001"),
            ("프로바이오 라이트 350", "FG-002"),
            ("프로바이오 라이트", "FG-002"),
            ("프로바이오", "FG-001"), // 기본값 플러스
            ("그린프로틴 딸기", "FG-003"),
            ("그린프로틴 초코", "FG-004"),
            ("그린프로틴", "FG-003"), // 기본값 딸기
            ("프로틴", "FG-003"),
            ("단백질", "FG-003"),
            ("비타퓨어 레몬", "FG-005"),
            ("스파클링 레몬", "FG-005"),
            ("비타퓨어 오렌지", "FG-006"),
            ("스파클링 오렌지", "FG-006"),
            ("비타퓨어", "FG-005"), // 기본값 레몬
            ("뷰티셀 콜라겐", "FG-007"),
            ("콜라겐 워터", "FG-007"),
            ("뷰티셀", "FG-007"),
            ("콜라겐", "FG-007"),
            ("키즈웰 면역", "FG-008"),
            ("면역쑥쑥", "FG-008"),
            ("키즈웰", "FG-008"),
            ("키즈", "FG-008"),
            // 단축 코드
            ("fg-001", "FG-001"),
            ("fg-002", "FG-002"),
            ("fg-003", "FG-003"),
            ("fg-004", "FG-004"),
            ("fg-005", "FG-005"),
            ("fg-006", "FG-006"),
            ("fg-007", "FG-007"),
            ("fg-008", "FG-008"),
        ];

        let item_id = item_patterns
            .iter()
            .find(|(pattern, _)| msg_lower.contains(pattern))
            .map(|(_, id)| id.to_string());

        Some((forecast_type.to_string(), item_id))
    }

    /// 수요/재고 예측 데이터 조회 및 통계 계산
    fn query_forecast_data(&self, forecast_type: &str, item_id: Option<&str>) -> Option<ForecastResult> {
        // judgify_large.db 경로 (seed_data.py가 생성한 DB)
        // 1. 프로젝트 루트 디렉토리에서 찾기
        // 2. 현재 디렉토리에서 찾기
        let possible_paths = [
            std::path::PathBuf::from("c:/dev/Judgify-core/judgify_large.db"),
            std::path::PathBuf::from("judgify_large.db"),
            std::path::PathBuf::from("../judgify_large.db"),
        ];

        let db_path = possible_paths
            .iter()
            .find(|p| p.exists())
            .cloned()
            .unwrap_or_else(|| std::path::PathBuf::from("c:/dev/Judgify-core/judgify_large.db"));

        let conn = match rusqlite::Connection::open(&db_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("⚠️ 예측 DB 연결 실패: {} (경로: {:?})", e, db_path);
                return None;
            }
        };

        println!("📊 [예측] DB 연결 성공: {:?}", db_path);

        match forecast_type {
            "demand" => self.calculate_demand_forecast(&conn, item_id),
            "inventory" => self.calculate_inventory_forecast(&conn, item_id),
            _ => None
        }
    }

    /// 수요 예측 계산 (이동평균 + 성장률)
    fn calculate_demand_forecast(&self, conn: &rusqlite::Connection, item_id: Option<&str>) -> Option<ForecastResult> {
        let now = chrono::Local::now();
        let current_month = now.format("%Y-%m").to_string();
        let current_year = now.format("%Y").to_string();
        let current_month_num: u32 = now.format("%m").to_string().parse().unwrap_or(1);

        // 다음 달 계산
        let next_month = if current_month_num == 12 {
            format!("{}-01", current_year.parse::<i32>().unwrap_or(2025) + 1)
        } else {
            format!("{}-{:02}", current_year, current_month_num + 1)
        };

        // 전년 동월
        let last_year_same_month = format!("{}-{:02}",
            current_year.parse::<i32>().unwrap_or(2025) - 1,
            current_month_num
        );

        // SQL 쿼리 (품목별 또는 전체)
        // judgify.db 스키마: sales_order(so_date), sales_order_dtl(item_cd, qty)
        // item_mst(item_cd, item_nm, item_type) - FG: 완제품, RM: 원료, PKG: 포장재
        let (sql, item_name) = if let Some(id) = item_id {
            // 품목명 조회 (item_mst 테이블의 item_nm 컬럼)
            let name: String = conn.query_row(
                "SELECT item_nm FROM item_mst WHERE item_cd = ?",
                [id],
                |row| row.get(0)
            ).unwrap_or_else(|_| id.to_string());

            (format!(r#"
                SELECT
                    strftime('%Y-%m', s.so_date) as month,
                    SUM(d.qty) as qty
                FROM sales_order s
                JOIN sales_order_dtl d ON s.so_no = d.so_no
                WHERE d.item_cd = '{}'
                GROUP BY strftime('%Y-%m', s.so_date)
                ORDER BY month DESC
                LIMIT 12
            "#, id), Some(name))
        } else {
            (r#"
                SELECT
                    strftime('%Y-%m', s.so_date) as month,
                    SUM(d.qty) as qty
                FROM sales_order s
                JOIN sales_order_dtl d ON s.so_no = d.so_no
                GROUP BY strftime('%Y-%m', s.so_date)
                ORDER BY month DESC
                LIMIT 12
            "#.to_string(), None)
        };

        // 월별 데이터 조회
        let mut monthly_data: Vec<(String, f64)> = Vec::new();
        if let Ok(mut stmt) = conn.prepare(&sql) {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, f64>(1)?
                ))
            }) {
                for row in rows.flatten() {
                    monthly_data.push(row);
                }
            }
        }

        if monthly_data.is_empty() {
            println!("⚠️ [예측] 데이터 없음");
            return None;
        }

        println!("📊 [예측] {}개월 데이터 조회 완료", monthly_data.len());

        // 통계 계산
        // 1. 최근 3개월 평균
        let recent_3m: Vec<f64> = monthly_data.iter().take(3).map(|(_, q)| *q).collect();
        let recent_avg = recent_3m.iter().sum::<f64>() / recent_3m.len().max(1) as f64;

        // 2. 6개월 이동평균
        let recent_6m: Vec<f64> = monthly_data.iter().take(6).map(|(_, q)| *q).collect();
        let moving_avg_6m = recent_6m.iter().sum::<f64>() / recent_6m.len().max(1) as f64;

        // 3. 전년 동월 실적
        let last_year_qty = monthly_data
            .iter()
            .find(|(m, _)| m == &last_year_same_month)
            .map(|(_, q)| *q)
            .unwrap_or(recent_avg);

        // 4. 성장률 계산 (전년 동월 대비)
        let growth_rate = if last_year_qty > 0.0 {
            ((recent_avg - last_year_qty) / last_year_qty) * 100.0
        } else {
            0.0
        };

        // 5. 예측 수량 (가중 평균: 최근 트렌드 60% + 전년 동월 40%)
        let forecast_qty = (recent_avg * 0.6) + (last_year_qty * (1.0 + growth_rate / 100.0) * 0.4);

        // 트렌드 데이터 (오름차순)
        let mut trend: Vec<(String, f64)> = monthly_data.into_iter().collect();
        trend.reverse();

        Some(ForecastResult {
            forecast_type: "demand".to_string(),
            item_id: item_id.map(|s| s.to_string()),
            item_name,
            forecast_period: next_month,
            recent_avg,
            moving_avg_6m,
            last_year_same_month: last_year_qty,
            growth_rate,
            forecast_qty,
            safety_stock: 0.0,
            current_stock: 0.0,
            monthly_trend: trend,
        })
    }

    /// 재고 예측 계산 (현재고 - 예상출고 + 예상입고)
    fn calculate_inventory_forecast(&self, conn: &rusqlite::Connection, item_id: Option<&str>) -> Option<ForecastResult> {
        // 먼저 수요 예측 계산
        let demand_forecast = self.calculate_demand_forecast(conn, item_id)?;

        // 현재 재고 조회 (fg_lot 테이블에서)
        // seed_data.py/judgify_large.db 기준: fg_lot(fg_item_id, qty, qc_status='PASS')
        let current_stock_sql = if let Some(id) = item_id {
            format!(r#"
                SELECT COALESCE(SUM(qty), 0)
                FROM fg_lot
                WHERE fg_item_id = '{}' AND qc_status = 'PASS'
            "#, id)
        } else {
            r#"
                SELECT COALESCE(SUM(qty), 0)
                FROM fg_lot
                WHERE qc_status = 'PASS'
            "#.to_string()
        };

        let current_stock: f64 = conn.query_row(&current_stock_sql, [], |row| row.get(0))
            .unwrap_or(0.0);

        // 평균 일출고량 계산 (최근 30일 기준)
        // seed_data.py/judgify_large.db 기준: sales_order(so_date), sales_order_dtl(item_id, order_qty)
        let daily_avg_sql = if let Some(id) = item_id {
            format!(r#"
                SELECT COALESCE(SUM(d.order_qty) / 30.0, 0)
                FROM sales_order s
                JOIN sales_order_dtl d ON s.so_no = d.so_no
                WHERE d.item_id = '{}'
                AND s.so_date >= date('now', '-30 days')
            "#, id)
        } else {
            r#"
                SELECT COALESCE(SUM(d.order_qty) / 30.0, 0)
                FROM sales_order s
                JOIN sales_order_dtl d ON s.so_no = d.so_no
                WHERE s.so_date >= date('now', '-30 days')
            "#.to_string()
        };

        let daily_avg: f64 = conn.query_row(&daily_avg_sql, [], |row| row.get(0))
            .unwrap_or(demand_forecast.forecast_qty / 30.0);

        // 안전재고 = 평균 일출고량 × 리드타임(7일) × 안전계수(1.5)
        let lead_time = 7.0;
        let safety_factor = 1.5;
        let safety_stock = daily_avg * lead_time * safety_factor;

        Some(ForecastResult {
            forecast_type: "inventory".to_string(),
            item_id: demand_forecast.item_id,
            item_name: demand_forecast.item_name,
            forecast_period: demand_forecast.forecast_period,
            recent_avg: demand_forecast.recent_avg,
            moving_avg_6m: demand_forecast.moving_avg_6m,
            last_year_same_month: demand_forecast.last_year_same_month,
            growth_rate: demand_forecast.growth_rate,
            forecast_qty: demand_forecast.forecast_qty,
            safety_stock,
            current_stock,
            monthly_trend: demand_forecast.monthly_trend,
        })
    }

    /// 예측 결과를 LLM 컨텍스트로 변환
    fn format_forecast_context(&self, result: &ForecastResult) -> String {
        let item_desc = result.item_name.as_ref()
            .map(|n| format!("{} ({})", n, result.item_id.as_ref().unwrap_or(&"전체".to_string())))
            .unwrap_or_else(|| "전체 제품".to_string());

        let mut ctx = String::from("\n<forecast_data>\n");
        ctx.push_str("아래는 통계 분석을 통해 계산된 실제 예측 데이터입니다. 이 수치를 정확히 사용하여 답변하세요:\n\n");

        ctx.push_str(&format!("예측 대상: {}\n", item_desc));
        ctx.push_str(&format!("예측 기간: {}\n", result.forecast_period));
        ctx.push_str(&format!("예측 유형: {}\n\n",
            if result.forecast_type == "demand" { "수요 예측" } else { "재고 예측" }
        ));

        ctx.push_str("=== 통계 분석 결과 ===\n");
        ctx.push_str(&format!("• 최근 3개월 평균: {}개\n", format_number(result.recent_avg)));
        ctx.push_str(&format!("• 6개월 이동평균: {}개\n", format_number(result.moving_avg_6m)));
        ctx.push_str(&format!("• 전년 동월 실적: {}개\n", format_number(result.last_year_same_month)));
        ctx.push_str(&format!("• 전년 대비 성장률: {:+.1}%\n", result.growth_rate));
        ctx.push_str(&format!("• 📈 예측 수량: {}개\n\n", format_number(result.forecast_qty)));

        if result.forecast_type == "inventory" {
            ctx.push_str("=== 재고 분석 ===\n");
            ctx.push_str(&format!("• 현재 재고: {}개\n", format_number(result.current_stock)));
            ctx.push_str(&format!("• 권장 안전재고: {}개\n", format_number(result.safety_stock)));

            let expected_stock = result.current_stock - result.forecast_qty;
            ctx.push_str(&format!("• 예상 기말재고: {}개\n", format_number(expected_stock)));

            if expected_stock < result.safety_stock {
                ctx.push_str(&format!("• ⚠️ 부족 예상: {}개 추가 필요\n", format_number(result.safety_stock - expected_stock)));
            } else {
                ctx.push_str("• ✅ 재고 여유 있음\n");
            }
            ctx.push_str("\n");
        }

        ctx.push_str("=== 최근 6개월 추이 ===\n");
        for (month, qty) in result.monthly_trend.iter().rev().take(6) {
            ctx.push_str(&format!("• {}: {}개\n", month, format_number(*qty)));
        }

        ctx.push_str("\n</forecast_data>\n");
        ctx
    }

    /// 메시지에서 ERP 조회 유형 추출
    ///
    /// # Arguments
    /// * `message` - 사용자 메시지
    ///
    /// # Returns
    /// * `Option<(String, String)>` - (조회 유형, 시간 필터) 또는 None
    fn extract_erp_query_type(&self, message: &str) -> Option<(String, String)> {
        let msg_lower = message.to_lowercase();

        // 시간 필터 추출
        // 현재 연도 동적 계산 (하드코딩 대신)
        let current_year = chrono::Local::now().format("%Y").to_string();
        let last_year_num: i32 = current_year.parse().unwrap_or(2025) - 1;
        let last_year_str = last_year_num.to_string();

        let time_filter = if msg_lower.contains("오늘") {
            "today"
        } else if msg_lower.contains("어제") {
            "yesterday"
        } else if msg_lower.contains("이번 주") || msg_lower.contains("이번주") || msg_lower.contains("금주") {
            "this_week"
        } else if msg_lower.contains("이번 달") || msg_lower.contains("이번달") || msg_lower.contains("금월") {
            "this_month"
        } else if msg_lower.contains("지난 달") || msg_lower.contains("지난달") || msg_lower.contains("전월") {
            "last_month"
        } else if msg_lower.contains("올해") || msg_lower.contains("금년") || msg_lower.contains(&current_year) {
            "this_year"
        } else if msg_lower.contains("작년") || msg_lower.contains("전년") || msg_lower.contains(&last_year_str) {
            "last_year"
        } else {
            "this_year" // 기본값: 올해
        };

        // 조회 유형 추출
        if msg_lower.contains("매출") || msg_lower.contains("판매") || msg_lower.contains("수주") || msg_lower.contains("sales") {
            Some(("sales".to_string(), time_filter.to_string()))
        } else if msg_lower.contains("구매") || msg_lower.contains("발주") || msg_lower.contains("입고") || msg_lower.contains("purchase") {
            Some(("purchase".to_string(), time_filter.to_string()))
        } else if msg_lower.contains("재고") || msg_lower.contains("inventory") || msg_lower.contains("stock") {
            Some(("inventory".to_string(), time_filter.to_string()))
        } else if msg_lower.contains("생산") || msg_lower.contains("production") {
            Some(("production".to_string(), time_filter.to_string()))
        } else {
            None
        }
    }

    /// 메시지에서 RAG 검색이 필요한 키워드 추출
    ///
    /// # Arguments
    /// * `message` - 사용자 메시지
    ///
    /// # Returns
    /// * `Option<String>` - 검색어 (필요 없으면 None)
    fn extract_rag_keywords(&self, message: &str) -> Option<String> {
        let msg_lower = message.to_lowercase();

        // RAG 검색 트리거 패턴들
        // ⚠️ 순서 중요: 더 구체적인 키워드가 먼저 매칭되어야 함!
        // 예: "살균 공정" → "살균"이 "공정"보다 먼저 매칭되어야 SOP-04 검색됨
        let trigger_patterns = [
            // ===== SOP 관련 (구체적인 공정명 먼저!) =====
            ("살균", "살균"),          // "살균 공정" → 살균 SOP 검색
            ("냉각", "냉각"),          // "냉각 공정" → 냉각 SOP 검색
            ("충진", "충진"),          // "충진 공정" → 충진 SOP 검색
            ("충전", "충전"),          // "충전 공정" → 충전 SOP 검색 (동의어)
            ("배합", "배합"),          // "배합 공정" → 배합 SOP 검색
            ("밀봉", "밀봉"),          // "밀봉 공정" → 밀봉 SOP 검색
            ("포장", "포장"),          // "포장 공정" → 포장 SOP 검색
            ("금속검출", "금속검출"),   // 금속검출 SOP
            ("cip", "CIP"),            // CIP 세척
            ("sip", "SIP"),            // SIP 살균
            ("ccp", "CCP"),            // CCP 관리
            ("haccp", "HACCP"),        // HACCP 인증
            // ===== 일반 공정 (구체적 공정명에 매칭 안 될 때) =====
            ("공정", "SOP"),           // 일반 공정 → SOP 전체 검색
            ("sop", "SOP"),            // SOP 직접 언급
            // ===== 품질/검사 관련 =====
            ("품질", "품질"),          // 품질 관련
            ("검사", "품질"),          // "품질 검사 절차" → 품질 검색
            ("체크", "CCP"),           // "CCP 체크 방법" → CCP 검색
            ("원료", "원료"),          // 원료 관련
            // ===== 절차/방법 (마지막 fallback) =====
            ("절차", "SOP"),           // "절차 알려줘" → SOP 검색
            ("방법", "SOP"),           // "방법 설명해줘" → SOP 검색
            // ===== 회사 정보 관련 =====
            ("회사", "회사"),
            ("퓨어웰", "퓨어웰"),
            ("기업", "기업"),
            ("조직", "조직"),
            ("시설", "시설"),
            ("공장", "시설"),
            ("인증", "인증"),
            ("제품", "제품"),
            // ===== 전략/DX 관련 =====
            ("dx", "디지털 전환"),
            ("디지털", "디지털 전환"),
            ("전략", "회사"),
            ("비전", "회사"),
            ("사업", "회사"),
            // ===== 일반 질문 (마지막) =====
            ("뭐하", "회사"),
            ("어떤", "회사"),
            ("무엇", "회사"),
        ];

        for (pattern, keyword) in trigger_patterns {
            if msg_lower.contains(pattern) {
                return Some(keyword.to_string());
            }
        }

        None
    }

    /// 대화형 응답 생성 (GeneralQuery용)
    ///
    /// # Arguments
    /// * `message` - 사용자 메시지
    /// * `history` - 최근 대화 이력 (컨텍스트)
    ///
    /// # Returns
    /// * `String` - Claude가 생성한 자연스러운 대화 응답
    pub async fn generate_conversational_response(
        &self,
        message: &str,
        history: Vec<ChatMessage>,
    ) -> Result<String> {
        // ==================== ERP 데이터 조회 (매출, 구매, 재고, 생산) ====================
        let erp_context = if let Some((query_type, time_filter)) = self.extract_erp_query_type(message) {
            if let Some(result) = self.query_erp_data(&query_type, &time_filter) {
                let mut ctx = String::from("\n<erp_data_context>\n");
                ctx.push_str("아래는 ERP 시스템에서 조회된 실제 데이터입니다. 이 데이터를 기반으로 정확하게 답변하세요:\n\n");
                ctx.push_str(&format!("조회 유형: {}\n", result.query_type));
                ctx.push_str(&format!("요약: {}\n", result.summary));
                ctx.push_str(&format!("상세 데이터: {}\n", serde_json::to_string_pretty(&result.data).unwrap_or_default()));
                ctx.push_str("\n</erp_data_context>\n");
                println!("📊 ERP 컨텍스트 추가: {} ({})", result.query_type, time_filter);
                ctx
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // ==================== 수요/재고 예측 (하이브리드 방식) ====================
        let forecast_context = if let Some((forecast_type, item_id)) = self.extract_forecast_query(message) {
            if let Some(result) = self.query_forecast_data(&forecast_type, item_id.as_deref()) {
                let ctx = self.format_forecast_context(&result);
                println!("📈 예측 컨텍스트 추가: {} (품목: {})",
                    forecast_type,
                    item_id.as_deref().unwrap_or("전체"));
                ctx
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // ==================== RAG 검색 (Knowledge Base) ====================
        let rag_context = if let Some(keyword) = self.extract_rag_keywords(message) {
            let results = self.search_knowledge_base(&keyword, 3);
            if !results.is_empty() {
                let mut ctx = String::from("\n<knowledge_base_context>\n");
                ctx.push_str("아래는 회사 지식베이스에서 검색된 관련 정보입니다. 사용자 질문에 답변할 때 참고하세요:\n\n");
                for (i, r) in results.iter().enumerate() {
                    ctx.push_str(&format!(
                        "--- 문서 {} ({}) ---\n제목: {}\n내용: {}\n\n",
                        i + 1,
                        r.category,
                        r.title,
                        // 내용이 너무 길면 자르기 (최대 1000자)
                        if r.content.chars().count() > 1000 {
                            format!("{}...", r.content.chars().take(1000).collect::<String>())
                        } else {
                            r.content.clone()
                        }
                    ));
                }
                ctx.push_str("</knowledge_base_context>\n");
                println!("📚 RAG 컨텍스트 추가: {} 문서", results.len());
                ctx
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // 시스템 프롬프트 (퓨어웰 음료㈜ AI Assistant 역할)
        let system_prompt = r#"You are TriFlow AI Assistant for 퓨어웰 음료㈜ (PUREWELL Beverage Co.), a helpful AI assistant for the TriFlow MES/ERP platform.

IMPORTANT - ERP Data:
- You have access to REAL ERP data in the <erp_data_context> section
- When users ask about 매출(sales), 구매(purchase), 재고(inventory), 생산(production), ALWAYS use the ERP data to answer
- The data is REAL and ACCURATE - use exact numbers from the data (e.g., "올해 매출은 394.2억원입니다")
- For sales queries: report total_sales in 억원 format, include order count and top customers if available
- NEVER say you don't have the data when <erp_data_context> is present

IMPORTANT - Forecast Data (하이브리드 예측):
- You have access to STATISTICAL FORECAST data in the <forecast_data> section
- The forecast is calculated using: (최근3개월평균 × 60%) + (전년동월 × 성장률 × 40%)
- When users ask about 예측, 전망, 예상, 다음달, ALWAYS use the forecast data
- Use EXACT numbers from the calculation (e.g., "다음달 예상 수요는 약 12,500개입니다")
- For inventory forecast: also mention 안전재고(safety stock), 현재재고(current stock), 부족 여부
- Provide BUSINESS INSIGHT based on the data:
  - 성장률 > 10%: "성장세가 두드러집니다"
  - 성장률 < -10%: "수요 감소 추세입니다"
  - 재고 부족 예상: "추가 생산 또는 발주가 필요합니다"
  - 재고 여유: "현재 재고 수준이 적정합니다"
- Explain the trend using 월별 추이 data

IMPORTANT - Company Knowledge:
- You have access to 퓨어웰 음료㈜ company information in the <knowledge_base_context> section
- When users ask about the company, products, processes, or SOPs, USE the knowledge base information to answer
- Always prioritize knowledge base data over generic responses
- If knowledge base has relevant info, quote specific details (e.g., "퓨어웰 음료㈜는 2010년 설립된 음료 제조 전문기업입니다")

=== 제품(품목) 데이터 구조 (item_mst) ===
품목 마스터 테이블 구조:
- item_cd: 품목코드 (FG-XXX: 완제품, RM-XXX: 원료, PKG-XXX: 포장재)
- item_nm: 품목명
- item_type: 품목유형 (FG=완제품/Finished Goods, RM=원료/Raw Material, PKG=포장재/Packaging)

현재 완제품 목록 (FG: Finished Goods) - 퓨어웰 음료㈜ 브랜드 제품:
- FG-001: 퓨어웰 프로바이오 플러스 500 (유산균 음료)
- FG-002: 퓨어웰 프로바이오 라이트 350 (유산균 음료)
- FG-003: 퓨어웰 그린프로틴 딸기맛 (식물성 단백질 쉐이크)
- FG-004: 퓨어웰 그린프로틴 초코맛 (식물성 단백질 쉐이크)
- FG-005: 비타퓨어 스파클링 레몬 (비타민 음료)
- FG-006: 비타퓨어 스파클링 오렌지 (비타민 음료)
- FG-007: 뷰티셀 콜라겐 워터 (콜라겐 음료)
- FG-008: 키즈웰 면역쑥쑥 (어린이 면역 음료)

IMPORTANT: 사용자가 "제품 목록", "우리 제품", "뭘 만들어?" 등을 물으면:
- 완제품(FG-XXX)만 응답하세요
- 원료(RM-XXX)나 포장재(PKG-XXX)는 제품이 아닙니다
- 예: "퓨어웰 음료㈜는 총 8종의 완제품을 생산합니다: 퓨어웰 프로바이오 시리즈(플러스, 라이트), 퓨어웰 그린프로틴(딸기, 초코), 비타퓨어 스파클링(레몬, 오렌지), 뷰티셀 콜라겐 워터, 키즈웰 면역쑥쑥이 있어요."

=== 현재 동작하는 기능 (시연 가능) ===
1. 회사 정보 조회: 퓨어웰 음료㈜ 기업 개요, 제품, 인증, 조직, 시설 정보
2. SOP 절차 안내: 살균, 배합, 충진, 냉각, 포장 등 제조 공정 표준작업절차
3. MES/ERP 데이터 조회: 매출, 구매, 재고, 생산 현황 실시간 조회
4. 수요/재고 예측: 이동평균 + 성장률 기반 다음달 수요 예측, 안전재고 분석
5. 제품(품목) 조회: 완제품 목록, 품목별 정보 안내
6. 일반 질문 응답: 식품안전, HACCP, 품질관리 관련 지식 답변

=== 다른 메뉴에서 가능한 기능 ===
- 워크플로우 생성/편집 → "워크플로우" 메뉴에서 가능
- 판단 실행 → 워크플로우 시뮬레이션에서 가능
- 대시보드/차트 → "대시보드" 메뉴에서 가능

=== 아직 개발 중인 기능 ===
- 채팅으로 차트 자동 생성
- AI 기반 BI 인사이트 자동 생성
- 채팅으로 워크플로우 생성

When users ask "뭘 할 수 있어?" or about capabilities:
- Focus on what actually works NOW (회사 정보, SOP, ERP 데이터 조회)
- If they want workflows or dashboards, guide them to the appropriate menu
- Be honest about features under development

Response guidelines:
- Be conversational, friendly, and helpful
- Use Korean language naturally
- When ERP data is available, USE IT to provide EXACT numbers
- When company knowledge is available, USE IT to provide specific answers
- Keep responses concise (2-4 sentences for simple queries, more detail if needed)
- Reference conversation history when relevant

Examples:
- User: "올해 매출 얼마야?" → [Use erp_data_context] "올해 퓨어웰 음료㈜의 매출은 약 XXX억원입니다. 총 XX건의 주문이 있었어요." (실제 erp_data_context의 숫자 사용)
- User: "작년 매출은?" → [Use erp_data_context with last_year filter] "작년 매출은 약 XXX억원이었습니다." (실제 erp_data_context의 숫자 사용, 연도는 데이터 기준으로 표시)
- User: "다음달 수요 예측해줘" → [Use forecast_data] "다음달 전체 제품 예상 수요는 약 12,500개입니다. 최근 3개월 평균이 11,800개이고, 전년 대비 +5.2% 성장률을 보이고 있어 완만한 성장세입니다."
- User: "프로바이오틱스 재고 예측" → [Use forecast_data with item] "프로바이오틱스 100의 다음달 예상 수요는 2,500개입니다. 현재 재고 3,000개로 안전재고(1,575개) 대비 여유가 있습니다."
- User: "우리 회사가 뭐하는 회사야?" → [Use knowledge_base_context] "퓨어웰 음료㈜는 2010년 설립된 음료 제조 전문기업입니다. 주스, 스무디, 건강음료 등을 생산하고 있으며, HACCP, ISO 22000 등의 인증을 보유하고 있어요."
- User: "살균 공정 어떻게 해?" → [Use SOP from knowledge_base] "살균 공정(SOP-04)은 CCP(중요관리점)로, 85°C에서 15초간 유지하는 것이 기준입니다. 온도가 83°C 미만이면 즉시 재살균이 필요해요."
- User: "제품 뭐 있어?" / "우리 제품 목록" → "퓨어웰 음료㈜는 총 8종의 완제품을 생산합니다: 퓨어웰 프로바이오 시리즈(플러스 500, 라이트 350), 퓨어웰 그린프로틴(딸기맛, 초코맛), 비타퓨어 스파클링(레몬, 오렌지), 뷰티셀 콜라겐 워터, 키즈웰 면역쑥쑥이 있어요." (완제품 FG-001~008만 응답, 원료/포장재 제외)
- User: "뭘 할 수 있어?" → "저는 퓨어웰 음료 회사 정보 안내, SOP 절차 설명, 매출/재고/생산 데이터 조회, 수요/재고 예측, 제품 목록 안내를 도와드릴 수 있어요. 워크플로우 생성은 좌측 '워크플로우' 메뉴에서, 대시보드는 '대시보드' 메뉴에서 이용하실 수 있습니다!"
"#;

        // 대화 이력을 안전하게 처리 (최근 5개)
        let mut conversation_context = String::new();
        if !history.is_empty() {
            conversation_context.push_str("\n<conversation_history trust_level=\"medium\">\n");
            for msg in history.iter().take(5) {
                // 각 메시지에서 프롬프트 인젝션 탐지
                if detect_injection_attempt(&msg.content) {
                    eprintln!("⚠️ 대화 이력에서 의심스러운 패턴 감지됨");
                }

                conversation_context.push_str(&format!(
                    "{}: {}\n",
                    if msg.role == "user" { "User" } else { "Assistant" },
                    sanitize_for_xml(&msg.content)  // XML 이스케이프 적용
                ));
            }
            conversation_context.push_str("</conversation_history>\n");
        }

        // ERP + Forecast + RAG 컨텍스트 + 대화 이력 + 사용자 메시지 조합
        let has_context = !conversation_context.is_empty() || !rag_context.is_empty() || !erp_context.is_empty() || !forecast_context.is_empty();
        let user_prompt = format!(
            "{}{}{}{}{}",
            erp_context,
            forecast_context,
            rag_context,
            conversation_context,
            if has_context {
                format!("\n\n<user_new_message trust_level=\"medium\">\n{}\n</user_new_message>",
                    sanitize_for_xml(message))
            } else {
                format!("<user_message trust_level=\"medium\">\n{}\n</user_message>",
                    sanitize_for_xml(message))
            }
        );

        println!("📤 [generate_conversational_response] Calling Claude API...");
        println!("   Context: {} history, RAG: {}, ERP: {}, Forecast: {}",
            history.len(),
            !rag_context.is_empty(),
            !erp_context.is_empty(),
            !forecast_context.is_empty());

        // Claude API 호출
        let request_body = json!({
            "model": "claude-sonnet-4-5-20250929",
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.7,  // 대화형 응답은 약간 더 창의적으로
            "max_tokens": 8192  // 긴 답변(전략 제안, 상세 설명 등) 대응
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
        println!("📥 [generate_conversational_response] Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await?;
            eprintln!("❌ [generate_conversational_response] Claude API error ({}): {}", status, error_text);
            anyhow::bail!("Claude API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await?;
        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in Claude response"))?;

        // 마크다운 코드 블록 제거 (혹시 JSON 형식으로 응답하는 경우 대비)
        let clean_content = strip_markdown_code_block(content);

        println!("✅ [generate_conversational_response] Response generated: {}",
            if clean_content.chars().count() > 100 {
                format!("{}...", clean_content.chars().take(100).collect::<String>())
            } else {
                clean_content.to_string()
            }
        );

        Ok(clean_content.to_string())
    }

    /// 차트 분석 요청에 대한 응답 생성 (프롬프트 라우터 사용)
    ///
    /// 사용자의 차트/그래프 요청을 분석하여 확장된 프롬프트 템플릿과 함께
    /// LLM을 호출하여 차트 데이터 + 분석 텍스트를 생성합니다.
    ///
    /// # Arguments
    /// * `message` - 사용자 메시지 (예: "라인별 생산량 보여줘", "CCP 합격률 현황")
    /// * `history` - 최근 대화 이력
    ///
    /// # Returns
    /// * `String` - 차트 JSON + 분석 텍스트가 포함된 응답
    pub async fn generate_chart_response(
        &self,
        message: &str,
        history: Vec<ChatMessage>,
    ) -> Result<String> {
        println!("📊 [generate_chart_response] Processing chart analysis request");
        println!("   Message: {}", message);
        println!("   History count: {} messages", history.len());
        if history.is_empty() {
            println!("   ⚠️ [WARNING] No conversation history - this is a NEW session");
        } else {
            println!("   ✅ History available - continuing conversation context");
        }

        // 1. 프롬프트 라우터로 확장 프롬프트 생성
        let router = PromptRouter::new();
        let expanded_prompt = router.get_final_prompt(message);

        println!("📋 [generate_chart_response] Expanded prompt length: {} chars", expanded_prompt.len());

        // 2. 대화 이력을 Claude 메시지 형식으로 변환
        let mut messages: Vec<serde_json::Value> = history
            .iter()
            .map(|msg| {
                json!({
                    "role": msg.role.clone(),
                    "content": msg.content.clone()
                })
            })
            .collect();

        // 3. 확장된 프롬프트를 현재 사용자 메시지로 추가
        messages.push(json!({
            "role": "user",
            "content": expanded_prompt
        }));

        // 4. 시스템 프롬프트 - 차트 분석 전문가 역할 (템플릿 응답 규칙 최우선 적용)
        let system_prompt = r#"당신은 퓨어웰 음료㈜ (PUREWELL Beverage Co.)의 AI 분석 전문가입니다.

핵심 역할:
1. 제공된 SQL 쿼리와 판단 기준을 기반으로 데이터를 분석합니다
2. 분석 결과를 명확한 한국어로 설명합니다
3. 응답 형식 예시에 맞춰 구조화된 응답을 생성합니다
4. 차트 렌더링을 위한 JSON 데이터를 포함합니다

🚨 최우선 규칙 - [7. 응답 규칙] 섹션 엄격 준수:
아래 사용자 메시지에 [7. 응답 규칙] 섹션이 포함되어 있다면, 해당 규칙을 반드시 최우선으로 따르세요.

구체적 준수 사항:
1. 수치 표시 형식을 정확히 따를 것:
   - 온도: 소수점 1자리 (예: 89.5℃)
   - 금액: 억원 단위 소수점 1자리 (예: 12.5억원)
   - 수량: 천단위 콤마 (예: 12,450병)
   - 백분율: 소수점 1~3자리 (템플릿 지시에 따름)

2. 필수 포함 항목을 빠뜨리지 말 것:
   - CCP 이탈 시 LOT ID 명시 (필수!)
   - 전월/전년 대비 화살표(↑↓) 표시
   - 이상 징후 발견 시 원인 분석 포함
   - 권장사항은 구체적이고 실행 가능하게

3. 시각적 표현을 템플릿 지시대로 사용할 것:
   - 상태 아이콘: ✅정상, ⚠️주의, 🚨경고
   - 색상 언급: 빨간색/노란색 강조
   - 표/테이블 형식 준수

4. 도메인별 특수 규칙:
   - HACCP/CCP: 100% 합격률이 필수조건임을 명시
   - OEE: 3요소(가동률/성능/품질) 모두 분석
   - 품질검사: PASS/HOLD/REJECT 3상태 구분

기본 응답 규칙 (위 규칙과 충돌 시 [7. 응답 규칙] 우선):
- 항상 한국어로 응답합니다
- 차트 JSON은 [6. 차트 렌더링 데이터] 섹션 형식을 정확히 따릅니다
- 판단 기준(Threshold)에 따라 상태를 표시합니다

중요: 응답 끝에 반드시 차트 JSON을 다음 형식으로 포함하세요:
```json:chart
{차트 데이터 JSON}
```"#;

        // 5. Claude API 호출
        let request_body = json!({
            "model": "claude-sonnet-4-5-20250929",
            "system": system_prompt,
            "messages": messages,
            "temperature": 0.3,  // 데이터 분석은 정확성 우선
            "max_tokens": 8192   // 차트 JSON 포함으로 더 긴 응답 허용
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
        let response_text = response.text().await?;

        if !status.is_success() {
            println!("❌ [generate_chart_response] API Error: {} - {}", status, response_text);
            return Err(anyhow::anyhow!("Claude API 오류: {}", status));
        }

        // 6. 응답 파싱
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
        let content = response_json["content"][0]["text"]
            .as_str()
            .unwrap_or("차트 분석 응답을 생성하지 못했습니다.");

        // 마크다운 코드 블록 제거 (JSON 부분 제외)
        let clean_content = strip_markdown_code_block(content);

        println!("✅ [generate_chart_response] Chart response generated: {} chars", clean_content.len());

        Ok(clean_content.to_string())
    }

    /// 테이블 데이터를 기반으로 자연어 응답 생성
    ///
    /// ERP/MES 테이블 조회 결과를 LLM에 전달하여 사용자 질문에 대한 자연어 답변 생성
    ///
    /// # Arguments
    /// * `message` - 사용자 질문 (예: "프로바이오틱스 2024년 6월 판매량은 얼마야?")
    /// * `table_data_json` - 테이블 데이터 JSON 문자열
    /// * `table_summary` - 테이블 요약 (예: "판매 주문에서 20건의 데이터를 찾았습니다")
    ///
    /// # Returns
    /// * `String` - 자연어 응답 (예: "2024년 6월 프로바이오틱스 판매량은 총 X개입니다.")
    pub async fn generate_response_from_table_data(
        &self,
        message: &str,
        table_data_json: &str,
        table_summary: &str,
    ) -> Result<String> {
        println!("🤖 [generate_response_from_table_data] Processing user question with table data");
        println!("   Question: {}", if message.chars().count() > 50 {
            format!("{}...", message.chars().take(50).collect::<String>())
        } else {
            message.to_string()
        });

        // 시스템 프롬프트 - 테이블 데이터를 해석하여 답변하도록 지시
        let system_prompt = r#"You are a helpful AI assistant for 퓨어웰 음료㈜ (PUREWELL Beverage Co.) that analyzes ERP/MES data to answer business questions.

CRITICAL INSTRUCTIONS:
1. You will receive ACTUAL table data from the ERP/MES system
2. ANALYZE the data carefully and ANSWER the user's question based on it
3. Provide a NATURAL LANGUAGE response in Korean
4. Include specific numbers, totals, counts, and relevant statistics from the data
5. Do NOT just describe the table - ANSWER the question using the data
6. Format numbers with Korean units (개, 원, 건 등)
7. For product names like "프로바이오틱스", filter and sum relevant rows
8. Always show totals, averages, or counts as appropriate for the question

RESPONSE FORMAT:
- Start with a direct answer to the question
- Include supporting details if relevant
- Keep the response concise and informative
- Speak in Korean naturally

Example:
Question: "프로바이오틱스 2024년 6월 판매량은 얼마야?"
Good Response: "2024년 6월 프로바이오틱스 제품 판매량은 총 15,000개입니다. 주요 고객사별로 보면 쿠팡이 5,000개, 마켓컬리가 3,000개를 주문했습니다."
Bad Response: "판매 주문에서 20건의 데이터를 찾았습니다." (This is what we DON'T want)
"#;

        // 사용자 프롬프트 - 질문 + 테이블 데이터
        let user_prompt = format!(
            r#"<user_question>
{}
</user_question>

<table_data>
조회 요약: {}

데이터:
{}
</table_data>

위 테이블 데이터를 분석하여 사용자의 질문에 정확하게 답변해주세요.
데이터에서 관련 정보를 찾아 총합, 평균, 건수 등을 계산하여 자연스러운 한국어로 답변하세요."#,
            sanitize_for_xml(message),
            sanitize_for_xml(table_summary),
            table_data_json
        );

        // Claude API 호출
        let request_body = json!({
            "model": "claude-sonnet-4-5-20250929",
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.3,  // 데이터 분석은 정확성 우선
            "max_tokens": 8192
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
        println!("📥 [generate_response_from_table_data] Response status: {}", status);

        if !status.is_success() {
            let error_text = response.text().await?;
            eprintln!("❌ [generate_response_from_table_data] Claude API error ({}): {}", status, error_text);
            anyhow::bail!("Claude API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await?;
        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in Claude response"))?;

        let clean_content = strip_markdown_code_block(content);

        println!("✅ [generate_response_from_table_data] Natural language response generated: {}",
            if clean_content.chars().count() > 100 {
                format!("{}...", clean_content.chars().take(100).collect::<String>())
            } else {
                clean_content.to_string()
            }
        );

        Ok(clean_content.to_string())
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
            "max_tokens": 8192
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

    // ========================================================================
    // Phase 9-2: AI Workflow Generator
    // ========================================================================

    /// 워크플로우 생성 요청 (자연어 → JSON 워크플로우)
    ///
    /// # Arguments
    /// * `system_prompt` - Manufacturing DSL 가이드
    /// * `user_prompt` - 사용자 자연어 입력
    ///
    /// # Returns
    /// * `Ok(String)` - JSON 워크플로우 문자열 (markdown 제거됨)
    /// * `Err(anyhow::Error)` - API 호출 또는 파싱 에러
    ///
    /// # Example
    /// ```rust
    /// let workflow_json = chat_service.generate_workflow_from_prompt(
    ///     &system_prompt,
    ///     "1호선 불량률 3% 초과시 알림"
    /// ).await?;
    /// ```
    pub async fn generate_workflow_from_prompt(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let request_body = json!({
            "model": "claude-sonnet-4-5-20250929",
            "system": system_prompt,
            "messages": [
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.3,  // 정확한 JSON 생성을 위해 낮은 temperature
            "max_tokens": 8192   // 긴 워크플로우 대응
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
        if !status.is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Claude API error ({}): {}", status, error_text);
        }

        let response_json: serde_json::Value = response.json().await?;
        let content = response_json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content in Claude response"))?;

        // Markdown code block 제거
        let clean_content = strip_markdown_code_block(content);
        Ok(clean_content.to_string())
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
        if service.claude_api_key == "sk-ant-test-key" {
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
                assert!(e.to_string().contains("Claude") || e.to_string().contains("API"));
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
                    e.to_string().contains("Claude") ||
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
        if service.claude_api_key == "sk-ant-test-key" {
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
                assert!(e.to_string().contains("Claude") || e.to_string().contains("API"));
            }
        }
    }

    #[tokio::test]
    async fn test_extract_workflow_params() {
        let service = ChatService::new().unwrap();

        // API 키가 없으면 테스트 스킵
        if service.claude_api_key == "sk-ant-test-key" {
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
                assert!(e.to_string().contains("Claude") || e.to_string().contains("API"));
            }
        }
    }
}

// ========================================================================
