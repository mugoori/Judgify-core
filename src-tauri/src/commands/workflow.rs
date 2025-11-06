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
    println!("📝 [IPC] create_workflow called! name: {:?}", request.name);
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
    println!("🔍 [IPC] get_workflow called! id: {:?}", id);
    let service = WorkflowService::new().map_err(|e| e.to_string())?;

    let workflow = service
        .get_workflow(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workflow not found".to_string())?;

    Ok(workflow.into())
}

#[tauri::command]
pub async fn get_all_workflows() -> Result<Vec<WorkflowResponse>, String> {
    println!("📋 [IPC] get_all_workflows called!");
    let service = WorkflowService::new().map_err(|e| e.to_string())?;

    let workflows = service.get_all_workflows().map_err(|e| e.to_string())?;

    Ok(workflows.into_iter().map(|w| w.into()).collect())
}

#[tauri::command]
pub async fn update_workflow(request: UpdateWorkflowRequest) -> Result<WorkflowResponse, String> {
    println!("✏️ [IPC] update_workflow called! id: {:?}, name: {:?}", request.id, request.name);
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
    println!("🗑️ [IPC] delete_workflow called! id: {:?}", id);
    let service = WorkflowService::new().map_err(|e| e.to_string())?;

    service.delete_workflow(&id).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn validate_workflow(definition: serde_json::Value) -> Result<bool, String> {
    println!("✅ [IPC] validate_workflow called!");
    let service = WorkflowService::new().map_err(|e| e.to_string())?;

    service
        .validate_workflow(&definition)
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub suggestions: Option<Vec<String>>,
}

#[tauri::command]
pub async fn validate_rule_expression(rule: String) -> Result<RuleValidationResult, String> {
    println!("🔍 [IPC] validate_rule_expression called! rule: {:?}", rule);

    use rhai::{Engine, Scope};

    // Rhai 엔진 직접 사용 (간단한 문법 검증용)
    let engine = Engine::new();
    let mut scope = Scope::new();

    // 테스트용 변수 등록
    scope.push("temperature", 90i64);
    scope.push("vibration", 45i64);
    scope.push("status", "normal".to_string());
    scope.push("count", 10i64);
    scope.push("pressure", 100.0);

    match engine.eval_with_scope::<bool>(&mut scope, &rule) {
        Ok(_) => Ok(RuleValidationResult {
            is_valid: true,
            errors: vec![],
            suggestions: None,
        }),
        Err(e) => {
            let error_msg = e.to_string();
            let mut suggestions = vec![];

            // Provide helpful suggestions based on error type
            if error_msg.contains("Unknown variable") || error_msg.contains("not found") {
                suggestions.push("사용 가능한 변수: temperature, vibration, status, count, pressure".to_string());
                suggestions.push("변수명 철자를 확인하세요.".to_string());
            } else if error_msg.contains("syntax") || error_msg.contains("parse") {
                suggestions.push("지원되는 연산자: >, <, ==, !=, >=, <=, &&, ||".to_string());
                suggestions.push("예시: temperature > 90 && vibration < 50".to_string());
            } else if error_msg.contains("type") {
                suggestions.push("타입이 일치하는지 확인하세요 (숫자, 문자열).".to_string());
            }

            Ok(RuleValidationResult {
                is_valid: false,
                errors: vec![error_msg],
                suggestions: if suggestions.is_empty() { None } else { Some(suggestions) },
            })
        }
    }
}
