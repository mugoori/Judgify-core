---
name: collect-metrics
description: Collect Prometheus metrics from all microservices and generate performance reports
---

Collect Prometheus metrics from all 9 microservices and generate comprehensive performance reports.

## 🎯 언제 사용하나요?

### ✅ 사용 조건
- 서비스 성능 모니터링이 필요할 때
- 일일/주간/월간 성능 리포트 생성할 때
- 성능 병목 지점을 빠르게 파악하고 싶을 때
- Before/After 비교 데이터가 필요할 때

### ❌ 사용하지 말아야 할 경우
- 복잡한 성능 분석 및 최적화 → **Task tool로 performance-engineer Agent 사용**
- 부하 테스트 실행 → **/run-load-test Skill 사용**
- 상세한 프로파일링 → **performance-engineer Agent + cProfile 사용**

---

## 📋 사용 방법

```bash
/collect-metrics [time-range]
```

**예시:**
```bash
/collect-metrics last-hour
/collect-metrics last-24h
/collect-metrics last-7d
/collect-metrics custom 2024-01-20 2024-01-22
```

---

## 🔧 수집되는 메트릭 카테고리

### 1. HTTP 요청 메트릭

```yaml
http_requests_total:
  - 총 요청 수 (서비스별, 엔드포인트별, 상태코드별)

http_request_duration_seconds:
  - 응답 시간 (평균, 중앙값, P95, P99)
  - 목표: < 500ms (Judgment Service)

http_requests_in_progress:
  - 동시 처리 중인 요청 수
```

### 2. 비즈니스 메트릭 (Judgify-core 특화)

```yaml
judgment_executions_total:
  - 판단 실행 횟수
  - 라벨: method (rule|llm|hybrid), result (true|false), workflow_id

judgment_confidence_score:
  - 평균 신뢰도 점수
  - 목표: >= 0.7

judgment_execution_duration_seconds:
  - 판단 처리 시간
  - 목표: < 500ms

learning_rule_extractions_total:
  - 자동 Rule 추출 횟수
  - 라벨: algorithm (frequency|decision_tree|llm)

learning_fewshot_samples_count:
  - Few-shot 학습 샘플 수
  - 목표: 10-20 samples per workflow

dashboard_auto_generations_total:
  - 대시보드 자동 생성 횟수
  - 목표: < 30초 생성 시간
```

### 3. 인프라 메트릭

```yaml
database_connections_active:
  - PostgreSQL 활성 연결 수
  - 목표: < 100 connections

database_query_duration_seconds:
  - 쿼리 실행 시간
  - 목표: < 100ms

redis_cache_hit_ratio:
  - Redis 캐시 히트율
  - 목표: >= 80%

memory_usage_bytes:
  - 메모리 사용량 (서비스별)

cpu_usage_percent:
  - CPU 사용률 (서비스별)
```

---

## 📊 생성되는 리포트 형식

### 1. JSON 형식 (원본 데이터)

```json
{
  "timestamp": "2024-01-22T10:00:00Z",
  "time_range": {
    "start": "2024-01-22T09:00:00Z",
    "end": "2024-01-22T10:00:00Z"
  },
  "services": {
    "judgment-service": {
      "http": {
        "requests_total": 12453,
        "requests_per_second": 3.46,
        "avg_response_time_ms": 245,
        "p95_response_time_ms": 480,
        "p99_response_time_ms": 650,
        "error_rate": 0.02
      },
      "business": {
        "judgment_executions": 8234,
        "avg_confidence_score": 0.85,
        "rule_only_percent": 72,
        "llm_fallback_percent": 28,
        "hybrid_avg_time_ms": 380
      },
      "infrastructure": {
        "db_connections": 45,
        "db_avg_query_time_ms": 78,
        "redis_hit_rate": 0.89,
        "memory_mb": 512,
        "cpu_percent": 35
      }
    },
    "learning-service": {
      "business": {
        "rule_extractions": 234,
        "fewshot_samples_avg": 15,
        "extraction_accuracy": 0.88,
        "algorithm_distribution": {
          "frequency": 45,
          "decision_tree": 35,
          "llm": 20
        }
      }
    }
  },
  "summary": {
    "total_requests": 45678,
    "avg_response_time_ms": 320,
    "overall_error_rate": 0.015,
    "services_healthy": 9,
    "alerts": []
  }
}
```

### 2. Markdown 리포트 (가독성)

