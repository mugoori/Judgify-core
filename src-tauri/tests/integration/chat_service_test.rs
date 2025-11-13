use judgify_core::services::chat_service::{ChatService, ChatMessage, ChatSession};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_send_message_integration() {
    let chat_service = ChatService::new();
    let session_id = "test-session-1";

    // 메시지 전송
    let user_message = ChatMessage {
        role: "user".to_string(),
        content: "Hello, how are you?".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = chat_service.send_message(session_id, user_message.clone());

    // 검증
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.content.len() > 0);
    assert_eq!(response.role, "assistant");
}

#[test]
fn test_message_history_integration() {
    let chat_service = ChatService::new();
    let session_id = "test-session-2";

    // 여러 메시지 전송
    for i in 1..=3 {
        let message = ChatMessage {
            role: "user".to_string(),
            content: format!("Message {}", i),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        chat_service.send_message(session_id, message).ok();
        thread::sleep(Duration::from_millis(10));
    }

    // 히스토리 조회
    let history = chat_service.get_message_history(session_id);

    // 검증 (3개 유저 메시지 + 3개 어시스턴트 응답 = 6개)
    assert!(history.len() >= 3);
    assert!(history.iter().any(|msg| msg.content.contains("Message 1")));
    assert!(history.iter().any(|msg| msg.content.contains("Message 2")));
    assert!(history.iter().any(|msg| msg.content.contains("Message 3")));
}

#[test]
fn test_session_management_integration() {
    let chat_service = ChatService::new();

    // 새 세션 생성
    let session_id = chat_service.create_session();
    assert!(session_id.len() > 0);

    // 세션 존재 확인
    let exists = chat_service.session_exists(&session_id);
    assert!(exists);

    // 세션 삭제
    chat_service.delete_session(&session_id);

    // 삭제 확인
    let exists_after_delete = chat_service.session_exists(&session_id);
    assert!(!exists_after_delete);
}

#[test]
fn test_streaming_response_integration() {
    let chat_service = ChatService::new();
    let session_id = "test-session-3";

    // 스트리밍 메시지 전송
    let user_message = ChatMessage {
        role: "user".to_string(),
        content: "Tell me a long story".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let mut stream = chat_service.send_message_stream(session_id, user_message);

    // 스트리밍 청크 수신
    let mut chunks = Vec::new();
    while let Some(chunk) = stream.next() {
        chunks.push(chunk);
        if chunks.len() >= 5 {
            break; // 최소 5개 청크 수신
        }
    }

    // 검증
    assert!(chunks.len() >= 1);
    assert!(chunks.iter().all(|c| c.len() > 0));
}

#[test]
fn test_context_preservation_integration() {
    let chat_service = ChatService::new();
    let session_id = "test-session-4";

    // 첫 번째 메시지
    let msg1 = ChatMessage {
        role: "user".to_string(),
        content: "My name is Alice".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    chat_service.send_message(session_id, msg1).ok();

    thread::sleep(Duration::from_millis(500));

    // 두 번째 메시지 (컨텍스트 참조)
    let msg2 = ChatMessage {
        role: "user".to_string(),
        content: "What is my name?".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    let response = chat_service.send_message(session_id, msg2).unwrap();

    // 컨텍스트 유지 확인 (응답에 "Alice" 포함 예상)
    assert!(
        response.content.to_lowercase().contains("alice") ||
        response.content.contains("이름") // 한글 응답 고려
    );
}

#[test]
fn test_error_handling_integration() {
    let chat_service = ChatService::new();
    let session_id = "test-session-5";

    // 빈 메시지 전송 시도
    let empty_message = ChatMessage {
        role: "user".to_string(),
        content: "".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let result = chat_service.send_message(session_id, empty_message);

    // 에러 또는 빈 응답 처리 확인
    match result {
        Err(_) => assert!(true), // 에러 발생 시 정상
        Ok(response) => {
            // 빈 메시지에 대한 기본 응답 확인
            assert!(response.content.len() > 0);
        }
    }
}

#[test]
fn test_concurrent_chat_sessions_integration() {
    let chat_service = Arc::new(ChatService::new());
    let mut handles = vec![];

    // 10개 동시 세션에서 메시지 전송
    for i in 0..10 {
        let service_clone = Arc::clone(&chat_service);
        let handle = thread::spawn(move || {
            let session_id = format!("concurrent-session-{}", i);
            let message = ChatMessage {
                role: "user".to_string(),
                content: format!("Concurrent message {}", i),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            service_clone.send_message(&session_id, message)
        });
        handles.push(handle);
    }

    // 모든 스레드 완료 대기
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap().is_ok() {
            success_count += 1;
        }
    }

    // 대부분의 메시지가 성공적으로 전송되어야 함
    assert!(success_count >= 8); // 최소 80% 성공률
}

#[test]
fn test_message_ordering_integration() {
    let chat_service = ChatService::new();
    let session_id = "test-session-6";

    // 순차적 메시지 전송
    let messages = vec!["First", "Second", "Third"];
    for content in messages.iter() {
        let message = ChatMessage {
            role: "user".to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        chat_service.send_message(session_id, message).ok();
        thread::sleep(Duration::from_millis(100));
    }

    // 히스토리 조회
    let history = chat_service.get_message_history(session_id);
    let user_messages: Vec<_> = history.iter()
        .filter(|msg| msg.role == "user")
        .collect();

    // 순서 확인
    assert!(user_messages.len() >= 3);
    assert!(user_messages[0].content.contains("First"));
    assert!(user_messages[1].content.contains("Second"));
    assert!(user_messages[2].content.contains("Third"));
}

#[test]
fn test_empty_session_handling_integration() {
    let chat_service = ChatService::new();
    let empty_session_id = "empty-session";

    // 빈 세션 히스토리 조회
    let history = chat_service.get_message_history(empty_session_id);

    // 빈 배열 반환 확인
    assert_eq!(history.len(), 0);
}

#[test]
fn test_chat_service_performance_metrics_integration() {
    let chat_service = ChatService::new();
    let session_id = "test-session-7";

    // 여러 메시지 전송 (메트릭 수집)
    for i in 1..=5 {
        let message = ChatMessage {
            role: "user".to_string(),
            content: format!("Performance test {}", i),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        chat_service.send_message(session_id, message).ok();
    }

    // 성능 메트릭 조회
    let metrics = chat_service.get_performance_metrics();

    // 메트릭 검증
    assert_eq!(metrics.total_messages_sent, 5);
    assert!(metrics.avg_response_time_ms > 0.0);
    assert!(metrics.total_sessions >= 1);
    assert!(metrics.avg_messages_per_session > 0.0);
}
