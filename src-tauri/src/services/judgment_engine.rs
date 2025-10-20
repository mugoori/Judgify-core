use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::database::{Database, Judgment};
use crate::services::{rule_engine::RuleEngine, llm_engine::LLMEngine};

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
    db: Database,
}

impl JudgmentEngine {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            rule_engine: RuleEngine::new()?,
            llm_engine: LLMEngine::new()?,
            db: Database::new()?,
        })
    }

    pub async fn execute(&self, input: JudgmentInput) -> anyhow::Result<JudgmentResult> {
        // 1. Try Rule Engine first
        match self.rule_engine.evaluate(&input) {
            Ok(rule_result) if rule_result.confidence >= 0.7 => {
                // Rule succeeded with high confidence
                self.save_result(&rule_result)?;
                return Ok(rule_result);
            }
            Ok(rule_result) => {
                // Rule succeeded but low confidence, try LLM
                match self.llm_engine.evaluate(&input).await {
                    Ok(llm_result) => {
                        let final_result = self.combine_results(rule_result, llm_result);
                        self.save_result(&final_result)?;
                        Ok(final_result)
                    }
                    Err(_) => {
                        // LLM failed, use rule result
                        self.save_result(&rule_result)?;
                        Ok(rule_result)
                    }
                }
            }
            Err(_) => {
                // Rule failed, use LLM only
                let llm_result = self.llm_engine.evaluate(&input).await?;
                self.save_result(&llm_result)?;
                Ok(llm_result)
            }
        }
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

    fn save_result(&self, result: &JudgmentResult) -> anyhow::Result<()> {
        let judgment = Judgment {
            id: result.id.clone(),
            workflow_id: result.workflow_id.clone(),
            input_data: serde_json::to_string(&serde_json::json!({}))?,
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
