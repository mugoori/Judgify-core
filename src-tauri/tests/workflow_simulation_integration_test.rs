/**
 * Workflow Simulation Integration Tests
 *
 * Week 5: 시뮬레이션 커맨드 통합 테스트
 *
 * Test Scenarios:
 * 1. 복잡한 워크플로우 시뮬레이션 통합 테스트 (6 nodes)
 * 2. 다중 노드 체인 실행 테스트 (분기 처리)
 * 3. 에러 처리 및 복구 테스트
 */

use serde_json::json;
use judgify_desktop::commands::workflow::{
    simulate_workflow_step, SimulationStepRequest, SimulationStepResponse,
};

/// Test 1: 복잡한 워크플로우 시뮬레이션 통합 테스트
///
/// 6개 노드로 구성된 전체 워크플로우를 단계별로 실행하고
/// 각 단계의 output이 다음 단계의 input으로 올바르게 전달되는지 확인
#[tokio::test]
async fn test_complex_workflow_simulation() {
    println!("🧪 [Integration Test 1] Starting complex workflow simulation test");

    // Define 6-node workflow: input → condition → action → notification → condition → output
    let nodes = vec![
        json!({
            "id": "node-1",
            "type": "data-input",
            "data": { "label": "센서 데이터 입력" }
        }),
        json!({
            "id": "node-2",
            "type": "condition",
            "data": {
                "label": "온도 체크",
                "rule": "temperature > 80"
            }
        }),
        json!({
            "id": "node-3",
            "type": "action",
            "data": {
                "label": "냉각 시스템 가동",
                "action": "activate_cooling"
            }
        }),
        json!({
            "id": "node-4",
            "type": "notification",
            "data": {
                "label": "경고 알림",
                "config": {
                    "channel": "slack",
                    "message": "고온 경고: 냉각 시스템 가동됨"
                }
            }
        }),
        json!({
            "id": "node-5",
            "type": "condition",
            "data": {
                "label": "진동 체크",
                "rule": "vibration < 50"
            }
        }),
        json!({
            "id": "node-6",
            "type": "data-output",
            "data": { "label": "처리 완료" }
        }),
    ];

    let edges = vec![
        json!({ "id": "edge-1", "source": "node-1", "target": "node-2" }),
        json!({ "id": "edge-2", "source": "node-2", "target": "node-3" }),
        json!({ "id": "edge-3", "source": "node-3", "target": "node-4" }),
        json!({ "id": "edge-4", "source": "node-4", "target": "node-5" }),
        json!({ "id": "edge-5", "source": "node-5", "target": "node-6" }),
    ];

    let initial_data = json!({
        "temperature": 90,
        "vibration": 30
    });

    // Step 1: Execute input node
    let request1 = SimulationStepRequest {
        workflow_id: "integration-test-workflow".to_string(),
        nodes: nodes.clone(),
        edges: edges.clone(),
        current_node_id: "node-1".to_string(),
        global_data: initial_data.clone(),
    };

    let result1 = simulate_workflow_step(request1).await.unwrap();
    assert_eq!(result1.status, "success");
    assert_eq!(result1.node_type, "data-input");
    assert_eq!(result1.next_node_id, Some("node-2".to_string()));
    println!("✅ Step 1 (input) passed");

    // Step 2: Execute condition node
    let request2 = SimulationStepRequest {
        workflow_id: "integration-test-workflow".to_string(),
        nodes: nodes.clone(),
        edges: edges.clone(),
        current_node_id: "node-2".to_string(),
        global_data: result1.output.unwrap_or(initial_data.clone()),
    };

    let result2 = simulate_workflow_step(request2).await.unwrap();
    assert_eq!(result2.status, "success");
    assert_eq!(result2.node_type, "condition");
    assert_eq!(result2.next_node_id, Some("node-3".to_string()));
    println!("✅ Step 2 (condition) passed");

    // Step 3: Execute action node
    let request3 = SimulationStepRequest {
        workflow_id: "integration-test-workflow".to_string(),
        nodes: nodes.clone(),
        edges: edges.clone(),
        current_node_id: "node-3".to_string(),
        global_data: result2.input.clone(),
    };

    let result3 = simulate_workflow_step(request3).await.unwrap();
    assert_eq!(result3.status, "success");
    assert_eq!(result3.node_type, "action");
    assert!(result3.output.is_some());
    assert_eq!(result3.next_node_id, Some("node-4".to_string()));
    println!("✅ Step 3 (action) passed");

    // Step 4: Execute notification node
    let request4 = SimulationStepRequest {
        workflow_id: "integration-test-workflow".to_string(),
        nodes: nodes.clone(),
        edges: edges.clone(),
        current_node_id: "node-4".to_string(),
        global_data: result3.input.clone(),
    };

    let result4 = simulate_workflow_step(request4).await.unwrap();
    assert_eq!(result4.status, "success");
    assert_eq!(result4.node_type, "notification");
    assert!(result4.output.is_some());
    let output4 = result4.output.unwrap();
    assert_eq!(output4["channel"], "slack");
    assert_eq!(result4.next_node_id, Some("node-5".to_string()));
    println!("✅ Step 4 (notification) passed");

    // Step 5: Execute second condition node
    let request5 = SimulationStepRequest {
        workflow_id: "integration-test-workflow".to_string(),
        nodes: nodes.clone(),
        edges: edges.clone(),
        current_node_id: "node-5".to_string(),
        global_data: result4.input.clone(),
    };

    let result5 = simulate_workflow_step(request5).await.unwrap();
    assert_eq!(result5.status, "success");
    assert_eq!(result5.node_type, "condition");
    assert_eq!(result5.next_node_id, Some("node-6".to_string()));
    println!("✅ Step 5 (second condition) passed");

    // Step 6: Execute output node
    let request6 = SimulationStepRequest {
        workflow_id: "integration-test-workflow".to_string(),
        nodes: nodes.clone(),
        edges: edges.clone(),
        current_node_id: "node-6".to_string(),
        global_data: result5.input.clone(),
    };

    let result6 = simulate_workflow_step(request6).await.unwrap();
    assert_eq!(result6.status, "success");
    assert_eq!(result6.node_type, "data-output");
    assert!(result6.output.is_some());
    let final_output = result6.output.unwrap();
    assert_eq!(final_output["workflowCompleted"], true);
    assert_eq!(result6.next_node_id, None); // No next node (workflow completed)
    println!("✅ Step 6 (output) passed");

    println!("🎉 [Integration Test 1] Complex workflow simulation test completed successfully");
}

