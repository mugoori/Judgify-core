# Core Judgement Engine - 초기 요구사항 정의서 (v2.0 Final)

이 문서는 **마이크로서비스 기반 AI 판단 플랫폼**의 핵심 요구사항과 기술 스택을 정의합니다. Ver2.0 Final에서는 자동학습 시스템, Visual Workflow Builder, MCP 기반 BI 등 혁신적 기능이 추가되었습니다.

## 1. 프로젝트 개요 (Ver2.0 Final)

- **프로젝트명**: Core Judgement Engine v2.0 Final (Judgify-core)
- **목표**: 제조업 SME를 위한 No-code AI 판단 플랫폼 - 하이브리드 판단 + 자동학습으로 지속적으로 똑똑해지는 시스템
- **핵심 혁신**:
  - **하이브리드 판단**: Rule Engine + LLM 조합으로 95% 정확도
  - **자동학습**: 사용자 피드백 → 자동 Rule 추출 → 지속적 성능 향상 (ML 대체)
  - **Visual Workflow**: n8n 스타일 드래그앤드롭 워크플로우 빌더
  - **MCP 기반 BI**: 사전 제작 컴포넌트 조립으로 강력한 BI 생성
  - **Chat Interface**: 전체 시스템 통합 마스터 컨트롤러
- **핵심 기술 스택**:
  - **Backend**: FastAPI + Python 3.11+
  - **Database**: PostgreSQL 15+ with pgvector (RAG + 자동학습)
  - **Cache**: Redis 7.0+ (판단 결과 캐싱)
  - **Message Queue**: Celery with Redis broker (비동기 처리)
  - **Frontend**: Next.js 14 + TypeScript (Visual Workflow Editor)
  - **MCP Protocol**: Component Library + External Integrations
- **타겟 고객**: 제조업 SME (연매출 50억~3000억원), IT 역량 낮음, MES/ERP 보유
- **비즈니스 모델**: 높은 구현비 + 모듈별 추가비 + 2년 무료 → 토큰 기반 과금


## 2. 핵심 기능 요구사항 (v2.0 Final)

### FR-001. Visual Workflow Builder (핵심!)
- **n8n 스타일 에디터**: 드래그앤드롭 UI로 판단 흐름 시각적 설계
  - 참조: https://goddaehee.tistory.com/408 (OpenAI Agent Builder)
- **노드 타입**: Trigger, Condition, Judgment, Action, Data Transform, Loop, Merge
- **JSON 기반 워크플로우**: 버전 관리 및 재사용 가능한 구조
- **입력 타입**: REST API, 센서 데이터, 스케줄 트리거
- **실시간 여부 결정**: LLM이 workflow 설정 분석 → Critical은 Rule, 분석은 LLM

### FR-002. 하이브리드 판단 엔진 + MES/ERP 통합 (핵심!)
- **AST 기반 Rule Engine**: 보안 강화된 조건식 평가 (eval 완전 제거)
- **LLM 판단 엔진**: GPT-4/Claude 기반 복합 판단
- **Hybrid 전략**: Rule 우선 → LLM 보완 방식
- **신뢰도 평가**: 각 판단 결과의 confidence score 제공 (0.0~1.0)
- **RAG 기반 설명**: pgvector로 유사 사례 검색 후 설명 생성
- **Connector Engine** (통합):
  - 확장 가능한 MES/ERP 연동 프레임워크
  - Adapter 패턴: API 우선(MCP) → DB 직접연결 대체
  - 플러그인 방식으로 미래 확장 준비 (즉시 개발 X)

### FR-003. 자동학습 시스템 (ML 대체, 신규!)
- **사용자 피드백 수집**:
  - 즉시 피드백: 👍👎 버튼 (판단 결과 직후)
  - 지연 피드백: LOG 클릭 → 과거 판단 재평가
  - 채팅 피드백: "그 때 판단 좋았어" / "잘못됐어"
