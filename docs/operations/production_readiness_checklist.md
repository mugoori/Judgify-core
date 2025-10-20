# Judgify-core v2.0 프로덕션 준비 체크리스트 📋

## 📖 개요

이 문서는 Judgify-core v2.0 마이크로서비스를 프로덕션 환경에 배포하기 전에 반드시 확인해야 할 모든 항목들을 정리한 체크리스트입니다.

**배포 담당자**: _______________  
**배포 예정일**: _______________  
**승인자**: _______________  
**문서 버전**: v2.0.0  

---

## 🏗️ 1. 인프라 준비사항

### 1.1 Kubernetes 클러스터 준비
- [ ] **Production Kubernetes 클러스터** 준비 완료
  - [ ] 최소 3개 워커 노드 (고가용성)
  - [ ] 노드당 최소 4vCPU, 16GB RAM
  - [ ] SSD 스토리지 500GB+ (영구 볼륨용)
  - [ ] 네트워크 정책 설정 완료

- [ ] **네임스페이스 설정**
  - [ ] `judgify-prod` 네임스페이스 생성
  - [ ] 리소스 쿼터 설정 (CPU: 20 cores, Memory: 64GB)
  - [ ] 네트워크 정책 적용

- [ ] **로드 밸런서 및 Ingress**
  - [ ] NGINX Ingress Controller 설치
  - [ ] SSL 인증서 준비 (Let's Encrypt 또는 상용 인증서)
  - [ ] 도메인 설정: `api.judgify.ai`, `dashboard.judgify.ai`

### 1.2 데이터베이스 준비
- [ ] **PostgreSQL 15 클러스터**
  - [ ] 프로덕션용 PostgreSQL 15 설치
  - [ ] 고가용성 설정 (Primary/Replica)
  - [ ] pgvector 확장 설치 및 설정
  - [ ] 백업 전략 구성 (매일 자동 백업)
  - [ ] 모니터링 설정 (pg_stat_statements, pg_stat_monitor)

- [ ] **Redis 클러스터**
  - [ ] Redis 7 클러스터 모드 설정
  - [ ] 고가용성 구성 (Sentinel 또는 Cluster)
  - [ ] 영구 저장소 설정 (RDB + AOF)
  - [ ] 메모리 최적화 설정

### 1.3 외부 서비스 준비
- [ ] **Elasticsearch 클러스터** (로깅용)
  - [ ] Elasticsearch 8.x 클러스터 설치
  - [ ] Kibana 대시보드 설정
  - [ ] 로그 보관 정책 설정 (30일)

- [ ] **모니터링 스택**
  - [ ] Prometheus 서버 설치
  - [ ] Grafana 대시보드 구성
  - [ ] AlertManager 설정

---

## 🔐 2. 보안 설정

### 2.1 시크릿 관리
- [ ] **Kubernetes Secrets 생성**
  ```bash
  # 필수 시크릿 확인
  kubectl get secrets -n judgify-prod
  - judgify-database-secret ✓
  - judgify-auth-secret ✓  
  - judgify-llm-secret ✓
  - judgify-external-secret ✓
  ```

- [ ] **환경 변수 검증**
  - [ ] `OPENAI_API_KEY` 설정 및 할당량 확인
  - [ ] `JWT_SECRET_KEY` 강력한 키 생성 (32자 이상)
  - [ ] `POSTGRES_PASSWORD` 복잡한 비밀번호 설정
  - [ ] `REDIS_PASSWORD` 설정

### 2.2 네트워크 보안
- [ ] **방화벽 규칙 설정**
  - [ ] 내부 서비스 간 통신만 허용
  - [ ] 외부에서는 API Gateway(8000)만 접근 가능
  - [ ] 관리 포트(9090, 5432, 6379) 외부 접근 차단

- [ ] **SSL/TLS 설정**
  - [ ] HTTPS 강제 리다이렉션 설정
  - [ ] TLS 1.2+ 강제 설정
  - [ ] HSTS 헤더 설정

### 2.3 인증 및 권한
- [ ] **JWT 설정 검증**
  - [ ] JWT 만료 시간 설정 (24시간)
  - [ ] Refresh Token 전략 구현
  - [ ] Rate Limiting 설정 (1000 req/min)

---

## 📊 3. 성능 및 확장성

### 3.1 리소스 할당
- [ ] **서비스별 리소스 설정 확인**
  ```yaml
  API Gateway:    CPU: 500m,  Memory: 1Gi
  Workflow:       CPU: 500m,  Memory: 1Gi  
  Judgment:       CPU: 1500m, Memory: 2Gi  # 핵심 서비스
  Action:         CPU: 500m,  Memory: 1Gi
  Logging:        CPU: 250m,  Memory: 512Mi
  Dashboard:      CPU: 1000m, Memory: 1.5Gi
  ```

### 3.2 오토스케일링 설정
- [ ] **HPA (Horizontal Pod Autoscaler)**
  - [ ] 각 서비스별 HPA 설정
  - [ ] CPU 75%, Memory 85% 임계값 설정
  - [ ] 최소 2개, 최대 10개 Pod 설정

- [ ] **VPA (Vertical Pod Autoscaler)** (선택사항)
  - [ ] 리소스 자동 추천 활성화

### 3.3 성능 테스트 완료
- [ ] **부하 테스트 실행 및 통과**
  ```bash
  # 성능 테스트 스크립트 실행
  python tests/performance/test_load_and_performance.py --environment production
  
  목표 성능 지표:
  - API 응답 시간: < 500ms (95 percentile)
  - 판단 실행 시간: < 2초
  - 대시보드 생성: < 30초
  - 동시 사용자: 1000명 이상
  ```

---

## 🔍 4. 모니터링 및 관찰성

### 4.1 로깅 설정
- [ ] **구조화된 로깅 활성화**
  - [ ] JSON 형식 로그 출력 설정
  - [ ] 로그 레벨: INFO (운영환경)
  - [ ] Correlation ID 추적 활성화

- [ ] **로그 수집 파이프라인**
  - [ ] Filebeat → Elasticsearch → Kibana 파이프라인 구성
  - [ ] 로그 보관 기간: 30일
  - [ ] 로그 압축 및 로테이션 설정

### 4.2 메트릭 수집
- [ ] **Prometheus 메트릭 확인**
  ```bash
  # 각 서비스의 /metrics 엔드포인트 확인
  curl https://api.judgify.ai/metrics
  
  필수 메트릭:
  - HTTP 요청 수/응답시간
  - 판단 실행 횟수/성공률  
  - 대시보드 생성 횟수
  - 데이터베이스 연결 상태
  - Redis 연결 상태
  ```

### 4.3 알림 설정  
- [ ] **AlertManager 규칙 구성**
  - [ ] 서비스 Down 알림 (30초 내 복구 안될 시)
  - [ ] CPU/Memory 사용률 임계값 알림 (85% 초과)
  - [ ] 에러율 임계값 알림 (5% 초과)
  - [ ] 판단 서비스 응답 시간 알림 (3초 초과)

- [ ] **알림 채널 설정**
  - [ ] Slack 채널 연동: `#alerts-prod`
  - [ ] 이메일 알림: devops-team@company.com
  - [ ] PagerDuty 연동 (Critical 알림)

---

## 🧪 5. 테스트 및 검증

### 5.1 스모크 테스트 실행
- [ ] **기본 스모크 테스트 통과**
  ```bash
  cd tests/smoke
  python smoke_tests.py --base-url https://api.judgify.ai
  # 모든 테스트 PASSED 확인
  ```

- [ ] **프로덕션 스모크 테스트 통과**
  ```bash
  python production_smoke_tests.py --base-url https://api.judgify.ai
  # Critical 실패 없음 확인
  ```

- [ ] **크리티컬 패스 테스트 통과**
  ```bash  
  python critical_path_tests.py --base-url https://api.judgify.ai
  # 모든 핵심 기능 정상 작동 확인
  ```

### 5.2 보안 스캔
- [ ] **컨테이너 이미지 취약점 스캔**
  ```bash
  # Trivy를 이용한 이미지 스캔
  trivy image judgify/api-gateway-service:v2.0.0
  trivy image judgify/judgment-service:v2.0.0
  trivy image judgify/dashboard-service:v2.0.0
  
  # Critical/High 취약점 0개 확인
  ```

- [ ] **의존성 보안 스캔**
  ```bash
  # Python 의존성 보안 검사
  safety check
  bandit -r services/
  
  # 보안 이슈 해결 확인
  ```

### 5.3 데이터 마이그레이션 테스트
- [ ] **데이터베이스 스키마 확인**
  ```sql
  -- 필수 테이블 존재 확인
  SELECT table_name FROM information_schema.tables 
  WHERE table_schema = 'public';
  
  필수 테이블:
  - workflows ✓
  - judgment_executions ✓
  - dashboards ✓
  - action_logs ✓
  - system_logs ✓
  ```

---

## 🔄 6. 백업 및 재해 복구

### 6.1 백업 전략
- [ ] **데이터베이스 백업**
  - [ ] 매일 자동 전체 백업 (02:00 AM)
  - [ ] 매시간 WAL 백업
  - [ ] S3/클라우드 스토리지 저장
  - [ ] 백업 보관 기간: 30일

- [ ] **설정 백업**
  - [ ] Kubernetes manifests Git 저장소 보관
  - [ ] 환경 설정 파일 암호화 백업
  - [ ] 인증서 백업

### 6.2 재해 복구 계획
- [ ] **RTO/RPO 목표 설정**
  - [ ] RTO (복구 시간): 4시간 이내
  - [ ] RPO (데이터 손실): 1시간 이내

- [ ] **복구 절차 문서화**
  - [ ] 데이터베이스 복구 절차서 작성
  - [ ] 서비스 복구 우선순위 정의
  - [ ] 비상 연락망 구성

---

## 📚 7. 문서 및 운영 준비

### 7.1 운영 문서
- [ ] **런북 작성 완료**
  - [ ] 배포 절차서
  - [ ] 장애 대응 매뉴얼  
  - [ ] 모니터링 가이드
  - [ ] 백업/복구 절차서

- [ ] **API 문서 업데이트**
  - [ ] OpenAPI/Swagger 문서 최신화
  - [ ] 사용자 가이드 작성
  - [ ] SDK 문서 (해당시)

### 7.2 운영팀 교육
- [ ] **시스템 아키텍처 교육**
  - [ ] 마이크로서비스 구조 이해
  - [ ] 서비스 간 의존성 파악
  - [ ] 핵심 비즈니스 로직 이해

- [ ] **운영 도구 교육**
  - [ ] Kubernetes 기본 명령어
  - [ ] Grafana 대시보드 활용
  - [ ] 로그 분석 (Kibana)
  - [ ] 장애 대응 절차

---

## 🚀 8. 배포 실행 체크리스트

### 8.1 사전 배포 확인
- [ ] **변경사항 리뷰**
  - [ ] 코드 리뷰 완료
  - [ ] 아키텍처 리뷰 완료
  - [ ] 보안 리뷰 완료
  - [ ] 성능 테스트 통과

- [ ] **배포 계획 승인**
  - [ ] 기술 팀장 승인
  - [ ] 운영팀 승인  
  - [ ] 서비스 책임자 승인

### 8.2 배포 실행
- [ ] **배포 창구 시간 확보** (권장: 새벽 2-6시)
- [ ] **배포팀 대기상태 확보**
  - [ ] 개발팀 대기
  - [ ] 운영팀 대기
  - [ ] 네트워크팀 대기 (필요시)

- [ ] **단계별 배포 실행**
  1. [ ] 스테이징 환경 배포 및 테스트
  2. [ ] 프로덕션 Blue 환경 배포
  3. [ ] Green 환경 스모크 테스트
  4. [ ] 트래픽 전환 (Blue → Green)
  5. [ ] 프로덕션 검증 테스트
  6. [ ] Blue 환경 정리

### 8.3 배포 후 검증
- [ ] **즉시 검증 (배포 후 30분)**
  - [ ] 전체 서비스 Health Check
  - [ ] 핵심 기능 동작 확인
  - [ ] 에러 로그 확인 (에러율 < 1%)
  - [ ] 응답 시간 확인 (< 500ms)

- [ ] **24시간 모니터링**
  - [ ] 시스템 리소스 사용률 모니터링
  - [ ] 비즈니스 메트릭 모니터링  
  - [ ] 사용자 피드백 수집

---

## ✅ 최종 승인 서명

### 기술 승인
- **개발팀장**: _________________ 날짜: _______
- **아키텍트**: _________________ 날짜: _______
- **보안 책임자**: ______________ 날짜: _______

### 운영 승인  
- **운영팀장**: _________________ 날짜: _______
- **인프라 책임자**: ____________ 날짜: _______
- **모니터링 책임자**: __________ 날짜: _______

### 비즈니스 승인
- **서비스 책임자**: ____________ 날짜: _______
- **제품 책임자**: ______________ 날짜: _______

---

## 📞 비상 연락망

| 역할 | 이름 | 전화번호 | 이메일 | 비상 전화 |
|------|------|----------|--------|-----------|
| 개발팀장 | _______ | _______ | _______ | _______ |
| 운영팀장 | _______ | _______ | _______ | _______ |
| 인프라 책임자 | _______ | _______ | _______ | _______ |
| 서비스 책임자 | _______ | _______ | _______ | _______ |

---

**📋 체크리스트 완료율: _____ / 총 항목**

**🚀 프로덕션 배포 준비 상태: [ ] 준비완료 [ ] 추가작업필요**

**최종 검토자**: _______________ **날짜**: _____________