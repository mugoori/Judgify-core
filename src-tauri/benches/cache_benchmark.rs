use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use judgify_desktop::services::cache_service::{CacheService, ChatMessage};

/// 벤치마크: CacheService GET 성능 (Cache HIT)
///
/// 목표: < 1ms (1,000,000 ns)
fn benchmark_cache_get_hit(c: &mut Criterion) {
    let cache = CacheService::new(5, 20);

    // 사전 준비: 캐시에 데이터 저장
    let messages = vec![ChatMessage {
        id: "msg1".to_string(),
        session_id: "session1".to_string(),
        role: "user".to_string(),
        content: "Hello, this is a test message!".to_string(),
        intent: Some("test".to_string()),
        created_at: "2025-11-05".to_string(),
    }];
    cache.put("session1".to_string(), messages);

    c.bench_function("cache_get_hit", |b| {
        b.iter(|| {
            let result = cache.get(black_box("session1"));
            assert!(result.is_some());
        })
    });
}

/// 벤치마크: CacheService GET 성능 (Cache MISS)
///
/// 목표: < 1ms (1,000,000 ns)
fn benchmark_cache_get_miss(c: &mut Criterion) {
    let cache = CacheService::new(5, 20);

    c.bench_function("cache_get_miss", |b| {
        b.iter(|| {
            let result = cache.get(black_box("nonexistent_session"));
            assert!(result.is_none());
        })
    });
}

/// 벤치마크: CacheService PUT 성능
///
/// 목표: < 1ms (1,000,000 ns)
fn benchmark_cache_put(c: &mut Criterion) {
    let cache = CacheService::new(5, 20);

    let messages = vec![ChatMessage {
        id: "msg1".to_string(),
        session_id: "session1".to_string(),
        role: "user".to_string(),
        content: "Hello, this is a test message!".to_string(),
        intent: Some("test".to_string()),
        created_at: "2025-11-05".to_string(),
    }];

    c.bench_function("cache_put", |b| {
        b.iter(|| {
            cache.put(black_box("session1".to_string()), black_box(messages.clone()));
        })
    });
}

/// 벤치마크: CacheService INVALIDATE 성능
///
/// 목표: < 0.5ms (500,000 ns)
fn benchmark_cache_invalidate(c: &mut Criterion) {
    let cache = CacheService::new(5, 20);

    // 사전 준비: 캐시에 데이터 저장
    let messages = vec![ChatMessage {
        id: "msg1".to_string(),
        session_id: "session1".to_string(),
        role: "user".to_string(),
        content: "Hello, this is a test message!".to_string(),
        intent: Some("test".to_string()),
        created_at: "2025-11-05".to_string(),
    }];
    cache.put("session1".to_string(), messages);

    c.bench_function("cache_invalidate", |b| {
        b.iter(|| {
            cache.invalidate(black_box("session1"));
        })
    });
}

/// 벤치마크: 다양한 메시지 수에 따른 PUT 성능
///
/// 테스트 케이스: 1, 5, 10, 20, 50 메시지
fn benchmark_cache_put_varying_size(c: &mut Criterion) {
    let cache = CacheService::new(5, 100); // 충분한 용량

    let mut group = c.benchmark_group("cache_put_varying_size");

    for size in [1, 5, 10, 20, 50].iter() {
        let messages: Vec<ChatMessage> = (0..*size)
            .map(|i| ChatMessage {
                id: format!("msg{}", i),
                session_id: "session1".to_string(),
                role: "user".to_string(),
                content: format!("Test message number {}", i),
                intent: Some("test".to_string()),
                created_at: "2025-11-05".to_string(),
            })
            .collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                cache.put(black_box("session1".to_string()), black_box(messages.clone()));
            });
        });
    }

    group.finish();
}

/// 벤치마크: 동시성 시나리오 (여러 세션 동시 액세스)
///
/// 목표: 멀티스레드 환경에서도 안정적인 성능 유지
fn benchmark_cache_concurrent_access(c: &mut Criterion) {
    let cache = CacheService::new(10, 20);

    // 사전 준비: 10개 세션 데이터 저장
    for i in 0..10 {
        let messages = vec![ChatMessage {
            id: format!("msg{}", i),
            session_id: format!("session{}", i),
            role: "user".to_string(),
            content: format!("Message for session {}", i),
            intent: None,
            created_at: "2025-11-05".to_string(),
        }];
        cache.put(format!("session{}", i), messages);
    }

    c.bench_function("cache_concurrent_access", |b| {
        b.iter(|| {
            // 여러 세션 순차 액세스 (실제 동시성은 Criterion 제약으로 시뮬레이션)
            for i in 0..10 {
                let _ = cache.get(black_box(&format!("session{}", i)));
            }
        })
    });
}

/// 벤치마크: LRU 교체 성능 (용량 초과시)
///
/// 목표: 가장 오래된 세션 제거 < 1ms
fn benchmark_cache_lru_eviction(c: &mut Criterion) {
    c.bench_function("cache_lru_eviction", |b| {
        b.iter(|| {
            let cache = CacheService::new(3, 20); // 최대 3개 세션

            // 4개 세션 저장 (1개는 자동 제거됨)
            for i in 0..4 {
                let messages = vec![ChatMessage {
                    id: format!("msg{}", i),
                    session_id: format!("session{}", i),
                    role: "user".to_string(),
                    content: format!("Message {}", i),
                    intent: None,
                    created_at: "2025-11-05".to_string(),
                }];
                cache.put(black_box(format!("session{}", i)), black_box(messages));
            }

            // 가장 오래된 session0은 제거되어야 함
            assert!(cache.get("session0").is_none());
            assert!(cache.get("session3").is_some());
        })
    });
}

criterion_group!(
    benches,
    benchmark_cache_get_hit,
    benchmark_cache_get_miss,
    benchmark_cache_put,
    benchmark_cache_invalidate,
    benchmark_cache_put_varying_size,
    benchmark_cache_concurrent_access,
    benchmark_cache_lru_eviction
);
criterion_main!(benches);