- **Few-shot 학습 관리**:
  - 10~20개 유사 예시 자동 선택
  - 정확도 우선 전략 (최신성보다)
  - pgvector 유사도 기반 검색
- **자동 Rule 추출** (핵심!):
  - 100+ 판단 데이터 축적시 자동 분석
  - 공통 패턴 추출 → Rule 자동 생성
  - 복수 알고리즘: 빈도 분석, 결정 트리, LLM 패턴 발견
  - 데이터 충분 → 통합 Rule 생성 → 반복 (지속적 개선)
- **성능 메트릭**: 정확도 개선율, Rule 추출 횟수, 피드백 수 추적

### FR-004. 실시간 액션 실행
- **Action Executor**: MCP 프로토콜 기반 외부 시스템 연동
- **비동기 처리**: Celery 기반 백그라운드 작업
- **재시도 로직**: 지수 백오프 알고리즘 적용

### FR-005. 통합 데이터 관리 (영구 보관 + 집계)
- **PostgreSQL 메인 DB**:
  - 워크플로우, 판단 이력, 사용자 데이터 통합 관리
  - Learning 데이터: predictions, user_feedback, training_samples, extracted_rules
- **PostgreSQL Raw Data DB** (별도):
  - 모든 센서/입력 데이터 영구 보관
  - 판단 메타데이터: 어떤 데이터를 어떻게 사용했는지
- **데이터 집계 알고리즘** (핵심!):
  - 할루시네이션 방지: 데이터량 줄이기
  - 통계값: 평균, 중간값, 표준편차
  - 평가값: 정상/경고/위험 3단계
  - 트렌드: 증가/감소/안정
  - LLM 입력용 최적화 (특수 알고리즘)
- **아카이빙 전략**:
  - 90일 이상 오래된 판단 데이터 → archived_judgments 테이블
  - 통계 데이터는 집계 형태로 유지
  - 원본 삭제, 아카이브 보관
- **pgvector**: RAG + Few-shot 검색용 임베딩 저장
- **Redis Cache**: 자주 사용되는 판단 결과 캐싱 (TTL 5분)

### FR-006. 데이터 시각화 서비스 (고정 대시보드)
- **역할**: 고정된 일간 대시보드 (구조 변경 불가)
- **기능**: 차트 이동/확대 가능, 실시간 데이터 업데이트
- **데이터 연결**: PostgreSQL 데이터 직접 조회 및 표시
- **미리 정의된 차트**: KPI 카드, 게이지, 라인/바 차트
- **WebSocket**: 실시간 데이터 스트리밍
- **vs BI Service**: Data Viz는 단순 조회, BI는 강력한 분석

### FR-007. BI(Business Intelligence) 서비스 (MCP 기반, 강화!)
- **MCP Component Assembly**:
  - 사전 제작된 차트/데이터 컴포넌트를 MCP로 가져오기
  - LLM이 최적 컴포넌트 선택 + 조립
  - React 코드 직접 생성 대신 컴포넌트 조합 방식
- **자연어 기반 인사이트**: "이번 주 불량률 분석해줘" → AI가 분석 + 시각화 제공
- **판단 엔진 통합**: Judgment Service와 연동하여 데이터 기반 자동 의사결정
- **LLM 인사이트 생성**: 데이터 분석 후 비즈니스 권장사항 제공
- **실시간 데이터 바인딩**: WebSocket 기반 동적 업데이트
- **vs Data Viz**: BI는 채팅으로 동적 생성, 매우 강력

### FR-008. Chat Interface Service (마스터 컨트롤러, 신규!)
- **역할**: "Overall engine managing everything" - 전체 시스템 통합 제어
- **Claude Desktop 수준 기능**:
  - 워크플로우 생성/실행: "온도 감시 워크플로우 만들어줘"
  - BI 생성/쿼리: "불량률 분석해줘"
  - 인터넷 검색: MCP 연결시 외부 정보 검색
