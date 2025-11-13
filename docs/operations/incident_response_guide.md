# Judgify-core v2.0 장애 대응 가이드 🚨

## 📖 개요

이 가이드는 Judgify-core v2.0 마이크로서비스 플랫폼에서 발생할 수 있는 장애 상황에 대한 체계적인 대응 절차를 정의합니다.

**대상**: 운영팀, DevOps 엔지니어, 개발팀  
**문서 버전**: v2.0.0  
**비상 연락처**: 📞 Emergency Hotline: +82-2-XXXX-XXXX

---

## 🎯 1. 장애 분류 및 우선순위

### 1.1 장애 심각도 분류

#### 🔴 Critical (P0) - 15분 이내 대응
- **정의**: 서비스 전체 중단, 데이터 손실, 보안 침해
- **영향**: 모든 사용자 서비스 불가
- **대응**: 즉시 대응팀 소집, 경영진 보고
- **예시**:
  - API Gateway 완전 다운
  - 데이터베이스 서버 장애
  - 보안 침해 사고
  - 데이터 손실/손상

#### 🟠 High (P1) - 30분 이내 대응  
- **정의**: 핵심 기능 장애, 성능 크게 저하
- **영향**: 주요 기능 사용 불가 또는 심각한 성능 저하
- **대응**: 대응팀 소집, 관리자 보고
- **예시**:
  - Judgment Service 장애
  - Dashboard 생성 불가
  - API 응답 시간 > 10초

#### 🟡 Medium (P2) - 2시간 이내 대응
- **정의**: 부분적 기능 장애, 성능 저하
- **영향**: 일부 기능 제한적 사용 가능
- **대응**: 담당자 배정, 일반 업무시간 내 해결
- **예시**:
  - Action Service 일부 실패
  - 로그 수집 지연
  - 모니터링 알람 누락

#### 🟢 Low (P3) - 24시간 이내 대응
- **정의**: 경미한 기능 오류, 사용자 불편
- **영향**: 우회 방법 존재, 서비스 지속 가능
- **대응**: 다음 유지보수 시간에 해결
- **예시**:
  - UI 표시 오류
  - 문서/도움말 오류
  - 로그 형식 문제

### 1.2 영향도별 우선 복구 순서
```
1. API Gateway (8000)     - 모든 요청의 진입점
2. Judgment Service (8002) - 핵심 비즈니스 로직  
3. Database/Redis         - 데이터 계층
4. Workflow Service (8001) - 워크플로우 관리
5. Dashboard Service (8006) - 시각화
6. Action Service (8003)   - 외부 연동
7. Logging Service (8005)  - 로그 수집
```

---

## 🚨 2. 장애 감지 및 알림

### 2.1 자동 감지 시스템

#### Prometheus + AlertManager 알림
```yaml
# 주요 알림 규칙
- alert: ServiceDown
  expr: up{job=~".*-service"} == 0
  for: 30s
  
- alert: HighErrorRate  
  expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
  for: 2m
  
- alert: HighResponseTime
  expr: histogram_quantile(0.95, http_request_duration_seconds_bucket) > 3
  for: 5m
```

#### 알림 채널
- **Slack**: `#alerts-critical`, `#alerts-warning`
- **이메일**: ops-team@company.com
- **PagerDuty**: Critical 알림 (24/7)
- **SMS**: P0/P1 장애 (대응팀)

### 2.2 수동 감지 방법
```bash
# 1. 시스템 전체 상태 확인
curl -f https://api.judgify.ai/health

# 2. 개별 서비스 상태
curl -f https://api.judgify.ai/api/v2/workflow/health
curl -f https://api.judgify.ai/api/v2/judgment/health
curl -f https://api.judgify.ai/api/v2/dashboard/health

# 3. Kubernetes 클러스터 상태
kubectl get pods -n judgify-prod
kubectl get services -n judgify-prod
kubectl top nodes
```

---

## 📋 3. 초기 대응 절차 (First 15 minutes)

### 3.1 장애 접수 및 확인

#### Step 1: 장애 접수 (1분)
```bash
# 1. 알림 수신 확인
- Slack 알림 채널 확인
- PagerDuty 알림 확인  
- 모니터링 대시보드 확인

# 2. 초기 대응자 지정
- P0/P1: 즉시 대응팀 소집
- P2/P3: 담당자 1명 배정
```

