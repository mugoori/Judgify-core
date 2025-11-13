use crate::utils::openai::OpenAIClient;

pub struct EmbeddingGenerator {
    client: OpenAIClient,
}

impl EmbeddingGenerator {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: OpenAIClient::new()?,
        })
    }

    pub async fn generate(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        self.client.create_embedding(text).await
    }

    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }

    pub fn find_most_similar(
        &self,
        query_embedding: &[f32],
        candidates: Vec<(String, Vec<f32>)>,
        top_k: usize,
    ) -> Vec<(String, f32)> {
        let mut similarities: Vec<(String, f32)> = candidates
            .into_iter()
            .map(|(id, embedding)| {
                let similarity = Self::cosine_similarity(query_embedding, &embedding);
                (id, similarity)
            })
            .collect();

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.into_iter().take(top_k).collect()
    }
}
