// algorithms/llm_pattern_discoverer.rs - LLM 기반 패턴 발견

use super::{ExtractedRule, FeedbackData};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// LLM 패턴 발견 알고리즘
///
/// ✅ Phase 2: Claude API를 사용하여 복잡한 패턴 발견 및 Rule 추출 (OpenAI에서 마이그레이션)
pub struct LLMPatternDiscoverer {
    /// Claude API 키
    api_key: String,

    /// 사용할 모델 (기본: claude-sonnet-4-5-20250929)
    model: String,

    /// HTTP 클라이언트
    client: Client,
}

/// 데이터 집계 결과
#[derive(Debug, Serialize)]
struct AggregatedSummary {
    positive_count: usize,
    negative_count: usize,
    positive_avg: HashMap<String, f64>,
    negative_avg: HashMap<String, f64>,
    positive_std: HashMap<String, f64>,
    correlations: HashMap<String, f64>,
}

/// ✅ Phase 2: Claude API 요청 구조
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    temperature: f64,
}

#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

/// ✅ Phase 2: Claude API 응답 구조
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug, Deserialize)]
struct ClaudeContent {
    text: String,
}

/// LLM이 반환하는 Rule 구조
#[derive(Debug, Deserialize)]
struct LLMRuleResponse {
    rules: Vec<LLMRule>,
    analysis: String,
}

#[derive(Debug, Deserialize)]
struct LLMRule {
    expression: String,
    confidence: f64,
    reasoning: String,
}

impl LLMPatternDiscoverer {
    pub fn new(api_key: String) -> Self {
        LLMPatternDiscoverer {
            api_key,
            model: "claude-sonnet-4-5-20250929".to_string(), // ✅ Phase 2: Claude 기본 모델
            client: Client::new(),
        }
    }

    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    /// 피드백 데이터에서 Rule 추출
    pub async fn extract_rules(&self, feedback_data: Vec<FeedbackData>) -> anyhow::Result<Vec<ExtractedRule>> {
        if feedback_data.is_empty() {
            return Ok(vec![]);
        }

        // 1. 데이터 집계
        let summary = self.aggregate_data(&feedback_data)?;

        // 2. LLM 프롬프트 생성
        let prompt = self.create_prompt(&summary);

        // 3. ✅ Phase 2: Claude API 호출 (OpenAI에서 변경)
        let response = self.call_claude_api(&prompt).await?;

        // 4. 응답 파싱
        let rules = self.parse_llm_response(&response)?;

        Ok(rules)
    }

    /// 데이터 집계 - 통계 계산
    fn aggregate_data(&self, feedback_data: &[FeedbackData]) -> anyhow::Result<AggregatedSummary> {
        let positive_data: Vec<_> = feedback_data
            .iter()
            .filter(|f| f.is_positive)
            .collect();

        let negative_data: Vec<_> = feedback_data
            .iter()
            .filter(|f| !f.is_positive)
            .collect();

        // 긍정 데이터 평균 계산
        let positive_avg = self.calculate_averages(&positive_data)?;
        let negative_avg = self.calculate_averages(&negative_data)?;

        // 긍정 데이터 표준편차 계산
        let positive_std = self.calculate_std_dev(&positive_data, &positive_avg)?;

        // 상관관계 계산 (간소화 버전)
        let correlations = HashMap::new(); // 추후 구현 가능

        Ok(AggregatedSummary {
            positive_count: positive_data.len(),
            negative_count: negative_data.len(),
            positive_avg,
            negative_avg,
            positive_std,
            correlations,
        })
    }

