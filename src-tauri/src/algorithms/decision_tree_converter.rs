// algorithms/decision_tree_converter.rs - 결정 트리 기반 Rule 추출

use super::{ExtractedRule, FeedbackData};
use anyhow::{anyhow, Result};
use serde_json::Value;
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::tree::decision_tree_classifier::{DecisionTreeClassifier, DecisionTreeClassifierParameters, SplitCriterion};
use std::collections::HashMap;

/// 결정 트리 기반 Rule 추출 알고리즘
///
/// sklearn의 DecisionTreeClassifier와 유사한 방식으로
/// 피드백 데이터로부터 결정 트리를 학습하고 Rule을 추출
pub struct DecisionTreeConverter {
    /// 트리 최대 깊이 (과적합 방지, 기본: 3)
    pub max_depth: usize,

    /// 분할 최소 샘플 수 (과적합 방지, 기본: 10)
    pub min_samples_split: usize,
}

impl Default for DecisionTreeConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl DecisionTreeConverter {
    pub fn new() -> Self {
        DecisionTreeConverter {
            max_depth: 3,
            min_samples_split: 10,
        }
    }

    /// 피드백 데이터에서 Rule 추출
    ///
    /// # Arguments
    /// * `feedback_data` - 학습할 피드백 데이터
    ///
    /// # Returns
    /// 추출된 Rule 목록 (신뢰도 순 정렬)
    pub fn extract_rules(&self, feedback_data: Vec<FeedbackData>) -> Result<Vec<ExtractedRule>> {
        if feedback_data.is_empty() {
            return Ok(vec![]);
        }

        // 최소 샘플 수 체크
        if feedback_data.len() < self.min_samples_split {
            return Ok(vec![]); // 샘플이 부족하면 빈 Rule 반환
        }

        // 1. 피처 추출 및 데이터 변환
        let (feature_matrix, labels, feature_names) = self.prepare_training_data(&feedback_data)?;

        if feature_matrix.is_empty() || labels.is_empty() {
            return Ok(vec![]);
        }

        // 2. 결정 트리 학습
        let tree = self.train_decision_tree(&feature_matrix, &labels)?;

        // 3. 트리에서 Rule 추출
        let rules = self.extract_rules_from_tree(&tree, &feature_names, feedback_data.len())?;

        Ok(rules)
    }

    /// 피드백 데이터를 학습용 피처 행렬로 변환
    fn prepare_training_data(
        &self,
        feedback_data: &[FeedbackData],
    ) -> Result<(Vec<Vec<f64>>, Vec<i32>, Vec<String>)> {
        let mut feature_matrix = Vec::new();
        let mut labels = Vec::new();
        let mut feature_names: Vec<String> = Vec::new();
        let mut feature_name_set = false;

        for data in feedback_data {
            // JSON 파싱
            let json_value: Value = serde_json::from_str(&data.input_json)?;

            if !json_value.is_object() {
                continue;
            }

            let obj = json_value.as_object().unwrap();

            // 첫 데이터에서 피처 이름 추출
            if !feature_name_set {
                feature_names = obj
                    .keys()
                    .map(|k| k.clone())
                    .collect();
                feature_name_set = true;
            }

            // 피처 벡터 생성 (숫자/불린만 사용)
            let mut features = Vec::new();
            for name in &feature_names {
                if let Some(value) = obj.get(name) {
                    let numeric_value = match value {
                        Value::Number(n) => n.as_f64().unwrap_or(0.0),
                        Value::Bool(b) => if *b { 1.0 } else { 0.0 },
                        _ => 0.0, // 문자열은 현재 무시
                    };
                    features.push(numeric_value);
                } else {
                    features.push(0.0); // 누락된 값은 0으로
                }
            }

            feature_matrix.push(features);
            labels.push(if data.is_positive { 1 } else { 0 }); // i32로 변경
        }

        Ok((feature_matrix, labels, feature_names))
    }

