// ✅ Phase 2: OpenAI 임베딩 전용 클라이언트 (Chat은 Claude로 마이그레이션)
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

/// ✅ Phase 2: OpenAI 클라이언트 (임베딩 전용, Few-shot 학습용)
pub struct OpenAIClient {
    client: Client,
    api_key: String,
}

impl OpenAIClient {
    pub fn new() -> anyhow::Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .unwrap_or_else(|_| "sk-test-key".to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    pub async fn create_embedding(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        #[derive(Serialize)]
        struct EmbeddingRequest {
            model: String,
            input: String,
        }

        #[derive(Deserialize)]
        struct EmbeddingResponse {
            data: Vec<EmbeddingData>,
        }

        #[derive(Deserialize)]
        struct EmbeddingData {
            embedding: Vec<f32>,
        }

        let request = EmbeddingRequest {
            model: "text-embedding-3-small".to_string(),
            input: text.to_string(),
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?
            .json::<EmbeddingResponse>()
            .await?;

        Ok(response.data[0].embedding.clone())
    }
}
