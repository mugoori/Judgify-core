use crate::database::Workflow;
use crate::services::workflow_service::WorkflowService;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub definition: serde_json::Value,
    pub rule_expression: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub id: String,
    pub name: Option<String>,
    pub definition: Option<serde_json::Value>,
    pub rule_expression: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowResponse {
    pub id: String,
    pub name: String,
    pub definition: serde_json::Value,
    pub rule_expression: Option<String>,
    pub version: i32,
    pub is_active: bool,
    pub created_at: String,
}

impl From<Workflow> for WorkflowResponse {
    fn from(w: Workflow) -> Self {
        Self {
            id: w.id,
            name: w.name,
            definition: serde_json::from_str(&w.definition).unwrap_or(serde_json::json!({})),
            rule_expression: w.rule_expression,
            version: w.version,
            is_active: w.is_active,
            created_at: w.created_at.to_rfc3339(),
        }
    }
}

#[tauri::command]
pub async fn create_workflow(request: CreateWorkflowRequest) -> Result<WorkflowResponse, String> {
    let service = WorkflowService::new().map_err(|e| e.to_string())?;

    // Validate workflow definition
    service
        .validate_workflow(&request.definition)
        .map_err(|e| e.to_string())?;

    let workflow = service
        .create_workflow(request.name, request.definition, request.rule_expression)
        .map_err(|e| e.to_string())?;

    Ok(workflow.into())
}

#[tauri::command]
pub async fn get_workflow(id: String) -> Result<WorkflowResponse, String> {
    let service = WorkflowService::new().map_err(|e| e.to_string())?;

    let workflow = service
        .get_workflow(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workflow not found".to_string())?;

    Ok(workflow.into())
}

#[tauri::command]
pub async fn get_all_workflows() -> Result<Vec<WorkflowResponse>, String> {
    let service = WorkflowService::new().map_err(|e| e.to_string())?;

    let workflows = service.get_all_workflows().map_err(|e| e.to_string())?;

    Ok(workflows.into_iter().map(|w| w.into()).collect())
}

#[tauri::command]
pub async fn update_workflow(request: UpdateWorkflowRequest) -> Result<WorkflowResponse, String> {
    let service = WorkflowService::new().map_err(|e| e.to_string())?;

    // Validate if definition is provided
    if let Some(ref def) = request.definition {
        service.validate_workflow(def).map_err(|e| e.to_string())?;
    }

    let workflow = service
        .update_workflow(
            request.id,
            request.name,
            request.definition,
            request.rule_expression,
            request.is_active,
        )
        .map_err(|e| e.to_string())?;

    Ok(workflow.into())
}

#[tauri::command]
pub async fn delete_workflow(id: String) -> Result<(), String> {
    let service = WorkflowService::new().map_err(|e| e.to_string())?;

    service.delete_workflow(&id).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn validate_workflow(definition: serde_json::Value) -> Result<bool, String> {
    let service = WorkflowService::new().map_err(|e| e.to_string())?;

    service
        .validate_workflow(&definition)
        .map_err(|e| e.to_string())
}
