use judgify_core::services::cache_service::{CacheService, ChatMessage};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_cache_put_and_get_integration() {
    let cache = CacheService::new(5, 20);
    let session_id = "test-session-1";

    // 메시지 생성
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "Hi there!".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    // PUT 연산
    cache.put(session_id, messages.clone());

    // GET 연산
    let retrieved = cache.get(session_id);

    // 검증
    assert!(retrieved.is_some());
    let retrieved_messages = retrieved.unwrap();
    assert_eq!(retrieved_messages.len(), 2);
    assert_eq!(retrieved_messages[0].content, "Hello");
    assert_eq!(retrieved_messages[1].content, "Hi there!");
}

#[test]
fn test_cache_invalidation_integration() {
    let cache = CacheService::new(5, 20);
    let session_id = "test-session-2";

    // 메시지 저장
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Test message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    cache.put(session_id, messages.clone());

    // 캐시에 있는지 확인
    assert!(cache.get(session_id).is_some());

    // 무효화
    cache.invalidate(session_id);

    // 캐시에서 제거되었는지 확인
    assert!(cache.get(session_id).is_none());
}

#[test]
fn test_cache_lru_eviction() {
    let cache = CacheService::new(3, 20); // 최대 3개 세션

    // 3개 세션 추가
    for i in 1..=3 {
        let session_id = format!("session-{}", i);
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: format!("Message {}", i),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ];
        cache.put(&session_id, messages);
        thread::sleep(Duration::from_millis(10)); // 타임스탬프 차이 보장
    }

    // 모두 캐시에 있어야 함
    assert!(cache.get("session-1").is_some());
    assert!(cache.get("session-2").is_some());
    assert!(cache.get("session-3").is_some());

    // 4번째 세션 추가 (LRU 정책에 따라 session-1 제거)
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Message 4".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];
    cache.put("session-4", messages);

    // session-1은 제거되고, session-2, 3, 4는 남아있어야 함
    assert!(cache.get("session-1").is_none());
    assert!(cache.get("session-2").is_some());
    assert!(cache.get("session-3").is_some());
    assert!(cache.get("session-4").is_some());
}

#[test]
fn test_cache_message_limit_per_session() {
    let cache = CacheService::new(5, 3); // 세션당 최대 3개 메시지

    let session_id = "test-session-3";

    // 5개 메시지 추가 (3개만 유지되어야 함)
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Message 1".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "Message 2".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Message 3".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "Message 4".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Message 5".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    cache.put(session_id, messages);

    // 최대 3개만 유지되어야 함 (최신 3개)
    let retrieved = cache.get(session_id);
    assert!(retrieved.is_some());
    let retrieved_messages = retrieved.unwrap();
    assert_eq!(retrieved_messages.len(), 3);
    assert_eq!(retrieved_messages[0].content, "Message 3");
    assert_eq!(retrieved_messages[2].content, "Message 5");
}

#[test]
fn test_cache_concurrent_access() {
    let cache = Arc::new(CacheService::new(10, 20));
    let mut handles = vec![];

    // 10개 스레드에서 동시 쓰기
    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let session_id = format!("session-{}", i);
            let messages = vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: format!("Concurrent message {}", i),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
            ];
            cache_clone.put(&session_id, messages);
        });
        handles.push(handle);
    }

    // 모든 스레드 대기
    for handle in handles {
        handle.join().unwrap();
    }

    // 모든 세션이 저장되었는지 확인
    for i in 0..10 {
        let session_id = format!("session-{}", i);
        let retrieved = cache.get(&session_id);
        assert!(retrieved.is_some());
        let messages = retrieved.unwrap();
        assert_eq!(messages[0].content, format!("Concurrent message {}", i));
    }
}

#[test]
fn test_cache_get_miss() {
    let cache = CacheService::new(5, 20);

    // 존재하지 않는 세션 조회
    let result = cache.get("non-existent-session");

    // None 반환되어야 함
    assert!(result.is_none());
}

#[test]
fn test_cache_update_existing_session() {
    let cache = CacheService::new(5, 20);
    let session_id = "test-session-4";

    // 초기 메시지 저장
    let initial_messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Initial message".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];
    cache.put(session_id, initial_messages);

    // 메시지 업데이트
    let updated_messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Updated message 1".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "Updated message 2".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];
    cache.put(session_id, updated_messages);

    // 업데이트된 메시지 확인
    let retrieved = cache.get(session_id);
    assert!(retrieved.is_some());
    let messages = retrieved.unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].content, "Updated message 1");
    assert_eq!(messages[1].content, "Updated message 2");
}

#[test]
fn test_cache_empty_messages() {
    let cache = CacheService::new(5, 20);
    let session_id = "test-session-5";

    // 빈 메시지 배열 저장
    let empty_messages: Vec<ChatMessage> = vec![];
    cache.put(session_id, empty_messages);

    // 빈 배열이 저장되어야 함
    let retrieved = cache.get(session_id);
    assert!(retrieved.is_some());
    let messages = retrieved.unwrap();
    assert_eq!(messages.len(), 0);
}

#[test]
fn test_cache_performance_metrics() {
    let cache = CacheService::new(5, 20);
    let session_id = "test-session-6";

    // PUT + GET 연산 수행
    let messages = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Performance test".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    cache.put(session_id, messages);
    cache.get(session_id);

    // 성능 메트릭 가져오기
    let metrics = cache.get_performance_metrics();

    // 메트릭이 기록되었는지 확인
    assert_eq!(metrics.total_puts, 1);
    assert_eq!(metrics.total_gets, 1);
    assert!(metrics.avg_put_duration_ns > 0);
    assert!(metrics.avg_get_duration_ns > 0);
}

#[test]
fn test_cache_hit_rate() {
    let cache = CacheService::new(5, 20);

    // 캐시 저장
    cache.put("session-1", vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Test".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ]);

    // 히트
    cache.get("session-1");
    cache.get("session-1");

    // 미스
    cache.get("session-2");

    // 성능 메트릭 확인
    let metrics = cache.get_performance_metrics();

    // Hit rate = 2 / 3 = 66.67%
    assert_eq!(metrics.total_gets, 3);
    assert!(metrics.cache_hits > 0);
    assert!(metrics.cache_misses > 0);
}

#[test]
fn test_cache_invalidate_all() {
    let cache = CacheService::new(5, 20);

    // 여러 세션 저장
    for i in 1..=3 {
        let session_id = format!("session-{}", i);
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: format!("Message {}", i),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ];
        cache.put(&session_id, messages);
    }

    // 모두 캐시에 있어야 함
    assert!(cache.get("session-1").is_some());
    assert!(cache.get("session-2").is_some());
    assert!(cache.get("session-3").is_some());

    // 모두 무효화
    cache.invalidate("session-1");
    cache.invalidate("session-2");
    cache.invalidate("session-3");

    // 모두 제거되었는지 확인
    assert!(cache.get("session-1").is_none());
    assert!(cache.get("session-2").is_none());
    assert!(cache.get("session-3").is_none());
}
