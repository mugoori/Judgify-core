use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Judgment {
    pub id: String,
    pub workflow_id: String,
    pub input_data: String,
    pub result: bool,
    pub confidence: f64,
    pub method_used: String,
    pub explanation: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub definition: String,
    pub rule_expression: Option<String>,
    pub version: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrainingSample {
    pub id: String,
    pub workflow_id: String,
    pub input_data: String,
    pub expected_result: bool,
    pub actual_result: Option<bool>,
    pub accuracy: Option<f64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feedback {
    pub id: String,
    pub judgment_id: String,
    pub feedback_type: String, // "thumbs_up", "thumbs_down", "comment"
    pub value: i32,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub template_type: String, // "judgment", "explanation", "insight"
    pub content: String, // Handlebars template with {{variables}}
    pub variables: String, // JSON array of variable names: ["workflow_context", "input_data"]
    pub version: i32,
    pub is_active: bool,
    pub token_limit: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenUsage {
    pub id: String,
    pub judgment_id: String,
    pub service: String, // "context7" | "openai" | "judgment"
    pub tokens_used: i32,
    pub cost_usd: f64, // Calculated based on service pricing
    pub complexity: String, // "simple" | "medium" | "complex"
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// CCP 데모용 데이터 모델 (RAG + 룰베이스 판단)
// ============================================================================

/// CCP 정책 문서 모델
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CcpDoc {
    pub id: i64,
    pub company_id: String,
    pub ccp_id: String,
    pub title: String,
    pub section_type: String, // "standard" | "monitoring" | "action"
    pub content: String,
    pub created_at: String, // ISO 8601
}

/// CCP 문서 검색 결과 (BM25 점수 포함)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CcpDocWithScore {
    pub id: i64,
    pub company_id: String,
    pub ccp_id: String,
    pub title: String,
    pub section_type: String,
    pub content: String,
    pub score: f64, // BM25 점수 (낮을수록 관련도 높음)
}

/// CCP 센서 로그 모델
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CcpSensorLog {
    pub log_id: i64,
    pub company_id: String,
    pub ccp_id: String,
    pub log_date: String, // ISO 8601 (YYYY-MM-DD)
    pub measured_value: f64,
    pub result: String, // "OK" | "NG"
    pub operator_name: Option<String>,
    pub action_taken: Option<String>,
    pub created_at: String, // ISO 8601
}

/// CCP 통계 데이터
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CcpStats {
    pub total_logs: i32,
    pub ng_count: i32,
    pub ng_rate: f64,
    pub avg_value: f64,
    pub min_value: f64,
    pub max_value: f64,
}

/// CCP 판단 결과 모델
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CcpJudgment {
    pub id: String,
    pub company_id: String,
    pub ccp_id: String,
    pub period_from: String, // ISO 8601 (YYYY-MM-DD)
    pub period_to: String,   // ISO 8601 (YYYY-MM-DD)
    pub total_logs: i32,
    pub ng_count: i32,
    pub ng_rate: f64,
    pub avg_value: f64,
    pub risk_level: String, // "LOW" | "MEDIUM" | "HIGH"
    pub rule_reason: Option<String>,
    pub llm_summary: Option<String>,
    pub evidence_docs: Option<String>, // JSON 문자열
    pub created_at: String, // ISO 8601
}

/// CCP 판단 요청 파라미터
#[derive(Debug, Serialize, Deserialize)]
pub struct CcpJudgmentRequest {
    pub company_id: String,
    pub ccp_id: String,
    pub period_from: String,
    pub period_to: String,
}

/// CCP 판단 응답 (통계 + 위험도 + AI 요약 + 증거 문서)
#[derive(Debug, Serialize, Deserialize)]
pub struct CcpJudgmentResponse {
    pub stats: CcpStats,
    pub risk_level: String,
    pub rule_reason: String,
    pub llm_summary: String,
    pub evidence_docs: Vec<CcpDocWithScore>,
    pub judgment_id: String,
}
