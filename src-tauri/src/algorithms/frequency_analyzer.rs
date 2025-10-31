// algorithms/frequency_analyzer.rs - 빈도 분석 기반 Rule 추출

use super::{ExtractedRule, FeedbackData};
use serde_json::Value;
use std::collections::HashMap;

/// 빈도 분석 알고리즘
///
/// 긍정 피드백 데이터에서 반복되는 조건 패턴을 찾아 Rule로 변환
/// 80% 이상 반복되는 패턴을 Rule로 추출
pub struct FrequencyAnalyzer {
    /// 패턴 추출 임계값 (기본: 0.80 = 80%)
    pub threshold: f64,

    /// 값 반올림 단위 (기본: 5.0)
    pub round_step: f64,
}

impl Default for FrequencyAnalyzer {
    fn default() -> Self {
        FrequencyAnalyzer {
            threshold: 0.80,
            round_step: 5.0,
        }
    }
}

impl FrequencyAnalyzer {
    pub fn new(threshold: f64) -> Self {
        FrequencyAnalyzer {
            threshold,
            round_step: 5.0,
        }
    }

    /// 피드백 데이터에서 Rule 추출
    pub fn extract_rules(&self, feedback_data: Vec<FeedbackData>) -> anyhow::Result<Vec<ExtractedRule>> {
        if feedback_data.is_empty() {
            return Ok(vec![]);
        }

        // 1. 긍정 피드백만 필터링
        let positive_data: Vec<_> = feedback_data
            .iter()
            .filter(|f| f.is_positive)
            .collect();

        if positive_data.is_empty() {
            return Ok(vec![]);
        }

        let total_count = positive_data.len();

        // 2. 각 데이터에서 조건 패턴 추출
        let mut pattern_counts: HashMap<String, usize> = HashMap::new();

        for data in &positive_data {
            if let Ok(conditions) = self.extract_conditions(&data.input_json) {
                for condition in conditions {
                    *pattern_counts.entry(condition).or_insert(0) += 1;
                }
            }
        }

        // 3. 임계값 이상 패턴 필터링
        let min_count = (total_count as f64 * self.threshold).ceil() as usize;

        let mut rules = Vec::new();
        for (pattern, count) in pattern_counts {
            if count >= min_count {
                let support_ratio = count as f64 / total_count as f64;
                let confidence = support_ratio * 0.9; // 과적합 방지

                let mut rule = ExtractedRule::new(
                    pattern.clone(),
                    confidence,
                    "frequency".to_string(),
                );
                rule.support_count = count;
                rule.total_count = total_count;

                rules.push(rule);
            }
        }

        // 4. 신뢰도 순으로 정렬
        rules.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(rules)
    }

    /// JSON 데이터에서 조건 추출
    fn extract_conditions(&self, input_json: &str) -> anyhow::Result<Vec<String>> {
        let value: Value = serde_json::from_str(input_json)?;

        if !value.is_object() {
            return Ok(vec![]);
        }

        let mut conditions = Vec::new();

        // 객체의 모든 키-값 쌍을 순회
        if let Some(obj) = value.as_object() {
            for (key, val) in obj {
                // 숫자 값만 처리 (비교 조건 생성 가능)
                if let Some(num) = val.as_f64() {
                    // 값 반올림 후 임계값 계산
                    let threshold = self.calculate_threshold(num);

                    // 조건 생성: "key > threshold"
                    let condition = format!("{} > {}", key, threshold);
                    conditions.push(condition);
                }
                // 불리언 값 처리
                else if let Some(bool_val) = val.as_bool() {
                    let condition = if bool_val {
                        format!("{} == true", key)
                    } else {
                        format!("{} == false", key)
                    };
                    conditions.push(condition);
                }
                // 문자열 값 처리
                else if let Some(str_val) = val.as_str() {
                    let condition = format!("{} == \"{}\"", key, str_val);
                    conditions.push(condition);
                }
            }
        }

        Ok(conditions)
    }

    /// 값 반올림 후 임계값 계산
    /// 예: 88 → 85 (5단위 반올림 후 -5)
    fn calculate_threshold(&self, value: f64) -> f64 {
        let rounded = (value / self.round_step).round() * self.round_step;
        rounded - self.round_step
    }

