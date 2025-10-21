# 시스템 구조 개요 (Ver2.0 Final)

이 문서는 **마이크로서비스 기반 AI 판단 플랫폼 Ver2.0 Final**의 전체 시스템 구조를 명확히 설명하는 기술 아키텍처 문서입니다.

### 주요 개선사항 (v1.0 → Ver2.0 Final):
- **Supabase 중심** → **PostgreSQL + 9개 마이크로서비스** 구조
- **단순 MCP 연동** → **하이브리드 판단 + 자동학습 + Visual Workflow Builder + MCP 기반 BI**
- **보안 강화**: AST 기반 Rule Engine, JWT 인증
- **성능 최적화**: Redis 캐싱, Celery 비동기, 데이터 집계 알고리즘 (할루시네이션 방지)
- **혁신 기능**: Learning Service (ML 대체), n8n 스타일 Workflow Editor, Chat Interface (마스터 컨트롤러)

---

## 1. Ver2.0 Final 전체 아키텍처 개요 (9개 마이크로서비스)

```
[사용자 인터페이스]
   ↓
┌──────────────────────────────────────────────┐
│           API Gateway (8000)                 │
│     인증/라우팅/Rate Limiting                │
└──────────────────────────────────────────────┘
   ↓
┌──────────────────────────────────────────────┐
│        마이크로서비스 계층 (9개 서비스)       │
├────────┬────────┬────────┬────────┬─────────┤
│Workflow│Judgment│DataViz │  BI    │ Chat    │
│(8001)  │(8002)  │(8006)  │(8007)  │(8008)   │
│        │        │        │        │         │
│Visual  │하이브리│단순    │MCP기반 │통합AI   │
│Builder │드판단  │대시보드│컴포넌트│어시스턴트│
│n8n     │+Conn   │        │조립    │(Master) │
└────────┴────────┴────────┴────────┴─────────┘
                    ↓
├────────┬────────┬────────┬─────────┤
│Action  │Notifi  │Logging │Learning │
│(8003)  │(8004)  │(8005)  │(8009)🔥 │
│        │        │        │         │
│외부    │알림    │로그    │자동학습 │
│시스템  │발송    │수집    │ML대체   │
│연동    │        │분석    │Rule추출 │
└────────┴────────┴────────┴─────────┘
   ↓
┌──────────────────────────────────────────────┐
│            데이터 계층                       │
├─────────┬─────────┬─────────┬──────┬────────┤
│PostgreSQL│ Redis   │pgvector │Celery│sklearn │
│(메인DB) │(캐시)   │(RAG+    │(큐)  │(Rule   │
│         │         │Few-shot)│      │추출)   │
└─────────┴─────────┴─────────┴──────┴────────┘
   ↓
┌──────────────────────────────────────────────┐
│          외부 연동 계층                      │
├─────────┬─────────┬─────────┬───────┬───────┤
│OpenAI   │  MCP    │ Slack   │Webhook│Next.js│
│API      │Protocol │Bot API  │       │14 UI  │
└─────────┴─────────┴─────────┴───────┴───────┘
```

---

## 2. 마이크로서비스별 상세 역할

### 🔧 API Gateway (Port 8000)
- **역할**: 모든 API 요청의 단일 진입점
- **기능**: 
  - JWT 기반 인증/인가
  - Rate Limiting (사용자당 1000 req/hour)
  - 로드밸런싱 및 라우팅
  - 요청/응답 로깅
- **기술 스택**: Kong 또는 Nginx

### 📋 Workflow Service (Port 8001) - **n8n 스타일 Visual Builder**
- **역할**: 워크플로우 생성, 관리, 버전 관리
- **Ver2.0 Final 핵심 기능**:
  - **n8n 스타일 드래그앤드롭 UI**: Next.js 14 기반 Visual Workflow Builder
  - 워크플로우 템플릿 라이브러리
  - A/B 테스트를 위한 워크플로우 버전 관리
  - 워크플로우 시뮬레이션 실행
- **기술 스택**: FastAPI + PostgreSQL + Next.js 14
- **데이터베이스**: PostgreSQL (workflows 테이블)

### 🧠 Judgment Service (Port 8002) - **핵심 서비스 (Connector 통합)**
- **역할**: AI + Rule 기반 하이브리드 판단 수행 + 외부 시스템 연동 (Connector 기능 통합)
- **판단 방식**:
  1. **Rule Engine 우선**: AST 기반 안전한 조건식 평가
  2. **LLM 보완**: Rule 실패 또는 신뢰도 < 0.7시 OpenAI API 호출 (Few-shot 활용)
  3. **Hybrid 전략**: 두 결과를 종합한 최적 판단