#### Step 2: 장애 범위 파악 (3분)
```bash
# 1. 영향 범위 확인
kubectl get pods -n judgify-prod --field-selector=status.phase!=Running

# 2. 사용자 영향도 파악
# Grafana에서 실시간 메트릭 확인:
- 활성 사용자 수
- 에러율 증가폭
- 응답 시간 변화

# 3. 장애 타임라인 확인
kubectl get events -n judgify-prod --sort-by='.lastTimestamp' | tail -20
```

#### Step 3: 장애 상황 전파 (2분)
```bash
# 1. 사내 공지 (Slack #general)
"🚨 [P0 장애] Judgify 서비스 장애 발생
- 발생시간: 2024-XX-XX 14:30
- 영향범위: 전체 서비스 사용 불가
- 대응상황: 긴급 복구 중
- 담당자: @ops-team"

# 2. 고객 공지 (해당시)
# 서비스 상태 페이지 업데이트
# 이메일/SMS 고객 통보

# 3. 관리자 보고
# P0/P1: 즉시 경영진 보고
# P2: 팀장 보고
```

### 3.2 응급 복구 시도 (10분)

#### 즉시 시도할 수 있는 복구 방법
```bash
# 1. Pod 재시작 (가장 간단한 해결책)
kubectl rollout restart deployment/api-gateway-service -n judgify-prod
kubectl rollout restart deployment/judgment-service -n judgify-prod

# 2. 이전 버전 즉시 롤백
kubectl rollout undo deployment/judgment-service -n judgify-prod

# 3. 수동 스케일 아웃 (리소스 부족시)
kubectl scale deployment/judgment-service --replicas=10 -n judgify-prod

# 4. 트래픽 차단 (더 큰 피해 방지)
kubectl patch ingress judgify-ingress -n judgify-prod \
  -p '{"metadata":{"annotations":{"nginx.ingress.kubernetes.io/server-snippet":"return 503;"}}}'
```

---

## 🔧 4. 서비스별 상세 장애 대응

### 4.1 API Gateway (8000) 장애

#### 증상 및 감지
- 모든 API 요청 실패 (502/503/504 오류)
- 헬스체크 실패: `curl https://api.judgify.ai/health`
- nginx/envoy 프록시 에러

#### 진단 절차
```bash
# 1. Pod 상태 확인
kubectl get pods -l app=api-gateway -n judgify-prod
kubectl describe pod <gateway-pod> -n judgify-prod

# 2. 로그 확인
kubectl logs -l app=api-gateway -n judgify-prod --tail=100

# 3. 인그레스 및 로드밸런서 확인
kubectl get ingress -n judgify-prod
kubectl describe ingress judgify-ingress -n judgify-prod

# 4. 업스트림 서비스 확인
kubectl get endpoints -n judgify-prod
```

#### 복구 절차
```bash
# 1. 빠른 복구 (2분)
kubectl rollout restart deployment/api-gateway-service -n judgify-prod

# 2. 롤백 (이전 버전이 안정적인 경우)
kubectl rollout undo deployment/api-gateway-service -n judgify-prod

# 3. 수동 스케일링 (리소스 부족시)
kubectl scale deployment/api-gateway-service --replicas=5 -n judgify-prod

# 4. 설정 문제 수정 (ConfigMap 오류시)
kubectl edit configmap api-gateway-config -n judgify-prod
kubectl rollout restart deployment/api-gateway-service -n judgify-prod
```

### 4.2 Judgment Service (8002) 장애

#### 증상 및 감지
- 판단 실행 실패 또는 극도로 느림
- `/api/v2/judgment/execute` 엔드포인트 오류
- LLM API 호출 실패

#### 진단 절차  
```bash
# 1. 서비스 상태 확인
kubectl get pods -l app=judgment-service -n judgify-prod
kubectl logs -l app=judgment-service -n judgify-prod --tail=100 | grep -i error

# 2. 리소스 사용량 확인 (CPU/Memory 부족시)
kubectl top pods -l app=judgment-service -n judgify-prod

# 3. 외부 의존성 확인
# OpenAI API 상태 확인
curl -H "Authorization: Bearer $OPENAI_API_KEY" https://api.openai.com/v1/models

# 데이터베이스 연결 확인
kubectl exec -it <judgment-pod> -n judgify-prod -- nc -zv postgres-service 5432
```

