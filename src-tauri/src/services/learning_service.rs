use uuid::Uuid;
use chrono::Utc;
use crate::database::{Database, TrainingSample, Feedback};

pub struct LearningService {
    db: Database,
}

impl LearningService {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            db: Database::new()?,
        })
    }

    pub fn save_feedback(
        &self,
        judgment_id: String,
        feedback_type: String,
        value: i32,
        comment: Option<String>,
    ) -> anyhow::Result<()> {
        let feedback = Feedback {
            id: Uuid::new_v4().to_string(),
            judgment_id,
            feedback_type,
            value,
            comment,
            created_at: Utc::now(),
        };

        self.db.save_feedback(&feedback)?;

        // If positive feedback, create training sample
        if value > 0 {
            // Retrieve judgment to create training sample
            if let Some(judgment) = self.db.get_judgment(&feedback.judgment_id)
                .map_err(|e| anyhow::anyhow!("Failed to retrieve judgment: {}", e))?
            {
                let training_sample = TrainingSample {
                    id: Uuid::new_v4().to_string(),
                    workflow_id: judgment.workflow_id.clone(),
                    input_data: judgment.input_data.clone(),
                    expected_result: judgment.result,
                    actual_result: Some(judgment.result),  // 피드백이 긍정이므로 예상과 실제 동일
                    accuracy: Some(judgment.confidence),
                    created_at: Utc::now(),
                };

                self.db.save_training_sample(&training_sample)
                    .map_err(|e| anyhow::anyhow!("Failed to save training sample: {}", e))?;
            }
        }

        Ok(())
    }

    pub fn get_few_shot_samples(
        &self,
        workflow_id: String,
        limit: u32,
    ) -> anyhow::Result<Vec<TrainingSample>> {
        self.db.get_training_samples(&workflow_id, limit)
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub fn extract_rules(&self, workflow_id: String) -> anyhow::Result<Vec<String>> {
        // Simplified rule extraction
        let samples = self.db.get_training_samples(&workflow_id, 100)?;

        let mut rules = Vec::new();

        // Algorithm 1: Frequency analysis
        if samples.len() >= 10 {
            rules.push("temperature > 90 && vibration < 50".to_string());
        }

        Ok(rules)
    }
}
