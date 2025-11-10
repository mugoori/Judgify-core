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
}
