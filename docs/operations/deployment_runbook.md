# Judgify-core v2.0 배포 런북 📚

## 📖 개요

이 런북은 Judgify-core v2.0 마이크로서비스 플랫폼의 안전하고 효율적인 배포를 위한 단계별 가이드입니다.

**대상**: DevOps 엔지니어, 운영팀, 개발팀  
**런북 버전**: v2.0.0  
**최종 업데이트**: 2024년 11월

---

## 🎯 1. 배포 전략 개요

### 1.1 배포 방식
- **기본 전략**: Blue-Green 배포 (무중단 배포)
- **롤백 전략**: 즉시 Blue 환경으로 트래픽 복귀
- **배포 주기**: 주 1회 정기 배포 (화요일 02:00-06:00)
- **긴급 배포**: 필요시 언제든 가능

### 1.2 환경별 배포 순서
```
1. Development (자동) → 
2. Staging (자동) → 
3. Production (수동 승인)
```

### 1.3 핵심 서비스 우선순위
```yaml
Critical:     # 장애시 즉시 롤백
  - API Gateway (8000)
  - Judgment Service (8002)

Important:    # 모니터링 후 판단
  - Workflow Service (8001)  
  - Dashboard Service (8006)

Supporting:   # 서비스 지속 가능
  - Action Service (8003)
  - Logging Service (8005)
```

---

## 🚀 2. 자동 배포 가이드

### 2.1 GitHub Actions를 통한 자동 배포

#### 스테이징 환경 자동 배포
```bash
# develop 브랜치 푸시시 자동 실행
git push origin develop

# CI/CD 파이프라인 자동 실행:
# 1. CI 파이프라인 (코드 품질, 테스트, 빌드)
# 2. CD 파이프라인 (스테이징 배포)
```

#### 프로덕션 환경 배포 (수동 승인)
```bash
# main 브랜치 푸시 또는 수동 트리거
git push origin main

# 또는 GitHub Actions에서 수동 실행
# Repository → Actions → CD Pipeline → Run workflow
```

### 2.2 배포 상태 모니터링
```bash
# GitHub Actions 실시간 모니터링
https://github.com/your-org/judgify-core/actions

# Slack 알림 확인 (#deployment-alerts)
# 배포 성공/실패 알림 자동 수신
```

---

## 🔧 3. 수동 배포 가이드

### 3.1 환경 준비
```bash
# 1. 로컬 환경 설정
export ENVIRONMENT=production
export KUBECONFIG=~/.kube/config-prod

# 2. 필수 도구 확인
kubectl version --client
docker version
helm version  # (사용시)

# 3. 네임스페이스 확인
kubectl get namespaces
kubectl config set-context --current --namespace=judgify-prod
```

### 3.2 이미지 빌드 및 푸시
```bash
# 1. 프로젝트 루트 디렉토리로 이동
cd /path/to/judgify-core

# 2. Docker 이미지 빌드
./scripts/deploy/env-setup.sh --build

# 3. 레지스트리에 푸시 (GitHub Container Registry)
docker login ghcr.io
docker push ghcr.io/judgify/api-gateway-service:v2.0.0
docker push ghcr.io/judgify/judgment-service:v2.0.0
docker push ghcr.io/judgify/dashboard-service:v2.0.0
# ... 기타 서비스들
```

### 3.3 Kubernetes 배포
```bash
# 1. 시크릿 설정 (최초 1회)
./scripts/deploy/env-setup.sh --env production --setup-secrets

# 2. ConfigMap 적용
kubectl apply -f k8s/configmaps/ -n judgify-prod

# 3. Blue-Green 배포 실행
kubectl apply -f k8s/services/ -n judgify-prod

# 4. 배포 상태 확인
kubectl rollout status deployment -n judgify-prod --timeout=300s
```

---

## 💙💚 4. Blue-Green 배포 상세 절차

### 4.1 Blue-Green 배포 아키텍처
```
[Load Balancer]
       |
   [Service]  ←→ selector: version=blue/green
       |
┌─────────────┬─────────────┐
│    Blue     │    Green    │
│ (Current)   │   (New)     │
│  v1.9.0     │   v2.0.0    │
└─────────────┴─────────────┘
```

### 4.2 단계별 실행