- **멀티턴 대화**: 컨텍스트 유지하며 연속 대화
- **Settings 관리**:
  - MCP 서버 연결 상태 실시간 표시
  - 연결 테스트, 로그 확인
- **MCP 통합**: PostgreSQL, GitHub, Notion, Playwright, Component Library 등
- **특징**: 많이 사용되진 않지만 강력한 전문가용 기능


## 3. 예외 처리 조건

- **입력 불완전**: 주요 입력값 누락 시, Claude는 `Missing key: {변수명}` 메시지 반환
- **DSL 실패**: 판단 조건 생성 실패 시, `DSL Parse Error` 반환 및 재PRP 수행 유도
- **MCP 실행 실패**: MCP에서 오류 발생 시 오류 메시지 수집 및 Slack 보고


## 4. 통합 데이터베이스 스키마 (PostgreSQL + pgvector)

### 메인 데이터베이스 (judgment_core)

#### workflows (워크플로우 정의)
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| name | VARCHAR(255) | 워크플로우 이름 |
| definition | JSONB | 워크플로우 노드 구조 (n8n 스타일) |
| version | INTEGER | 버전 번호 |
| created_by | UUID | 생성자 ID |
| is_active | BOOLEAN | 활성 상태 |
| realtime_enabled | BOOLEAN | 실시간 처리 여부 (LLM 결정) |

#### judgment_executions (판단 실행 이력)
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| workflow_id | UUID | 워크플로우 참조 |
| input_data | JSONB | 입력 데이터 |
| rule_result | JSONB | Rule Engine 결과 |
| llm_result | JSONB | LLM Engine 결과 |
| final_result | JSONB | 최종 판단 결과 |
| confidence_score | DECIMAL(3,2) | 신뢰도 점수 |
| method_used | VARCHAR(20) | 사용된 방법 (rule/llm/hybrid) |
| execution_time_ms | INTEGER | 실행 시간 |
| data_usage_metadata | JSONB | 어떤 데이터를 어떻게 사용했는지 |
| explanation_embedding | VECTOR(1536) | RAG용 임베딩 |

#### action_executions (액션 실행 이력)
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| judgment_id | UUID | 판단 실행 참조 |
| action_type | VARCHAR(50) | 액션 유형 (slack/mcp/webhook) |
| target_system | VARCHAR(100) | 대상 시스템 |
| command | JSONB | 실행 명령 |
| status | VARCHAR(20) | 실행 상태 |
| result | JSONB | 실행 결과 |

### Learning Service 테이블 (신규!)

#### predictions (LLM 예측 저장)
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| judgment_id | UUID | 판단 실행 참조 |
| predicted_result | JSONB | 예측 결과 |
| confidence | DECIMAL(3,2) | 신뢰도 |
| created_at | TIMESTAMP | 생성 시간 |

#### user_feedback (사용자 피드백)
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| prediction_id | UUID | 예측 참조 |
| feedback_type | VARCHAR(20) | thumbs_up/thumbs_down/chat/log_review |
| feedback_value | INTEGER | -1 (나쁨), 0 (중립), 1 (좋음) |
| feedback_text | TEXT | 채팅 피드백 내용 (선택) |
| created_at | TIMESTAMP | 생성 시간 |

#### training_samples (Few-shot 학습 샘플)
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| workflow_id | UUID | 워크플로우 참조 |
| input_data | JSONB | 입력 예시 |
| expected_output | JSONB | 기대 출력 |
| accuracy_score | DECIMAL(3,2) | 정확도 점수 |
| usage_count | INTEGER | 사용 횟수 (Few-shot으로 몇 번 사용됐는지) |
| last_used_at | TIMESTAMP | 마지막 사용 시간 |
| sample_embedding | VECTOR(1536) | 유사도 검색용 임베딩 |

