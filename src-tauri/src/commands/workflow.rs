use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowGenerationRequest {
    pub description: String,
    pub context: Option<WorkflowContext>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub industry: Option<String>,
    pub complexity: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowGenerationResponse {
    pub nodes: Vec<serde_json::Value>,
    pub edges: Vec<serde_json::Value>,
    pub metadata: WorkflowMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub provider: String,
    pub model: String,
    pub confidence: f64,
    #[serde(rename = "generationTime")]
    pub generation_time: u64,
}

/// Tauri command to generate workflow using Claude API
/// This bypasses browser CORS restrictions by calling from Rust backend
#[tauri::command]
pub async fn generate_workflow_with_llm(
    request: WorkflowGenerationRequest,
    api_key: String,
    model: Option<String>,
) -> Result<WorkflowGenerationResponse, String> {
    println!("🚀 [Workflow] Generating workflow via Tauri backend");
    println!("   Description: {}", request.description);
    println!("   Model: {}", model.as_ref().unwrap_or(&"claude-3-5-sonnet-20241022".to_string()));

    let start_time = std::time::Instant::now();

    // Build the prompt
    let mut prompt = format!(
        "Generate a workflow JSON for the following requirement:\n\nDescription: {}\n",
        request.description
    );

    if let Some(context) = &request.context {
        if let Some(industry) = &context.industry {
            prompt.push_str(&format!("Industry: {}\n", industry));
        }
        if let Some(complexity) = &context.complexity {
            prompt.push_str(&format!("Complexity: {}\n", complexity));
        }
    }

    prompt.push_str(r#"
IMPORTANT: Return ONLY valid JSON in this exact format:
{
  "nodes": [
    {
      "id": "node-1",
      "type": "data-input",
      "label": "Node Label",
      "config": {},
      "position": { "x": 100, "y": 100 }
    }
  ],
  "edges": [
    {
      "id": "edge-1",
      "source": "node-1",
      "target": "node-2"
    }
  ]
}

Available node types: data-input, condition, action, notification, data-output

Rules:
1. First node must be data-input
2. Last node must be data-output
3. All nodes must be connected
4. Use descriptive labels
5. Position nodes left-to-right (increment x by 250)
"#);

    // Call Claude API
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&json!({
            "model": model.unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string()),
            "max_tokens": 4096,
            "temperature": 0.7,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to call Claude API: {}", e))?;

    // Check response status
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Claude API error ({}): {}", status, error_text));
    }

    // Parse response
    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Claude response: {}", e))?;

    // Extract content from Claude response
    let content = response_json["content"][0]["text"]
        .as_str()
        .ok_or("Invalid Claude response format")?;

    // Parse the workflow JSON from the response
    let workflow_json = parse_workflow_from_text(content)?;

    let generation_time = start_time.elapsed().as_millis() as u64;

    println!("✅ [Workflow] Generated successfully in {}ms", generation_time);

    Ok(WorkflowGenerationResponse {
        nodes: workflow_json["nodes"]
            .as_array()
            .ok_or("Missing nodes array")?
            .clone(),
        edges: workflow_json["edges"]
            .as_array()
            .ok_or("Missing edges array")?
            .clone(),
        metadata: WorkflowMetadata {
            provider: "Claude (Anthropic)".to_string(),
            model: response_json["model"]
                .as_str()
                .unwrap_or("claude-3-5-sonnet-20241022")
                .to_string(),
            confidence: 0.85,
            generation_time,
        },
    })
}

/// Parse workflow JSON from Claude's text response
fn parse_workflow_from_text(text: &str) -> Result<serde_json::Value, String> {
    // Try to extract JSON from markdown code blocks if present
    let json_text = if let Some(start) = text.find("```") {
        let start_idx = text[start..].find('\n').map(|i| start + i + 1).unwrap_or(start + 3);
        let end = text[start_idx..]
            .find("```")
            .map(|i| start_idx + i)
            .unwrap_or(text.len());
        &text[start_idx..end]
    } else {
        text
    };

    // Parse JSON
    serde_json::from_str(json_text.trim())
        .map_err(|e| format!("Failed to parse workflow JSON: {}", e))
}

/// Simulation request for executing workflow step-by-step
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationStepRequest {
    pub workflow_id: String,
    pub nodes: Vec<serde_json::Value>,
    pub edges: Vec<serde_json::Value>,
    pub current_node_id: String,
    pub global_data: serde_json::Value,
}

/// Simulation step response
#[derive(Debug, Serialize, Deserialize)]
pub struct SimulationStepResponse {
    pub node_id: String,
    pub node_name: String,
    pub node_type: String,
    pub status: String, // 'success' | 'error' | 'running'
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub next_node_id: Option<String>,
}

/// Tauri command to simulate a single workflow step
/// This executes one node and returns the result
#[tauri::command]
pub async fn simulate_workflow_step(
    request: SimulationStepRequest,
) -> Result<SimulationStepResponse, String> {
    println!("🎭 [Simulation] Executing step for node: {}", request.current_node_id);

    let start_time = std::time::Instant::now();

    // Find the current node
    let current_node = request
        .nodes
        .iter()
        .find(|n| n["id"].as_str() == Some(&request.current_node_id))
        .ok_or(format!("Node not found: {}", request.current_node_id))?;

    let node_type = current_node["type"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();
    let node_name = current_node["data"]["label"]
        .as_str()
        .unwrap_or(&request.current_node_id)
        .to_string();

    // Execute node logic based on type
    let (status, output, error) = match node_type.as_str() {
        "data-input" | "input" => {
            // Input node: return initial data
            ("success".to_string(), Some(request.global_data.clone()), None)
        }
        "condition" | "decision" => {
            // Decision node: evaluate rule
            let rule = current_node["data"]["rule"].as_str();
            if let Some(rule_expr) = rule {
                match evaluate_simple_rule(rule_expr, &request.global_data) {
                    Ok(result) => (
                        "success".to_string(),
                        Some(json!({
                            "decision": result,
                            "rule": rule_expr,
                            "context": request.global_data
                        })),
                        None,
                    ),
                    Err(e) => ("error".to_string(), None, Some(e)),
                }
            } else {
                ("error".to_string(), None, Some("No rule defined".to_string()))
            }
        }
        "action" => {
            // Action node: simulate action execution
            let action = current_node["data"]["action"]
                .as_str()
                .unwrap_or("default_action");
            (
                "success".to_string(),
                Some(json!({
                    "action": action,
                    "executed": true,
                    "result": format!("Action executed: {}", action)
                })),
                None,
            )
        }
        "data-output" | "output" => {
            // Output node: return final result
            (
                "success".to_string(),
                Some(json!({
                    "finalResult": request.global_data,
                    "workflowCompleted": true
                })),
                None,
            )
        }
        "notification" => {
            // Notification node: simulate notification
            let channel = current_node["data"]["config"]["channel"]
                .as_str()
                .unwrap_or("email");
            let message = current_node["data"]["config"]["message"]
                .as_str()
                .unwrap_or("Notification message");
            (
                "success".to_string(),
                Some(json!({
                    "channel": channel,
                    "message": message,
                    "sentAt": chrono::Utc::now().to_rfc3339(),
                    "status": "sent"
                })),
                None,
            )
        }
        _ => {
            // Unknown node type
            (
                "error".to_string(),
                None,
                Some(format!("Unsupported node type: {}", node_type)),
            )
        }
    };

    // Find next node(s) from edges
    let next_node_id = request
        .edges
        .iter()
        .find(|e| e["source"].as_str() == Some(&request.current_node_id))
        .and_then(|e| e["target"].as_str())
        .map(|s| s.to_string());

    let execution_time = start_time.elapsed().as_millis() as u64;

    println!("✅ [Simulation] Step completed in {}ms (status: {})", execution_time, status);

    Ok(SimulationStepResponse {
        node_id: request.current_node_id.clone(),
        node_name,
        node_type,
        status,
        input: request.global_data,
        output,
        error,
        execution_time_ms: execution_time,
        next_node_id,
    })
}

/// Simple rule evaluator (unsafe eval - for prototype only!)
/// Production should use AST-based safe evaluation
fn evaluate_simple_rule(rule: &str, data: &serde_json::Value) -> Result<bool, String> {
    // For prototype: simple string-based evaluation
    // Production: Use AST parser from algorithms/rule_engine.rs

    // Example: "temperature > 80 && vibration < 50"
    // For now, return a mock result
    println!("⚠️  [Simulation] Mock rule evaluation: {}", rule);

    // Mock evaluation: check if data has required fields
    if rule.contains("temperature") && !data["temperature"].is_null() {
        let temp = data["temperature"].as_f64().unwrap_or(0.0);
        return Ok(temp > 80.0); // Simple mock logic
    }

    // Default: return true
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workflow_from_text() {
        let text = r#"```json
{
  "nodes": [{"id": "node-1", "type": "data-input"}],
  "edges": []
}
```"#;

        let result = parse_workflow_from_text(text).unwrap();
        assert!(result["nodes"].is_array());
        assert!(result["edges"].is_array());
    }

    #[test]
    fn test_parse_workflow_from_plain_json() {
        let text = r#"{"nodes": [], "edges": []}"#;
        let result = parse_workflow_from_text(text).unwrap();
        assert!(result["nodes"].is_array());
        assert!(result["edges"].is_array());
    }

    #[test]
    fn test_simulate_input_node() {
        let request = SimulationStepRequest {
            workflow_id: "test-workflow".to_string(),
            nodes: vec![json!({
                "id": "node-1",
                "type": "data-input",
                "data": {
                    "label": "Input Node"
                }
            })],
            edges: vec![],
            current_node_id: "node-1".to_string(),
            global_data: json!({"temperature": 90}),
        };

        let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = runtime.block_on(simulate_workflow_step(request)).unwrap();

        assert_eq!(result.status, "success");
        assert_eq!(result.node_type, "data-input");
        assert!(result.output.is_some());
    }

    #[test]
    fn test_evaluate_simple_rule() {
        let data = json!({"temperature": 90});
        let result = evaluate_simple_rule("temperature > 80", &data).unwrap();
        assert_eq!(result, true);

        let data2 = json!({"temperature": 70});
        let result2 = evaluate_simple_rule("temperature > 80", &data2).unwrap();
        assert_eq!(result2, false);
    }
}
