use serde::{Deserialize, Serialize};

/// Ver2.0 10개 노드 타입 정의 (TypeScript NodeType enum과 동기화)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    // v1 호환 타입 (기존 5개)
    Input,
    Decision,
    Action,
    Output,
    Notification,

    // v2 신규 타입 (추가 5개)
    DataInput,
    RuleJudgment,
    LlmJudgment,
    ActionExecution,
    DataAggregation,
}

impl NodeType {
    /// TypeScript 노드 타입 문자열을 Rust enum으로 변환
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            // v1 호환 (kebab-case, snake_case, lowercase 모두 지원)
            "input" | "data-input" => Some(Self::Input),
            "decision" | "condition" => Some(Self::Decision),
            "action" => Some(Self::Action),
            "output" | "data-output" => Some(Self::Output),
            "notification" => Some(Self::Notification),

            // v2 신규 타입
            "data_input" => Some(Self::DataInput),
            "rule_judgment" | "rule-judgment" => Some(Self::RuleJudgment),
            "llm_judgment" | "llm-judgment" => Some(Self::LlmJudgment),
            "action_execution" | "action-execution" => Some(Self::ActionExecution),
            "data_aggregation" | "data-aggregation" => Some(Self::DataAggregation),

            _ => None,
        }
    }

    /// Rust enum을 TypeScript 호환 문자열로 변환
    pub fn to_string(&self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Decision => "decision",
            Self::Action => "action",
            Self::Output => "output",
            Self::Notification => "notification",
            Self::DataInput => "data_input",
            Self::RuleJudgment => "rule_judgment",
            Self::LlmJudgment => "llm_judgment",
            Self::ActionExecution => "action_execution",
            Self::DataAggregation => "data_aggregation",
        }
    }
}

/// 노드 설정 데이터 (JSON으로 직렬화/역직렬화)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    // Rule 관련 설정
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,

    // Action 관련 설정
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,

    // Notification 관련 설정
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    // LLM 관련 설정
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    // 데이터 집계 관련 설정
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation_type: Option<String>, // sum, avg, count, min, max
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_from_str() {
        assert_eq!(NodeType::from_str("input"), Some(NodeType::Input));
        assert_eq!(NodeType::from_str("data-input"), Some(NodeType::Input));
        assert_eq!(NodeType::from_str("decision"), Some(NodeType::Decision));
        assert_eq!(NodeType::from_str("rule_judgment"), Some(NodeType::RuleJudgment));
        assert_eq!(NodeType::from_str("llm-judgment"), Some(NodeType::LlmJudgment));
        assert_eq!(NodeType::from_str("unknown"), None);
    }

    #[test]
    fn test_node_type_to_string() {
        assert_eq!(NodeType::Input.to_string(), "input");
        assert_eq!(NodeType::RuleJudgment.to_string(), "rule_judgment");
        assert_eq!(NodeType::DataAggregation.to_string(), "data_aggregation");
    }

    #[test]
    fn test_node_config_serialization() {
        let config = NodeConfig {
            label: "Test Node".to_string(),
            node_type: "rule_judgment".to_string(),
            description: Some("Test rule evaluation".to_string()),
            rule: Some("temperature > 80".to_string()),
            action: None,
            channel: None,
            message: None,
            prompt: None,
            model: None,
            aggregation_type: None,
            field: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"label\":\"Test Node\""));
        assert!(json.contains("\"rule\":\"temperature > 80\""));

        // 역직렬화 검증
        let deserialized: NodeConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.label, "Test Node");
        assert_eq!(deserialized.rule, Some("temperature > 80".to_string()));
    }
}