#### 복구 절차
```bash
# 1. 즉시 재시작
kubectl rollout restart deployment/judgment-service -n judgify-prod

# 2. 리소스 증가 (메모리/CPU 부족시)
kubectl patch deployment judgment-service -n judgify-prod \
  -p '{"spec":{"template":{"spec":{"containers":[{"name":"judgment-service","resources":{"limits":{"memory":"4Gi","cpu":"2000m"}}}]}}}}'

# 3. 외부 API 실패시 임시 조치
# Rule Engine만 사용하도록 설정
kubectl set env deployment/judgment-service -n judgify-prod \
  ENABLE_LLM_ENGINE=false

# 4. 데이터베이스 연결 문제 해결
kubectl rollout restart statefulset/postgres -n judgify-prod
```

### 4.3 Database 장애

#### 증상 및 감지
- 모든 서비스에서 데이터베이스 연결 오류
- PostgreSQL 연결 실패
- 데이터베이스 응답 없음

#### 진단 절차
```bash
# 1. PostgreSQL Pod 상태 확인
kubectl get pods -l app=postgres -n judgify-prod
kubectl describe pod <postgres-pod> -n judgify-prod

# 2. 데이터베이스 로그 확인
kubectl logs -l app=postgres -n judgify-prod --tail=100

# 3. 디스크 공간 확인
kubectl exec -it <postgres-pod> -n judgify-prod -- df -h

# 4. 연결 테스트
kubectl exec -it <app-pod> -n judgify-prod -- nc -zv postgres-service 5432
```

#### 복구 절차 (매우 주의!)
```bash
# 1. PostgreSQL 재시작 (READ-ONLY 우선)
kubectl patch statefulset postgres -n judgify-prod \
  -p '{"spec":{"template":{"spec":{"containers":[{"name":"postgres","env":[{"name":"POSTGRES_READ_ONLY","value":"true"}]}]}}}}'

kubectl rollout restart statefulset/postgres -n judgify-prod

# 2. 백업에서 복구 (데이터 손실시)
# 별도 데이터베이스 복구 절차 참조
./scripts/backup/restore_database.sh --backup-date 2024-XX-XX

# 3. 슬레이브 DB로 임시 전환 (고가용성 설정시)
kubectl patch configmap postgres-config -n judgify-prod \
  --patch '{"data":{"primary_host":"postgres-slave-service"}}'

# 4. 응급시 외부 DB 사용
kubectl set env deployment/api-gateway-service -n judgify-prod \
  DATABASE_URL="postgresql://backup_user:backup_pass@backup-db.company.com:5432/judgify_backup"
```

### 4.4 Redis 장애

#### 증상 및 감지
- 캐시 오류, 성능 저하
- 세션 정보 손실
- Redis 연결 실패

#### 진단 및 복구
```bash
# 1. Redis 상태 확인
kubectl get pods -l app=redis -n judgify-prod
kubectl exec -it <redis-pod> -n judgify-prod -- redis-cli ping

# 2. 메모리 사용량 확인
kubectl exec -it <redis-pod> -n judgify-prod -- redis-cli info memory

# 3. 재시작 복구
kubectl rollout restart deployment/redis -n judgify-prod

# 4. 캐시 무력화 (임시)
kubectl set env deployment/api-gateway-service -n judgify-prod \
  REDIS_ENABLED=false
```

---

## 📊 5. 장애 중 모니터링

### 5.1 실시간 모니터링 대시보드

#### Grafana 대시보드 모니터링
```bash
# 주요 메트릭 실시간 확인:
https://grafana.company.com/d/judgify-incident

핵심 지표:
- HTTP 요청 수 (QPS)
- HTTP 에러율 (%)  
- API 응답 시간 (P95)
- 활성 사용자 수
- 데이터베이스 연결 수
- 메모리/CPU 사용률
```

