---
name: compare-metrics
description: Compare Before/After metrics to measure optimization impact and generate delta reports
---

Compare performance metrics before and after optimization to measure improvement impact.

## 🎯 언제 사용하나요?

### ✅ 사용 조건
- 성능 최적화 전후 비교가 필요할 때
- 배포 전후 성능 변화를 측정할 때
- 코드 리팩토링 효과를 정량적으로 확인할 때
- A/B 테스트 결과를 비교할 때

### ❌ 사용하지 말아야 할 경우
- 메트릭 수집이 안 된 상태 → **먼저 /collect-metrics Skill 실행 필요**
- 복잡한 성능 분석 → **Task tool로 performance-engineer Agent 사용**
- 실시간 모니터링 → **Grafana 대시보드 사용**

---

## 📋 사용 방법

```bash
/compare-metrics before-file after-file
/compare-metrics auto  # 최근 2개 메트릭 파일 자동 비교
```

**예시:**
```bash
/compare-metrics metrics/2024-01-21-10-00.json metrics/2024-01-22-10-00.json
/compare-metrics auto
```

---

## 🔧 비교 항목

### 1. HTTP 성능 비교

```yaml
비교 지표:
  - Requests per second (RPS)
  - 평균 응답 시간 (Mean)
  - P95 응답 시간
  - P99 응답 시간
  - 에러율 (%)

목표:
  - RPS 증가 >= 10%
  - 응답 시간 감소 >= 20%
  - 에러율 감소
```

### 2. 비즈니스 메트릭 비교 (Judgify-core 특화)

```yaml
Judgment Service:
  - 판단 실행 속도 변화
  - 평균 신뢰도 점수 변화
  - Rule vs LLM 비율 변화
  - 하이브리드 판단 효율성

Learning Service:
  - Few-shot 샘플 최적화 효과
  - Rule 추출 정확도 변화
  - 알고리즘 선택 분포 변화

BI Service:
  - 대시보드 생성 시간 변화
  - MCP 컴포넌트 조립 효율성
```

### 3. 인프라 리소스 비교

```yaml
데이터베이스:
  - 연결 수 변화
  - 쿼리 실행 시간 변화
  - 인덱스 효율성

캐시:
  - Redis 히트율 변화
  - 캐시 메모리 사용량

시스템:
  - CPU 사용률 변화
  - 메모리 사용량 변화
```

---

## 📊 생성되는 비교 리포트

### 1. 요약 대시보드

```
📊 Before/After 성능 비교 리포트
===============================

비교 기간:
  BEFORE: 2024-01-21 10:00 ~ 11:00
  AFTER:  2024-01-22 10:00 ~ 11:00

전체 개선도:
  ✅ 응답 시간: 320ms → 198ms (-38.1%) 🎉
  ✅ RPS: 142 → 221 (+55.6%) 🎉
  ✅ 에러율: 2.3% → 0.8% (-65.2%) 🎉
  ✅ CPU 사용률: 68% → 42% (-38.2%) 🎉

종합 평가: 🏆 EXCELLENT - 모든 지표 개선
```

### 2. 서비스별 상세 비교

```
## Judgment Service (8002) 성능 비교

### HTTP 성능
| 지표 | Before | After | 변화 | 평가 |
|------|--------|-------|------|------|
| **RPS** | 142 | 221 | +55.6% | 🎉 |
| **평균 응답** | 452ms | 245ms | -45.8% | 🎉 |
| **P95 응답** | 680ms | 380ms | -44.1% | 🎉 |
| **P99 응답** | 920ms | 520ms | -43.5% | 🎉 |
| **에러율** | 1.5% | 0.2% | -86.7% | 🎉 |

### 비즈니스 메트릭
| 지표 | Before | After | 변화 | 평가 |
|------|--------|-------|------|------|
| **판단 실행 수** | 8,234 | 12,450 | +51.2% | ✅ |
| **평균 신뢰도** | 0.82 | 0.88 | +7.3% | ✅ |
| **Rule Only %** | 68% | 78% | +14.7% | ✅ |
| **LLM Fallback %** | 32% | 22% | -31.3% | ✅ |

### 인프라
| 지표 | Before | After | 변화 | 평가 |
|------|--------|-------|------|------|
| **DB 연결** | 78 | 45 | -42.3% | ✅ |
| **쿼리 시간** | 145ms | 78ms | -46.2% | 🎉 |
| **캐시 히트율** | 72% | 89% | +23.6% | 🎉 |
| **메모리** | 812MB | 512MB | -36.9% | ✅ |
| **CPU** | 68% | 35% | -48.5% | 🎉 |

### 개선 요약
✅ 데이터베이스 쿼리 최적화 성공 (-46.2% 쿼리 시간)
✅ Redis 캐싱 전략 개선 (+23.6% 히트율)
✅ Rule Engine 성능 향상 (+14.7% Rule Only 비율)
✅ 전반적인 리소스 효율성 향상

### 최적화 기법
1. PostgreSQL 인덱스 추가 (workflow_id, created_at)
2. Redis 캐시 TTL 조정 (300초 → 600초)
3. Rule Engine AST 파싱 캐싱
4. 데이터베이스 연결 풀 크기 최적화 (100 → 50)
```