    /// 평균 계산
    fn calculate_averages(&self, data: &[&FeedbackData]) -> anyhow::Result<HashMap<String, f64>> {
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        let mut sums: HashMap<String, f64> = HashMap::new();
        let mut counts: HashMap<String, usize> = HashMap::new();

        for feedback in data {
            let value: Value = serde_json::from_str(&feedback.input_json)?;

            if let Some(obj) = value.as_object() {
                for (key, val) in obj {
                    if let Some(num) = val.as_f64() {
                        *sums.entry(key.clone()).or_insert(0.0) += num;
                        *counts.entry(key.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut averages = HashMap::new();
        for (key, sum) in sums {
            if let Some(&count) = counts.get(&key) {
                averages.insert(key, sum / count as f64);
            }
        }

        Ok(averages)
    }

    /// 표준편차 계산
    fn calculate_std_dev(
        &self,
        data: &[&FeedbackData],
        averages: &HashMap<String, f64>,
    ) -> anyhow::Result<HashMap<String, f64>> {
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        let mut variances: HashMap<String, f64> = HashMap::new();
        let mut counts: HashMap<String, usize> = HashMap::new();

        for feedback in data {
            let value: Value = serde_json::from_str(&feedback.input_json)?;

            if let Some(obj) = value.as_object() {
                for (key, val) in obj {
                    if let Some(num) = val.as_f64() {
                        if let Some(&avg) = averages.get(key) {
                            let diff = num - avg;
                            *variances.entry(key.clone()).or_insert(0.0) += diff * diff;
                            *counts.entry(key.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }

        let mut std_devs = HashMap::new();
        for (key, variance) in variances {
            if let Some(&count) = counts.get(&key) {
                std_devs.insert(key, (variance / count as f64).sqrt());
            }
        }

        Ok(std_devs)
    }

    /// LLM 프롬프트 생성
    fn create_prompt(&self, summary: &AggregatedSummary) -> String {
        let positive_avg_json = serde_json::to_string_pretty(&summary.positive_avg).unwrap_or_default();
        let negative_avg_json = serde_json::to_string_pretty(&summary.negative_avg).unwrap_or_default();
        let positive_std_json = serde_json::to_string_pretty(&summary.positive_std).unwrap_or_default();

        format!(
            r#"You are an expert data analyst specializing in rule extraction from feedback data.

Analyze the following aggregated statistics to extract business rules:

Positive Feedback Statistics:
- Count: {}
- Average values: {}
- Standard deviation: {}

Negative Feedback Statistics:
- Count: {}
- Average values: {}

Task:
1. Identify patterns that distinguish positive from negative cases
2. Generate 1-3 rule expressions in the format: "key > threshold && key2 < threshold2"
3. Calculate confidence (0.0-1.0) based on data separation quality

Return response in this exact JSON format:
{{
  "rules": [
    {{
      "expression": "temperature > 85 && vibration > 40",
      "confidence": 0.85,
      "reasoning": "Positive cases have avg temperature 88.5 (std 3.2), negative avg 75.2"
    }}
  ],
  "analysis": "Overall pattern summary"
}}

IMPORTANT: Return ONLY valid JSON, no additional text."#,
            summary.positive_count,
            positive_avg_json,
            positive_std_json,
            summary.negative_count,
            negative_avg_json
        )
    }

    /// ✅ Phase 2: Claude API 호출 (OpenAI에서 마이그레이션)
    async fn call_claude_api(&self, prompt: &str) -> anyhow::Result<String> {
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096, // ✅ Claude는 max_tokens 필수!
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: 0.3, // 낮은 온도 = 일관성 높음
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key) // ✅ Claude는 x-api-key 사용
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Claude API error: {}", error_text);
        }

        let api_response: ClaudeResponse = response.json().await?;

        if let Some(content) = api_response.content.first() {
            Ok(content.text.clone()) // ✅ Claude는 content[0].text
        } else {
            anyhow::bail!("No response from Claude API")
        }
    }

    /// LLM 응답 파싱
    fn parse_llm_response(&self, response_text: &str) -> anyhow::Result<Vec<ExtractedRule>> {
        let llm_response: LLMRuleResponse = serde_json::from_str(response_text)?;

        let rules = llm_response
            .rules
            .into_iter()
            .map(|llm_rule| {
                let mut rule = ExtractedRule::new(
                    llm_rule.expression,
                    llm_rule.confidence,
                    "llm".to_string(),
                );
                rule.total_count = 1; // LLM은 집계 데이터 기반
                rule
            })
            .collect();

        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_averages() {
        let discoverer = LLMPatternDiscoverer::new("test_key".to_string());

        let feedback_data = vec![
            FeedbackData {
                input_json: r#"{"temperature": 90, "vibration": 45}"#.to_string(),
                is_positive: true,
                judgment_result: true,
                judgment_id: "1".to_string(),
            },
            FeedbackData {
                input_json: r#"{"temperature": 88, "vibration": 40}"#.to_string(),
                is_positive: true,
                judgment_result: true,
                judgment_id: "2".to_string(),
            },
        ];

        let data_refs: Vec<_> = feedback_data.iter().collect();
        let averages = discoverer.calculate_averages(&data_refs).unwrap();

        assert_eq!(averages.get("temperature"), Some(&89.0));
        assert_eq!(averages.get("vibration"), Some(&42.5));
    }

    #[test]
    fn test_calculate_std_dev() {
        let discoverer = LLMPatternDiscoverer::new("test_key".to_string());

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
        ];

        let data_refs: Vec<_> = feedback_data.iter().collect();
        let averages = HashMap::from([("temperature".to_string(), 89.0)]);
        let std_devs = discoverer.calculate_std_dev(&data_refs, &averages).unwrap();

        // std = sqrt((1^2 + 1^2) / 2) = sqrt(1) = 1.0
        assert_eq!(std_devs.get("temperature"), Some(&1.0));
    }

    #[test]
    fn test_create_prompt() {
        let discoverer = LLMPatternDiscoverer::new("test_key".to_string());

        let summary = AggregatedSummary {
            positive_count: 10,
            negative_count: 5,
            positive_avg: HashMap::from([("temperature".to_string(), 89.0)]),
            negative_avg: HashMap::from([("temperature".to_string(), 75.0)]),
            positive_std: HashMap::from([("temperature".to_string(), 3.0)]),
            correlations: HashMap::new(),
        };

        let prompt = discoverer.create_prompt(&summary);

        assert!(prompt.contains("Positive Feedback Statistics"));
        assert!(prompt.contains("Count: 10"));
        assert!(prompt.contains("temperature"));
    }

    #[test]
    fn test_parse_llm_response() {
        let discoverer = LLMPatternDiscoverer::new("test_key".to_string());

        let response = r#"{
            "rules": [
                {
                    "expression": "temperature > 85",
                    "confidence": 0.85,
                    "reasoning": "Test reasoning"
                }
            ],
            "analysis": "Test analysis"
        }"#;

        let rules = discoverer.parse_llm_response(response).unwrap();

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].expression, "temperature > 85");
        assert_eq!(rules[0].confidence, 0.85);
        assert_eq!(rules[0].method, "llm");
    }
}