```markdown
# Judgify-core Ver2.0 성능 리포트

**기간**: 2024-01-22 09:00 ~ 10:00 (1시간)
**생성 시각**: 2024-01-22 10:05:00 UTC

---

## 📊 전체 요약

| 지표 | 값 | 목표 | 상태 |
|------|-----|------|------|
| **총 요청** | 45,678 | - | ✅ |
| **평균 응답 시간** | 320ms | < 500ms | ✅ |
| **에러율** | 1.5% | < 2% | ✅ |
| **서비스 상태** | 9/9 정상 | 9/9 | ✅ |

---

## 🎯 Judgment Service (8002)

### HTTP 성능
- **요청 수**: 12,453 (3.46 req/s)
- **평균 응답**: 245ms ✅
- **P95 응답**: 480ms ✅
- **P99 응답**: 650ms ⚠️
- **에러율**: 2.0% ✅

### 비즈니스 메트릭
- **판단 실행**: 8,234회
- **평균 신뢰도**: 0.85 ✅
- **Rule Only**: 72% (5,929회)
- **LLM Fallback**: 28% (2,305회)
- **하이브리드 평균 시간**: 380ms ✅

### 권장사항
⚠️ P99 응답 시간이 650ms로 목표(500ms) 초과
   → performance-engineer Agent로 병목 분석 권장

---

## 🧠 Learning Service (8009)

### 자동학습 메트릭
- **Rule 추출**: 234회
- **평균 Few-shot 샘플**: 15개 ✅
- **추출 정확도**: 88% ✅
- **알고리즘 분포**:
  - 빈도 분석: 45%
  - 결정 트리: 35%
  - LLM 패턴: 20%

---

## 💾 인프라 상태

| 서비스 | DB 연결 | 캐시 히트율 | 메모리 | CPU |
|--------|---------|------------|--------|-----|
| Judgment | 45 ✅ | 89% ✅ | 512MB ✅ | 35% ✅ |
| Learning | 23 ✅ | 85% ✅ | 256MB ✅ | 28% ✅ |
| Workflow | 32 ✅ | 92% ✅ | 384MB ✅ | 22% ✅ |

---

## 🚨 알림 및 권장사항

1. ⚠️ Judgment Service P99 응답 시간 최적화 필요
2. ✅ 모든 서비스 정상 작동 중
3. ✅ 캐시 히트율 목표(80%) 초과 달성
```

---

## 🚀 실행 결과 예시

```bash
$ /collect-metrics last-hour

📊 Collecting metrics from 9 microservices...

✅ API Gateway (8000): 15,234 requests, 180ms avg
✅ Workflow Service (8001): 4,567 requests, 220ms avg
✅ Judgment Service (8002): 12,453 requests, 245ms avg
✅ Action Service (8003): 3,456 requests, 150ms avg
✅ Notification Service (8004): 2,345 requests, 90ms avg
✅ Logging Service (8005): 45,678 requests, 50ms avg
✅ Data Visualization (8006): 1,234 requests, 320ms avg
✅ BI Service (8007): 789 requests, 1,200ms avg
✅ Chat Interface (8008): 567 requests, 450ms avg
✅ Learning Service (8009): 234 requests, 680ms avg

📄 Reports generated:
- metrics/2024-01-22-10-00.json (raw data)
- metrics/2024-01-22-10-00.md (readable report)

📈 Summary:
- Total requests: 45,678
- Avg response: 320ms ✅
- Error rate: 1.5% ✅
- Services healthy: 9/9 ✅

💾 Data saved to: metrics/ directory
```

---

## 🚀 다음 단계 추천

메트릭 수집 후:

1. **성능 분석**: performance-engineer Agent로 병목 지점 분석
2. **비교 분석**: `/compare-metrics` Skill로 이전 데이터와 비교
3. **부하 테스트**: `/run-load-test` Skill로 성능 한계 테스트
4. **최적화**: 문제 발견시 해당 Agent에게 최적화 요청
5. **모니터링**: Grafana 대시보드에서 실시간 모니터링

---

## 💡 주의사항

- **Prometheus 필수**: Prometheus 서버가 실행 중이어야 함
- **시간대**: UTC 기준으로 수집
- **데이터 보관**: 최근 30일 데이터만 로컬 저장
- **대용량 쿼리**: 30일 이상 데이터는 Prometheus 직접 쿼리

---

## 🔗 관련 리소스

- **Agent 활용**: performance-engineer (성능 분석), observability-engineer (모니터링 설정)
- **다음 Skill**: `/compare-metrics`, `/run-load-test`
- **도구**: Prometheus, Grafana
- **문서**: [docs/operations/deployment_guide.md](../../docs/operations/deployment_guide.md)