    /// 조건들을 AND로 결합하여 복합 Rule 생성
    pub fn combine_conditions(&self, conditions: Vec<String>) -> String {
        if conditions.is_empty() {
            return "true".to_string();
        }

        if conditions.len() == 1 {
            return conditions[0].clone();
        }

        conditions.join(" && ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_conditions_numeric() {
        let analyzer = FrequencyAnalyzer::default();
        let json = r#"{"temperature": 90, "vibration": 45}"#;

        let conditions = analyzer.extract_conditions(json).unwrap();

        assert_eq!(conditions.len(), 2);
        assert!(conditions.contains(&"temperature > 85".to_string()));
        assert!(conditions.contains(&"vibration > 40".to_string()));
    }

    #[test]
    fn test_extract_conditions_boolean() {
        let analyzer = FrequencyAnalyzer::default();
        let json = r#"{"is_critical": true, "is_normal": false}"#;

        let conditions = analyzer.extract_conditions(json).unwrap();

        assert!(conditions.contains(&"is_critical == true".to_string()));
        assert!(conditions.contains(&"is_normal == false".to_string()));
    }

    #[test]
    fn test_extract_conditions_string() {
        let analyzer = FrequencyAnalyzer::default();
        let json = r#"{"status": "error", "level": "high"}"#;

        let conditions = analyzer.extract_conditions(json).unwrap();

        assert!(conditions.contains(&"status == \"error\"".to_string()));
        assert!(conditions.contains(&"level == \"high\"".to_string()));
    }

    #[test]
    fn test_calculate_threshold() {
        let analyzer = FrequencyAnalyzer::default();

        // 88 → round(88/5)*5 = 90 → 90-5 = 85
        assert_eq!(analyzer.calculate_threshold(88.0), 85.0);
        // 92 → round(92/5)*5 = 90 → 90-5 = 85
        assert_eq!(analyzer.calculate_threshold(92.0), 85.0);
        // 43 → round(43/5)*5 = 45 → 45-5 = 40
        assert_eq!(analyzer.calculate_threshold(43.0), 40.0);
    }

    #[test]
    fn test_extract_rules_with_threshold() {
        let analyzer = FrequencyAnalyzer::new(0.80);

        let feedback_data = vec![
            FeedbackData {
                input_json: r#"{"temperature": 90}"#.to_string(),
                is_positive: true,
                judgment_result: true,
                judgment_id: "1".to_string(),
            },
            FeedbackData {
                input_json: r#"{"temperature": 88}"#.to_string(),
                is_positive: true,
                judgment_result: true,
                judgment_id: "2".to_string(),
            },
            FeedbackData {
                input_json: r#"{"temperature": 92}"#.to_string(),
                is_positive: true,
                judgment_result: true,
                judgment_id: "3".to_string(),
            },
            FeedbackData {
                input_json: r#"{"temperature": 89}"#.to_string(),
                is_positive: true,
                judgment_result: true,
                judgment_id: "4".to_string(),
            },
            FeedbackData {
                input_json: r#"{"temperature": 91}"#.to_string(),
                is_positive: true,
                judgment_result: true,
                judgment_id: "5".to_string(),
            },
        ];

        let rules = analyzer.extract_rules(feedback_data).unwrap();

        assert!(!rules.is_empty());
        assert_eq!(rules[0].expression, "temperature > 85");
        assert!(rules[0].confidence > 0.7);
        assert_eq!(rules[0].support_count, 5);
        assert_eq!(rules[0].total_count, 5);
    }

    #[test]
    fn test_combine_conditions() {
        let analyzer = FrequencyAnalyzer::default();

        let conditions = vec![
            "temperature > 85".to_string(),
            "vibration > 40".to_string(),
        ];

        let combined = analyzer.combine_conditions(conditions);
        assert_eq!(combined, "temperature > 85 && vibration > 40");
    }

    #[test]
    fn test_empty_feedback() {
        let analyzer = FrequencyAnalyzer::default();
        let rules = analyzer.extract_rules(vec![]).unwrap();
        assert!(rules.is_empty());
    }

    #[test]
    fn test_no_positive_feedback() {
        let analyzer = FrequencyAnalyzer::default();

        let feedback_data = vec![
            FeedbackData {
                input_json: r#"{"temperature": 90}"#.to_string(),
                is_positive: false,
                judgment_result: false,
                judgment_id: "1".to_string(),
            },
        ];

        let rules = analyzer.extract_rules(feedback_data).unwrap();
        assert!(rules.is_empty());
    }
}
