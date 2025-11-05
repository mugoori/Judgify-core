use handlebars::Handlebars;
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::database::Database;
use crate::database::models::PromptTemplate;

/// Prompt Template Engine with Handlebars variable system
pub struct PromptEngine {
    db: Database,
    handlebars: Handlebars<'static>,
}

impl PromptEngine {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            db: Database::new()?,
            handlebars: Handlebars::new(),
        })
    }

    /// Render prompt template with variables
    pub fn render(
        &self,
        template_type: &str,
        variables: HashMap<String, Value>,
    ) -> anyhow::Result<String> {
        // Get active template by type
        let template = self
            .db
            .get_active_template_by_type(template_type)?
            .ok_or_else(|| anyhow::anyhow!("No active template found for type: {}", template_type))?;

        // Validate variables against template requirements
        self.validate_variables(&template, &variables)?;

        // Render with Handlebars
        let rendered = self
            .handlebars
            .render_template(&template.content, &variables)
            .map_err(|e| anyhow::anyhow!("Template rendering failed: {}", e))?;

        Ok(rendered)
    }

    /// Render judgment prompt with Few-shot samples
    pub async fn render_judgment_prompt(
        &self,
        workflow_id: &str,
        input_data: &Value,
        few_shot_limit: usize,
    ) -> anyhow::Result<String> {
        // Get workflow context
        let workflow = self
            .db
            .get_workflow(workflow_id)?
            .ok_or_else(|| anyhow::anyhow!("Workflow not found: {}", workflow_id))?;

        // Get Few-shot training samples
        let training_samples = self
            .db
            .get_training_samples(workflow_id, few_shot_limit as u32)?;

        // Format Few-shot samples
        let few_shot_text = if training_samples.is_empty() {
            "없음 (첫 판단)".to_string()
        } else {
            training_samples
                .iter()
                .enumerate()
                .map(|(i, sample)| {
                    format!(
                        "**예시 {}:**\n입력: {}\n예상 결과: {}\n정확도: {:.1}%",
                        i + 1,
                        sample.input_data,
                        if sample.expected_result { "합격" } else { "불합격" },
                        sample.accuracy.unwrap_or(0.0) * 100.0
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        };

        // Prepare variables
        let mut variables = HashMap::new();
        variables.insert(
            "workflow_context".to_string(),
            json!({
                "name": workflow.name,
                "rule": workflow.rule_expression.unwrap_or_else(|| "없음".to_string()),
                "version": workflow.version
            })
        );
        variables.insert("input_data".to_string(), input_data.clone());
        variables.insert("few_shot_samples".to_string(), json!(few_shot_text));

        // Render judgment template
        self.render("judgment", variables)
    }

    /// Render explanation prompt
    pub fn render_explanation_prompt(
        &self,
        workflow_name: &str,
        input_data: &Value,
        result: bool,
        confidence: f64,
        method_used: &str,
        similar_cases: &[String],
    ) -> anyhow::Result<String> {
        let mut variables = HashMap::new();
        variables.insert("workflow_name".to_string(), json!(workflow_name));
        variables.insert("input_data".to_string(), input_data.clone());
        variables.insert("result".to_string(), json!(if result { "합격" } else { "불합격" }));
        variables.insert("confidence".to_string(), json!((confidence * 100.0).round()));
        variables.insert("method_used".to_string(), json!(method_used));
        variables.insert(
            "similar_cases".to_string(),
            json!(similar_cases.join("\n- "))
        );

        self.render("explanation", variables)
    }

    /// Render insight prompt
    pub fn render_insight_prompt(
        &self,
        data_summary: &Value,
        trend_data: &Value,
        key_metrics: &Value,
    ) -> anyhow::Result<String> {
        let mut variables = HashMap::new();
        variables.insert("data_summary".to_string(), data_summary.clone());
        variables.insert("trend_data".to_string(), trend_data.clone());
        variables.insert("key_metrics".to_string(), key_metrics.clone());

        self.render("insight", variables)
    }

    /// Validate variables against template requirements
    fn validate_variables(
        &self,
        template: &PromptTemplate,
        variables: &HashMap<String, Value>,
    ) -> anyhow::Result<()> {
        // Parse required variables from template metadata
        let required_vars: Vec<String> = serde_json::from_str(&template.variables)
            .unwrap_or_else(|_| vec![]);

        // Check for missing variables
        let missing_vars: Vec<String> = required_vars
            .iter()
            .filter(|var| !variables.contains_key(*var))
            .cloned()
            .collect();

        if !missing_vars.is_empty() {
            return Err(anyhow::anyhow!(
                "Missing required variables: {}",
                missing_vars.join(", ")
            ));
        }

        Ok(())
    }

    /// Count tokens in rendered prompt (using tiktoken)
    pub fn count_tokens(&self, text: &str) -> usize {
        use tiktoken_rs::cl100k_base;
        let bpe = cl100k_base().unwrap();
        bpe.encode_with_special_tokens(text).len()
    }

    /// Check if rendered prompt exceeds template token limit
    pub fn check_token_limit(&self, template_type: &str, rendered_text: &str) -> anyhow::Result<bool> {
        let template = self
            .db
            .get_active_template_by_type(template_type)?
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_type))?;

        if let Some(limit) = template.token_limit {
            let token_count = self.count_tokens(rendered_text);
            Ok(token_count <= limit as usize)
        } else {
            Ok(true) // No limit set
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_render_with_variables() {
        let engine = PromptEngine::new().unwrap();

        // Manual template rendering test
        let template = "Hello {{name}}! Your score is {{score}}.";
        let mut variables = HashMap::new();
        variables.insert("name".to_string(), json!("Alice"));
        variables.insert("score".to_string(), json!(95));

        let rendered = engine.handlebars.render_template(template, &variables).unwrap();
        assert_eq!(rendered, "Hello Alice! Your score is 95.");
    }

    #[test]
    fn test_token_counting() {
        let engine = PromptEngine::new().unwrap();

        let text = "This is a test prompt with some tokens.";
        let token_count = engine.count_tokens(text);

        // Approximate token count (GPT-4 tokenizer)
        assert!(token_count > 5 && token_count < 15);
    }

    #[test]
    fn test_validate_variables() {
        let engine = PromptEngine::new().unwrap();

        let template = PromptTemplate {
            id: "test".to_string(),
            name: "Test".to_string(),
            template_type: "test".to_string(),
            content: "{{var1}} {{var2}}".to_string(),
            variables: r#"["var1","var2"]"#.to_string(),
            version: 1,
            is_active: true,
            token_limit: Some(100),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut valid_vars = HashMap::new();
        valid_vars.insert("var1".to_string(), json!("value1"));
        valid_vars.insert("var2".to_string(), json!("value2"));

        // Should pass
        assert!(engine.validate_variables(&template, &valid_vars).is_ok());

        // Missing var2 - should fail
        let mut invalid_vars = HashMap::new();
        invalid_vars.insert("var1".to_string(), json!("value1"));
        assert!(engine.validate_variables(&template, &invalid_vars).is_err());
    }
}