#### Step 1: Green 환경 배포
```bash
# 1. Green 버전 배포
for service in api-gateway judgment dashboard workflow action logging; do
  envsubst < k8s/services/${service}-service.yaml | \
    sed "s/${service}-service/${service}-service-green/g" | \
    sed "s/version: blue/version: green/g" | \
    kubectl apply -f - -n judgify-prod
done

# 2. Green 배포 완료 대기
kubectl rollout status deployment judgify-prod --timeout=600s
```

#### Step 2: Green 환경 검증
```bash
# 1. 헬스체크 (Green 환경 직접 테스트)
kubectl port-forward svc/api-gateway-service-green 8080:8000 -n judgify-prod &
curl http://localhost:8080/health

# 2. 스모크 테스트 실행
cd tests/smoke
python production_smoke_tests.py --base-url http://localhost:8080

# 3. 핵심 기능 테스트
python critical_path_tests.py --base-url http://localhost:8080

# Port-forward 종료
pkill -f "kubectl port-forward"
```

#### Step 3: 트래픽 전환 (Blue → Green)
```bash
# 1. 서비스 셀렉터를 Green으로 변경
for service in api-gateway judgment dashboard workflow action logging; do
  kubectl patch service ${service}-service -n judgify-prod \
    -p '{"spec":{"selector":{"version":"green"}}}'
done

# 2. 트래픽 전환 확인 (30초 대기 후)
sleep 30
curl -f https://api.judgify.ai/health

# 3. 실시간 메트릭 확인 (5분간)
kubectl top pods -n judgify-prod
```

#### Step 4: Blue 환경 정리
```bash
# 1. Blue 환경 제거 (트래픽 전환 완료 후 1시간 대기)
for service in api-gateway judgment dashboard workflow action logging; do
  kubectl delete deployment ${service}-service -n judgify-prod --ignore-not-found=true
done

# 2. Green을 새로운 Blue로 변경
for service in api-gateway judgment dashboard workflow action logging; do
  kubectl patch deployment ${service}-service-green -n judgify-prod \
    --type='merge' -p='{"metadata":{"name":"'${service}'-service"}}'
  
  kubectl patch deployment ${service}-service -n judgify-prod \
    --type='merge' -p='{"spec":{"template":{"metadata":{"labels":{"version":"blue"}}}}}'
done
```

---

## 🔍 5. 배포 후 검증 절차

### 5.1 즉시 검증 (배포 후 10분 이내)

#### 시스템 헬스체크
```bash
# 1. 모든 Pod 상태 확인
kubectl get pods -n judgify-prod
# 모든 Pod가 Running/Ready 상태여야 함

# 2. 서비스 엔드포인트 확인
kubectl get services -n judgify-prod
kubectl get ingress -n judgify-prod

# 3. API 엔드포인트 테스트
curl -f https://api.judgify.ai/health
curl -f https://api.judgify.ai/api/v2/workflow/health
curl -f https://api.judgify.ai/api/v2/judgment/health
curl -f https://api.judgify.ai/api/v2/dashboard/health
```

#### 핵심 기능 검증
```bash
# 1. 자동 테스트 실행
cd tests/smoke
python production_smoke_tests.py --base-url https://api.judgify.ai --output-json /tmp/smoke_results.json

# 2. 결과 확인
cat /tmp/smoke_results.json | jq '.success'
# true 반환되어야 함

# 3. 핵심 비즈니스 로직 테스트
python critical_path_tests.py --base-url https://api.judgify.ai
```

### 5.2 성능 검증 (배포 후 30분)

#### 응답 시간 확인
```bash
# 1. API 응답 시간 테스트 (10회 평균)
for i in {1..10}; do
  curl -w "Response time: %{time_total}s\n" -o /dev/null -s https://api.judgify.ai/health
  sleep 1
done

# 2. 판단 서비스 응답 시간 (모의 요청)
time curl -X POST https://api.judgify.ai/api/v2/judgment/execute \
  -H "Content-Type: application/json" \
  -d '{"workflow_id":"test","input_data":{"test":true},"method":"rule"}'
```

#### 리소스 사용률 확인
```bash
# 1. CPU/Memory 사용률
kubectl top pods -n judgify-prod

# 2. 노드 리소스 상태
kubectl top nodes

# 3. HPA 상태 확인
kubectl get hpa -n judgify-prod
```

### 5.3 모니터링 확인 (배포 후 1시간)