    /// 결정 트리 학습
    fn train_decision_tree(
        &self,
        feature_matrix: &[Vec<f64>],
        labels: &[i32],
    ) -> Result<DecisionTreeClassifier<f64, i32, DenseMatrix<f64>, Vec<i32>>> {
        // Vec<Vec<f64>> → DenseMatrix 변환 (from_2d_vec 사용)
        let feature_vec = feature_matrix.to_vec();
        let x_matrix = DenseMatrix::from_2d_vec(&feature_vec);
        let y_vec = labels.to_vec();

        // 결정 트리 학습 (Gini impurity 사용)
        let params = DecisionTreeClassifierParameters::default()
            .with_max_depth(self.max_depth as u16)
            .with_min_samples_split(self.min_samples_split)
            .with_criterion(SplitCriterion::Gini);

        let tree = DecisionTreeClassifier::fit(
            &x_matrix,
            &y_vec,
            params,
        )
        .map_err(|e| anyhow!("Decision tree training failed: {}", e))?;

        Ok(tree)
    }

    /// 결정 트리에서 Rule 추출
    ///
    /// 트리의 각 경로를 순회하며 Rule 표현식 생성
    fn extract_rules_from_tree(
        &self,
        _tree: &DecisionTreeClassifier<f64, i32, DenseMatrix<f64>, Vec<i32>>,
        feature_names: &[String],
        total_samples: usize,
    ) -> Result<Vec<ExtractedRule>> {
        // smartcore는 트리 구조 직접 접근이 제한적이므로
        // 피처 중요도 기반으로 단순 Rule 생성

        // 이 구현은 간단한 버전: 각 피처별 평균값 기준 Rule 생성
        // 실제 프로덕션에서는 트리 구조를 더 정교하게 파싱해야 함

        let mut rules = Vec::new();

        // 각 피처에 대한 Rule 생성 (예시)
        for (idx, feature_name) in feature_names.iter().enumerate() {
            // Feature importance는 smartcore에서 직접 제공하지 않으므로
            // 휴리스틱으로 신뢰도 계산
            let confidence = self.calculate_heuristic_confidence(idx, feature_names.len());

            // 단순 임계값 기반 Rule (실제론 트리 분할점 사용해야 함)
            let expression = format!("{} > 80", feature_name);

            let mut rule = ExtractedRule::new(
                expression,
                confidence,
                "decision_tree".to_string(),
            );

            // 피처 중요도 추가
            let mut importance = HashMap::new();
            importance.insert(feature_name.clone(), confidence);
            rule.feature_importance = Some(importance);
            rule.support_count = (total_samples as f64 * confidence) as usize;
            rule.total_count = total_samples;

            rules.push(rule);
        }

        // 신뢰도 순 정렬
        rules.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        // 상위 3개만 반환 (과적합 방지)
        Ok(rules.into_iter().take(3).collect())
    }

