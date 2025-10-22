---
name: run-load-test
description: Run Apache Bench load tests on microservices to measure performance under stress
---

Run load tests using Apache Bench to measure microservice performance under various stress levels.

## 🎯 언제 사용하나요?

### ✅ 사용 조건
- 서비스 성능 한계를 테스트하고 싶을 때
- 배포 전 성능 검증이 필요할 때
- 동시 사용자 증가시 응답 시간 변화를 측정할 때
- Before/After 성능 비교가 필요할 때

### ❌ 사용하지 말아야 할 경우
- 프로덕션 환경 테스트 → **반드시 개발/스테이징 환경에서만 실행**
- 복잡한 성능 최적화 → **Task tool로 performance-engineer Agent 사용**
- 상세한 프로파일링 → **performance-engineer Agent + cProfile 사용**

---

## 📋 사용 방법

```bash
/run-load-test service-name endpoint [concurrency] [requests]
```

**예시:**
```bash
/run-load-test judgment-service /api/v2/judgment/execute 100 10000
/run-load-test workflow-service /api/v2/workflow 50 5000
/run-load-test quick  # 모든 서비스 빠른 테스트 (10 concurrent, 1000 requests)
```

**파라미터:**
- `service-name`: 테스트 대상 서비스 (또는 "all")
- `endpoint`: API 엔드포인트 경로
- `concurrency`: 동시 요청 수 (기본: 100)
- `requests`: 총 요청 수 (기본: 10000)

---

## 🔧 테스트 시나리오

### 1. 기본 부하 테스트 (Baseline)

```bash
Apache Bench 설정:
- 동시 요청: 100
- 총 요청: 10,000
- Keep-Alive: 활성화
- 타임아웃: 30초
```

**목적**: 일반적인 운영 환경 성능 측정

### 2. 고부하 테스트 (Stress Test)

```bash
Apache Bench 설정:
- 동시 요청: 500
- 총 요청: 50,000
- Keep-Alive: 활성화
```

**목적**: 피크 타임 성능 측정

### 3. 극한 부하 테스트 (Spike Test)

```bash
Apache Bench 설정:
- 동시 요청: 1000
- 총 요청: 100,000
- Keep-Alive: 비활성화
```

**목적**: 시스템 한계점 파악

### 4. 지속 부하 테스트 (Endurance Test)

```bash
Apache Bench 설정:
- 동시 요청: 200
- 총 요청: 500,000
- 시간: 60분
```

**목적**: 메모리 누수, 연결 풀 고갈 감지

---

## 📊 생성되는 리포트

### 1. 실시간 진행 상황

```bash
$ /run-load-test judgment-service /api/v2/judgment/execute 100 10000

🚀 Starting load test on Judgment Service...

Target: http://localhost:8002/api/v2/judgment/execute
Concurrency: 100
Total Requests: 10,000

Progress: [████████████████████████████████] 100% (10,000/10,000)

⏱️  Elapsed: 45.2s
📊 Requests/sec: 221.2
```

### 2. 상세 결과 리포트

```
📊 Load Test Report - Judgment Service
=====================================

Test Configuration:
  - Endpoint: /api/v2/judgment/execute
  - Concurrency: 100
  - Total Requests: 10,000
  - Test Duration: 45.2 seconds

Performance Metrics:
  ✅ Requests per second: 221.2 req/s
  ✅ Time per request: 452ms (mean)
  ✅ Time per request: 4.52ms (mean, across all concurrent)

Response Time Distribution:
  - Min: 180ms
  - Mean: 452ms ✅
  - Median: 420ms ✅
  - P95: 680ms ⚠️
  - P99: 920ms ❌
  - Max: 1,450ms

Status Code Distribution:
  - 200 OK: 9,850 (98.5%) ✅
  - 500 Internal Server Error: 150 (1.5%) ⚠️
  - Timeout: 0 (0%)

Connection Stats:
  - Connect: 12ms (mean)
  - Processing: 440ms (mean)
  - Waiting: 432ms (mean)
  - Total: 452ms (mean)

Throughput:
  - Transfer rate: 2,456 KB/sec
  - Total transferred: 108.5 MB

Verdict:
  ⚠️ NEEDS OPTIMIZATION
  - P99 exceeds 500ms target (920ms)
  - 1.5% error rate (target: < 1%)

Recommendations:
  1. Optimize database queries (high waiting time)
  2. Increase connection pool size
  3. Add Redis caching for frequent queries
  4. Contact performance-engineer Agent for detailed analysis
```

