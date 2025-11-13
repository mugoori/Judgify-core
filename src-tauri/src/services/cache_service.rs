use lru::LruCache;
use std::sync::{Arc, Mutex};
use std::num::NonZeroUsize;
use std::time::{Duration, Instant};

/// 메모리 기반 LRU 캐시 서비스
/// - Key: session_id (String)
/// - Value: Vec<ChatMessage> (최근 20개 메시지)
///
/// 성능 목표:
/// - 캐시 히트: < 10ms
/// - 캐시 히트율: ~80%
/// - 메모리 사용: < 10MB (5 세션 × 20 메시지)
pub struct CacheService {
    /// LRU 캐시 (최대 5개 세션 유지)
    cache: Arc<Mutex<LruCache<String, Vec<ChatMessage>>>>,
    /// 세션당 최대 메시지 수
    max_messages_per_session: usize,
    /// 캐시 통계 (디버깅/모니터링용)
    stats: Arc<Mutex<CacheStats>>,
    /// 성능 메트릭 (Week 1-2 Task 1.1: Performance Instrumentation)
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
}

/// 채팅 메시지 캐시 구조체 (ChatMessage와 동일 구조)
#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub intent: Option<String>,
    pub created_at: String,
}

/// 캐시 통계 구조체
#[derive(Clone, Debug, Default)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub invalidations: usize,
}

/// 성능 메트릭 구조체 (Week 1-2 Task 1.1: Performance Instrumentation)
#[derive(Clone, Debug, Default)]
pub struct PerformanceMetrics {
    /// 총 GET 호출 횟수
    pub total_gets: usize,
    /// 총 PUT 호출 횟수
    pub total_puts: usize,
    /// 총 INVALIDATE 호출 횟수
    pub total_invalidates: usize,

    /// GET 메서드 평균 응답 시간 (나노초)
    pub avg_get_duration_ns: u128,
    /// PUT 메서드 평균 응답 시간 (나노초)
    pub avg_put_duration_ns: u128,
    /// INVALIDATE 메서드 평균 응답 시간 (나노초)
    pub avg_invalidate_duration_ns: u128,

    /// GET 최대 응답 시간 (나노초)
    pub max_get_duration_ns: u128,
    /// PUT 최대 응답 시간 (나노초)
    pub max_put_duration_ns: u128,
    /// INVALIDATE 최대 응답 시간 (나노초)
    pub max_invalidate_duration_ns: u128,

    /// GET 최소 응답 시간 (나노초)
    pub min_get_duration_ns: u128,
    /// PUT 최소 응답 시간 (나노초)
    pub min_put_duration_ns: u128,
    /// INVALIDATE 최소 응답 시간 (나노초)
    pub min_invalidate_duration_ns: u128,

    /// 현재 캐시된 총 메시지 수 (모든 세션 합계)
    pub total_cached_messages: usize,
    /// 현재 캐시 메모리 사용량 (바이트 추정치)
    pub estimated_memory_bytes: usize,
}

impl PerformanceMetrics {
    /// GET 메서드 성능 업데이트
    fn update_get_duration(&mut self, duration: Duration) {
        let duration_ns = duration.as_nanos();
        self.total_gets += 1;

        // 평균 계산 (누적 이동 평균)
        if self.avg_get_duration_ns == 0 {
            self.avg_get_duration_ns = duration_ns;
        } else {
            self.avg_get_duration_ns =
                (self.avg_get_duration_ns * (self.total_gets - 1) as u128 + duration_ns) / self.total_gets as u128;
        }

        // 최대/최소 업데이트
        if duration_ns > self.max_get_duration_ns {
            self.max_get_duration_ns = duration_ns;
        }
        if self.min_get_duration_ns == 0 || duration_ns < self.min_get_duration_ns {
            self.min_get_duration_ns = duration_ns;
        }
    }

    /// PUT 메서드 성능 업데이트
    fn update_put_duration(&mut self, duration: Duration) {
        let duration_ns = duration.as_nanos();
        self.total_puts += 1;

        if self.avg_put_duration_ns == 0 {
            self.avg_put_duration_ns = duration_ns;
        } else {
            self.avg_put_duration_ns =
                (self.avg_put_duration_ns * (self.total_puts - 1) as u128 + duration_ns) / self.total_puts as u128;
        }

        if duration_ns > self.max_put_duration_ns {
            self.max_put_duration_ns = duration_ns;
        }
        if self.min_put_duration_ns == 0 || duration_ns < self.min_put_duration_ns {
            self.min_put_duration_ns = duration_ns;
        }
    }

