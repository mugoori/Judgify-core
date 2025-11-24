// Phase 9-2: AI Workflow Generator - Unit Tests
// 목적: generate_workflow_draft 커맨드와 create_workflow_dsl_prompt 함수의 유닛 테스트

#[cfg(test)]
mod workflow_ai_tests {
    use crate::commands::workflow_v2::{WorkflowStep, NodeType};

    // ============================================================================
    // 1. System Prompt 검증 테스트
    // ============================================================================

    #[test]
    fn test_system_prompt_contains_all_node_types() {
        // Given: System prompt 생성
        let system_prompt = super::super::create_workflow_dsl_prompt();

        // When: 6개 NodeType이 모두 포함되어 있는지 검증
        let expected_types = vec![
            "TRIGGER", "QUERY", "CALC", "JUDGMENT", "APPROVAL", "ALERT"
        ];

        // Then: 모든 NodeType이 시스템 프롬프트에 포함되어야 함
        for node_type in expected_types {
            assert!(
                system_prompt.contains(node_type),
                "System prompt should contain NodeType: {}",
                node_type
            );
        }
    }

    #[test]
    fn test_system_prompt_contains_few_shot_examples() {
        // Given: System prompt 생성
        let system_prompt = super::super::create_workflow_dsl_prompt();

        // When: 5개 Few-shot 예시가 포함되어 있는지 검증
        let expected_examples = vec![
            "불량률 모니터링",      // Example 1
            "설비 가동률 분석",     // Example 2
            "AI 품질 판단",         // Example 3
            "주기적 품질 모니터링", // Example 4
            "다단계 승인 프로세스", // Example 5
        ];

        // Then: 모든 예시가 시스템 프롬프트에 포함되어야 함
        for example in expected_examples {
            assert!(
                system_prompt.contains(example),
                "System prompt should contain example: {}",
                example
            );
        }
    }

    // ============================================================================
    // 2. JSON 파싱 검증 테스트
    // ============================================================================

    #[test]
    fn test_parse_simple_workflow_json() {
        // Given: Claude가 반환한 간단한 워크플로우 JSON
        let json_response = r#"[
            {
                "id": "trigger_1",
                "type": "TRIGGER",
                "label": "불량 감지",
                "config": {
                    "triggerType": "threshold",
                    "metric": "불량률",
                    "condition": "> 3%"
                }
            },
            {
                "id": "alert_1",
                "type": "ALERT",
                "label": "알림 전송",
                "config": {
                    "channel": "slack",
                    "message": "불량률 초과"
                }
            }
        ]"#;

        // When: JSON 파싱
        let result: Result<Vec<WorkflowStep>, _> = serde_json::from_str(json_response);

