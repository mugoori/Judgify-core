use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::services::judgment_engine::{JudgmentInput, JudgmentResult};
use crate::database::Database;

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

pub struct LLMEngine {
    client: Client,
    api_key: String,
    db: Database,
}

impl LLMEngine {
    pub fn new() -> anyhow::Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "sk-test-key".to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
            db: Database::new()?,
        })
    }

    pub async fn evaluate(&self, input: &JudgmentInput) -> anyhow::Result<JudgmentResult> {
        // Few-shot 학습 샘플 가져오기 (10-20개)
        let few_shot_samples = self.get_few_shot_samples(&input.workflow_id, 15)?;

        let prompt = self.build_prompt(input, &few_shot_samples)?;

        let mut messages = vec![
            Message {
                role: "system".to_string(),
                content: "당신은 제조 품질 판단 전문가입니다. 주어진 데이터를 분석하여 합격/불합격을 판단하고, 그 이유를 명확하게 설명하세요.\n\n응답 형식:\n판단: [합격/불합격]\n이유: [상세 설명]\n신뢰도: [0.0-1.0]".to_string(),
            },
        ];

        // Few-shot 예시를 메시지에 추가
        for sample in &few_shot_samples {
            messages.push(Message {
                role: "user".to_string(),
                content: format!("입력 데이터:\n{}", sample.input_data),
            });
            messages.push(Message {
                role: "assistant".to_string(),
                content: format!(
                    "판단: {}\n이유: 이전 사례를 기반으로 한 판단입니다.",
                    if sample.expected_result { "합격" } else { "불합격" }
                ),
            });
        }

        // 현재 요청 추가
        messages.push(Message {
            role: "user".to_string(),
            content: prompt,
        });

        let request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages,
            temperature: 0.3,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<OpenAIResponse>()
            .await?;

        let llm_response = &response.choices[0].message.content;
        let (result, confidence, explanation) = self.parse_llm_response(llm_response)?;

        // Few-shot 샘플 수에 따라 신뢰도 보정
        let adjusted_confidence = if few_shot_samples.len() >= 10 {
            (confidence * 1.1).min(1.0) // 10개 이상 샘플이 있으면 신뢰도 향상
        } else if few_shot_samples.len() >= 5 {
            confidence
        } else {
            confidence * 0.9 // 샘플이 부족하면 신뢰도 감소
        };

        Ok(JudgmentResult {
            id: Uuid::new_v4().to_string(),
            workflow_id: input.workflow_id.clone(),
            result,
            confidence: adjusted_confidence,
            method_used: if few_shot_samples.is_empty() { "llm".to_string() } else { "llm_few_shot".to_string() },
            explanation: format!(
                "{}\n\n📚 Few-shot 학습: {} 개 유사 사례 참조",
                explanation,
                few_shot_samples.len()
            ),
        })
    }

    fn get_few_shot_samples(&self, workflow_id: &str, limit: u32) -> anyhow::Result<Vec<crate::database::TrainingSample>> {
        // 정확도가 높은 훈련 샘플만 가져오기 (accuracy >= 0.8)
        Ok(self.db.get_training_samples(workflow_id, limit)
            .unwrap_or_default()
            .into_iter()
            .filter(|s| s.accuracy.unwrap_or(0.0) >= 0.8)
            .collect())
    }

    fn build_prompt(&self, input: &JudgmentInput, few_shot_samples: &[crate::database::TrainingSample]) -> anyhow::Result<String> {
        let mut prompt = String::new();

        if !few_shot_samples.is_empty() {
            prompt.push_str(&format!("아래 {} 개의 유사 사례를 참고하여 판단하세요:\n\n", few_shot_samples.len()));
            for (idx, sample) in few_shot_samples.iter().enumerate().take(5) {
                prompt.push_str(&format!(
                    "사례 {}:\n입력: {}\n결과: {}\n정확도: {:.1}%\n\n",
                    idx + 1,
                    sample.input_data,
                    if sample.expected_result { "합격" } else { "불합격" },
                    sample.accuracy.unwrap_or(0.0) * 100.0
                ));
            }
            prompt.push_str("---\n\n");
        }

        prompt.push_str(&format!(
            "다음 데이터를 분석하여 품질 합격/불합격을 판단하세요:\n\n입력 데이터:\n{}",
            serde_json::to_string_pretty(&input.input_data)?
        ));

        Ok(prompt)
    }

    fn parse_llm_response(&self, response: &str) -> anyhow::Result<(bool, f64, String)> {
        let result = response.contains("합격") && !response.contains("불합격");

        // 신뢰도 파싱 시도
        let confidence = if let Some(conf_str) = response.split("신뢰도:").nth(1) {
            conf_str
                .trim()
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.8)
        } else {
            0.8 // 기본 신뢰도
        };

        Ok((result, confidence, response.to_string()))
    }
}