#### extracted_rules (자동 추출된 Rule)
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| workflow_id | UUID | 워크플로우 참조 |
| rule_expression | TEXT | AST 파싱 가능한 조건식 |
| confidence | DECIMAL(3,2) | Rule 신뢰도 |
| sample_count | INTEGER | 몇 개 판단에서 추출됐는지 |
| extraction_method | VARCHAR(50) | frequency/decision_tree/llm |
| created_at | TIMESTAMP | 생성 시간 |
| is_active | BOOLEAN | 활성 상태 |

#### learning_metrics (학습 성과 메트릭)
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| workflow_id | UUID | 워크플로우 참조 |
| accuracy_improvement | DECIMAL(5,2) | 정확도 개선율 (%) |
| rule_extraction_count | INTEGER | 추출된 Rule 개수 |
| feedback_count | INTEGER | 받은 피드백 개수 |
| measured_at | TIMESTAMP | 측정 시간 |

### 데이터 집계 및 아카이빙

#### aggregated_data (집계 데이터)
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| source_data_ids | UUID[] | 원본 데이터 참조 |
| aggregation_type | VARCHAR(50) | statistical/evaluation/trend |
| aggregated_value | JSONB | 집계값 (평균/중간값/평가값) |
| time_window_start | TIMESTAMP | 집계 시작 시간 |
| time_window_end | TIMESTAMP | 집계 종료 시간 |
| created_at | TIMESTAMP | 생성 시간 |

#### archived_judgments (아카이빙된 판단)
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| original_judgment_id | UUID | 원본 판단 ID |
| judgment_data | JSONB | 판단 데이터 전체 |
| archived_at | TIMESTAMP | 아카이빙 시간 |
| archive_reason | VARCHAR(100) | age_based/manual/policy |

### Raw Data 데이터베이스 (judgment_raw_data) - 별도 DB

#### raw_sensor_data (센서 원본 데이터)
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| sensor_id | VARCHAR(100) | 센서 식별자 |
| timestamp | TIMESTAMP | 수집 시간 |
| raw_value | JSONB | 원본 데이터 그대로 |
| archived_at | TIMESTAMP | 아카이빙 시간 (NULL = 활성) |


## 5. 마이크로서비스 구성 (Ver2.0 Final - 9개 서비스)

| # | 서비스명 | 포트 | 책임 | 기술 스택 | UI 매핑 | 변경 |
|---|----------|------|------|-----------|---------|------|
| 1 | **API Gateway** | 8000 | 라우팅/인증/Rate Limiting | Kong/Nginx + JWT | - | 유지 |
| 2 | **Workflow Service** | 8001 | **Visual workflow builder (n8n)** | FastAPI + PostgreSQL + Next.js | - | 🔥강화 |
| 3 | **Judgment Service** | 8002 | **하이브리드 판단 + Connector + Learning** | FastAPI + Redis + OpenAI + pgvector + Adapter | - | 🔥강화 |
| 4 | **Action Service** | 8003 | 외부 시스템 액션 실행 | FastAPI + Celery | - | 유지 |
| 5 | **Notification Service** | 8004 | Slack/Teams/Email 알림 | FastAPI + Message Queue | - | 유지 |
| 6 | **Logging Service** | 8005 | 통합 로그 수집/관리 | FastAPI + PostgreSQL + ELK | - | 유지 |
| 7 | **Data Visualization Service** | 8006 | **고정 일간 대시보드** | FastAPI + PostgreSQL + WebSocket | `judgify-inventory-dashboard.html` | ✨재정의 |
| 8 | **BI Service** | 8007 | **MCP Component Assembly** | FastAPI + LLM + MCP Component Library | `judgify-inventory-chat.html` | 🔥강화 |
| 9 | **Chat Interface Service** | 8008 | **마스터 컨트롤러** | FastAPI + LLM + Multiple MCP | `judgify-enterprise-ui.html` | 🔥강화 |
| 10 | **Learning Service** | **8009** | **자동학습 + Rule 추출** | FastAPI + PostgreSQL + pgvector + sklearn | - | ⭐신규 |