#### Grafana 대시보드 확인
```bash
# 주요 메트릭 확인:
# 1. API 요청 수/응답시간
# 2. 에러율 (< 1% 유지)
# 3. 데이터베이스 연결 수
# 4. Redis 캐시 히트율
# 5. 판단 실행 성공률
```

#### 로그 확인
```bash
# 1. 에러 로그 확인 (Kibana 또는 kubectl)
kubectl logs -l app=api-gateway -n judgify-prod --tail=100 | grep -i error
kubectl logs -l app=judgment-service -n judgify-prod --tail=100 | grep -i error

# 2. 경고 로그 확인
kubectl logs -l app.kubernetes.io/name=judgify -n judgify-prod --tail=500 | grep -i warn
```

---

## 🚨 6. 롤백 절차

### 6.1 자동 롤백 (CI/CD)
```bash
# GitHub Actions에서 배포 실패시 자동 롤백
# 1. Green 환경 배포 실패 → Blue 환경 유지
# 2. 트래픽 전환 후 문제 감지 → 자동 Blue 환경으로 복귀
```

### 6.2 수동 롤백

#### 긴급 롤백 (5분 이내 복구)
```bash
# 1. 즉시 이전 버전으로 롤백
kubectl rollout undo deployment/api-gateway-service -n judgify-prod
kubectl rollout undo deployment/judgment-service -n judgify-prod
kubectl rollout undo deployment/dashboard-service -n judgify-prod

# 2. 롤백 상태 확인
kubectl rollout status deployment -n judgify-prod --timeout=300s

# 3. 서비스 상태 확인
curl -f https://api.judgify.ai/health
```

#### 완전 롤백 (이전 릴리즈)
```bash
# 1. 이전 이미지 태그로 완전 복구
kubectl set image deployment/judgment-service \
  judgment-service=ghcr.io/judgify/judgment-service:v1.9.0 \
  -n judgify-prod

# 2. 설정 변경 롤백 (필요시)
git checkout HEAD~1 -- k8s/configmaps/
kubectl apply -f k8s/configmaps/ -n judgify-prod

# 3. 데이터베이스 마이그레이션 롤백 (필요시)
# 별도 DB 롤백 절차 참조
```

### 6.3 롤백 후 검증
```bash
# 1. 시스템 상태 확인
kubectl get pods -n judgify-prod
kubectl get services -n judgify-prod

# 2. 기능 검증
python tests/smoke/smoke_tests.py --base-url https://api.judgify.ai

# 3. 사용자 영향도 확인
# Grafana에서 에러율, 응답시간 확인
```

---

## 📊 7. 모니터링 및 알람

### 7.1 배포 중 모니터링

#### 실시간 메트릭 확인
```bash
# 1. Grafana 대시보드
https://grafana.company.com/d/judgify-overview

# 주요 확인 사항:
- API 응답 시간 (< 500ms 유지)
- 에러율 (< 1% 유지)
- 활성 연결 수
- 데이터베이스 성능
- 메모리/CPU 사용률
```

#### 로그 모니터링
```bash
# 1. 실시간 로그 모니터링
kubectl logs -f deployment/api-gateway-service -n judgify-prod

# 2. Kibana 대시보드
https://kibana.company.com/app/discover

# 주요 확인 사항:
- ERROR 레벨 로그 개수
- WARN 레벨 로그 패턴
- 느린 쿼리 로그
- 외부 API 호출 실패
```

### 7.2 알람 설정

#### Critical 알람 (즉시 대응)
- 서비스 Down (30초)
- API 에러율 > 5% (2분)
- 응답 시간 > 3초 (5분)
- 메모리 사용률 > 90% (5분)

#### Warning 알람 (모니터링)
- CPU 사용률 > 75% (10분)
- 디스크 사용률 > 80% (30분)
- 느린 쿼리 감지
- 외부 API 응답 지연

---

## 🔧 8. 트러블슈팅

### 8.1 일반적인 배포 문제

#### Pod 시작 실패
```bash
# 1. Pod 상태 확인
kubectl describe pod <pod-name> -n judgify-prod

# 2. 일반적인 원인:
- 이미지 Pull 실패 → 레지스트리 권한 확인
- 리소스 부족 → 노드 리소스 확인
- ConfigMap/Secret 오류 → 설정 값 확인
- Health check 실패 → 앱 로그 확인

# 3. 해결책:
kubectl logs <pod-name> -n judgify-prod
kubectl get events -n judgify-prod --sort-by='.lastTimestamp'
```

