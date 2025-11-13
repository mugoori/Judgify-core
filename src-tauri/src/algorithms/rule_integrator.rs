// algorithms/rule_integrator.rs - Rule 통합 로직

use super::ExtractedRule;
use std::collections::HashMap;

/// Rule 통합기
///
/// 여러 알고리즘에서 추출된 Rule들을 통합하여 최적의 Rule 선택
pub struct RuleIntegrator {
    /// 알고리즘별 가중치 (기본: 균등)
    pub weights: HashMap<String, f64>,

    /// 일치 보너스 (두 알고리즘이 같은 Rule 제안시)
    pub agreement_bonus: f64,

    /// 최소 신뢰도 임계값
    pub min_confidence: f64,
}

impl Default for RuleIntegrator {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert("frequency".to_string(), 0.5);
        weights.insert("llm".to_string(), 0.5);
        weights.insert("decision_tree".to_string(), 0.4); // 구현시

        RuleIntegrator {
            weights,
            agreement_bonus: 0.05,
            min_confidence: 0.70,
        }
    }
}

impl RuleIntegrator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_weights(mut self, weights: HashMap<String, f64>) -> Self {
        self.weights = weights;
        self
    }

    pub fn with_agreement_bonus(mut self, bonus: f64) -> Self {
        self.agreement_bonus = bonus;
        self
    }

    /// 여러 알고리즘 결과를 통합하여 최적의 Rule 선택
    pub fn integrate_rules(&self, rule_sets: Vec<Vec<ExtractedRule>>) -> anyhow::Result<Option<ExtractedRule>> {
        if rule_sets.is_empty() {
            return Ok(None);
        }

        // 1. 모든 Rule을 하나의 벡터로 병합
        let mut all_rules: Vec<ExtractedRule> = rule_sets.into_iter().flatten().collect();

        if all_rules.is_empty() {
            return Ok(None);
        }

        // 2. Rule 표현식 정규화
        for rule in &mut all_rules {
            rule.expression = self.normalize_expression(&rule.expression);
        }

        // 3. 동일 표현식끼리 그룹화
        let grouped_rules = self.group_by_expression(&all_rules);

        // 4. 각 그룹별로 가중 평균 신뢰도 계산
        let mut integrated_rules: Vec<ExtractedRule> = Vec::new();

        for (expression, rules) in grouped_rules {
            let integrated = self.calculate_weighted_confidence(&expression, &rules);
            integrated_rules.push(integrated);
        }

        // 5. 최소 신뢰도 이상인 Rule만 필터링
        integrated_rules.retain(|r| r.confidence >= self.min_confidence);

        // 6. 신뢰도 순으로 정렬 후 최고 신뢰도 Rule 반환
        integrated_rules.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(integrated_rules.into_iter().next())
    }

    /// Rule 표현식 정규화
    ///
    /// 공백 정리, 일관된 형식 적용
    fn normalize_expression(&self, expression: &str) -> String {
        expression
            .trim()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }

    /// 동일 표현식끼리 그룹화
    fn group_by_expression(&self, rules: &[ExtractedRule]) -> HashMap<String, Vec<ExtractedRule>> {
        let mut groups: HashMap<String, Vec<ExtractedRule>> = HashMap::new();

        for rule in rules {
            groups
                .entry(rule.expression.clone())
                .or_insert_with(Vec::new)
                .push(rule.clone());
        }

        groups
    }

    /// 가중 평균 신뢰도 계산
    fn calculate_weighted_confidence(&self, expression: &str, rules: &[ExtractedRule]) -> ExtractedRule {
        let mut total_weighted_confidence = 0.0;
        let mut total_weight = 0.0;
        let mut total_support = 0;
        let mut total_count = 0;

        for rule in rules {
            let weight = self.weights.get(&rule.method).copied().unwrap_or(0.5);
            total_weighted_confidence += rule.confidence * weight;
            total_weight += weight;
            total_support += rule.support_count;
            total_count = total_count.max(rule.total_count);
        }

        let mut final_confidence = if total_weight > 0.0 {
            total_weighted_confidence / total_weight
        } else {
            0.0
        };

        // 일치 보너스 (2개 이상 알고리즘이 같은 Rule 제안시)
        if rules.len() >= 2 {
            final_confidence += self.agreement_bonus;
            final_confidence = final_confidence.min(1.0); // 최대 1.0
        }

        let mut integrated_rule = ExtractedRule::new(
            expression.to_string(),
            final_confidence,
            "integrated".to_string(),
        );

        integrated_rule.support_count = total_support;
        integrated_rule.total_count = total_count;

        // 피처 중요도 병합 (선택적)
        let mut merged_importance: HashMap<String, f64> = HashMap::new();
        for rule in rules {
            if let Some(importance) = &rule.feature_importance {
                for (feature, score) in importance {
                    *merged_importance.entry(feature.clone()).or_insert(0.0) += score;
                }
            }
        }

        if !merged_importance.is_empty() {
            // 평균으로 정규화
            for value in merged_importance.values_mut() {
                *value /= rules.len() as f64;
            }
            integrated_rule.feature_importance = Some(merged_importance);
        }

        integrated_rule
    }

    /// 두 Rule 세트에서 최적 Rule 선택 (편의 함수)
    pub fn integrate_two(
        &self,
        rules1: Vec<ExtractedRule>,
        rules2: Vec<ExtractedRule>,
    ) -> anyhow::Result<Option<ExtractedRule>> {
        self.integrate_rules(vec![rules1, rules2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_expression() {
        let integrator = RuleIntegrator::new();

        let expr1 = "Temperature  >  85  &&  Vibration  <  50";
        let normalized = integrator.normalize_expression(expr1);

        assert_eq!(normalized, "temperature > 85 && vibration < 50");
    }

    #[test]
    fn test_group_by_expression() {
        let integrator = RuleIntegrator::new();

        let rules = vec![
            ExtractedRule::new("temp > 85".to_string(), 0.8, "frequency".to_string()),
            ExtractedRule::new("temp > 85".to_string(), 0.85, "llm".to_string()),
            ExtractedRule::new("vib > 40".to_string(), 0.75, "frequency".to_string()),
        ];

        let groups = integrator.group_by_expression(&rules);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("temp > 85").unwrap().len(), 2);
        assert_eq!(groups.get("vib > 40").unwrap().len(), 1);
    }

    #[test]
    fn test_calculate_weighted_confidence() {
        let integrator = RuleIntegrator::new();

        let rules = vec![
            ExtractedRule::new("temp > 85".to_string(), 0.8, "frequency".to_string()),
            ExtractedRule::new("temp > 85".to_string(), 0.9, "llm".to_string()),
        ];

        let integrated = integrator.calculate_weighted_confidence("temp > 85", &rules);

        // (0.8 * 0.5 + 0.9 * 0.5) / 1.0 + 0.05 (agreement bonus) = 0.9
        assert!((integrated.confidence - 0.9).abs() < 0.01);
        assert_eq!(integrated.method, "integrated");
    }

    #[test]
    fn test_integrate_rules() {
        let integrator = RuleIntegrator::new();

        let freq_rules = vec![
            ExtractedRule::new("temp > 85".to_string(), 0.8, "frequency".to_string()),
            ExtractedRule::new("vib > 40".to_string(), 0.75, "frequency".to_string()),
        ];

        let llm_rules = vec![
            ExtractedRule::new("temp > 85".to_string(), 0.9, "llm".to_string()),
        ];

        let result = integrator.integrate_rules(vec![freq_rules, llm_rules]).unwrap();

        assert!(result.is_some());
        let best_rule = result.unwrap();
        assert_eq!(best_rule.expression, "temp > 85");
        assert!(best_rule.confidence >= 0.85); // 가중 평균 + 보너스
    }

    #[test]
    fn test_integrate_two() {
        let integrator = RuleIntegrator::new();

        let freq_rules = vec![
            ExtractedRule::new("temp > 85".to_string(), 0.8, "frequency".to_string()),
        ];

        let llm_rules = vec![
            ExtractedRule::new("vib > 40".to_string(), 0.85, "llm".to_string()),
        ];

        let result = integrator.integrate_two(freq_rules, llm_rules).unwrap();

        assert!(result.is_some());
        let best_rule = result.unwrap();
        assert!(best_rule.confidence >= 0.80);
    }

    #[test]
    fn test_empty_rules() {
        let integrator = RuleIntegrator::new();

        let result = integrator.integrate_rules(vec![]).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_min_confidence_filter() {
        let mut integrator = RuleIntegrator::new();
        integrator.min_confidence = 0.9; // 높은 임계값

        let rules = vec![
            ExtractedRule::new("temp > 85".to_string(), 0.8, "frequency".to_string()),
        ];

        let result = integrator.integrate_rules(vec![rules]).unwrap();
        assert!(result.is_none()); // 0.8 < 0.9이므로 필터링됨
    }

    #[test]
    fn test_agreement_bonus() {
        let integrator = RuleIntegrator::new();

        let rules = vec![
            ExtractedRule::new("temp > 85".to_string(), 0.8, "frequency".to_string()),
            ExtractedRule::new("temp > 85".to_string(), 0.8, "llm".to_string()),
        ];

        let integrated = integrator.calculate_weighted_confidence("temp > 85", &rules);

        // (0.8 * 0.5 + 0.8 * 0.5) / 1.0 + 0.05 = 0.85
        assert!((integrated.confidence - 0.85).abs() < 0.001);
    }
}
