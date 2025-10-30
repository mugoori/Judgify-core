use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::database::{Database, Judgment};
use crate::services::{rule_engine::RuleEngine, llm_engine::LLMEngine, learning_service::LearningService};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JudgmentInput {
    pub workflow_id: String,
    pub input_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JudgmentResult {
    pub id: String,
    pub workflow_id: String,
    pub result: bool,
    pub confidence: f64,
    pub method_used: String,
    pub explanation: String,
}

pub struct JudgmentEngine {
    rule_engine: RuleEngine,
    llm_engine: LLMEngine,
    learning_service: LearningService,
    db: Database,
}

impl JudgmentEngine {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            rule_engine: RuleEngine::new()?,
            llm_engine: LLMEngine::new()?,
            learning_service: LearningService::new()?,
            db: Database::new()?,
        })
    }

    /// Few-shot 학습을 포함한 하이브리드 판단 (새로운 기본 메서드!)
    pub async fn judge_with_few_shot(&self, input: JudgmentInput) -> anyhow::Result<JudgmentResult> {
        // 1. Few-shot 샘플 검색 (Learning Service)
        let few_shot_samples = self.learning_service
            .get_few_shot_samples(input.workflow_id.clone(), 15)?;

        println!("📚 Few-shot 샘플 개수: {}", few_shot_samples.len());

        // 2. Rule Engine 실행
        match self.rule_engine.evaluate(&input) {
            Ok(rule_result) if rule_result.confidence >= 0.7 => {
                // Rule 성공, Few-shot 불필요
                println!("✅ Rule Engine 성공 (신뢰도: {:.1}%), Few-shot 생략", rule_result.confidence * 100.0);
                self.save_result(&rule_result, &input)?;
                return Ok(rule_result);
            }
            Ok(rule_result) => {
                // Rule 저신뢰도, LLM + Few-shot 실행
                println!("⚠️  Rule Engine 저신뢰도 ({:.1}%), LLM + Few-shot 실행", rule_result.confidence * 100.0);

                match self.llm_engine.evaluate_with_few_shot(&input, &few_shot_samples).await {
                    Ok(llm_result) => {
                        let final_result = self.combine_results(rule_result, llm_result);
                        self.save_result(&final_result, &input)?;
                        Ok(final_result)
                    }
                    Err(_) => {
                        // LLM 실패, Rule 결과 사용
                        self.save_result(&rule_result, &input)?;
                        Ok(rule_result)
                    }
                }
            }
            Err(_) => {
                // Rule 실패, LLM + Few-shot만 실행
                println!("❌ Rule Engine 실패, LLM + Few-shot만 사용");
                let llm_result = self.llm_engine.evaluate_with_few_shot(&input, &few_shot_samples).await?;
                self.save_result(&llm_result, &input)?;
                Ok(llm_result)
            }
        }
    }

    /// 기존 execute() 메서드 (하위 호환성)
    pub async fn execute(&self, input: JudgmentInput) -> anyhow::Result<JudgmentResult> {
        // 기본적으로 Few-shot 학습 활성화
        self.judge_with_few_shot(input).await
    }

    fn combine_results(&self, rule: JudgmentResult, llm: JudgmentResult) -> JudgmentResult {
        if llm.confidence > rule.confidence {
            JudgmentResult {
                id: Uuid::new_v4().to_string(),
                method_used: "hybrid".to_string(),
                explanation: format!(
                    "하이브리드 판단 결과:\n\n[Rule Engine (신뢰도: {:.1}%)]\n{}\n\n[LLM Engine (신뢰도: {:.1}%)]\n{}",
                    rule.confidence * 100.0,
                    rule.explanation,
                    llm.confidence * 100.0,
                    llm.explanation
                ),
                ..llm
            }
        } else {
            rule
        }
    }

    fn save_result(&self, result: &JudgmentResult, input: &JudgmentInput) -> anyhow::Result<()> {
        let judgment = Judgment {
            id: result.id.clone(),
            workflow_id: result.workflow_id.clone(),
            input_data: serde_json::to_string(&input.input_data)?,
            result: result.result,
            confidence: result.confidence,
            method_used: result.method_used.clone(),
            explanation: result.explanation.clone(),
            created_at: Utc::now(),
        };

        self.db.save_judgment(&judgment)?;
        Ok(())
    }

    pub async fn get_history(
        &self,
        workflow_id: Option<String>,
        limit: u32,
    ) -> anyhow::Result<Vec<JudgmentResult>> {
        let judgments = self.db.get_judgment_history(workflow_id, limit)?;

        Ok(judgments
            .into_iter()
            .map(|j| JudgmentResult {
                id: j.id,
                workflow_id: j.workflow_id,
                result: j.result,
                confidence: j.confidence,
                method_used: j.method_used,
                explanation: j.explanation,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use crate::database::{Workflow, TrainingSample};
    use chrono::Utc;

    /// 테스트용 Workflow 및 TrainingSample 생성 헬퍼
    fn setup_test_data() -> (String, Vec<TrainingSample>) {
        let workflow_id = Uuid::new_v4().to_string();

        // 테스트용 훈련 샘플 생성 (정확도 높은 샘플들)
        let samples = vec![
            TrainingSample {
                id: Uuid::new_v4().to_string(),
                workflow_id: workflow_id.clone(),
                input_data: r#"{"temperature": 88, "vibration": 42}"#.to_string(),
                expected_result: true,
                actual_result: Some(true),
                accuracy: Some(0.95),
                created_at: Utc::now(),
            },
            TrainingSample {
                id: Uuid::new_v4().to_string(),
                workflow_id: workflow_id.clone(),
                input_data: r#"{"temperature": 91, "vibration": 45}"#.to_string(),
                expected_result: true,
                actual_result: Some(true),
                accuracy: Some(0.92),
                created_at: Utc::now(),
            },
            TrainingSample {
                id: Uuid::new_v4().to_string(),
                workflow_id: workflow_id.clone(),
                input_data: r#"{"temperature": 75, "vibration": 30}"#.to_string(),
                expected_result: false,
                actual_result: Some(false),
                accuracy: Some(0.88),
                created_at: Utc::now(),
            },
        ];

        (workflow_id, samples)
    }

    #[tokio::test]
    async fn test_judge_with_few_shot_basic() {
        // 기본 기능 테스트: Few-shot 샘플 검색 및 판단 실행
        let engine = JudgmentEngine::new().unwrap();
        let (workflow_id, samples) = setup_test_data();

        // Workflow 먼저 생성 (외래 키 제약 때문에)
        let workflow = Workflow {
            id: workflow_id.clone(),
            name: "Test Workflow".to_string(),
            definition: "{}".to_string(),
            rule_expression: Some("temperature > 85 && vibration > 40".to_string()),
            version: 1,
            is_active: true,
            created_at: Utc::now(),
        };
        engine.db.save_workflow(&workflow).unwrap();

        // DB에 샘플 저장
        for sample in &samples {
            engine.db.save_training_sample(sample).unwrap();
        }

        // 판단 실행
        let input = JudgmentInput {
            workflow_id: workflow_id.clone(),
            input_data: serde_json::json!({"temperature": 90, "vibration": 43}),
        };

        let result = engine.judge_with_few_shot(input).await;

        // 결과 검증
        assert!(result.is_ok());
        let judgment = result.unwrap();
        assert_eq!(judgment.workflow_id, workflow_id);
        assert!(judgment.confidence > 0.0);
    }

    #[test]
    fn test_combine_results() {
        let engine = JudgmentEngine::new().unwrap();

        let rule_result = JudgmentResult {
            id: Uuid::new_v4().to_string(),
            workflow_id: "test".to_string(),
            result: true,
            confidence: 0.6,
            method_used: "rule".to_string(),
            explanation: "Rule 판단".to_string(),
        };

        let llm_result = JudgmentResult {
            id: Uuid::new_v4().to_string(),
            workflow_id: "test".to_string(),
            result: true,
            confidence: 0.9,
            method_used: "llm".to_string(),
            explanation: "LLM 판단".to_string(),
        };

        let combined = engine.combine_results(rule_result, llm_result.clone());

        // LLM 신뢰도가 더 높으면 LLM 결과 반환
        assert_eq!(combined.method_used, "hybrid");
        assert_eq!(combined.result, llm_result.result);
        assert!(combined.explanation.contains("하이브리드 판단 결과"));
    }

    #[tokio::test]
    async fn test_get_history() {
        let engine = JudgmentEngine::new().unwrap();
        let workflow_id = Uuid::new_v4().to_string();

        // 테스트 판단 결과 저장
        let input = JudgmentInput {
            workflow_id: workflow_id.clone(),
            input_data: serde_json::json!({"temp": 85}),
        };

        let result = JudgmentResult {
            id: Uuid::new_v4().to_string(),
            workflow_id: workflow_id.clone(),
            result: true,
            confidence: 0.85,
            method_used: "rule".to_string(),
            explanation: "Test".to_string(),
        };

        engine.save_result(&result, &input).unwrap();

        // 히스토리 조회
        let history = engine.get_history(Some(workflow_id.clone()), 10).await.unwrap();

        assert!(!history.is_empty());
        assert_eq!(history[0].workflow_id, workflow_id);
    }
}