#### 로그 분석
```bash
# 1. 실시간 에러 로그
kubectl logs -f deployment/api-gateway-service -n judgify-prod | grep ERROR

# 2. Kibana에서 에러 패턴 분석
https://kibana.company.com/app/discover
# 쿼리: level:ERROR AND @timestamp:[now-15m TO now]

# 3. 느린 쿼리 감지
kubectl logs -l app=postgres -n judgify-prod | grep "duration:"
```

### 5.2 장애 영향도 측정
```bash
# 1. 사용자 영향도
# 활성 사용자 수 변화
# 에러 발생 비율
# 완료되지 못한 요청 수

# 2. 비즈니스 영향도  
# 판단 실행 건수 변화
# 대시보드 생성 건수
# API 호출량 변화

# 3. SLA 지표
# 가용성: 99.9% 목표
# 응답시간: 95% < 500ms
# 에러율: < 0.1%
```

---

## 🔄 6. 장애 해결 프로세스

### 6.1 단계별 해결 절차

#### Phase 1: 응급 복구 (0-30분)
```bash
# 목표: 서비스 가용성 최우선 복구
1. Pod 재시작/롤백으로 빠른 복구 시도
2. 트래픽 차단으로 추가 피해 방지  
3. 리소스 스케일링으로 용량 확보
4. 외부 의존성 문제 우회
```

#### Phase 2: 임시 해결 (30분-2시간)
```bash
# 목표: 안정적 서비스 운영 확보
1. 근본 원인 파악 및 임시 해결책 적용
2. 모니터링 강화 및 재발 방지책
3. 성능 튜닝 및 최적화
4. 상세 장애 보고서 작성 시작
```

#### Phase 3: 영구 해결 (2시간 이후)
```bash
# 목표: 근본 원인 해결 및 개선
1. 근본 원인 완전 분석 및 수정
2. 테스트 환경에서 충분한 검증
3. 안전한 운영 환경 배포
4. 재발 방지를 위한 시스템 개선
```

### 6.2 에스컬레이션 매트릭스

| 시간 | P0 (Critical) | P1 (High) | P2 (Medium) | P3 (Low) |
|------|---------------|-----------|-------------|----------|
| 0-15분 | 대응팀 + DevOps | 담당자 + DevOps | 담당자 | 담당자 |
| 15-30분 | + 팀장 | + 팀장 | 담당자 | - |
| 30-60분 | + 개발팀장 | + 개발팀장 | + 팀장 | - |
| 60분+ | + CTO/CEO | + CTO | + 개발팀장 | + 팀장 |

---

## 📝 7. 장애 문서화

### 7.1 실시간 기록 (장애 중)

#### 장애 로그 템플릿
```markdown
## 장애 정보
- **장애 ID**: INC-2024-XXXX
- **발생시간**: 2024-XX-XX 14:30 KST
- **감지방법**: AlertManager 알림
- **심각도**: P0 (Critical)
- **영향범위**: 전체 서비스

## 타임라인
- 14:30 - 장애 최초 감지 (Slack 알림)
- 14:31 - 대응팀 소집, 상황 파악 시작
- 14:35 - API Gateway Pod 재시작 시도
- 14:38 - 이전 버전으로 롤백 시작
- 14:42 - 서비스 정상화 확인
- 14:45 - 모니터링 정상화 확인

## 대응 조치
1. API Gateway 재시작
2. v1.9.0으로 긴급 롤백
3. 리소스 모니터링 강화

## 근본 원인
- 메모리 누수로 인한 OOM Kill
- 새 버전(v2.0.0)의 버그

## 해결 방안
- 즉시: 안정 버전 유지
- 단기: 메모리 누수 버그 수정
- 장기: 메모리 프로파일링 강화
```

### 7.2 사후 분석 보고서