- **기능**:
  - 실시간 판단 (목표: 2초 이내)
  - 신뢰도 점수 계산 (0.0~1.0)
  - RAG 기반 판단 설명 생성
  - Redis 캐싱으로 반복 판단 최적화
  - **Learning Service 연동**: Few-shot 샘플 자동 요청
- **외부 연동**: OpenAI API, pgvector (RAG + Few-shot), Learning Service (8009)

### ⚡ Action Service (Port 8003)
- **역할**: 판단 결과에 따른 외부 시스템 액션 실행
- **지원 액션**:
  - Slack/Teams 알림 전송
  - MCP 프로토콜 기반 설비 제어
  - Webhook 호출
  - 이메일/SMS 발송
- **신뢰성**: Celery 기반 비동기 처리 + 재시도 로직

### 🔔 Notification Service (Port 8004)
- **역할**: 통합 알림 발송 서비스
- **기능**: Slack, Teams, Email 알림 통합 관리

### 📝 Logging Service (Port 8005)
- **역할**: 모든 서비스의 로그 중앙 집중 관리
- **기능**: 구조화된 로그 수집, 검색, 분석, 감사 추적

### 📊 Data Visualization Service (Port 8006) - **단순 데이터 대시보드**
- **역할**: 미리 정의된 차트로 단순 데이터 표시 (편집 가능)
- **기능**:
  - 미리 정의된 대시보드 템플릿 렌더링
  - 드래그앤드롭으로 차트 배치 변경
  - PostgreSQL에서 데이터 직접 조회
- **지원 차트**: KPI Card, Gauge, Line Chart, Bar Chart
- **기술**: FastAPI + PostgreSQL + WebSocket

### 🎨 BI Service (Port 8007) - **MCP 기반 컴포넌트 조립 (신규)**
- **역할**: AI 기반 인사이트 생성 + MCP 컴포넌트 조립
- **Ver2.0 Final 핵심 변경**: React 코드 생성 → **MCP 사전 제작 컴포넌트 조립**
- **프로세스**:
  1. 사용자 자연어 요청 분석 (LLM)
  2. Judgment Service 호출 → 데이터 기반 판단
  3. **MCP Components에서 적합한 사전 제작 컴포넌트 검색 및 조립**
  4. 비즈니스 권장사항 생성 (RAG 기반)
- **기술**: FastAPI + LLM + MCP Components + RAG

### 💬 Chat Interface Service (Port 8008) - **통합 AI 어시스턴트 (마스터 컨트롤러)**
- **역할**: 통합 AI 채팅 어시스턴트 - 모든 서비스 통합 제어
- **기능**:
  - 멀티턴 대화 컨텍스트 관리
  - 의도 분석 및 라우팅 (워크플로우 실행, BI 생성, 설정 변경)
  - **MCP 서버 상태 실시간 표시** (Settings 화면)
  - Learning Service 피드백 전송 (👍👎, LOG, 채팅)
- **기술**: FastAPI + LLM + WebSocket + NLP Engine

### 🎓 Learning Service (Port 8009) 🔥 - **자동학습 시스템 (ML 대체!)**
- **역할**: 자동학습 + Rule 추출 (전통적 ML 대체)
- **Ver2.0 Final 혁신 기능**:
  1. **사용자 피드백 수집**: 👍👎, LOG 리뷰, 채팅 피드백
  2. **Few-shot 학습 관리**: pgvector로 유사한 10-20개 예시 자동 검색
  3. **자동 Rule 추출 (3개 알고리즘)**:
     - 빈도 분석: 패턴 카운팅 → 80%+ 임계값 → Rule 추출
     - 결정 트리 학습: sklearn DecisionTreeClassifier → 트리를 Rule로 변환
     - LLM 패턴 발견: LLM이 요약 분석 → Rule 제안
- **기술 스택**: FastAPI + PostgreSQL + pgvector + sklearn
- **데이터베이스**: PostgreSQL (training_samples, feedback, extracted_rules 테이블)

---

## 3. Ver2.0 Final 데이터 흐름 시나리오

