// algorithms/mod.rs - Rule 추출 알고리즘 모듈

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod frequency_analyzer;
pub mod llm_pattern_discoverer;
pub mod rule_integrator;
pub mod decision_tree_converter;

// 공통 데이터 구조

/// 추출된 Rule 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRule {
    /// Rule 표현식 (예: "temperature > 85 && vibration > 40")
    pub expression: String,

    /// 신뢰도 점수 (0.0 ~ 1.0)
    pub confidence: f64,

    /// 추출 방법 ("frequency" | "llm" | "integrated")
    pub method: String,

    /// 피처 중요도 (선택적)
    pub feature_importance: Option<HashMap<String, f64>>,

    /// 지원 샘플 수
    pub support_count: usize,

    /// 총 샘플 수
    pub total_count: usize,
}

/// 피드백 데이터 (알고리즘 입력)
#[derive(Debug, Clone)]
pub struct FeedbackData {
    /// 입력 데이터 (JSON 문자열)
    pub input_json: String,

    /// 긍정 피드백 여부 (true: 👍, false: 👎)
    pub is_positive: bool,

    /// 실제 판단 결과
    pub judgment_result: bool,

    /// 판단 ID (추적용)
    pub judgment_id: String,
}

/// 알고리즘 실행 결과
#[derive(Debug)]
pub struct AlgorithmResult {
    pub rules: Vec<ExtractedRule>,
    pub execution_time_ms: u64,
    pub algorithm_name: String,
}

impl ExtractedRule {
    /// 새 ExtractedRule 생성
    pub fn new(expression: String, confidence: f64, method: String) -> Self {
        ExtractedRule {
            expression,
            confidence,
            method,
            feature_importance: None,
            support_count: 0,
            total_count: 0,
        }
    }

    /// 신뢰도가 임계값 이상인지 확인
    pub fn is_confident(&self, threshold: f64) -> bool {
        self.confidence >= threshold
    }

    /// 지원율 계산 (support_count / total_count)
    pub fn support_ratio(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.support_count as f64 / self.total_count as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extracted_rule_creation() {
        let rule = ExtractedRule::new(
            "temperature > 85".to_string(),
            0.85,
            "frequency".to_string(),
        );

        assert_eq!(rule.expression, "temperature > 85");
        assert_eq!(rule.confidence, 0.85);
        assert_eq!(rule.method, "frequency");
        assert!(rule.is_confident(0.7));
        assert!(!rule.is_confident(0.9));
    }

    #[test]
    fn test_support_ratio() {
        let mut rule = ExtractedRule::new(
            "vibration > 40".to_string(),
            0.8,
            "frequency".to_string(),
        );
        rule.support_count = 80;
        rule.total_count = 100;

        assert_eq!(rule.support_ratio(), 0.8);
    }
}
