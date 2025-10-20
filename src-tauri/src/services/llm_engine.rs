use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::services::judgment_engine::{JudgmentInput, JudgmentResult};

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
}

impl LLMEngine {
    pub fn new() -> anyhow::Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "sk-test-key".to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    pub async fn evaluate(&self, input: &JudgmentInput) -> anyhow::Result<JudgmentResult> {
        let prompt = self.build_prompt(input)?;

        let request = OpenAIRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "당신은 제조 품질 판단 전문가입니다. 주어진 데이터를 분석하여 합격/불합격을 판단하고, 그 이유를 명확하게 설명하세요. 응답 형식: 판단: [합격/불합격]\\n이유: [상세 설명]".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
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
        let (result, explanation) = self.parse_llm_response(llm_response)?;

        Ok(JudgmentResult {
            id: Uuid::new_v4().to_string(),
            workflow_id: input.workflow_id.clone(),
            result,
            confidence: 0.8,
            method_used: "llm".to_string(),
            explanation,
        })
    }

    fn build_prompt(&self, input: &JudgmentInput) -> anyhow::Result<String> {
        Ok(format!(
            "다음 데이터를 분석하여 품질 합격/불합격을 판단하세요:\n\n입력 데이터:\n{}",
            serde_json::to_string_pretty(&input.input_data)?
        ))
    }

    fn parse_llm_response(&self, response: &str) -> anyhow::Result<(bool, String)> {
        let result = response.contains("합격") && !response.contains("불합격");
        Ok((result, response.to_string()))
    }
}