#### Post-Mortem 템플릿
```markdown
# 장애 사후 분석 보고서

## 요약
- **장애일시**: 2024-XX-XX 14:30~14:45 (15분)
- **영향도**: 전체 사용자 서비스 불가
- **근본원인**: 메모리 누수로 인한 Pod 재시작
- **비즈니스 영향**: 매출 손실 추정 $X,XXX

## What Went Wrong
1. 새 배포 버전에서 메모리 누수 발생
2. OOM Killer에 의한 Pod 강제 종료
3. 자동 복구 실패 (이미지 Pull 지연)

## What Went Well  
1. 15분 내 빠른 감지 및 대응
2. 롤백 절차 정상 수행
3. 고객 커뮤니케이션 적절

## 개선 사항
1. **즉시 (1주 이내)**:
   - 메모리 누수 버그 수정
   - 배포 전 메모리 프로파일링 강화

2. **단기 (1개월 이내)**:
   - Canary 배포 도입
   - 자동 롤백 로직 개선
   - 리소스 모니터링 강화

3. **장기 (3개월 이내)**:
   - Chaos Engineering 도입
   - SRE 팀 구성
   - 장애 예방 시스템 구축

## 액션 아이템
- [ ] 메모리 누수 수정 (@dev-team, 2024-XX-XX)
- [ ] Canary 배포 구현 (@devops-team, 2024-XX-XX)  
- [ ] 자동 롤백 개선 (@platform-team, 2024-XX-XX)
```

---

## 🛠️ 8. 장애 예방 및 개선

### 8.1 예방 조치

#### 정기 점검 항목 (Weekly)
```bash
# 1. 시스템 리소스 점검
kubectl top nodes
kubectl top pods -n judgify-prod --sort-by=cpu
kubectl top pods -n judgify-prod --sort-by=memory

# 2. 디스크 사용량 점검
kubectl exec -it <postgres-pod> -n judgify-prod -- df -h
kubectl exec -it <logging-pod> -n judgify-prod -- df -h

# 3. 로그 에러 패턴 분석
kubectl logs -l app.kubernetes.io/name=judgify -n judgify-prod --since=168h | \
  grep -i error | sort | uniq -c | sort -nr | head -10

# 4. 성능 지표 리뷰
# Grafana에서 주간 성능 트렌드 확인
```

#### 자동화 개선
```bash
# 1. 자동 복구 스크립트
# Kubernetes Liveness/Readiness Probe 최적화
# 자동 재시작 조건 개선

# 2. 예측적 스케일링
# HPA 메트릭 개선 (CPU, Memory, Custom Metrics)
# VPA로 리소스 자동 최적화

# 3. Chaos Engineering
# 정기적 장애 주입 테스트
# 복구 시간 단축 훈련
```

### 8.2 모니터링 개선

#### 추가 메트릭 구성
```yaml
# 비즈니스 메트릭
- judgment_success_rate
- dashboard_generation_time  
- user_session_duration

# 기술 메트릭
- database_connection_pool_usage
- redis_memory_fragmentation
- api_request_queue_depth

# SLI/SLO 메트릭
- availability_percentage
- error_budget_consumption  
- mttr_minutes (평균 복구 시간)
```

---

## 📚 9. 참고 자료

### 9.1 관련 문서
- [배포 런북](deployment_runbook.md)
- [모니터링 가이드](monitoring_guide.md) 
- [백업 복구 가이드](backup_recovery_guide.md)
- [보안 대응 절차](security_incident_guide.md)

### 9.2 외부 도구 및 대시보드
- **Grafana**: https://grafana.company.com/d/judgify-overview
- **Kibana**: https://kibana.company.com/app/discover  
- **PagerDuty**: https://company.pagerduty.com
- **Status Page**: https://status.judgify.ai

### 9.3 비상 연락처
| 역할 | 이름 | 전화 | 이메일 | Slack |
|------|------|------|--------|-------|
| 운영팀장 | _______ | +82-10-XXXX | ops@company.com | @ops-lead |
| DevOps Lead | _______ | +82-10-XXXX | devops@company.com | @devops-lead |  
| 개발팀장 | _______ | +82-10-XXXX | dev@company.com | @dev-lead |
| 인프라 엔지니어 | _______ | +82-10-XXXX | infra@company.com | @infra-eng |

---

**📱 24/7 비상 핫라인: +82-2-XXXX-XXXX**

**🚨 생명 안전이 관련된 Critical 장애의 경우 즉시 112/119 신고 후 대응**

---

**문서 버전**: v2.0.0  
**최종 업데이트**: 2024-11-XX  
**다음 훈련 일정**: 2024-12-XX  

**작성자**: SRE Team  
**검토자**: Operations Team  
**승인자**: Engineering Director