### 3. Learning Service 비교

```
## Learning Service (8009) 성능 비교

### 자동학습 메트릭
| 지표 | Before | After | 변화 | 평가 |
|------|--------|-------|------|------|
| **Rule 추출 횟수** | 234 | 298 | +27.4% | ✅ |
| **평균 Few-shot 샘플** | 18 | 15 | -16.7% | ✅ |
| **추출 정확도** | 82% | 91% | +11.0% | 🎉 |
| **처리 시간** | 1,245ms | 680ms | -45.4% | 🎉 |

### 알고리즘 분포
| 알고리즘 | Before | After | 변화 |
|----------|--------|-------|------|
| **빈도 분석** | 35% | 45% | +28.6% |
| **결정 트리** | 40% | 35% | -12.5% |
| **LLM 패턴** | 25% | 20% | -20.0% |

📝 인사이트:
- Few-shot 샘플 수 최적화로 처리 시간 45% 단축
- 빈도 분석 알고리즘 성능 향상으로 사용 비율 증가
- 전체 정확도 9%p 향상 (82% → 91%)
```

---

## 🎯 성능 개선 판정 기준

| 개선도 | 판정 | 아이콘 | 설명 |
|--------|------|--------|------|
| **>= 30%** | Excellent | 🎉 | 탁월한 최적화 |
| **20-30%** | Great | 🚀 | 우수한 최적화 |
| **10-20%** | Good | ✅ | 양호한 최적화 |
| **5-10%** | Acceptable | 📊 | 수용 가능 |
| **< 5%** | Marginal | ⚠️ | 미미한 개선 |
| **음수** | Regression | ❌ | 성능 저하 (문제!) |

---

## 📈 ROI 계산

```
## 💰 비용/효과 분석

### 서버 리소스 절감
- CPU 사용률 감소: 68% → 42% (-26%p)
  → 예상 클라우드 비용 절감: $450/월 → $333/월 (-26%)

- 메모리 사용량 감소: 812MB → 512MB (-37%)
  → 인스턴스 다운그레이드 가능 (m5.large → m5.medium)
  → 예상 절감: $70/월

### 처리 용량 증가
- RPS 증가: 142 → 221 (+55.6%)
  → 동일 인프라로 55.6% 더 많은 요청 처리 가능
  → 스케일 아웃 시기 지연 (6개월 → 12개월 예상)

### 사용자 경험 개선
- 평균 응답 시간 감소: 452ms → 245ms (-45.8%)
  → 사용자 만족도 향상 예상
  → 이탈률 감소 예상 (측정 필요)

### ROI 요약
💰 월간 비용 절감: $120
📈 처리 용량 증가: +55.6%
⏱️ 응답 시간 개선: -45.8%
🎯 종합 ROI: 🏆 EXCELLENT
```

---

## 🚀 실행 예시

```bash
$ /compare-metrics auto

🔍 Finding latest 2 metric files...

BEFORE: metrics/2024-01-21-10-00.json
AFTER:  metrics/2024-01-22-10-00.json

📊 Comparing metrics...

✅ Overall Performance:
  🎉 Response Time: -38.1% (320ms → 198ms)
  🎉 RPS: +55.6% (142 → 221)
  🎉 Error Rate: -65.2% (2.3% → 0.8%)

✅ Service-level Changes:
  🎉 Judgment Service: -45.8% response time
  🎉 Learning Service: +11.0% accuracy
  ✅ BI Service: -28.3% generation time

📄 Detailed report saved to:
  - comparisons/before-after-2024-01-22.md
  - comparisons/before-after-2024-01-22.json

🎯 Verdict: 🏆 EXCELLENT OPTIMIZATION
```

---

## 🚀 다음 단계 추천

비교 분석 후:

1. **성능 개선 확인**: 🎉 아이콘이 많으면 성공적 최적화
2. **성능 저하 수정**: ❌ 아이콘이 있으면 즉시 롤백 또는 재최적화
3. **추가 최적화**: ⚠️ 아이콘 항목에 대해 performance-engineer Agent 협업
4. **문서화**: 최적화 기법을 문서화하여 다른 서비스에 적용
5. **모니터링**: 개선 효과가 지속되는지 일주일간 모니터링

---

## 💡 주의사항

- **공정한 비교**: 동일한 시간대, 동일한 부하 조건에서 비교
- **통계적 유의성**: 1회 측정보다 3회 평균 권장
- **외부 요인**: 네트워크, 데이터베이스 상태 등 외부 요인 통제
- **다양한 지표**: 단일 지표만 보지 말고 종합적으로 평가

---

## 🔗 관련 리소스

- **Agent 활용**: performance-engineer (상세 분석), business-analyst (ROI 분석)
- **이전 Skill**: `/collect-metrics`, `/run-load-test`
- **도구**: Grafana (시각화), Prometheus (메트릭 저장)
- **문서**: [docs/operations/deployment_guide.md](../../docs/operations/deployment_guide.md)