        // Then: 파싱 성공 및 2개 스텝 확인
        assert!(result.is_ok(), "JSON parsing should succeed");
        let steps = result.unwrap();
        assert_eq!(steps.len(), 2, "Should have 2 steps");
        assert_eq!(steps[0].step_type, NodeType::Trigger);
        assert_eq!(steps[1].step_type, NodeType::Alert);
    }

    #[test]
    fn test_parse_complex_workflow_json() {
        // Given: Claude가 반환한 복잡한 워크플로우 JSON (6개 NodeType 모두 포함)
        let json_response = r#"[
            {
                "id": "trigger_1",
                "type": "TRIGGER",
                "label": "매 시간 실행",
                "config": { "cron": "0 * * * *" }
            },
            {
                "id": "query_1",
                "type": "QUERY",
                "label": "불량률 조회",
                "config": { "sql": "SELECT AVG(defect_rate) FROM line_1" }
            },
            {
                "id": "calc_1",
                "type": "CALC",
                "label": "평균 계산",
                "config": { "formula": "SUM(values) / COUNT(values)" }
            },
            {
                "id": "judgment_1",
                "type": "JUDGMENT",
                "label": "판단 실행",
                "config": { "rule": "defect_rate > 3%" }
            },
            {
                "id": "approval_1",
                "type": "APPROVAL",
                "label": "팀장 승인",
                "config": { "approver": "생산팀장" }
            },
            {
                "id": "alert_1",
                "type": "ALERT",
                "label": "알림 전송",
                "config": { "channel": "slack" }
            }
        ]"#;

        // When: JSON 파싱
        let result: Result<Vec<WorkflowStep>, _> = serde_json::from_str(json_response);

        // Then: 파싱 성공 및 6개 NodeType 모두 확인
        assert!(result.is_ok(), "JSON parsing should succeed");
        let steps = result.unwrap();
        assert_eq!(steps.len(), 6, "Should have 6 steps");

        // 각 NodeType 검증
        assert_eq!(steps[0].step_type, NodeType::Trigger);
        assert_eq!(steps[1].step_type, NodeType::Query);
        assert_eq!(steps[2].step_type, NodeType::Calc);
        assert_eq!(steps[3].step_type, NodeType::Judgment);
        assert_eq!(steps[4].step_type, NodeType::Approval);
        assert_eq!(steps[5].step_type, NodeType::Alert);
    }

    #[test]
    fn test_parse_invalid_json_should_fail() {
        // Given: 잘못된 JSON (type 필드 누락)
        let invalid_json = r#"[
            {
                "id": "trigger_1",
                "label": "불량 감지",
                "config": {}
            }
        ]"#;

        // When: JSON 파싱 시도
        let result: Result<Vec<WorkflowStep>, _> = serde_json::from_str(invalid_json);

        // Then: 파싱 실패해야 함
        assert!(result.is_err(), "Invalid JSON should fail to parse");
    }

    // ============================================================================
    // 3. Markdown Code Block 제거 테스트
    // ============================================================================

    #[test]
    fn test_strip_markdown_json_block() {
        // Given: Markdown JSON code block으로 감싸진 응답
        let markdown_response = r#"```json
[
    {
        "id": "trigger_1",
        "type": "TRIGGER",
        "label": "테스트",
        "config": {}
    }
]
```"#;

        // When: ChatService.strip_markdown_code_block 호출
        let clean_json = strip_markdown_code_block(markdown_response);

        // Then: Markdown 제거되고 순수 JSON만 남아야 함
        assert!(!clean_json.starts_with("```"), "Should not start with ```");
        assert!(!clean_json.ends_with("```"), "Should not end with ```");
        assert!(clean_json.trim().starts_with("["), "Should start with [");
    }

    #[test]
    fn test_strip_markdown_plain_code_block() {
        // Given: 일반 Markdown code block으로 감싸진 응답
        let markdown_response = r#"```
[
    {
        "id": "trigger_1",
        "type": "TRIGGER",
        "label": "테스트",
        "config": {}
    }
]
```"#;

        // When: ChatService.strip_markdown_code_block 호출
        let clean_json = strip_markdown_code_block(markdown_response);

        // Then: Markdown 제거되고 순수 JSON만 남아야 함
        assert!(!clean_json.starts_with("```"), "Should not start with ```");
        assert!(!clean_json.ends_with("```"), "Should not end with ```");
        assert!(clean_json.trim().starts_with("["), "Should start with [");
    }

    #[test]
    fn test_no_strip_for_clean_json() {
        // Given: 이미 깨끗한 JSON (Markdown 없음)
        let clean_json_response = r#"[
    {
        "id": "trigger_1",
        "type": "TRIGGER",
        "label": "테스트",
        "config": {}
    }
]"#;

        // When: ChatService.strip_markdown_code_block 호출
        let result = strip_markdown_code_block(clean_json_response);

        // Then: 원본 그대로 반환되어야 함
        assert_eq!(
            result.trim(),
            clean_json_response.trim(),
            "Clean JSON should remain unchanged"
        );
    }

    // ============================================================================
    // Helper: strip_markdown_code_block 로직 복사 (테스트용)
    // ============================================================================

    fn strip_markdown_code_block(content: &str) -> &str {
        let trimmed = content.trim();

        if trimmed.starts_with("```json") {
            let without_start = trimmed.strip_prefix("```json").unwrap().trim();
            return without_start.strip_suffix("```").unwrap_or(without_start).trim();
        }

        if trimmed.starts_with("```") {
            let without_start = trimmed.strip_prefix("```").unwrap().trim();
            return without_start.strip_suffix("```").unwrap_or(without_start).trim();
        }

        trimmed
    }

    // ============================================================================
    // 4. Mock Integration Test (선택사항)
    // ============================================================================

    #[tokio::test]
    #[ignore] // CI에서는 실행 안 함 (실제 API 키 필요)
    async fn test_generate_workflow_draft_integration() {
        // Given: 실제 사용자 입력 시뮬레이션
        let user_prompt = "1호선 불량률이 3% 초과하면 팀장에게 알림 보내기";

        // When: generate_workflow_draft 호출 (Mock ChatService 필요)
        // Note: 실제 구현시 ChatService를 Mock으로 대체 필요

        // Then: WorkflowStep 배열 반환 확인
        // assert!(result.is_ok());
        // let steps = result.unwrap();
        // assert!(!steps.is_empty());

        // 테스트 작성 가이드:
        // 1. ChatService를 trait으로 추출
        // 2. MockChatService 구현 (고정된 JSON 반환)
        // 3. generate_workflow_draft에 ChatService trait 주입
        // 4. 이 테스트에서 MockChatService 사용
    }
}