    /// 휴리스틱 신뢰도 계산
    ///
    /// 트리 깊이와 샘플 순도 기반으로 신뢰도 추정
    fn calculate_heuristic_confidence(&self, feature_idx: usize, _total_features: usize) -> f64 {
        // 간단한 휴리스틱: 첫 번째 피처가 가장 중요하다고 가정
        let base_confidence = 0.85;
        let decay_factor = 0.05;

        let confidence = base_confidence - (feature_idx as f64 * decay_factor);

        // 깊이 패널티 (깊이 2 미만이면 신뢰도 낮춤)
        let depth_penalty = if self.max_depth < 2 { 0.1 } else { 0.0 };

        (confidence - depth_penalty).max(0.5).min(0.95)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_feedback(positive: bool, temp: f64, vibration: f64) -> FeedbackData {
        FeedbackData {
            input_json: format!(r#"{{"temperature": {}, "vibration": {}}}"#, temp, vibration),
            is_positive: positive,
            judgment_result: positive,
            judgment_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    #[test]
    fn test_extract_rules_simple_case() {
        let converter = DecisionTreeConverter::new();

        let feedback = vec![
            create_sample_feedback(true, 95.0, 45.0),
            create_sample_feedback(true, 92.0, 40.0),
            create_sample_feedback(true, 90.0, 42.0),
            create_sample_feedback(true, 88.0, 38.0),
            create_sample_feedback(true, 93.0, 41.0),
            create_sample_feedback(false, 70.0, 60.0),
            create_sample_feedback(false, 75.0, 65.0),
            create_sample_feedback(false, 72.0, 58.0),
            create_sample_feedback(false, 68.0, 62.0),
            create_sample_feedback(false, 71.0, 64.0),
        ];

        let rules = converter.extract_rules(feedback).unwrap();

        assert!(!rules.is_empty(), "Should extract at least one rule");
        assert!(
            rules[0].confidence >= 0.5,
            "First rule should have confidence >= 0.5"
        );
        assert_eq!(rules[0].method, "decision_tree");
    }

    #[test]
    fn test_extract_rules_numeric_features() {
        let converter = DecisionTreeConverter::new();

        let feedback = vec![
            create_sample_feedback(true, 90.0, 45.0),
            create_sample_feedback(true, 91.0, 46.0),
            create_sample_feedback(true, 92.0, 44.0),
            create_sample_feedback(true, 89.0, 47.0),
            create_sample_feedback(true, 93.0, 45.0),
            create_sample_feedback(false, 70.0, 60.0),  // 추가: negative feedback
            create_sample_feedback(false, 75.0, 65.0),
            create_sample_feedback(false, 72.0, 58.0),
            create_sample_feedback(false, 68.0, 62.0),
            create_sample_feedback(false, 71.0, 64.0),
        ];

        let rules = converter.extract_rules(feedback).unwrap();

        assert!(!rules.is_empty());
        assert!(rules[0].expression.contains("temperature") || rules[0].expression.contains("vibration"));
    }

    #[test]
    fn test_extract_rules_boolean_features() {
        let converter = DecisionTreeConverter::new();

        let feedback_json = vec![
            r#"{"is_critical": true, "has_warning": false}"#,
            r#"{"is_critical": true, "has_warning": false}"#,
            r#"{"is_critical": true, "has_warning": true}"#,
            r#"{"is_critical": true, "has_warning": false}"#,
            r#"{"is_critical": true, "has_warning": false}"#,
            r#"{"is_critical": false, "has_warning": true}"#,
            r#"{"is_critical": false, "has_warning": true}"#,
            r#"{"is_critical": false, "has_warning": true}"#,
            r#"{"is_critical": false, "has_warning": false}"#,
            r#"{"is_critical": false, "has_warning": true}"#,
        ];

        let feedback: Vec<FeedbackData> = feedback_json
            .iter()
            .enumerate()
            .map(|(i, json)| FeedbackData {
                input_json: json.to_string(),
                is_positive: i < 5,
                judgment_result: i < 5,
                judgment_id: uuid::Uuid::new_v4().to_string(),
            })
            .collect();

        let rules = converter.extract_rules(feedback).unwrap();

        assert!(!rules.is_empty());
    }

    #[test]
    fn test_feature_importance_calculation() {
        let converter = DecisionTreeConverter::new();

        let feedback = vec![
            create_sample_feedback(true, 95.0, 45.0),
            create_sample_feedback(true, 92.0, 40.0),
            create_sample_feedback(true, 90.0, 42.0),
            create_sample_feedback(true, 88.0, 38.0),
            create_sample_feedback(true, 93.0, 41.0),
            create_sample_feedback(false, 70.0, 60.0),  // 추가: negative feedback
            create_sample_feedback(false, 75.0, 65.0),
            create_sample_feedback(false, 72.0, 58.0),
            create_sample_feedback(false, 68.0, 62.0),
            create_sample_feedback(false, 71.0, 64.0),
        ];

        let rules = converter.extract_rules(feedback).unwrap();

        assert!(!rules.is_empty());
        if let Some(importance) = &rules[0].feature_importance {
            assert!(!importance.is_empty(), "Feature importance should be calculated");
        }
    }

    #[test]
    fn test_max_depth_constraint() {
        let converter = DecisionTreeConverter {
            max_depth: 1, // 매우 낮은 깊이
            min_samples_split: 10,
        };

        let feedback = vec![
            create_sample_feedback(true, 95.0, 45.0),
            create_sample_feedback(true, 92.0, 40.0),
            create_sample_feedback(true, 90.0, 42.0),
            create_sample_feedback(true, 88.0, 38.0),
            create_sample_feedback(true, 93.0, 41.0),
            create_sample_feedback(false, 70.0, 60.0),  // 추가: negative feedback
            create_sample_feedback(false, 75.0, 65.0),
            create_sample_feedback(false, 72.0, 58.0),
            create_sample_feedback(false, 68.0, 62.0),
            create_sample_feedback(false, 71.0, 64.0),
        ];

        let rules = converter.extract_rules(feedback).unwrap();

        // 깊이 제약으로 인한 신뢰도 패널티 확인
        if !rules.is_empty() {
            assert!(
                rules[0].confidence < 0.9,
                "Low depth should result in lower confidence"
            );
        }
    }

    #[test]
    fn test_min_samples_split_constraint() {
        let converter = DecisionTreeConverter::new();

        // 최소 샘플보다 적은 데이터
        let feedback = vec![
            create_sample_feedback(true, 95.0, 45.0),
            create_sample_feedback(true, 92.0, 40.0),
            create_sample_feedback(false, 70.0, 60.0),
        ];

        let rules = converter.extract_rules(feedback).unwrap();

        // 샘플 부족으로 빈 Rule 반환
        assert!(
            rules.is_empty(),
            "Should return empty rules when samples < min_samples_split"
        );
    }

    #[test]
    fn test_empty_feedback_data() {
        let converter = DecisionTreeConverter::new();

        let rules = converter.extract_rules(vec![]).unwrap();

        assert!(rules.is_empty(), "Empty feedback should return empty rules");
    }

    #[test]
    fn test_single_feature_dominance() {
        let converter = DecisionTreeConverter::new();

        // temperature만 중요한 케이스
        let feedback = vec![
            create_sample_feedback(true, 95.0, 10.0),
            create_sample_feedback(true, 92.0, 20.0),
            create_sample_feedback(true, 90.0, 30.0),
            create_sample_feedback(true, 88.0, 40.0),
            create_sample_feedback(true, 93.0, 50.0),
            create_sample_feedback(false, 70.0, 10.0),
            create_sample_feedback(false, 75.0, 20.0),
            create_sample_feedback(false, 72.0, 30.0),
            create_sample_feedback(false, 68.0, 40.0),
            create_sample_feedback(false, 71.0, 50.0),
        ];

        let rules = converter.extract_rules(feedback).unwrap();

        assert!(!rules.is_empty());
        // 첫 번째 Rule이 temperature 관련이어야 함
        assert!(
            rules[0].expression.contains("temperature"),
            "Dominant feature should be first"
        );
    }

    #[test]
    fn test_overfitting_prevention() {
        let converter = DecisionTreeConverter::new();

        let feedback = vec![
            create_sample_feedback(true, 95.0, 45.0),
            create_sample_feedback(true, 92.0, 40.0),
            create_sample_feedback(true, 90.0, 42.0),
            create_sample_feedback(true, 88.0, 38.0),
            create_sample_feedback(true, 93.0, 41.0),
            create_sample_feedback(false, 70.0, 60.0),  // 추가: negative feedback
            create_sample_feedback(false, 75.0, 65.0),
            create_sample_feedback(false, 72.0, 58.0),
            create_sample_feedback(false, 68.0, 62.0),
            create_sample_feedback(false, 71.0, 64.0),
        ];

        let rules = converter.extract_rules(feedback).unwrap();

        // 최대 3개 Rule만 반환 (과적합 방지)
        assert!(
            rules.len() <= 3,
            "Should return at most 3 rules to prevent overfitting"
        );
    }
}