    /// INVALIDATE 메서드 성능 업데이트
    fn update_invalidate_duration(&mut self, duration: Duration) {
        let duration_ns = duration.as_nanos();
        self.total_invalidates += 1;

        if self.avg_invalidate_duration_ns == 0 {
            self.avg_invalidate_duration_ns = duration_ns;
        } else {
            self.avg_invalidate_duration_ns =
                (self.avg_invalidate_duration_ns * (self.total_invalidates - 1) as u128 + duration_ns) / self.total_invalidates as u128;
        }

        if duration_ns > self.max_invalidate_duration_ns {
            self.max_invalidate_duration_ns = duration_ns;
        }
        if self.min_invalidate_duration_ns == 0 || duration_ns < self.min_invalidate_duration_ns {
            self.min_invalidate_duration_ns = duration_ns;
        }
    }
}

impl CacheService {
    /// 새 캐시 서비스 생성
    ///
    /// # Arguments
    /// * `max_sessions` - 최대 캐시할 세션 수 (권장: 5)
    /// * `max_messages_per_session` - 세션당 최대 메시지 수 (권장: 20)
    pub fn new(max_sessions: usize, max_messages_per_session: usize) -> Self {
        let capacity = NonZeroUsize::new(max_sessions)
            .expect("max_sessions must be greater than 0");

        println!("📦 [CacheService] Initialized with capacity: {} sessions, {} messages/session",
            max_sessions, max_messages_per_session);

        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            max_messages_per_session,
            stats: Arc::new(Mutex::new(CacheStats::default())),
            performance_metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
        }
    }

    /// 캐시에서 히스토리 조회
    ///
    /// # Returns
    /// - `Some(Vec<ChatMessage>)` - 캐시 히트
    /// - `None` - 캐시 미스
    pub fn get(&self, session_id: &str) -> Option<Vec<ChatMessage>> {
        let start = Instant::now();

        let mut cache = self.cache.lock().unwrap();
        let result = cache.get(session_id).cloned();

        // 성능 메트릭 업데이트 (Week 1-2 Task 1.1)
        let duration = start.elapsed();
        let mut perf = self.performance_metrics.lock().unwrap();
        perf.update_get_duration(duration);

        // 통계 업데이트
        let mut stats = self.stats.lock().unwrap();
        if result.is_some() {
            stats.hits += 1;
            println!("✅ [Cache] HIT - session: {} | duration: {:.3}ms | hits: {}, misses: {} | hit_rate: {:.1}%",
                session_id,
                duration.as_secs_f64() * 1000.0,
                stats.hits,
                stats.misses,
                (stats.hits as f64 / (stats.hits + stats.misses) as f64) * 100.0
            );
        } else {
            stats.misses += 1;
            println!("❌ [Cache] MISS - session: {} | duration: {:.3}ms | hits: {}, misses: {} | hit_rate: {:.1}%",
                session_id,
                duration.as_secs_f64() * 1000.0,
                stats.hits,
                stats.misses,
                (stats.hits as f64 / (stats.hits + stats.misses) as f64) * 100.0
            );
        }

        result
    }

    /// 캐시에 히스토리 저장
    ///
    /// # Arguments
    /// * `session_id` - 세션 ID
    /// * `messages` - 저장할 메시지 목록 (최신순)
    pub fn put(&self, session_id: String, messages: Vec<ChatMessage>) {
        let start = Instant::now();

        let mut cache = self.cache.lock().unwrap();

        // 최대 메시지 수 제한
        let limited_messages = if messages.len() > self.max_messages_per_session {
            println!("⚠️  [Cache] Limiting messages from {} to {} for session: {}",
                messages.len(), self.max_messages_per_session, session_id);
            messages[..self.max_messages_per_session].to_vec()
        } else {
            messages
        };

        cache.put(session_id.clone(), limited_messages.clone());

        // 성능 메트릭 업데이트 (Week 1-2 Task 1.1)
        let duration = start.elapsed();
        let mut perf = self.performance_metrics.lock().unwrap();
        perf.update_put_duration(duration);

        println!("💾 [Cache] PUT - session: {} | messages: {} | duration: {:.3}ms | avg_put: {:.3}ms",
            session_id,
            limited_messages.len(),
            duration.as_secs_f64() * 1000.0,
            perf.avg_put_duration_ns as f64 / 1_000_000.0
        );
    }

    /// 특정 세션 캐시 무효화
    ///
    /// # Note
    /// save_message() 호출시 자동으로 호출됨
    pub fn invalidate(&self, session_id: &str) {
        let start = Instant::now();

        let mut cache = self.cache.lock().unwrap();
        cache.pop(session_id);

        // 성능 메트릭 업데이트 (Week 1-2 Task 1.1)
        let duration = start.elapsed();
        let mut perf = self.performance_metrics.lock().unwrap();
        perf.update_invalidate_duration(duration);

        // 통계 업데이트
        let mut stats = self.stats.lock().unwrap();
        stats.invalidations += 1;

        println!("🧹 [Cache] INVALIDATE - session: {} | duration: {:.3}ms | total: {} | avg_invalidate: {:.3}ms",
            session_id,
            duration.as_secs_f64() * 1000.0,
            stats.invalidations,
            perf.avg_invalidate_duration_ns as f64 / 1_000_000.0
        );
    }

    /// 전체 캐시 클리어
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        println!("🧹 [Cache] CLEAR - all sessions");
    }

    /// 캐시 통계 조회 (디버깅/모니터링용)
    ///
    /// # Returns
    /// - `(current_size, capacity, cache_stats)` - (현재 캐시된 세션 수, 최대 용량, 통계)
    pub fn stats(&self) -> (usize, usize, CacheStats) {
        let cache = self.cache.lock().unwrap();
        let stats = self.stats.lock().unwrap().clone();
        (cache.len(), cache.cap().get(), stats)
    }

    /// 성능 메트릭 조회 (Week 1-2 Task 1.1: Performance Instrumentation)
    ///
    /// # Returns
    /// - `PerformanceMetrics` - 실시간 성능 데이터 (응답 시간, 메모리 사용량 등)
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        let cache = self.cache.lock().unwrap();
        let mut perf = self.performance_metrics.lock().unwrap();

        // 메모리 사용량 계산 (추정치)
        let mut total_messages = 0;
        for (_, messages) in cache.iter() {
            total_messages += messages.len();
        }

        perf.total_cached_messages = total_messages;

        // 메시지당 평균 300바이트로 추정 (id + session_id + role + content + intent + created_at)
        // + 세션 ID 문자열 크기 (평균 36바이트, UUID)
        const AVG_MESSAGE_SIZE: usize = 300;
        const AVG_SESSION_ID_SIZE: usize = 36;
        perf.estimated_memory_bytes =
            (total_messages * AVG_MESSAGE_SIZE) + (cache.len() * AVG_SESSION_ID_SIZE);

        perf.clone()
    }

    /// 캐시 히트율 + 성능 정보 출력 (디버깅용)
    pub fn print_performance_summary(&self) {
        let (size, capacity, stats) = self.stats();
        let perf = self.get_performance_metrics();
        let hit_rate = self.hit_rate();

        println!("\n📊 [CacheService] Performance Summary");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📦 캐시 상태:");
        println!("   - 현재 세션 수: {}/{}", size, capacity);
        println!("   - 총 캐시된 메시지: {}", perf.total_cached_messages);
        println!("   - 메모리 사용량: {:.2} KB", perf.estimated_memory_bytes as f64 / 1024.0);
        println!("\n📈 히트/미스 통계:");
        println!("   - 히트: {}", stats.hits);
        println!("   - 미스: {}", stats.misses);
        println!("   - 히트율: {:.1}%", hit_rate);
        println!("   - 무효화: {}", stats.invalidations);
        println!("\n⏱️  평균 응답 시간:");
        println!("   - GET:        {:.3} ms", perf.avg_get_duration_ns as f64 / 1_000_000.0);
        println!("   - PUT:        {:.3} ms", perf.avg_put_duration_ns as f64 / 1_000_000.0);
        println!("   - INVALIDATE: {:.3} ms", perf.avg_invalidate_duration_ns as f64 / 1_000_000.0);
        println!("\n⚡ 최대 응답 시간:");
        println!("   - GET:        {:.3} ms", perf.max_get_duration_ns as f64 / 1_000_000.0);
        println!("   - PUT:        {:.3} ms", perf.max_put_duration_ns as f64 / 1_000_000.0);
        println!("   - INVALIDATE: {:.3} ms", perf.max_invalidate_duration_ns as f64 / 1_000_000.0);
        println!("\n🎯 최소 응답 시간:");
        println!("   - GET:        {:.3} ms", perf.min_get_duration_ns as f64 / 1_000_000.0);
        println!("   - PUT:        {:.3} ms", perf.min_put_duration_ns as f64 / 1_000_000.0);
        println!("   - INVALIDATE: {:.3} ms", perf.min_invalidate_duration_ns as f64 / 1_000_000.0);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }

    /// 캐시 히트율 계산
    pub fn hit_rate(&self) -> f64 {
        let stats = self.stats.lock().unwrap();
        let total = stats.hits + stats.misses;
        if total == 0 {
            0.0
        } else {
            (stats.hits as f64 / total as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = CacheService::new(3, 10);

        // 캐시 미스
        assert!(cache.get("session1").is_none());

        // 캐시 저장
        let messages = vec![
            ChatMessage {
                id: "msg1".to_string(),
                session_id: "session1".to_string(),
                role: "user".to_string(),
                content: "Hello".to_string(),
                intent: None,
                created_at: "2025-11-03".to_string(),
            }
        ];
        cache.put("session1".to_string(), messages.clone());

        // 캐시 히트
        let result = cache.get("session1");
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = CacheService::new(3, 10);

        let messages = vec![
            ChatMessage {
                id: "msg1".to_string(),
                session_id: "session1".to_string(),
                role: "user".to_string(),
                content: "Hello".to_string(),
                intent: None,
                created_at: "2025-11-03".to_string(),
            }
        ];

        cache.put("session1".to_string(), messages);
        assert!(cache.get("session1").is_some());

        // 무효화
        cache.invalidate("session1");
        assert!(cache.get("session1").is_none());
    }

    #[test]
    fn test_cache_message_limit() {
        let cache = CacheService::new(3, 5);

        // 10개 메시지 저장 (최대 5개만 유지)
        let messages: Vec<ChatMessage> = (0..10).map(|i| {
            ChatMessage {
                id: format!("msg{}", i),
                session_id: "session1".to_string(),
                role: "user".to_string(),
                content: format!("Message {}", i),
                intent: None,
                created_at: "2025-11-03".to_string(),
            }
        }).collect();

        cache.put("session1".to_string(), messages);

        let result = cache.get("session1").unwrap();
        assert_eq!(result.len(), 5); // 5개로 제한됨
    }

    #[test]
    fn test_cache_stats() {
        let cache = CacheService::new(3, 10);

        // 미스
        cache.get("session1");
        cache.get("session2");

        // 저장 후 히트
        let messages = vec![
            ChatMessage {
                id: "msg1".to_string(),
                session_id: "session1".to_string(),
                role: "user".to_string(),
                content: "Hello".to_string(),
                intent: None,
                created_at: "2025-11-03".to_string(),
            }
        ];
        cache.put("session1".to_string(), messages);
        cache.get("session1");
        cache.get("session1");

        let (size, capacity, stats) = cache.stats();
        assert_eq!(size, 1); // 1개 세션 캐시됨
        assert_eq!(capacity, 3); // 최대 3개
        assert_eq!(stats.hits, 2); // 2번 히트
        assert_eq!(stats.misses, 2); // 2번 미스
    }

    #[test]
    fn test_performance_instrumentation() {
        let cache = CacheService::new(5, 20);

        // 여러 작업 수행하여 성능 데이터 수집
        for i in 0..10 {
            let session_id = format!("session{}", i);
            let messages = vec![
                ChatMessage {
                    id: format!("msg{}", i),
                    session_id: session_id.clone(),
                    role: "user".to_string(),
                    content: format!("Test message {}", i),
                    intent: None,
                    created_at: "2025-11-04".to_string(),
                }
            ];

            // PUT
            cache.put(session_id.clone(), messages);

            // GET (HIT)
            assert!(cache.get(&session_id).is_some());

            // INVALIDATE
            cache.invalidate(&session_id);

            // GET (MISS)
            assert!(cache.get(&session_id).is_none());
        }

        // 성능 메트릭 조회
        let perf = cache.get_performance_metrics();

        // 검증: 각 메서드가 10번씩 호출되어야 함
        assert_eq!(perf.total_gets, 20); // HIT 10번 + MISS 10번
        assert_eq!(perf.total_puts, 10);
        assert_eq!(perf.total_invalidates, 10);

        // 평균 응답 시간이 기록되어야 함
        assert!(perf.avg_get_duration_ns > 0);
        assert!(perf.avg_put_duration_ns > 0);
        assert!(perf.avg_invalidate_duration_ns > 0);

        // 최대/최소 응답 시간이 기록되어야 함
        assert!(perf.max_get_duration_ns > 0);
        assert!(perf.min_get_duration_ns > 0);

        // 성능 요약 출력 (목표: <10ms for cache hits)
        cache.print_performance_summary();

        // 목표 검증: GET 평균 < 10ms (10,000,000 나노초)
        assert!(perf.avg_get_duration_ns < 10_000_000,
            "Average GET duration should be < 10ms, but was {:.3}ms",
            perf.avg_get_duration_ns as f64 / 1_000_000.0);
    }
}