### 서비스 간 통신 아키텍처 (Ver2.0 Final)

#### 핵심 통신 흐름

**1. Judgment Service (8002) - 핵심 허브**
```
├── Learning Service (8009) 연동: Few-shot 요청, 예측 저장, Rule 적용
├── Connector Engine (내부): MES/ERP 연동 (MCP 우선 → DB 대체)
└── RAG Engine: pgvector 유사 사례 검색, 설명 생성
```

**2. Learning Service (8009) - 자동학습**
```
├── 피드백 수집: 👍👎 버튼, LOG 재평가, 채팅 피드백
├── Few-shot 관리: pgvector 검색, 정확도 우선 10~20개 선택
└── Rule 추출: 빈도 분석, 결정 트리, LLM 패턴 발견
```

**3. BI Service (8007) - Component Assembly**
```
├── MCP Component Library: list_components, get_component
├── LLM Orchestrator: 요청 분석, 컴포넌트 선택, 데이터 바인딩
└── Dashboard Assembly: 컴포넌트 조립, WebSocket 실시간 연결
```

**4. Chat Interface Service (8008) - 마스터 컨트롤러**
```
├── Workflow Service (8001): 워크플로우 생성/실행
├── BI Service (8007): BI 생성/쿼리
├── Judgment Service (8002): 직접 판단 요청
├── Learning Service (8009): 피드백 전달, 메트릭 조회
└── Multiple MCP: PostgreSQL, GitHub, Notion, Playwright, Component Library
```

**5. Workflow Service (8001) - Visual Builder**
```
├── n8n-like Editor: Node Types (Trigger, Condition, Judgment, Action, Loop, Merge)
├── Workflow CRUD: 생성, 조회, 수정, 삭제, 버전 관리
└── LLM Integration: 실시간 여부 결정 (Critical → Rule, 분석 → LLM)
```

### 외부 연동 프로토콜
- **MCP (Model Context Protocol)**: 외부 시스템 명령 실행
  - PostgreSQL MCP: 데이터베이스 직접 연결
  - GitHub MCP: 코드 관리 및 이슈 트래킹
  - Notion MCP: 문서 및 지식베이스 연동
  - Playwright MCP: E2E 테스트 자동화
  - Word Document MCP: 보고서 자동 생성
- **Slack Bot API**: 실시간 알림 및 상호작용
- **OpenAI API**: LLM 기반 판단 및 설명 생성
- **Webhook**: 외부 시스템으로부터 트리거 수신

### MCP 통합 기능 (FR-008)
**Settings 화면에서 MCP 서버 상태 표시**:
- **실시간 연결 상태**: 각 MCP 서버의 연결 상태 (연결됨/연결 안됨/오류)
- **서버 정보**: 서버명, 버전, 마지막 통신 시간
- **사용 가이드**: 각 MCP 서버의 활용 방법 및 예시
- **연결 테스트**: 수동으로 연결 테스트 실행
- **로그 확인**: MCP 통신 로그 실시간 조회


## 6. 개발 환경 및 배포 전략

