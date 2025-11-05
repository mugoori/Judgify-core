// services/complexity_analyzer.rs - MCP Conditional Activation
//
// Purpose: Analyze judgment complexity to optimize MCP usage and reduce token costs
// Strategy:
//   - Simple: Rule-only (MCP OFF) → 0 tokens, <10ms
//   - Medium: Rule + LLM (Context7 OFF) → 2K tokens, <500ms
//   - Complex: Full MCP (Context7 ON) → 5K tokens, <2,000ms

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::database::models::Workflow;

/// Judgment complexity levels for MCP activation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JudgmentComplexity {
    /// Rule-only execution (MCP disabled)
    /// - Cost: 0 tokens
    /// - Target: <10ms response time
    /// - Use case: Clear rule exists + high historical confidence
    Simple,

    /// Rule + LLM execution (Context7 disabled)
    /// - Cost: ~2,000 tokens (OpenAI only)
    /// - Target: <500ms response time
    /// - Use case: Rule exists but moderate confidence OR simple data structure
    Medium,

    /// Full MCP execution (Context7 enabled)
    /// - Cost: ~5,000 tokens (Context7 + OpenAI)
    /// - Target: <2,000ms response time
    /// - Use case: No rule OR low confidence OR complex nested data
    Complex,
}

impl JudgmentComplexity {
    /// Get expected token usage for this complexity level
    pub fn expected_tokens(&self) -> usize {
        match self {
            Self::Simple => 0,
            Self::Medium => 2000,
            Self::Complex => 5000,
        }
    }

    /// Get target response time in milliseconds
    pub fn target_response_ms(&self) -> u64 {
        match self {
            Self::Simple => 10,
            Self::Medium => 500,
            Self::Complex => 2000,
        }
    }

    /// Convert to string for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Medium => "medium",
            Self::Complex => "complex",
        }
    }
}