### 시나리오 1: 하이브리드 판단 실행 (Few-shot 학습 활용)
```
1. [센서/API] → 온도 88도, 진동 42 데이터 수신
   ↓
2. [Workflow Service] → 해당 워크플로우 정의 조회
   ↓
3. [Judgment Service] → Learning Service에 Few-shot 샘플 요청
   ↓
4. [Learning Service] → pgvector로 유사 10개 예시 검색 및 반환
   ↓
5. [Judgment Service] → Rule Engine 실행: "temp > 85 AND vib > 40" (Few-shot 보강)
   ↓
6. Rule 성공 → 신뢰도 1.0, 결과: "작업자 호출 필요"
   ↓
7. [Action Service] → Slack 알림 + MCP 설비 제어
   ↓
8. [Logging Service] → 판단 이력 PostgreSQL 저장
   ↓
9. [pgvector] → RAG용 임베딩 생성 및 저장
```

### 시나리오 2: LLM 보완 판단 (Few-shot 활용)
```
1. [복잡한 입력] → Rule로 평가하기 어려운 상황 발생
   ↓
2. [Rule Engine] → 조건 매칭 실패 또는 신뢰도 < 0.7
   ↓
3. [Learning Service] → Few-shot 학습용 유사 예시 10-20개 제공
   ↓
4. [LLM Engine] → OpenAI API 호출, Few-shot + 컨텍스트 포함 판단
   ↓
5. [Hybrid Logic] → Rule + LLM 결과 종합, 최종 판단
   ↓
6. [설명 생성기] → RAG 기반 상세 설명 생성
```

### 시나리오 3: MCP 기반 BI 컴포넌트 조립 (Ver2.0 Final)
```
1. [사용자] → Chat Interface에서 "지난 주 불량률 분석해줘"
   ↓
2. [Chat Interface] → BI Service에 인사이트 생성 요청
   ↓
3. [BI Service] → LLM으로 요청 분석
   ↓
4. [BI Service] → Judgment Service 호출: 데이터 기반 판단
   ↓
5. [BI Service] → MCP Components에서 적합한 사전 제작 컴포넌트 검색
   ↓
6. [BI Service] → 컴포넌트 조립 + 비즈니스 권장사항 생성 (RAG)
   ↓
7. [Chat Interface] → 사용자에게 인사이트 + 컴포넌트 표시
```

### 시나리오 4: 자동학습 및 Rule 추출 (Ver2.0 Final 혁신!) 🔥
```
1. [사용자] → Chat Interface에서 판단 결과에 👍 피드백
   ↓
2. [Chat Interface] → Learning Service에 피드백 전송
   ↓
3. [Learning Service] → 피드백 데이터 저장 (training_samples)
   ↓
4. [Learning Service] → Few-shot 샘플 업데이트 (긍정 피드백 샘플 추가)
   ↓
5. [자동 Rule 추출 시스템 (주기적 실행)]
   ↓
6. [알고리즘 1: 빈도 분석] → 패턴 카운팅, 80%+ 임계값 Rule 후보 생성
   ↓
7. [알고리즘 2: 결정 트리] → sklearn DecisionTreeClassifier로 Rule 추출
   ↓
8. [알고리즘 3: LLM 패턴] → LLM이 데이터 요약 분석, Rule 제안
   ↓
9. [최적 Rule 선택] → 3개 알고리즘 결과 비교, 최고 성능 Rule 선택
   ↓
10. [Workflow Service] → 자동 추출된 Rule을 워크플로우에 반영
```

---

## 4. Ver2.0 보안 및 성능 최적화

### 🔒 보안 강화 사항
- **AST 기반 Rule Engine**: JavaScript eval 완전 제거, 코드 인젝션 방지
- **JWT 인증**: API Gateway에서 통합 인증 처리
- **RBAC**: 역할 기반 접근 제어 (관리자/운영자/조회자)
- **입력 검증**: Pydantic 모델로 모든 API 입력 검증
- **데이터 암호화**: 민감한 판단 데이터 AES-256 암호화
- **감사 로그**: 모든 중요 작업 추적 및 기록

### ⚡ 성능 최적화
- **Redis 캐싱**: 자주 사용되는 판단 결과 캐싱 (TTL 5분)
- **Celery 비동기**: 무거운 외부 API 호출 백그라운드 처리
- **DB 최적화**: 적절한 인덱스 설계, 쿼리 최적화
- **로드밸런싱**: 각 마이크로서비스 독립적 스케일링
- **CDN**: 정적 리소스 글로벌 배포

---

## 5. Ver2.0 확장성 및 운영 전략