### 개발 환경
```yaml
# docker-compose.dev.yml
services:
  postgres:
    image: pgvector/pgvector:pg15
    environment:
      POSTGRES_DB: judgment_dev
      POSTGRES_USER: dev_user
      POSTGRES_PASSWORD: dev_pass

  redis:
    image: redis:7-alpine

  # 9개 마이크로서비스 (Ver2.0 Final)
  api-gateway:
    build: ./services/api-gateway
    ports: ["8000:8000"]

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

  action-service:
    build: ./services/action
    ports: ["8003:8003"]
    environment:
      - DATABASE_URL=postgresql://dev_user:dev_pass@postgres:5432/judgment_dev
      - REDIS_URL=redis://redis:6379

  notification-service:
    build: ./services/notification
    ports: ["8004:8004"]
    environment:
      - SLACK_BOT_TOKEN=${SLACK_BOT_TOKEN}

  logging-service:
    build: ./services/logging
    ports: ["8005:8005"]
    environment:
      - DATABASE_URL=postgresql://dev_user:dev_pass@postgres:5432/judgment_dev

  data-visualization-service:
    build: ./services/data-visualization
    ports: ["8006:8006"]
    environment:
      - DATABASE_URL=postgresql://dev_user:dev_pass@postgres:5432/judgment_dev

  bi-service:
    build: ./services/bi
    ports: ["8007:8007"]
    environment:
      - DATABASE_URL=postgresql://dev_user:dev_pass@postgres:5432/judgment_dev
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - MCP_COMPONENT_LIBRARY_URL=${MCP_COMPONENT_LIBRARY_URL}

  chat-interface-service:
    build: ./services/chat-interface
    ports: ["8008:8008"]
    environment:
      - DATABASE_URL=postgresql://dev_user:dev_pass@postgres:5432/judgment_dev
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - MCP_SERVERS=${MCP_SERVERS}

  learning-service:
    build: ./services/learning
    ports: ["8009:8009"]
    environment:
      - DATABASE_URL=postgresql://dev_user:dev_pass@postgres:5432/judgment_dev
      - REDIS_URL=redis://redis:6379
      - OPENAI_API_KEY=${OPENAI_API_KEY}
```

### 프로덕션 배포
- **컨테이너화**: Docker + Kubernetes
- **모니터링**: Prometheus + Grafana
- **로깅**: ELK Stack
- **CI/CD**: GitHub Actions + ArgoCD

## 7. 보안 및 성능 기준

### 보안
- **AST 기반 Rule Engine**: JavaScript eval 제거로 코드 인젝션 방지
- **JWT 인증**: API Gateway에서 통합 인증 처리
- **RBAC**: 역할 기반 접근 제어
- **입력 검증**: Pydantic 모델로 모든 API 입력 검증

### 성능
- **판단 응답시간**: 평균 2초, 95% < 5초
- **캐싱**: Redis로 자주 사용되는 판단 결과 캐싱
- **비동기 처리**: Celery로 무거운 작업 백그라운드 처리
- **DB 최적화**: 적절한 인덱스 및 쿼리 최적화

## 8. Ver2.0 Final 아키텍처 변경사항 요약

### Ver1.0 → Ver2.0 Final 주요 변경점

#### 1. 서비스 구조 재설계 (6 → 9개 서비스)
- **기존 Ver1.0**: Supabase 중심 단순 구조 (6개 서비스)
- **Ver2.0 Final**: PostgreSQL + 마이크로서비스 아키텍처 (9개 서비스)
  - **신규 서비스 3개**:
    - **Data Visualization Service (8006)**: 고정 일간 대시보드
    - **BI Service (8007)**: MCP 기반 Component Assembly
    - **Chat Interface Service (8008)**: Claude Desktop 수준 마스터 컨트롤러
    - **Learning Service (8009)**: ML 대체 자동학습 시스템

#### 2. 핵심 혁신 기능 추가
- **Visual Workflow Builder**: n8n 스타일 드래그앤드롭 에디터
- **자동학습 시스템**: 피드백 수집 → Few-shot 관리 → 자동 Rule 추출
- **MCP Component Assembly**: React 코드 생성 대신 사전 제작 컴포넌트 조립
- **데이터 집계 알고리즘**: LLM 할루시네이션 방지를 위한 특수 알고리즘
- **Connector Engine**: Judgment Service 내 MES/ERP 통합 (Adapter 패턴)

#### 3. 아키텍처 전략 변경
- **Supabase 제거** → PostgreSQL 15+ with pgvector로 통합
- **Dashboard 개념 분리**:
  - Data Viz (8006): 고정된 일간 대시보드 (차트 이동/확대만 가능)
  - BI Service (8007): 자연어 기반 동적 BI 생성 (매우 강력)
