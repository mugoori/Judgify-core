use rhai::{Engine, Scope};
use uuid::Uuid;
use crate::database::Database;
use crate::services::judgment_engine::{JudgmentInput, JudgmentResult};

pub struct RuleEngine {
    engine: Engine,
    db: Database,
}

impl RuleEngine {
    pub fn new() -> anyhow::Result<Self> {
        let mut engine = Engine::new();
        engine.set_max_operations(10000);

        Ok(Self {
            engine,
            db: Database::new()?,
        })
    }

    pub fn evaluate(&self, input: &JudgmentInput) -> anyhow::Result<JudgmentResult> {
        let workflow = self
            .db
            .get_workflow(&input.workflow_id)?
            .ok_or_else(|| anyhow::anyhow!("Workflow not found"))?;

        let rule_expression = workflow
            .rule_expression
            .ok_or_else(|| anyhow::anyhow!("No rule expression defined"))?;

        let mut scope = Scope::new();

        // Register input data as variables
        if let Some(obj) = input.input_data.as_object() {
            for (key, value) in obj {
                match value {
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            scope.push(key.clone(), i);
                        } else if let Some(f) = n.as_f64() {
                            scope.push(key.clone(), f);
                        }
                    }
                    serde_json::Value::String(s) => {
                        scope.push(key.clone(), s.clone());
                    }
                    serde_json::Value::Bool(b) => {
                        scope.push(key.clone(), *b);
                    }
                    _ => {}
                }
            }
        }

        // Execute rule
        let result: bool = self
            .engine
            .eval_with_scope(&mut scope, &rule_expression)
            .map_err(|e| anyhow::anyhow!("Rule evaluation failed: {}", e))?;

        Ok(JudgmentResult {
            id: Uuid::new_v4().to_string(),
            workflow_id: input.workflow_id.clone(),
            result,
            confidence: 0.9,
            method_used: "rule".to_string(),
            explanation: format!(
                "Rule 기반 판단 완료\n\nRule: {}\n결과: {}",
                rule_expression,
                if result { "합격" } else { "불합격" }
            ),
        })
    }
}
