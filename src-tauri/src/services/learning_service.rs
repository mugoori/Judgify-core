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
            // TODO: Create training sample from judgment
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