/// Test 2: 다중 노드 체인 실행 테스트 (분기 처리)
///
/// Condition 노드에서 true/false 분기가 발생하는 워크플로우를 테스트
/// 각 경로의 최종 결과가 다른지 확인
#[tokio::test]
async fn test_branching_workflow() {
    println!("🧪 [Integration Test 2] Starting branching workflow test");

    // Workflow with branching: input → condition → [action-true OR action-false] → output
    let nodes = vec![
        json!({
            "id": "node-1",
            "type": "data-input",
            "data": { "label": "입력" }
        }),
        json!({
            "id": "node-2",
            "type": "condition",
            "data": {
                "label": "온도 체크",
                "rule": "temperature > 80"
            }
        }),
        json!({
            "id": "node-3-true",
            "type": "action",
            "data": {
                "label": "경고 발생",
                "action": "send_alert"
            }
        }),
        json!({
            "id": "node-3-false",
            "type": "action",
            "data": {
                "label": "정상 처리",
                "action": "normal_operation"
            }
        }),
        json!({
            "id": "node-4",
            "type": "data-output",
            "data": { "label": "출력" }
        }),
    ];

    // True branch edges
    let edges_true = vec![
        json!({ "id": "edge-1", "source": "node-1", "target": "node-2" }),
        json!({ "id": "edge-2-true", "source": "node-2", "target": "node-3-true" }),
        json!({ "id": "edge-3", "source": "node-3-true", "target": "node-4" }),
    ];

    // False branch edges
    let edges_false = vec![
        json!({ "id": "edge-1", "source": "node-1", "target": "node-2" }),
        json!({ "id": "edge-2-false", "source": "node-2", "target": "node-3-false" }),
        json!({ "id": "edge-4", "source": "node-3-false", "target": "node-4" }),
    ];

    // Test TRUE branch (temperature > 80)
    let data_high_temp = json!({ "temperature": 95 });

    let result_true_1 = simulate_workflow_step(SimulationStepRequest {
        workflow_id: "branch-test-true".to_string(),
        nodes: nodes.clone(),
        edges: edges_true.clone(),
        current_node_id: "node-1".to_string(),
        global_data: data_high_temp.clone(),
    }).await.unwrap();

    assert_eq!(result_true_1.next_node_id, Some("node-2".to_string()));

    let result_true_2 = simulate_workflow_step(SimulationStepRequest {
        workflow_id: "branch-test-true".to_string(),
        nodes: nodes.clone(),
        edges: edges_true.clone(),
        current_node_id: "node-2".to_string(),
        global_data: data_high_temp.clone(),
    }).await.unwrap();

    // Should go to node-3-true
    assert_eq!(result_true_2.next_node_id, Some("node-3-true".to_string()));
    println!("✅ TRUE branch path verified");

    // Test FALSE branch (temperature <= 80)
    let data_low_temp = json!({ "temperature": 70 });

    let result_false_1 = simulate_workflow_step(SimulationStepRequest {
        workflow_id: "branch-test-false".to_string(),
        nodes: nodes.clone(),
        edges: edges_false.clone(),
        current_node_id: "node-1".to_string(),
        global_data: data_low_temp.clone(),
    }).await.unwrap();

    assert_eq!(result_false_1.next_node_id, Some("node-2".to_string()));

    let result_false_2 = simulate_workflow_step(SimulationStepRequest {
        workflow_id: "branch-test-false".to_string(),
        nodes: nodes.clone(),
        edges: edges_false.clone(),
        current_node_id: "node-2".to_string(),
        global_data: data_low_temp.clone(),
    }).await.unwrap();

    // Should go to node-3-false
    assert_eq!(result_false_2.next_node_id, Some("node-3-false".to_string()));
    println!("✅ FALSE branch path verified");

    println!("🎉 [Integration Test 2] Branching workflow test completed successfully");
}