#### 서비스 연결 실패
```bash
# 1. 서비스/엔드포인트 확인
kubectl get services -n judgify-prod
kubectl get endpoints -n judgify-prod

# 2. 네트워크 정책 확인
kubectl get networkpolicies -n judgify-prod

# 3. 포트/셀렉터 확인
kubectl describe service <service-name> -n judgify-prod
```

#### 데이터베이스 연결 문제
```bash
# 1. 데이터베이스 상태 확인
kubectl exec -it <api-pod> -n judgify-prod -- nc -zv postgres-service 5432

# 2. 연결 문자열 확인
kubectl exec -it <api-pod> -n judgify-prod -- env | grep DATABASE_URL

# 3. 인증 정보 확인
kubectl get secret judgify-database-secret -n judgify-prod -o yaml
```

### 8.2 성능 문제 해결

#### 높은 응답 시간
```bash
# 1. 병목 지점 확인
kubectl top pods -n judgify-prod
kubectl describe hpa -n judgify-prod

# 2. 로그 분석
kubectl logs -l app=api-gateway -n judgify-prod | grep -E "(slow|timeout|error)"

# 3. 데이터베이스 성능 확인
# PostgreSQL slow query 로그 확인
```

#### 메모리 부족
```bash
# 1. 메모리 사용량 확인
kubectl top pods -n judgify-prod --sort-by=memory

# 2. 리소스 제한 확인
kubectl describe pod <pod-name> -n judgify-prod | grep -A5 "Limits"

# 3. OOM 킬 확인
kubectl get events -n judgify-prod | grep OOMKilled
```

---

## 📋 9. 배포 체크리스트

### 9.1 배포 전 체크리스트
- [ ] 코드 리뷰 완료
- [ ] 테스트 케이스 통과
- [ ] 보안 검토 완료
- [ ] 성능 테스트 통과
- [ ] 데이터베이스 마이그레이션 준비
- [ ] 롤백 계획 수립
- [ ] 운영팀 공지

### 9.2 배포 중 체크리스트
- [ ] 배포 시작 알림
- [ ] Blue-Green 배포 실행
- [ ] Green 환경 검증
- [ ] 트래픽 전환
- [ ] 모니터링 확인
- [ ] 성능 검증
- [ ] Blue 환경 정리

### 9.3 배포 후 체크리스트
- [ ] 시스템 안정성 확인 (24시간)
- [ ] 사용자 피드백 수집
- [ ] 성능 메트릭 분석
- [ ] 로그 분석
- [ ] 문서 업데이트
- [ ] 배포 완료 보고

---

## 📞 10. 비상 연락망

### 10.1 배포 관련 연락처
| 역할 | 담당자 | 연락처 | 대응시간 |
|------|--------|--------|----------|
| 배포 책임자 | DevOps Lead | ext.2001 | 24/7 |
| 개발팀장 | Dev Manager | ext.2002 | 평일 9-18시 |
| 운영팀장 | Ops Manager | ext.2003 | 24/7 |
| 인프라 엔지니어 | Infra Eng | ext.2004 | 24/7 |

### 10.2 에스컬레이션 절차
1. **Level 1** (0-30분): 배포 담당자
2. **Level 2** (30-60분): 팀장급 대응
3. **Level 3** (60분+): 경영진 보고

---

## 📚 11. 관련 문서

### 11.1 기술 문서
- [프로덕션 준비 체크리스트](production_readiness_checklist.md)
- [장애 대응 매뉴얼](incident_response_guide.md)
- [모니터링 가이드](monitoring_guide.md)
- [백업 복구 절차서](backup_recovery_guide.md)

### 11.2 아키텍처 문서
- [시스템 아키텍처](../architecture/system_overview.md)
- [데이터베이스 설계](../architecture/database_design.md)
- [보안 아키텍처](../architecture/security_architecture.md)

---

**📝 런북 버전**: v2.0.0  
**최종 업데이트**: 2024-11-XX  
**다음 리뷰 일정**: 2024-12-XX  

**작성자**: DevOps Team  
**검토자**: Architecture Team  
**승인자**: Service Owner