/// Complexity analyzer for judgment decisions
pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
    /// Analyze judgment complexity based on workflow and input data
    ///
    /// Decision criteria (in order):
    /// 1. Rule expression exists? → Check confidence
    /// 2. Historical confidence ≥ 0.8? → Simple
    /// 3. Historical confidence ≥ 0.6? → Medium
    /// 4. Input JSON depth > 2? → Complex
    /// 5. Input field count > 5? → Medium/Complex
    /// 6. Default: Complex (play it safe)
    pub fn analyze(
        workflow: &Workflow,
        input_data: &Value,
        historical_confidence: Option<f64>,
    ) -> JudgmentComplexity {
        // Criterion 1: No rule defined → Always Complex
        if workflow.rule_expression.is_none() {
            return JudgmentComplexity::Complex;
        }

        // Criterion 2: Very low confidence → Complex (need full MCP assistance)
        if let Some(confidence) = historical_confidence {
            if confidence < 0.6 {
                return JudgmentComplexity::Complex;
            }
        }

        // Criterion 3: Analyze input data structure complexity
        let json_depth = Self::calculate_json_depth(input_data);
        let field_count = Self::count_fields(input_data);

        // Deep nesting → Complex (likely needs Context7 documentation)
        if json_depth > 2 {
            return JudgmentComplexity::Complex;
        }

        // Many fields + moderate confidence → Medium
        if field_count > 5 {
            if let Some(confidence) = historical_confidence {
                if confidence >= 0.6 && confidence < 0.8 {
                    return JudgmentComplexity::Medium;
                }
            }
        }

        // High confidence + rule exists → Simple
        if let Some(confidence) = historical_confidence {
            if confidence >= 0.8 {
                return JudgmentComplexity::Simple;
            }
        }

        // Default: Medium (safe middle ground)
        JudgmentComplexity::Medium
    }

    /// Calculate maximum nesting depth of JSON object
    ///
    /// Examples:
    /// - `{"a": 1}` → depth 1
    /// - `{"a": {"b": 2}}` → depth 2
    /// - `{"a": {"b": {"c": 3}}}` → depth 3
    fn calculate_json_depth(value: &Value) -> usize {
        match value {
            Value::Object(obj) => {
                if obj.is_empty() {
                    1
                } else {
                    1 + obj.values()
                        .map(|v| Self::calculate_json_depth(v))
                        .max()
                        .unwrap_or(0)
                }
            }
            Value::Array(arr) => {
                if arr.is_empty() {
                    1
                } else {
                    1 + arr.iter()
                        .map(|v| Self::calculate_json_depth(v))
                        .max()
                        .unwrap_or(0)
                }
            }
            _ => 0, // Primitive values don't add to depth (base case)
        }
    }

    /// Count total number of fields in JSON object (flattened)
    ///
    /// Examples:
    /// - `{"a": 1, "b": 2}` → 2 fields
    /// - `{"a": {"b": 2, "c": 3}}` → 3 fields (a, b, c)
    fn count_fields(value: &Value) -> usize {
        match value {
            Value::Object(obj) => {
                obj.len() + obj.values()
                    .map(|v| Self::count_fields(v))
                    .sum::<usize>()
            }
            Value::Array(arr) => {
                arr.iter()
                    .map(|v| Self::count_fields(v))
                    .sum::<usize>()
            }
            _ => 0, // Primitives don't count as fields
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_workflow(has_rule: bool) -> Workflow {
        Workflow {
            id: "test-workflow".to_string(),
            name: "Test Workflow".to_string(),
            definition: r#"{"nodes":[],"edges":[]}"#.to_string(),
            rule_expression: if has_rule {
                Some("temperature > 85 && vibration < 50".to_string())
            } else {
                None
            },
            version: 1,
            is_active: true,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_simple_case_high_confidence() {
        let workflow = create_test_workflow(true);
        let input = serde_json::json!({"temperature": 90, "vibration": 45});
        let confidence = Some(0.95);

        let complexity = ComplexityAnalyzer::analyze(&workflow, &input, confidence);

        assert_eq!(complexity, JudgmentComplexity::Simple);
        assert_eq!(complexity.expected_tokens(), 0);
        assert_eq!(complexity.target_response_ms(), 10);
    }

    #[test]
    fn test_medium_case_moderate_confidence() {
        let workflow = create_test_workflow(true);
        let input = serde_json::json!({"temperature": 90, "vibration": 45});
        let confidence = Some(0.7);

        let complexity = ComplexityAnalyzer::analyze(&workflow, &input, confidence);

        assert_eq!(complexity, JudgmentComplexity::Medium);
        assert_eq!(complexity.expected_tokens(), 2000);
    }

    #[test]
    fn test_complex_case_no_rule() {
        let workflow = create_test_workflow(false);
        let input = serde_json::json!({"temperature": 90});
        let confidence = Some(0.9); // Even high confidence won't help

        let complexity = ComplexityAnalyzer::analyze(&workflow, &input, confidence);

        assert_eq!(complexity, JudgmentComplexity::Complex);
        assert_eq!(complexity.expected_tokens(), 5000);
    }

    #[test]
    fn test_complex_case_low_confidence() {
        let workflow = create_test_workflow(true);
        let input = serde_json::json!({"temperature": 90});
        let confidence = Some(0.5); // Below 0.6 threshold

        let complexity = ComplexityAnalyzer::analyze(&workflow, &input, confidence);

        assert_eq!(complexity, JudgmentComplexity::Complex);
    }

    #[test]
    fn test_complex_case_deep_nesting() {
        let workflow = create_test_workflow(true);
        let input = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "value": 123
                    }
                }
            }
        });
        let confidence = Some(0.9); // Even high confidence won't help

        let complexity = ComplexityAnalyzer::analyze(&workflow, &input, confidence);

        assert_eq!(complexity, JudgmentComplexity::Complex);
    }

    #[test]
    fn test_json_depth_calculation() {
        // Depth 1: Flat object
        let depth1 = serde_json::json!({"a": 1, "b": 2});
        assert_eq!(ComplexityAnalyzer::calculate_json_depth(&depth1), 1);

        // Depth 2: Nested once
        let depth2 = serde_json::json!({"a": {"b": 2}});
        assert_eq!(ComplexityAnalyzer::calculate_json_depth(&depth2), 2);

        // Depth 3: Nested twice
        let depth3 = serde_json::json!({"a": {"b": {"c": 3}}});
        assert_eq!(ComplexityAnalyzer::calculate_json_depth(&depth3), 3);

        // Array nesting
        let arr_depth = serde_json::json!({"items": [{"nested": 1}]});
        assert_eq!(ComplexityAnalyzer::calculate_json_depth(&arr_depth), 3);
    }

    #[test]
    fn test_field_counting() {
        // 2 fields
        let two_fields = serde_json::json!({"a": 1, "b": 2});
        assert_eq!(ComplexityAnalyzer::count_fields(&two_fields), 2);

        // 3 fields (a, b, c) - nested counts
        let nested_fields = serde_json::json!({"a": {"b": 2, "c": 3}});
        assert_eq!(ComplexityAnalyzer::count_fields(&nested_fields), 3);

        // Array items
        let array_fields = serde_json::json!({"items": [{"x": 1}, {"y": 2}]});
        assert_eq!(ComplexityAnalyzer::count_fields(&array_fields), 3); // items + x + y
    }

    #[test]
    fn test_medium_case_many_fields() {
        let workflow = create_test_workflow(true);
        let input = serde_json::json!({
            "field1": 1,
            "field2": 2,
            "field3": 3,
            "field4": 4,
            "field5": 5,
            "field6": 6  // More than 5 fields
        });
        let confidence = Some(0.7); // Moderate confidence

        let complexity = ComplexityAnalyzer::analyze(&workflow, &input, confidence);

        assert_eq!(complexity, JudgmentComplexity::Medium);
    }

    #[test]
    fn test_complexity_as_str() {
        assert_eq!(JudgmentComplexity::Simple.as_str(), "simple");
        assert_eq!(JudgmentComplexity::Medium.as_str(), "medium");
        assert_eq!(JudgmentComplexity::Complex.as_str(), "complex");
    }
}
