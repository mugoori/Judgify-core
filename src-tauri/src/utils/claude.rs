use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Claude API 클라이언트
pub struct ClaudeClient {
    api_key: String,
    http_client: Client,
}

impl ClaudeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http_client: Client::new(),
        }
    }

    /// Claude API로 메시지 생성
    pub async fn create_message(
        &self,
        system: &str,
        messages: Vec<Message>,
        model: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String> {
        let request = ClaudeRequest {
            model: model.to_string(),
            system: system.to_string(),
            messages,
            max_tokens,
            temperature,
        };

        let response = self
            .http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Claude API error: {}", error_text);
        }

        let response_json: ClaudeResponse = response.json().await?;

        // Claude는 여러 content 블록을 반환할 수 있음, 첫 번째 텍스트 블록만 사용
        Ok(response_json
            .content
            .into_iter()
            .find(|c| !c.text.is_empty())
            .map(|c| c.text)
            .unwrap_or_default())
    }
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    system: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String, // "user" | "assistant"
    pub content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_claude_client_creation() {
        let client = ClaudeClient::new("sk-ant-test-key".to_string());
        assert_eq!(client.api_key, "sk-ant-test-key");
    }
}