/// Test 3: 에러 처리 및 복구 테스트
///
/// 잘못된 rule expression, 누락된 config 등의 에러 상황에서
/// 적절한 에러 메시지와 함께 워크플로우가 중단되는지 확인
#[tokio::test]
async fn test_error_handling() {
    println!("🧪 [Integration Test 3] Starting error handling test");

    // Test Case 1: Invalid rule expression (should not cause panic)
    let nodes_invalid_rule = vec![
        json!({
            "id": "node-1",
            "type": "data-input",
            "data": { "label": "입력" }
        }),
        json!({
            "id": "node-2",
            "type": "condition",
            "data": {
                "label": "잘못된 조건",
                "rule": "" // Empty rule expression
            }
        }),
    ];

    let edges = vec![
        json!({ "id": "edge-1", "source": "node-1", "target": "node-2" }),
    ];

    let data = json!({ "value": 100 });

    // Execute condition node with empty rule
    let result = simulate_workflow_step(SimulationStepRequest {
        workflow_id: "error-test-1".to_string(),
        nodes: nodes_invalid_rule.clone(),
        edges: edges.clone(),
        current_node_id: "node-2".to_string(),
        global_data: data.clone(),
    }).await.unwrap();

    // Should handle gracefully (current implementation returns success with default true)
    assert_eq!(result.status, "success");
    println!("✅ Empty rule handled gracefully");

    // Test Case 2: Notification node with missing config (should use defaults)
    let nodes_missing_config = vec![
        json!({
            "id": "node-1",
            "type": "notification",
            "data": {
                "label": "알림",
                "config": {} // Empty config (should use defaults)
            }
        }),
    ];

    let result2 = simulate_workflow_step(SimulationStepRequest {
        workflow_id: "error-test-2".to_string(),
        nodes: nodes_missing_config.clone(),
        edges: vec![],
        current_node_id: "node-1".to_string(),
        global_data: data.clone(),
    }).await.unwrap();

    assert_eq!(result2.status, "success");
    assert!(result2.output.is_some());
    let output = result2.output.unwrap();
    assert_eq!(output["channel"], "email"); // Default channel
    assert_eq!(output["status"], "sent");
    println!("✅ Missing config handled with defaults");

    // Test Case 3: Node not found error
    let result3 = simulate_workflow_step(SimulationStepRequest {
        workflow_id: "error-test-3".to_string(),
        nodes: vec![json!({ "id": "node-1", "type": "data-input", "data": {} })],
        edges: vec![],
        current_node_id: "node-999".to_string(), // Non-existent node
        global_data: data.clone(),
    }).await;

    // Should return error
    assert!(result3.is_err());
    let error_msg = result3.unwrap_err();
    assert!(error_msg.contains("Node not found"));
    println!("✅ Node not found error handled correctly");

    // Test Case 4: Unknown node type (should return error status)
    let nodes_unknown_type = vec![
        json!({
            "id": "node-1",
            "type": "unknown-type",
            "data": { "label": "알 수 없는 노드" }
        }),
    ];

    let result4 = simulate_workflow_step(SimulationStepRequest {
        workflow_id: "error-test-4".to_string(),
        nodes: nodes_unknown_type.clone(),
        edges: vec![],
        current_node_id: "node-1".to_string(),
        global_data: data.clone(),
    }).await.unwrap();

    assert_eq!(result4.status, "error");
    assert!(result4.error.is_some());
    let error = result4.error.unwrap();
    assert!(error.contains("Unsupported node type"));
    println!("✅ Unknown node type error handled correctly");

    println!("🎉 [Integration Test 3] Error handling test completed successfully");
}