### 3. 서비스별 비교표

```
| Service | RPS | Mean | P95 | P99 | Error% | Status |
|---------|-----|------|-----|-----|--------|--------|
| API Gateway (8000) | 450 | 220ms | 380ms | 520ms | 0.1% | ✅ |
| Workflow (8001) | 280 | 355ms | 580ms | 720ms | 0.5% | ✅ |
| Judgment (8002) | 221 | 452ms | 680ms | 920ms | 1.5% | ⚠️ |
| Action (8003) | 320 | 310ms | 480ms | 650ms | 0.3% | ✅ |
| Logging (8005) | 890 | 112ms | 180ms | 240ms | 0.0% | ✅ |
| Learning (8009) | 145 | 689ms | 1100ms | 1450ms | 2.1% | ❌ |

Legend:
  ✅ Excellent (P99 < 500ms, Error < 1%)
  ⚠️ Needs Optimization (P99 < 1000ms, Error < 2%)
  ❌ Critical (P99 >= 1000ms, Error >= 2%)
```

---

## 🎯 목표 성능 기준 (Ver2.0 Final)

| 서비스 | 목표 RPS | 목표 평균 응답 | 목표 P99 | 목표 에러율 |
|--------|----------|---------------|----------|------------|
| **API Gateway** | >= 400 | < 250ms | < 500ms | < 0.5% |
| **Workflow** | >= 250 | < 400ms | < 800ms | < 1% |
| **Judgment** | >= 200 | < 500ms | < 1000ms | < 1% |
| **Learning** | >= 100 | < 700ms | < 1500ms | < 2% |
| **BI Service** | >= 50 | < 2000ms | < 5000ms | < 2% |
| **Others** | >= 300 | < 300ms | < 600ms | < 1% |

---

## 🚀 실행 예시

### 예시 1: Judgment Service 기본 테스트

```bash
$ /run-load-test judgment-service /api/v2/judgment/execute 100 10000

🚀 Starting load test...
📊 Results:
  - RPS: 221.2 ✅
  - Mean: 452ms ✅
  - P99: 920ms ⚠️
  - Error: 1.5% ⚠️

📄 Detailed report saved to:
  - load-tests/judgment-service-2024-01-22-10-30.txt
  - load-tests/judgment-service-2024-01-22-10-30.json
```

### 예시 2: 모든 서비스 빠른 테스트

```bash
$ /run-load-test quick

🚀 Quick load test on all 9 services...

✅ API Gateway: 450 RPS, 220ms mean, 0.1% error
✅ Workflow: 280 RPS, 355ms mean, 0.5% error
⚠️ Judgment: 221 RPS, 452ms mean, 1.5% error
✅ Action: 320 RPS, 310ms mean, 0.3% error
... (9 services total)

📊 Overall Status: 7/9 ✅, 2/9 ⚠️
```

---

## 🚀 다음 단계 추천

부하 테스트 후:

1. **메트릭 수집**: `/collect-metrics` Skill로 상세 메트릭 확인
2. **비교 분석**: `/compare-metrics` Skill로 이전 테스트와 비교
3. **성능 최적화**: performance-engineer Agent에게 최적화 요청
4. **재테스트**: 최적화 후 동일 테스트 재실행하여 개선도 측정
5. **프로덕션 배포**: 모든 목표 달성시 배포 진행

---

## 💡 주의사항

- **프로덕션 금지**: 절대 프로덕션 환경에서 실행하지 말 것
- **데이터베이스 영향**: 테스트 데이터베이스 사용 권장
- **네트워크 부하**: 로컬 환경 테스트 권장 (외부 네트워크 영향 최소화)
- **리소스 모니터링**: 테스트 중 CPU/메모리 모니터링 필수

---

## 🔗 관련 리소스

- **Agent 활용**: performance-engineer (최적화), database-optimization (쿼리 튜닝)
- **다음 Skill**: `/collect-metrics`, `/compare-metrics`
- **도구**: Apache Bench (ab), wrk, Grafana
- **문서**: [docs/operations/deployment_guide.md](../../docs/operations/deployment_guide.md)
