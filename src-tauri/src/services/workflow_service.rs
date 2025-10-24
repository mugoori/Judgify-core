use crate::database::{Database, Workflow};
use uuid::Uuid;
use chrono::Utc;

pub struct WorkflowService {
    db: Database,
}

impl WorkflowService {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            db: Database::new()?,
        })
    }

    pub fn create_workflow(
        &self,
        name: String,
        definition: serde_json::Value,
        rule_expression: Option<String>,
    ) -> anyhow::Result<Workflow> {
        let workflow = Workflow {
            id: Uuid::new_v4().to_string(),
            name,
            definition: serde_json::to_string(&definition)?,
            rule_expression,
            version: 1,
            is_active: true,
            created_at: Utc::now(),
        };

        self.db.save_workflow(&workflow)?;
        Ok(workflow)
    }

    pub fn get_workflow(&self, id: &str) -> anyhow::Result<Option<Workflow>> {
        self.db.get_workflow(id).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn get_all_workflows(&self) -> anyhow::Result<Vec<Workflow>> {
        self.db.get_all_workflows().map_err(|e| anyhow::anyhow!(e))
    }

    pub fn update_workflow(
        &self,
        id: String,
        name: Option<String>,
        definition: Option<serde_json::Value>,
        rule_expression: Option<String>,
        is_active: Option<bool>,
    ) -> anyhow::Result<Workflow> {
        let mut workflow = self
            .db
            .get_workflow(&id)?
            .ok_or_else(|| anyhow::anyhow!("Workflow not found"))?;

        if let Some(n) = name {
            workflow.name = n;
        }
        if let Some(d) = definition {
            workflow.definition = serde_json::to_string(&d)?;
        }
        if let Some(r) = rule_expression {
            workflow.rule_expression = Some(r);
        }
        if let Some(a) = is_active {
            workflow.is_active = a;
        }

        workflow.version += 1;
        self.db.save_workflow(&workflow)?;
        Ok(workflow)
    }

    pub fn delete_workflow(&self, id: &str) -> anyhow::Result<()> {
        // Soft delete by setting is_active to false
        let mut workflow = self
            .db
            .get_workflow(id)?
            .ok_or_else(|| anyhow::anyhow!("Workflow not found"))?;

        workflow.is_active = false;
        self.db.save_workflow(&workflow)?;
        Ok(())
    }

    pub fn validate_workflow(&self, definition: &serde_json::Value) -> anyhow::Result<bool> {
        // Basic validation
        if !definition.is_object() {
            return Err(anyhow::anyhow!("Workflow definition must be an object"));
        }

        let obj = definition.as_object().unwrap();

        // Check required fields
        if !obj.contains_key("nodes") || !obj.contains_key("edges") {
            return Err(anyhow::anyhow!(
                "Workflow must contain 'nodes' and 'edges'"
            ));
        }

        // Validate nodes array
        if let Some(nodes) = obj.get("nodes") {
            if !nodes.is_array() {
                return Err(anyhow::anyhow!("'nodes' must be an array"));
            }
        }

        // Validate edges array
        if let Some(edges) = obj.get("edges") {
            if !edges.is_array() {
                return Err(anyhow::anyhow!("'edges' must be an array"));
            }
        }

        Ok(true)
    }
}
