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
