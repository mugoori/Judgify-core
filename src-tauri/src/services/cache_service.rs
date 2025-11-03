use lru::LruCache;
use std::sync::{Arc, Mutex};
use std::num::NonZeroUsize;

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
        }
    }

    /// 캐시에서 히스토리 조회
    ///
    /// # Returns
    /// - `Some(Vec<ChatMessage>)` - 캐시 히트
    /// - `None` - 캐시 미스
    pub fn get(&self, session_id: &str) -> Option<Vec<ChatMessage>> {
        let mut cache = self.cache.lock().unwrap();
        let result = cache.get(session_id).cloned();

        // 통계 업데이트
        let mut stats = self.stats.lock().unwrap();
        if result.is_some() {
            stats.hits += 1;
            println!("✅ [Cache] HIT - session: {} (hits: {}, misses: {})",
                session_id, stats.hits, stats.misses);
        } else {
            stats.misses += 1;
            println!("❌ [Cache] MISS - session: {} (hits: {}, misses: {})",
                session_id, stats.hits, stats.misses);
        }

        result
    }

    /// 캐시에 히스토리 저장
    ///
    /// # Arguments
    /// * `session_id` - 세션 ID
    /// * `messages` - 저장할 메시지 목록 (최신순)
    pub fn put(&self, session_id: String, messages: Vec<ChatMessage>) {
        let mut cache = self.cache.lock().unwrap();

        // 최대 메시지 수 제한
        let limited_messages = if messages.len() > self.max_messages_per_session {
            println!("⚠️  [Cache] Limiting messages from {} to {} for session: {}",
                messages.len(), self.max_messages_per_session, session_id);
            messages[..self.max_messages_per_session].to_vec()
        } else {
            messages
        };

        println!("💾 [Cache] PUT - session: {}, messages: {}",
            session_id, limited_messages.len());

        cache.put(session_id, limited_messages);
    }

    /// 특정 세션 캐시 무효화
    ///
    /// # Note
    /// save_message() 호출시 자동으로 호출됨
    pub fn invalidate(&self, session_id: &str) {
        let mut cache = self.cache.lock().unwrap();
        cache.pop(session_id);

        // 통계 업데이트
        let mut stats = self.stats.lock().unwrap();
        stats.invalidations += 1;

        println!("🧹 [Cache] INVALIDATE - session: {} (total invalidations: {})",
            session_id, stats.invalidations);
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
    /// - `(current_size, capacity)` - (현재 캐시된 세션 수, 최대 용량)
    pub fn stats(&self) -> (usize, usize, CacheStats) {
        let cache = self.cache.lock().unwrap();
        let stats = self.stats.lock().unwrap().clone();
        (cache.len(), cache.cap().get(), stats)
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
}