- **Learning Service 독립**: ML 없이 LLM + 복수 알고리즘으로 자동 개선
- **Chat Interface**: 전체 시스템 통합 관리 (많이 사용되진 않지만 전문가용)

#### 4. 데이터 전략 고도화
- **영구 보관**: 모든 데이터 영구 보존 (Raw Data DB 별도)
- **데이터 집계**: 통계/평가/트렌드 3가지 방식으로 LLM 입력 최적화
- **아카이빙**: 90일 이상 데이터 → archived_judgments 테이블로 이동
- **메타데이터 추적**: 판단시 어떤 데이터를 어떻게 사용했는지 기록

#### 5. 비즈니스 모델 반영
- **타겟 고객**: 제조업 SME (연매출 50억~3000억원), IT 역량 낮음
- **UI 철학**: 누구나 쉽게 사용, 강력한 결과 제공
- **과금 방식**: 높은 구현비 + 모듈별 추가비 + 2년 무료 → 토큰 기반
- **토큰 최적화**: 데이터 집계 알고리즘으로 필수 효율화

### UI 파일 매핑 (Ver2.0 Final)
- `UI/judgify-inventory-dashboard.html` → Data Visualization Service (8006)
- `UI/judgify-inventory-chat.html` → BI Service (8007)
- `UI/judgify-enterprise-ui.html` → Chat Interface Service (8008)

### MVP 범위
- **현재 설계 전체가 MVP**: 지금 정의된 9개 서비스 + 모든 기능
- **향후 확장**: 제조업 특화 모듈 (재고관리, 생산관리, 품질관리 등)

## 9. 다음 단계 문서 참조 (Ver2.0 Final)

### 아키텍처 문서 (전체 시스템)
1. `docs/architecture/system_overview.md`: 상세 아키텍처 설계 (9개 서비스)
2. `docs/architecture/database_design.md`: PostgreSQL + pgvector 스키마
3. `docs/architecture/microservices_communication.md`: 서비스 간 통신 패턴

### 서비스별 설계 문서 (9개 마이크로서비스)
4. `docs/services/judgment_engine.md`: **판단 엔진 구현 명세** (핵심! Connector + Learning 통합)
5. `docs/services/workflow_editor.md`: **Visual Workflow Builder 설계** (n8n 스타일 에디터)
6. `docs/services/learning_service.md`: **자동학습 서비스 설계** (신규! ML 대체)
7. `docs/services/bi_service.md`: **BI 서비스 설계** (신규! MCP Component Assembly)
8. `docs/services/chat_interface_service.md`: **채팅 인터페이스 서비스** (신규! 마스터 컨트롤러)
9. `docs/services/data_visualization_service.md`: **데이터 시각화 서비스** (고정 대시보드)
10. `docs/services/action_service.md`: 액션 실행 서비스
11. `docs/services/notification_service.md`: 알림 서비스
12. `docs/services/logging_service.md`: 로깅 서비스

### 알고리즘 설계 문서 (핵심!)
13. `docs/algorithms/auto_rule_extraction.md`: **자동 Rule 추출 알고리즘 설계**
    - 빈도 분석 (Frequency Analysis)
    - 결정 트리 학습 (Decision Tree Learning)
    - LLM 패턴 발견 (LLM Pattern Discovery)
14. `docs/algorithms/data_aggregation.md`: **데이터 집계 알고리즘 설계** (특수 알고리즘)
    - 통계 집계 (Statistical Aggregation)
    - 평가 집계 (Evaluation Aggregation)
    - 트렌드 분석 (Trend Analysis)

### 운영 및 모니터링 문서
15. `docs/operations/monitoring_guide.md`: 모니터링 및 운영 가이드 (Learning 메트릭 추가)
16. `docs/operations/mcp_integration_guide.md`: MCP 서버 연동 및 관리 가이드


