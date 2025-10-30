use uuid::Uuid;
use chrono::Utc;
use crate::database::{Database, TrainingSample, Feedback, Workflow};
use crate::algorithms::{
    frequency_analyzer::FrequencyAnalyzer,
    llm_pattern_discoverer::LLMPatternDiscoverer,
    rule_integrator::RuleIntegrator,
    FeedbackData,
};

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

    /// 추출된 Rule을 Workflow에 저장
    pub fn save_extracted_rule(
        &self,
        workflow_id: String,
        rule_expression: String,
        confidence: f64,
    ) -> anyhow::Result<()> {
        // 1. Workflow 로드
        let workflow = self.db.get_workflow(&workflow_id)?
            .ok_or_else(|| anyhow::anyhow!("Workflow not found: {}", workflow_id))?;

        // 2. Rule 업데이트 + 버전 증가
        let updated_workflow = Workflow {
            rule_expression: Some(rule_expression.clone()),
            version: workflow.version + 1,
            ..workflow
        };

        // 3. DB 저장
        self.db.save_workflow(&updated_workflow)?;

        println!(
            "✅ Rule saved to workflow {}: {} (confidence: {:.2}, version: {} → {})",
            workflow_id, rule_expression, confidence,
            workflow.version, updated_workflow.version
        );

        Ok(())
    }

    pub async fn extract_rules(&self, workflow_id: String) -> anyhow::Result<String> {
        // 1. Load feedback data from database
        let samples = self.db.get_training_samples(&workflow_id, 100)?;

        if samples.is_empty() {
            return Ok("No training samples available".to_string());
        }

        // Convert TrainingSample to FeedbackData
        let feedback_data: Vec<FeedbackData> = samples
            .into_iter()
            .map(|sample| {
                let is_positive = sample.accuracy.unwrap_or(0.0) >= 0.7;
                FeedbackData {
                    input_json: sample.input_data,
                    is_positive,
                    judgment_result: sample.expected_result,
                    judgment_id: sample.id.clone(),
                }
            })
            .collect();

        // 2. Run Algorithm 1: Frequency Analysis
        let freq_analyzer = FrequencyAnalyzer::new(0.80);
        let freq_rules = freq_analyzer.extract_rules(feedback_data.clone())?;

        // 3. Run Algorithm 3: LLM Pattern Discovery (parallel execution possible)
        let llm_rules = if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let llm_discoverer = LLMPatternDiscoverer::new(api_key);
            llm_discoverer.extract_rules(feedback_data).await?
        } else {
            Vec::new()
        };

        // 4. Integrate rules from both algorithms
        let integrator = RuleIntegrator::new();
        let best_rule = integrator.integrate_rules(vec![freq_rules, llm_rules])?;

        // 5. Save rule to workflow and return result
        if let Some(rule) = best_rule {
            // 자동으로 Workflow에 저장
            self.save_extracted_rule(
                workflow_id.clone(),
                rule.expression.clone(),
                rule.confidence,
            )?;

            Ok(format!(
                "Extracted Rule: {} (confidence: {:.2}, method: {}) - Saved to workflow {}",
                rule.expression, rule.confidence, rule.method, workflow_id
            ))
        } else {
            Ok("No confident rules found".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_save_extracted_rule_success() {
        // 테스트용 Workflow 생성
        let service = LearningService::new().unwrap();
        let workflow_id = Uuid::new_v4().to_string();

        // 테스트 Workflow 저장
        let workflow = Workflow {
            id: workflow_id.clone(),
            name: "Test Workflow".to_string(),
            definition: "{}".to_string(),
            rule_expression: None,
            version: 1,
            is_active: true,
            created_at: Utc::now(),
        };
        service.db.save_workflow(&workflow).unwrap();

        // Rule 저장 테스트
        let result = service.save_extracted_rule(
            workflow_id.clone(),
            "temperature > 85".to_string(),
            0.92,
        );

        assert!(result.is_ok());

        // 검증: Workflow에 Rule이 저장되었는지 확인
        let updated = service.db.get_workflow(&workflow_id).unwrap().unwrap();
        assert_eq!(updated.rule_expression, Some("temperature > 85".to_string()));
        assert_eq!(updated.version, 2); // 버전 증가 확인
    }

    #[test]
    fn test_save_extracted_rule_workflow_not_found() {
        let service = LearningService::new().unwrap();
        let non_existent_id = Uuid::new_v4().to_string();

        let result = service.save_extracted_rule(
            non_existent_id.clone(),
            "temperature > 85".to_string(),
            0.92,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Workflow not found"));
    }

    #[test]
    fn test_save_extracted_rule_version_increment() {
        let service = LearningService::new().unwrap();
        let workflow_id = Uuid::new_v4().to_string();

        // 초기 Workflow 생성 (version 1)
        let workflow = Workflow {
            id: workflow_id.clone(),
            name: "Test Workflow".to_string(),
            definition: "{}".to_string(),
            rule_expression: None,
            version: 1,
            is_active: true,
            created_at: Utc::now(),
        };
        service.db.save_workflow(&workflow).unwrap();

        // 첫 번째 Rule 저장 (version 1 → 2)
        service.save_extracted_rule(
            workflow_id.clone(),
            "temperature > 85".to_string(),
            0.92,
        ).unwrap();

        let workflow_v2 = service.db.get_workflow(&workflow_id).unwrap().unwrap();
        assert_eq!(workflow_v2.version, 2);

        // 두 번째 Rule 저장 (version 2 → 3)
        service.save_extracted_rule(
            workflow_id.clone(),
            "temperature > 90 && vibration > 40".to_string(),
            0.95,
        ).unwrap();

        let workflow_v3 = service.db.get_workflow(&workflow_id).unwrap().unwrap();
        assert_eq!(workflow_v3.version, 3);
        assert_eq!(workflow_v3.rule_expression, Some("temperature > 90 && vibration > 40".to_string()));
    }
}