### 🚀 확장성 설계
- **마이크로서비스**: 각 서비스 독립적 개발/배포/스케일링
- **수평 확장**: Kubernetes 기반 Auto-scaling
- **데이터베이스**: PostgreSQL 읽기 복제본 활용
- **캐시 분산**: Redis Cluster로 캐시 분산 처리
- **API 버전 관리**: 각 서비스별 독립적 버전 관리

### 📊 모니터링 및 관찰성
- **메트릭**: Prometheus로 비즈니스/시스템 메트릭 수집
- **대시보드**: Grafana로 실시간 시스템 상태 시각화
- **로깅**: ELK Stack으로 중앙 집중식 로그 관리
- **추적**: Jaeger로 분산 트레이싱
- **알림**: 시스템 장애 및 임계치 초과 시 자동 알림

### 🔧 운영 자동화
- **CI/CD**: GitHub Actions + ArgoCD
- **컨테이너**: Docker + Kubernetes
- **백업**: PostgreSQL 자동 백업 및 복구
- **로그 순환**: 자동 로그 아카이빙 및 삭제
- **성능 튜닝**: 자동 쿼리 성능 분석 및 최적화 제안

## 6. Ver2.0 배포 및 환경 구성

### 개발 환경 (Docker Compose)
```yaml
# docker-compose.dev.yml
version: '3.8'
services:
  postgres:
    image: pgvector/pgvector:pg15
    environment:
      POSTGRES_DB: judgment_dev
      POSTGRES_USER: dev_user
      POSTGRES_PASSWORD: dev_pass
    ports: ["5432:5432"]
  
  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]
  
  api-gateway:
    build: ./api-gateway
    ports: ["8000:8000"]
    depends_on: [postgres, redis]
  
  workflow-service:
    build: ./services/workflow
    ports: ["8001:8001"]
    environment:
      - DATABASE_URL=postgresql://dev_user:dev_pass@postgres:5432/judgment_dev
      - REDIS_URL=redis://redis:6379
  
  judgment-service:
    build: ./services/judgment
    ports: ["8002:8002"]
    environment:
      - DATABASE_URL=postgresql://dev_user:dev_pass@postgres:5432/judgment_dev
      - REDIS_URL=redis://redis:6379
      - OPENAI_API_KEY=${OPENAI_API_KEY}
```

### 프로덕션 환경 (Kubernetes)
- **네임스페이스**: 환경별 분리 (dev/staging/prod)
- **리소스 제한**: CPU/메모리 제한 설정
- **헬스체크**: 각 서비스별 health endpoint
- **롤링 업데이트**: 무중단 배포
- **시크릿 관리**: Kubernetes Secrets로 민감정보 관리

## 7. 다음 단계 상세 문서 연결 (Ver2.0 Final)

이 시스템 구조를 기반으로 다음 상세 문서들이 작성됩니다:

### 마이크로서비스 상세 문서
1. **`docs/services/judgment_engine.md`**: Judgment Service 내부 구현 (하이브리드 판단 + Connector)
2. **`docs/services/dashboard_service.md`**: Dashboard Service 상세 설계 (Data Visualization + BI)
3. **`docs/services/workflow_editor.md`**: Workflow Service UI/UX 설계 (n8n 스타일)
4. **`docs/services/learning_service.md`** 🔥: Learning Service 자동학습 알고리즘 (Ver2.0 Final 신규)
5. **`docs/services/chat_interface.md`**: Chat Interface Service 통합 어시스턴트
6. **`docs/services/external_integration.md`**: MCP 및 외부 시스템 연동

### 알고리즘 및 아키텍처 문서
7. **`docs/algorithms/auto_rule_extraction.md`** 🔥: 3가지 자동 Rule 추출 알고리즘 (Ver2.0 Final 신규)
8. **`docs/algorithms/data_aggregation.md`** 🔥: 데이터 집계 알고리즘 - LLM 할루시네이션 방지 (Ver2.0 Final 신규)
9. **`docs/architecture/database_design.md`**: 통합 데이터베이스 스키마 (9개 서비스 + Learning)
10. **`docs/architecture/system_overview.md`**: Ver2.0 Final 전체 시스템 아키텍처

### 운영 및 모니터링 문서
11. **`docs/operations/monitoring_guide.md`**: 운영 및 모니터링 가이드
12. **`docs/operations/deployment_guide.md`**: Docker/Kubernetes 배포 가이드

각 문서는 이 시스템 구조 정의를 기반으로 작성되며, **Ver2.0 Final의 9개 마이크로서비스 아키텍처**와 일관된 설계 원칙을 따릅니다.
