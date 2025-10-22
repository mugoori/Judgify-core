# CLAUDE.md - Ver2.0 Final 마이크로서비스 개발 가이드 🤖⚡

이 문서는 **Claude Code가 Judgify-core Ver2.0 Final 마이크로서비스 기반 AI 판단 플랫폼**을 개발할 때 참조하는 **포괄적 컨텍스트 엔지니어링 가이드**입니다.

Ver2.0 Final에서는 Supabase 중심 구조에서 **PostgreSQL + 마이크로서비스 아키텍처**로 전면 전환하여, **하이브리드 판단**, **자동학습 시스템**, **Visual Workflow Builder**, **MCP 기반 BI** 등이 핵심 기능입니다.

---

## 📋 0. Ver2.0 Final 문서 목적 및 범위

Claude는 **마이크로서비스 아키텍처 설계자 + 하이브리드 AI 엔지니어 + 자동학습 전문가 + 풀스택 개발자**로서 다음 역할을 수행합니다:

### 🎯 Claude의 핵심 역할 (Ver2.0 Final)
1. **마이크로서비스 설계**: **9개 독립 서비스 아키텍처** 구성 (Learning Service 추가!)
2. **하이브리드 판단 로직**: Rule Engine + LLM의 최적 조합 설계
3. **자동학습 시스템 (ML 대체)**:
   - 사용자 피드백 수집 (👍👎, LOG, 채팅)
   - Few-shot 학습 관리 (10-20개 유사 예시)
   - 자동 Rule 추출 (3개 알고리즘: 빈도 분석, 결정 트리, LLM 패턴)
4. **Visual Workflow Builder**: n8n 스타일 드래그앤드롭 워크플로우 에디터
5. **MCP 기반 BI**: 사전 제작 컴포넌트 조립 (React 코드 생성 대신)
6. **Chat Interface**: Claude Desktop 수준 마스터 컨트롤러
7. **데이터 집계 알고리즘**: LLM 할루시네이션 방지 특수 알고리즘
8. **PostgreSQL + pgvector**: RAG + Few-shot + 자동학습 통합
9. **DevOps 자동화**: Docker + Kubernetes 배포 전략
10. **AI 에이전트 협업**: 18개 전문 에이전트와의 효율적 협업 관리

### 📚 Ver2.0 핵심 문서 구조
```
핵심 가이드 (루트)
├── CLAUDE.md           ← 이 문서 (Claude 개발 가이드)
├── README.md          ← 프로젝트 전체 개요
├── initial.md         ← Ver2.0 통합 요구사항
├── prompt-guide.md    ← LLM Prompt 설계 전략
└── system-structure.md ← 시스템 아키텍처 개요

상세 설계 (docs/)
├── architecture/       ← 시스템 아키텍처
│   ├── system_overview.md              ← 전체 아키텍처
│   ├── database_design.md              ← DB 스키마
│   ├── api_specifications.md           ← API 명세 (2,555줄)
│   ├── security_architecture.md        ← 보안 설계 (2,240줄)
│   ├── microservices_communication.md  ← 서비스 간 통신 (1,534줄)
│   └── data_pipeline_architecture.md   ← 데이터 파이프라인 (1,752줄)
├── services/          ← 마이크로서비스별 설계
│   ├── judgment_engine.md              ← 핵심 판단 서비스
│   ├── learning_service.md             ← 자동학습 시스템 (ML 대체, 신규!)
│   ├── workflow_editor.md              ← Visual Workflow Builder (1,669줄)
│   ├── dashboard_service.md            ← BI + 대시보드 (1,381줄)
│   ├── external_integration.md         ← 외부 시스템 연동 (1,852줄)
│   └── mcp_optimization.md             ← MCP 최적화 전략 (신규!)
├── operations/        ← 운영 관리
│   ├── deployment_strategy.md          ← 배포 전략 (1,721줄)
│   └── monitoring_guide.md             ← 모니터링
└── development/       ← 개발 관리
    ├── plan.md                         ← Windows Desktop App 개발 계획 (1,130줄)
    ├── requirements.md                 ← 요구사항
    └── status.md                       ← 진행 상황
```

### 📐 문서 관리 전략 (신규 추가!)

Claude가 문서를 생성하거나 수정할 때 **반드시** 따라야 하는 규칙:

#### 🎯 기본 원칙: **통합 문서 우선**

**1. 통합 문서 유지 조건**
```yaml
조건:
  - 파일 크기 < 2,500줄
  - 동일한 주제 범위 (예: 개발 계획)
  - 동일한 독자층 (예: 개발팀)
  - 팀 규모 < 5명

행동:
  - 기존 문서에 섹션 추가 (예: plan.md에 섹션 14 추가)
  - 새 파일 생성 금지
  - 섹션 번호 순차 증가
```

**2. 별도 문서 분리 조건**
```yaml
조건:
  - 파일 크기 > 2,500줄
  - 독립적인 주제 (예: API 레퍼런스)
  - 다른 독자층 (예: 사용자 매뉴얼)
  - 팀 규모 > 5명

행동:
  - 새 파일 생성 고려
  - 사용자에게 분리 여부 확인
  - 기존 문서에서 참조 링크 추가
```

**3. 현재 프로젝트 적용 상황**
```yaml
docs/development/plan.md:
  - 현재 줄 수: 1,130줄 ✅ 통합 유지
  - 임계값: 2,500줄
  - 전략: 섹션 추가로 확장 (현재 섹션 1~14)

상태:
  - ✅ 통합 문서 방식 유지
  - ⏳ 2,500줄 도달시 분리 검토
```

#### 📋 Claude의 문서 작업 체크리스트

새 정보 추가시:
1. **파일 크기 확인**: `wc -l {파일명}` 실행
2. **통합 가능 여부 판단**:
   - < 2,500줄 → 기존 문서에 섹션 추가
   - > 2,500줄 → 사용자에게 분리 여부 확인
3. **섹션 번호 부여**: 순차적 증가 (14 → 15 → 16)
4. **목차(TOC) 업데이트**: 필요시 상단 목차 추가

#### 🚫 금지 사항

```yaml
❌ 하지 말 것:
  - 사용자 확인 없이 새 문서 파일 생성
  - 2,500줄 미만 파일을 임의로 분리
  - 기존 통합 문서를 임의로 분해
  - 중복 내용을 여러 파일에 작성

✅ 해야 할 것:
  - 항상 기존 문서 활용 우선
  - 파일 크기 임계값 확인
  - 사용자에게 전략 제안 및 확인
  - Single Source of Truth 유지
```

#### 💡 실전 예시

**시나리오 1: GitHub Actions 가이드 추가 (2025-01-21 작업)**
```
판단:
  - plan.md 크기: 1,130줄 < 2,500줄
  - 주제: 개발 프로세스의 일부
  - 독자: 동일 (개발팀)

행동:
  ✅ plan.md에 섹션 14 추가
  ❌ workflows-guide.md 별도 생성 안 함
```

**시나리오 2: 미래 - API 문서 1,500줄 추가 요청**
```
판단:
  - plan.md 현재: 2,300줄
  - 추가 시: 3,800줄 > 2,500줄

행동:
  1. 사용자에게 확인 요청:
     "plan.md가 2,500줄 임계값을 초과합니다.
      API 문서를 별도 파일(api-reference.md)로
      생성할까요?"
  2. 승인시 분리, 거부시 통합 유지
```

#### 🌿 Git 브랜치 백업 전략

**큰 변화 발생시 브랜치 백업 필수**:
- 아키텍처 변경 (예: 9개 → 12개 서비스)
- CLAUDE.md 200줄 이상 수정
- 개발 전략 변경 (예: 에이전트 구성 변경)
- 기술 스택 교체 (예: PostgreSQL → MongoDB)

**워크플로우 (5단계)**:
```
1. 현재 상태 커밋 (백업)
2. 백업 브랜치 생성 ({category}/{description})
3. 변경 작업 수행
4. 비교 보고서 작성 (COMPARISON_{category}_{date}.md)
5. 사용자 선택 (채택 or 유지)
```

**실전 예시**: CLAUDE.md Phase 3 최적화 (2025-01-21)
- 브랜치: `docs/claude-md-phase3-test`
- 비교 보고서: [PHASE_COMPARISON.md](PHASE_COMPARISON.md)
- 결과: Phase 3 채택 (7.3% 파일 크기 감소, 의사코드 전환)

**상세 가이드**: [docs/development/git-branch-strategy.md](docs/development/git-branch-strategy.md)

---

### 🔄 Ver2.0 Final 아키텍처 변경 요약

**서비스 구조 변화**:
- Ver1.0: 6개 서비스 → Ver2.0 Final: **9개 마이크로서비스**
- **Learning Service (8009) 신규 추가**: 자동학습 + Rule 추출 (ML 대체!)
- **용어 정정**: "Dashboard" → 3개 독립 서비스
  - Data Visualization Service (8006): 단순 데이터 대시보드
  - BI Service (8007): MCP 기반 AI 인사이트
  - Chat Interface Service (8008): 통합 AI 어시스턴트

**핵심 혁신 기능**:
1. **하이브리드 판단**: Rule Engine + LLM 순차 실행 ([섹션 2.1](#21-하이브리드-판단-전략-rule--llm))
2. **자동학습 시스템**: 3개 알고리즘 (빈도 분석 + 결정 트리 + LLM) ([섹션 2.3](#23-자동학습-시스템-전략-ver20-final---ml-대체))
3. **데이터 집계**: 토큰 90% 절감 + 할루시네이션 방지 ([섹션 2.4](#24-데이터-집계-알고리즘-할루시네이션-방지))
4. **Visual Workflow Builder**: n8n 스타일 드래그앤드롭
5. **MCP 컴포넌트 조립**: 사전 제작 컴포넌트 활용

**UI 매핑**:
- `UI/judgify-inventory-dashboard.html` → Data Visualization (8006)
- `UI/judgify-inventory-chat.html` → BI Service (8007)
- `UI/judgify-enterprise-ui.html` → Chat Interface (8008)

---

### 🚀 Quick Reference (빠른 참조)

#### 자주 찾는 정보
| 질문 | 답변 위치 |
|------|----------|
| **9개 서비스 목록은?** | [섹션 1](#-1-ver20-final-마이크로서비스-아키텍처-이해) 테이블 참조 |
| **개발 우선순위는?** | [섹션 1](#-1-ver20-final-마이크로서비스-아키텍처-이해) → 🧠 핵심 개발 우선순위 |
| **하이브리드 판단 로직은?** | [섹션 2.1](#21-하이브리드-판단-전략-rule--llm) 코드 예제 |
| **자동학습 시스템은?** | [섹션 2.3](#23-자동학습-시스템-전략-ver20-final---ml-대체) 상세 구현 |
| **데이터 집계 알고리즘은?** | [섹션 2.4](#24-데이터-집계-알고리즘-할루시네이션-방지) 구현 패턴 |
| **AI 에이전트 팀 구성은?** | [섹션 6](#-6-ver20-ai-에이전트-팀-구성) Phase 1-3 |
| **서비스별 에이전트 매핑은?** | [섹션 6.4](#64-서비스별-에이전트-매핑-전략-ver20-final) 테이블 |
| **MCP 도구 목록은?** | [섹션 7.1-7.2](#71-핵심-mcp-도구-1단계---즉시-필요) 핵심/확장 도구 |
| **문서 관리 규칙은?** | [섹션 0](#-문서-관리-전략-신규-추가) 2,500줄 임계값 |
| **개발 체크리스트는?** | [섹션 11](#-11-ver20-claude-개발-체크리스트) 전체 체크리스트 |

#### 핵심 파일 경로
| 문서 종류 | 파일 경로 | 용도 |
|----------|----------|------|
| **개발 계획** | [docs/development/plan.md](docs/development/plan.md) | Windows Desktop App 계획 (1,130줄) |
| **통합 요구사항** | [initial.md](initial.md) | Ver2.0 전체 요구사항 |
| **Prompt 설계** | [prompt-guide.md](prompt-guide.md) | LLM Prompt 템플릿 |
| **Judgment 설계** | [docs/services/judgment_engine.md](docs/services/judgment_engine.md) | 하이브리드 판단 엔진 |
| **Learning 설계** | [docs/services/learning_service.md](docs/services/learning_service.md) | 자동학습 시스템 (ML 대체) |
| **DB 스키마** | [docs/architecture/database_design.md](docs/architecture/database_design.md) | PostgreSQL + pgvector |
| **API 명세** | [docs/architecture/api_specifications.md](docs/architecture/api_specifications.md) | REST API 스펙 (2,555줄) |

#### 개발 시작 가이드
1. **처음 시작시**: [섹션 12](#-12-ver20-quick-start-for-claude) Quick Start 4단계
2. **서비스 개발시**: [섹션 3.2](#32-서비스별-개발-컨텍스트-매핑-ver20-final) 컨텍스트 매핑
3. **에이전트 협업시**: [섹션 13](#-13-ver20-ai-에이전트-활용-가이드) 에이전트별 역할
4. **테스트시**: [섹션 9](#-9-ver20-개발-검증-및-테스트-전략) 테스트 패턴
5. **배포시**: [섹션 10](#-10-ver20-배포-및-운영-자동화) Docker + K8s

---

## 🏗 1. Ver2.0 Final 마이크로서비스 아키텍처 이해

Claude가 개발할 **9개 핵심 마이크로서비스**:

| 서비스 | 포트 | Claude의 개발 역할 | 핵심 기술 | UI 매핑 |
|--------|------|-------------------|-----------|---------|
| **API Gateway** | 8000 | JWT 인증 + 라우팅 로직 설계 | Kong/Nginx + JWT | - |
| **Workflow Service** | 8001 | **Visual Workflow Builder (n8n 스타일)** | FastAPI + PostgreSQL + Next.js 14 | - |
| **Judgment Service** | 8002 | **하이브리드 판단 엔진 핵심 로직 + Connector 통합** | FastAPI + OpenAI + AST Parser + pgvector | - |
| **Action Service** | 8003 | 외부 시스템 연동 + Celery 비동기 | FastAPI + Celery + MCP | - |
| **Notification Service** | 8004 | Slack/Teams/Email 알림 | FastAPI + Message Queue | - |
| **Logging Service** | 8005 | 중앙집중 로그 수집/분석 시스템 | FastAPI + PostgreSQL + ELK | - |
| **Data Visualization Service** | 8006 | 단순 데이터 대시보드 (편집 가능) | FastAPI + PostgreSQL + WebSocket | `judgify-inventory-dashboard.html` |
| **BI Service** | 8007 | **MCP 기반 컴포넌트 조립 + 인사이트** | FastAPI + LLM + MCP Components | `judgify-inventory-chat.html` |
| **Chat Interface Service** | 8008 | **통합 AI 채팅 어시스턴트 (마스터 컨트롤러)** | FastAPI + LLM + WebSocket | `judgify-enterprise-ui.html` |
| **Learning Service** | **8009** | **자동학습 + Rule 추출 (ML 대체)** | FastAPI + PostgreSQL + pgvector + sklearn | - |

### 🧠 핵심 개발 우선순위 (Ver2.0 Final)
1. **Judgment Service (8002)** - 하이브리드 판단 엔진 (가장 중요!)
2. **Learning Service (8009)** - 자동학습 시스템 (혁신 기능! ML 대체)
3. **BI Service (8007)** - MCP 기반 컴포넌트 조립 (React 생성 대신)
4. **Chat Interface Service (8008)** - 통합 AI 어시스턴트 (마스터 컨트롤러)
5. **Workflow Service (8001)** - Visual Workflow Builder (n8n 스타일)
6. **Data Visualization Service (8006)** - 단순 데이터 대시보드
7. **나머지 서비스들** - 지원 시스템

---

## 🎯 2. Ver2.0 핵심 개발 철학

### 2.1 하이브리드 판단 전략 (Rule + LLM)

**실행 흐름**:
```
1. Rule Engine 우선 실행 (AST 기반, 안전함)
   ├─ 성공 && 신뢰도 ≥ 0.7 → 즉시 반환 (종료)
   └─ 실패 || 저신뢰도 → 2단계로 진행

2. LLM 보완 실행
   └─ OpenAI API 호출 (workflow context 활용)

3. 최종 결과 종합
   └─ Rule 결과 + LLM 결과 → 하이브리드 판단
```

**핵심 파라미터**:
- 신뢰도 임계값: `0.7`
- Rule Engine: AST 기반 (eval 금지)
- LLM Engine: OpenAI API

### 2.2 3-Tier Frontend 전략 (Ver2.0 핵심 변경!)

**용어 정정**: "Dashboard" → 3개 서비스로 분리

#### 2.2.1 Data Visualization Service (8006) - 단순 대시보드

**기능**: 미리 정의된 차트로 데이터 표시 (편집 가능)

**처리 흐름**:
```
render_dashboard:
  1. DB에서 대시보드 설정 로드
  2. PostgreSQL 데이터 직접 조회
  3. 미리 정의된 차트 렌더링
     (KPI 카드, 게이지, 라인/바 차트)

edit_dashboard:
  └─ 드래그앤드롭으로 차트 배치 변경 저장
```

#### 2.2.2 BI Service (8007) - AI 기반 인사이트 생성

**기능**: MCP 기반 컴포넌트 조립 + AI 인사이트 생성

**처리 흐름**:
```
generate_insight(user_request):
  1. LLM 요청 분석
     └─ 필요 데이터 + 비즈니스 컨텍스트 추출

  2. Judgment Service 호출
     └─ 데이터 기반 판단 실행

  3. React 컴포넌트 자동 생성
     └─ 인사이트 + 최적 차트 타입 기반

  4. 비즈니스 권장사항 생성
     ├─ 판단 결과 분석
     └─ RAG 엔진으로 유사 사례 검색

  반환: 대시보드 + 인사이트 + 권장사항
```

#### 2.2.3 Chat Interface Service (8008) - 통합 AI 어시스턴트

**기능**: Claude Desktop 수준 마스터 컨트롤러

**처리 흐름**:
```
handle_chat(user_message, session_id):
  1. 의도 분석 (NLP)
     └─ workflow_execution | data_visualization | settings_change

  2. 라우팅 로직
     ├─ workflow_execution → Workflow Service 호출
     ├─ data_visualization → BI Service 호출
     └─ settings_change → Settings 변경 (MCP 서버 상태 포함)

  3. 컨텍스트 유지 (멀티턴 대화)
     └─ 세션별 대화 이력 저장

  반환: 결과 + 세션 컨텍스트
```

### 2.3 자동학습 시스템 전략 (Ver2.0 Final - ML 대체!)

**핵심 개념**: 전통적 머신러닝 대신 3개 알고리즘 + Few-shot 학습으로 자동 Rule 추출

#### 처리 흐름

**1. 피드백 수집**:
```
collect_feedback(judgment_id, feedback_type, value):
  ├─ 피드백 저장 (👍👎, LOG 리뷰, 채팅)
  └─ value == 1 (긍정) → Few-shot 샘플 자동 추가
```

**2. Few-shot 학습**:
```
manage_few_shot(input_data):
  1. 입력 임베딩 생성 (OpenAI API)
  2. pgvector 유사 샘플 검색
     ├─ 테이블: training_samples
     ├─ 개수: 10-20개
     └─ 최소 정확도: 0.8
  반환: 유사 예시 목록
```

**3. 자동 Rule 추출 (3개 알고리즘)**:
```
extract_rules(workflow_id):
  알고리즘 1: 빈도 분석
    └─ 반복 패턴 발견

  알고리즘 2: 결정 트리 학습 (sklearn)
    └─ 조건문 자동 생성

  알고리즘 3: LLM 패턴 발견
    └─ OpenAI로 복잡한 패턴 추출

  → 최적 Rule 선택 및 저장
```

### 2.4 데이터 집계 알고리즘 (할루시네이션 방지!)

**목적**: LLM 토큰 최적화 + 할루시네이션 방지

**처리 흐름**:
```
aggregate_for_llm(raw_data, time_range):
  1. 통계 집계 (Statistical Aggregation)
     ├─ mean (평균)
     ├─ median (중앙값)
     ├─ std_dev (표준편차)
     ├─ min (최소)
     └─ max (최대)

  2. 평가 집계 (Evaluation Aggregation)
     ├─ status: normal | warning | critical
     │   (평균 < threshold → normal, 아니면 critical)
     └─ trend: increasing | decreasing
         (현재 평균 vs 이전 평균 비교)

  3. 트렌드 분석 (Trend Analysis)
     ├─ direction: 데이터 방향성 계산
     └─ change_rate: 변화율 계산

  4. 집계 데이터 저장 (아카이빙)
     └─ 통계 + 평가 + 트렌드 → DB 저장

  반환: {stats, evaluation, trend}
```

**핵심 효과**:
- 토큰 사용량: 원본 데이터 대비 90% 감소
- 할루시네이션 방지: 집계된 정확한 통계 값 전달

### 2.5 보안 우선 개발
- **AST 기반 Rule Engine**: JavaScript `eval()` 절대 금지
- **입력 검증**: 모든 API에 Pydantic 모델 적용
- **인증**: JWT + RBAC 철저히 구현

---

## 🔧 3. Ver2.0 Final 개발 컨텍스트 전략

### 3.1 문서 기반 컨텍스트 우선순위
Claude는 개발시 **반드시 다음 순서로** 문서를 참조해야 함:

1. **`CLAUDE.md`** (이 문서) - 전역 개발 규칙 및 아키텍처 이해
2. **`initial.md`** - Ver2.0 요구사항 및 제약조건
3. **`docs/services/{서비스명}.md`** - 구체적 구현 스펙
4. **`docs/architecture/system_overview.md`** - 전체 시스템 설계
5. **`prompt-guide.md`** - LLM 관련 개발시 Prompt 설계 가이드

### 3.2 서비스별 개발 컨텍스트 매핑 (Ver2.0 Final)
```bash
# Judgment Service 개발시 (최우선!)
docs/services/judgment_engine.md → AST Rule Engine + LLM 통합 로직
docs/architecture/database_design.md → judgment_executions 테이블 설계
prompt-guide.md → LLM 판단용 Prompt 템플릿

# Learning Service 개발시 (혁신 기능! ML 대체)
docs/services/learning_service.md → 자동학습 시스템 상세 설계
docs/algorithms/auto_rule_extraction.md → 3개 알고리즘 구현 가이드
docs/algorithms/data_aggregation.md → 데이터 집계 알고리즘 설계
docs/architecture/database_design.md → Learning 관련 테이블들
prompt-guide.md → Few-shot 학습 + Rule 추출 Prompt 템플릿

# Workflow Service 개발시 (Visual Builder!)
docs/services/workflow_editor.md → n8n 스타일 드래그앤드롭 에디터
docs/architecture/database_design.md → workflows 테이블 설계

# BI Service 개발시 (MCP 기반 컴포넌트 조립)
docs/services/bi_service.md → MCP 컴포넌트 조립 + 인사이트 생성
docs/services/judgment_engine.md → Judgment Service와 통합 방법
prompt-guide.md → BI 인사이트 생성용 Prompt 템플릿
UI/judgify-inventory-chat.html → UI 디자인 참조

# Chat Interface Service 개발시 (마스터 컨트롤러)
docs/services/chat_interface_service.md → 멀티턴 대화 + 의도 분류 로직
prompt-guide.md → 채팅 어시스턴트용 Prompt 설계
UI/judgify-enterprise-ui.html → UI 디자인 참조

# Data Visualization Service 개발시 (단순 대시보드)
docs/services/data_visualization_service.md → 미리 정의된 차트 렌더링 로직
UI/judgify-inventory-dashboard.html → UI 디자인 참조

# 전체 시스템 이해시
system-structure.md → 9개 마이크로서비스 간 통신 구조
docs/architecture/system_overview.md → 상세 아키텍처 및 기술 선택
```

---

## 🚀 4. Ver2.0 개발 흐름 및 패턴

### 4.1 마이크로서비스 개발 패턴

**FastAPI 서비스 구조**:
```
1. 기본 구조
   └─ FastAPI(title="서비스명", version="2.0.0")

2. 의존성 주입
   ├─ get_database() → PostgreSQL 연결
   └─ get_redis_cache() → Redis 캐시

3. 라우터 분리
   ├─ /api/v2/{service} → 비즈니스 로직
   └─ /health → 헬스체크

4. 에러 처리
   └─ @exception_handler → JSON 응답

5. 구조화 로깅
   └─ structured_logger → workflow_id + result 포함
```

### 4.2 하이브리드 판단 개발 패턴

**클래스 구조**:
```
HybridJudgmentEngine:
  초기화:
    ├─ rule_engine: ASTRuleEngine
    └─ llm_engine: OpenAIEngine

  judge(workflow_input) → JudgmentResult:
    1. Rule Engine 평가
    2. 성공 조건 체크 (신뢰도 ≥ 0.7 && 에러 없음)
       └─ 성공시 → method="rule"로 즉시 반환
    3. LLM 보완 실행
    4. 최종 결과 종합 (Rule + LLM)
```

### 4.3 자동 대시보드 생성 패턴

**처리 흐름**:
```
DashboardAutoGenerator.generate(user_request):
  1. LLM 요청 분석
     ├─ 필요 데이터 소스 식별
     └─ 비즈니스 의도 파악

  2. 컴포넌트 선택
     ├─ 데이터 타입 → 차트 타입 매핑
     └─ 시각화 의도 → 최적 레이아웃

  3. React 코드 생성
     └─ 컴포넌트 + 데이터 바인딩

  반환: DashboardConfig
    ├─ title (LLM 제안)
    ├─ components (선택된 차트들)
    ├─ react_code (생성된 코드)
    └─ real_time_config (업데이트 주기)
```

---

## 💾 5. Ver2.0 데이터베이스 개발 전략

### 5.1 PostgreSQL + pgvector 활용
```sql
-- Claude가 생성해야 하는 핵심 테이블들

-- 워크플로우 정의  
CREATE TABLE workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    definition JSONB NOT NULL,  -- 워크플로우 노드 구조
    rule_expression TEXT,       -- AST 파싱용 Rule 표현식
    version INTEGER DEFAULT 1,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW()
);

-- 판단 실행 결과 (핵심!)
CREATE TABLE judgment_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID REFERENCES workflows(id),
    input_data JSONB NOT NULL,
    rule_result JSONB,           -- Rule Engine 결과
    llm_result JSONB,            -- LLM Engine 결과  
    final_result JSONB NOT NULL, -- 최종 하이브리드 결과
    confidence_score DECIMAL(3,2), -- 신뢰도 점수
    method_used VARCHAR(20),     -- rule|llm|hybrid
    execution_time_ms INTEGER,
    explanation_embedding VECTOR(1536), -- RAG용 임베딩
    created_at TIMESTAMP DEFAULT NOW()
);
```

### 5.2 RAG 기반 설명 생성
```python
# Claude가 구현해야 하는 RAG 패턴

class RAGExplanationGenerator:
    async def generate_explanation(self, judgment_result: JudgmentResult) -> str:
        # 1. 유사한 과거 판단 검색 (pgvector)
        similar_cases = await self.vector_search(
            query_embedding=judgment_result.input_embedding,
            table="judgment_executions", 
            limit=5
        )
        
        # 2. 컨텍스트 구성
        context = {
            "current_judgment": judgment_result.dict(),
            "similar_cases": similar_cases,
            "domain_knowledge": self.get_domain_rules()
        }
        
        # 3. LLM으로 설명 생성
        explanation = await self.llm_explainer.generate(
            template="explanation_template",
            context=context
        )
        
        return explanation
```

---

## 🤖 6. Ver2.0 AI 에이전트 팀 구성

### 6.1 Phase 1: 핵심 기능 구현 에이전트 (8개)
```bash
# AI 판단 엔진 전문가
ai-engineer → 하이브리드 판단 로직 설계 및 구현
prompt-engineer → LLM 판단용 프롬프트 최적화

# 데이터 및 백엔드 전문가  
database-optimization → PostgreSQL + pgvector 최적화
data-engineer → ETL 파이프라인 및 데이터 처리
graphql-architect → 마이크로서비스 간 API 설계

# 비즈니스 로직 전문가
business-analyst → KPI 설계 및 비즈니스 메트릭
task-decomposition-expert → 복잡한 워크플로우 분해 설계
search-specialist → RAG 시스템 및 벡터 검색 구현
```

### 6.2 Phase 2: 확장 및 연동 에이전트 (6개)
```bash
# 인프라 및 보안
devops-engineer → Docker/Kubernetes 배포 자동화
security-engineer → JWT, RBAC, 데이터 암호화
performance-engineer → 성능 테스트 및 최적화

# MLOps 및 운영
mlops-engineer → AI 모델 배포 및 모니터링
customer-support → 사용자 가이드 및 문서화
risk-manager → 시스템 안정성 및 장애 대응
```

### 6.3 Phase 3: 고급 기능 에이전트 (4개)
```bash
# 문서화 및 모니터링
technical-writer → 프로젝트 문서화 표준화
observability-engineer → 모니터링 및 로그 분석

# UI/UX 및 연구
frontend-architect → 자동 생성 대시보드 UI/UX
academic-researcher → 최신 AI 논문 및 기술 동향 분석
```

### 6.4 서비스별 에이전트 매핑 전략 (Ver2.0 Final)
| 서비스 | 주담당 에이전트 | 협업 에이전트 |
|--------|-----------------|---------------|
| **Judgment Service (8002)** | ai-engineer, prompt-engineer | search-specialist, mlops-engineer |
| **Learning Service (8009)** 🔥 | **ai-engineer, mlops-engineer** | **search-specialist, database-optimization** |
| **Workflow Service (8001)** | task-decomposition-expert, graphql-architect | frontend-architect (Visual Builder) |
| **BI Service (8007)** | ai-engineer, business-analyst | prompt-engineer, frontend-architect |
| **Chat Interface Service (8008)** | prompt-engineer, frontend-architect | ai-engineer, technical-writer |
| **Data Visualization Service (8006)** | frontend-architect, data-engineer | business-analyst |
| **Action Service (8003)** | data-engineer, graphql-architect | security-engineer |
| **Notification Service (8004)** | data-engineer | devops-engineer |
| **Logging Service (8005)** | devops-engineer, observability-engineer | risk-manager |
| **API Gateway (8000)** | security-engineer, performance-engineer | devops-engineer |

### 6.5 하이브리드 AI 전략: Agent + Skill 통합 🆕

Ver2.0 Final에서는 **Agent (전략)** 와 **Skill (전술)** 을 결합한 하이브리드 접근법을 사용합니다.

#### 📊 Agent vs Skill 비교

| 구분 | Agent | Skill |
|------|-------|-------|
| **복잡도** | 높음 (전문가 판단) | 낮음 (절차 따라가기) |
| **소요 시간** | 20-60분 (깊이 분석) | 초-5분 (빠른 실행) |
| **창의성** | 필요 (새로운 해결책) | 불필요 (정해진 패턴) |
| **사용 방법** | `Task tool` 호출 | `/명령어` 실행 |
| **재사용성** | 낮음 (매번 다름) | 높음 (같은 패턴) |
| **비용** | 높음 (Opus 모델) | 낮음 (템플릿 실행) |

#### 🎯 언제 무엇을 사용할까?

**Agent 사용 조건**:
- ✅ 복잡한 아키텍처 설계 필요
- ✅ 창의적 문제 해결 필요
- ✅ 전문가 수준 판단 필요
- ✅ 컨텍스트 이해 필수
- ✅ 시간 여유 있음 (20-60분)
- ✅ 고품질 결과 중요

**Skill 사용 조건**:
- ✅ 반복적인 작업
- ✅ 정해진 절차 실행
- ✅ 빠른 결과 필요 (초~분)
- ✅ 일관성 중요
- ✅ 간단한 템플릿 생성
- ✅ 자동화 가능한 검증

#### 📋 8개 Skill 목록 (.claude/skills/)

1. **`/create-service`** - 새 FastAPI 서비스 템플릿 생성 (30초)
2. **`/generate-api`** - CRUD API 엔드포인트 자동 생성 (1분)
3. **`/generate-tests`** - pytest 테스트 템플릿 생성 (2분)
4. **`/sync-docs`** - API 문서 자동 동기화 (1분)
5. **`/validate-architecture`** - 아키텍처 규칙 검증 (30초)
6. **`/collect-metrics`** - Prometheus 메트릭 수집 (30초)
7. **`/run-load-test`** - Apache Bench 성능 테스트 (5분)
8. **`/compare-metrics`** - Before/After 비교 보고서 (1분)

#### 🔄 하이브리드 워크플로우 패턴

**패턴 1: Agent 설계 → Skill 생성 → Manual 개발**
```
Task tool (ai-engineer): 아키텍처 설계 (40분)
  ↓
/create-service: 템플릿 생성 (30초)
  ↓
수동 개발: 비즈니스 로직 구현 (60분)
  ↓
/generate-tests: 테스트 생성 (2분)
  ↓
/validate-architecture: 검증 (30초)
```

**패턴 2: Skill 데이터 수집 → Agent 분석 → Skill 실행**
```
/collect-metrics: 메트릭 수집 (30초)
  ↓
Task tool (performance-engineer): 문제 분석 (30분)
  ↓
수동 수정: 코드 최적화 (45분)
  ↓
/run-load-test: 성능 테스트 (5분)
  ↓
/compare-metrics: Before/After 비교 (1분)
```

**패턴 3: 반복 작업 자동화 (Skill 체인)**
```
/validate-architecture (30초)
  ↓
/generate-tests (2분)
  ↓
/sync-docs (1분)
  ↓
/collect-metrics (30초)
```

#### 📊 서비스별 Agent/Skill 비율 (예상)

| 서비스 | Agent 작업 | Skill 작업 | 비율 |
|--------|-----------|-----------|------|
| **Judgment Service (8002)** | 설계, 최적화 | 테스트, 검증 | 70% / 30% |
| **Learning Service (8009)** | 알고리즘 개선 | 벤치마크, 검증 | 75% / 25% |
| **Workflow Service (8001)** | UI/UX 설계 | 템플릿 생성 | 60% / 40% |
| **BI Service (8007)** | 인사이트 로직 | 컴포넌트 조립 | 65% / 35% |
| **전체 평균** | - | - | **58% / 42%** |

#### 💡 하이브리드 전략의 핵심

- **Agent는 "전략"**, **Skill은 "전술"**
- **복잡한 설계는 Agent**, **반복 실행은 Skill**
- **시간이 중요하면 Skill**, **품질이 중요하면 Agent**
- **처음엔 Agent로 설계**, **이후엔 Skill로 자동화**

#### 🎯 예상 효과

- **작업 시간**: 45분 → 19분 (58% 절감)
- **일일 처리량**: 10개 → 24개 (2.4배)
- **자동화율**: 반복 작업 95% Skill로 처리

**상세 가이드**: [docs/guides/hybrid-ai-strategy.md](docs/guides/hybrid-ai-strategy.md)

---

## 🔄 7. Ver2.0 MCP 및 외부 연동 전략

### 7.1 핵심 MCP 도구 (1단계 - 즉시 필요)
```python
# Ver2.0에서 Claude가 활용하는 핵심 MCP 도구들

CORE_MCP_TOOLS = {
    # 데이터베이스 및 파일시스템
    "postgresql-integration": "PostgreSQL 직접 연결 (Supabase 대체)",
    "filesystem-access": "프로젝트 코드 관리 및 파일 처리",
    
    # 코드 관리 및 협업
    "github-integration": "코드 관리 및 CI/CD 파이프라인",
    "memory-integration": "AI 판단 컨텍스트 및 세션 관리",
    
    # 테스트 및 모니터링
    "playwright-mcp-server": "마이크로서비스 E2E 테스트 자동화"
}
```

### 7.2 확장 MCP 도구 (2단계 - 기능 확장)
```python
EXTENDED_MCP_TOOLS = {
    # 개발 도구
    "context7": "최신 라이브러리 문서 및 API 참조",
    "circleci": "CI/CD 파이프라인 자동화",
    "deepgraph-typescript": "코드 분석 및 아키텍처 검증",
    
    # AI/LLM 관련  
    "openai": "하이브리드 판단 및 대시보드 생성용 LLM",
    
    # 외부 연동
    "slack": "판단 결과 알림 및 실시간 보고",
    "notion": "프로젝트 문서 및 설계 문서 관리",
    
    # 운영 도구
    "terminal": "Docker/Kubernetes 배포 명령",
    "redis": "캐시 및 세션 관리"
}
```

### 7.3 Judgify-core 특화 MCP 활용 시나리오
```bash
# PostgreSQL MCP 활용 예시
/query "SELECT * FROM judgment_executions WHERE confidence_score > 0.8"
/analyze-workflow-performance
/optimize-database-schema

# Memory MCP 활용 예시  
/save-context "하이브리드 판단 로직 개선사항"
/restore-context "마이크로서비스 아키텍처 설계"

# GitHub MCP 활용 예시
/create-issue "Judgment Service 성능 최적화"
/review-pr 123
/generate-release-notes v2.0.0

# Context7 MCP 활용 예시
/get-docs "fastapi async patterns"
/search-examples "postgresql pgvector integration"
```

### 7.4 외부 시스템 연동 패턴
```python
# Claude가 구현해야 하는 Action Service 패턴

class ActionExecutor:
    async def execute_action(self, judgment_result: JudgmentResult) -> ActionResult:
        actions = judgment_result.recommended_actions
        
        results = []
        for action in actions:
            if action.type == "slack_notification":
                result = await self.slack_client.send_alert(
                    channel="#alerts",
                    message=f"판단 완료: {judgment_result.result}",
                    confidence=judgment_result.confidence
                )
            
            elif action.type == "mcp_control":
                result = await self.mcp_client.execute_command(
                    system=action.target_system,
                    command=action.command,
                    parameters=action.parameters
                )
            
            results.append(result)
            
        return ActionResult(executed_actions=results)
```

---

## 🎨 8. Ver2.0 Frontend 자동 생성 전략

### 8.1 React 컴포넌트 자동 생성
```typescript
// Claude가 생성해야 하는 자동 대시보드 컴포넌트 패턴

export const AutoGeneratedDashboard = ({ config }: DashboardProps) => {
  const { data, loading } = useRealTimeData({
    dataSource: config.dataSource,
    refreshInterval: config.refreshInterval || 30000
  });

  return (
    <div className="dashboard-container">
      <h1 className="text-2xl font-bold mb-6">{config.title}</h1>
      <div className="grid grid-cols-12 gap-4">
        {config.components.map((component, index) => (
          <div 
            key={index} 
            className={`col-span-${component.width} h-${component.height}`}
          >
            {component.type === 'BarChart' && (
              <BarChart data={data[component.dataKey]} {...component.props} />
            )}
            {component.type === 'MetricCard' && (
              <MetricCard value={data[component.dataKey]} {...component.props} />
            )}
            {/* 기타 차트 타입들 */}
          </div>
        ))}
      </div>
    </div>
  );
};

// 실시간 데이터 훅 (Claude가 구현)
const useRealTimeData = ({ dataSource, refreshInterval }) => {
  const [data, setData] = useState(null);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    const ws = new WebSocket(`ws://localhost:8006/realtime/${dataSource}`);
    ws.onmessage = (event) => {
      setData(JSON.parse(event.data));
      setLoading(false);
    };
    return () => ws.close();
  }, [dataSource]);
  
  return { data, loading };
};
```

---

## 🔍 9. Ver2.0 개발 검증 및 테스트 전략

### 9.1 마이크로서비스 테스트 패턴
```python
# Claude가 생성해야 하는 테스트 코드 패턴

import pytest
from fastapi.testclient import TestClient
from judgment_service.app import app

client = TestClient(app)

class TestJudgmentService:
    def test_hybrid_judgment_rule_success(self):
        """Rule Engine 성공 케이스 테스트"""
        response = client.post("/api/v2/judgment/execute", json={
            "workflow_id": "test-workflow-123",
            "input_data": {"temperature": 90, "vibration": 45},
            "method": "hybrid"
        })
        
        assert response.status_code == 200
        result = response.json()
        assert result["result"] is True
        assert result["method_used"] == "rule"
        assert result["confidence"] >= 0.9
    
    def test_hybrid_judgment_llm_fallback(self):
        """LLM 보완 실행 케이스 테스트"""  
        response = client.post("/api/v2/judgment/execute", json={
            "workflow_id": "complex-workflow-456", 
            "input_data": {"complex_scenario": "unexpected situation"},
            "method": "hybrid"
        })
        
        assert response.status_code == 200
        result = response.json()
        assert result["method_used"] in ["llm", "hybrid"]
        assert "explanation" in result
```

### 9.2 E2E 테스트 자동화
```python
# Claude가 구현하는 Playwright E2E 테스트

async def test_dashboard_auto_generation_e2e():
    """대시보드 자동 생성 E2E 테스트"""
    
    # 1. 사용자 요청 시뮬레이션
    page = await browser.new_page()
    await page.goto("http://localhost:3000/dashboard")
    
    # 2. 자연어 요청 입력
    await page.fill('[data-testid="dashboard-request"]', 
                   "지난 주 워크플로우별 성공률을 보여줘")
    await page.click('[data-testid="generate-button"]')
    
    # 3. 대시보드 생성 확인
    await page.wait_for_selector('[data-testid="generated-dashboard"]')
    
    # 4. 차트 컴포넌트 로딩 확인
    chart = await page.query_selector('[data-testid="bar-chart"]')
    assert chart is not None
    
    # 5. 실시간 데이터 업데이트 확인
    await page.wait_for_function("() => document.querySelectorAll('.chart-data').length > 0")
```

---

## 🚀 10. Ver2.0 배포 및 운영 자동화

### 10.1 Docker + Kubernetes 배포 패턴
```yaml
# Claude가 생성하는 Kubernetes 배포 설정

# judgment-service-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: judgment-service
  namespace: judgify-prod
spec:
  replicas: 3
  selector:
    matchLabels:
      app: judgment-service
  template:
    spec:
      containers:
      - name: judgment-service
        image: judgify/judgment-service:v2.0.0
        ports:
        - containerPort: 8002
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: connection-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: redis-secret  
              key: connection-url
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: openai-secret
              key: api-key
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi" 
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8002
          initialDelaySeconds: 30
          periodSeconds: 10
```

### 10.2 모니터링 및 알림 자동화
```python
# Claude가 구현하는 모니터링 메트릭

from prometheus_client import Counter, Histogram, Gauge

# 비즈니스 메트릭
judgment_executions_total = Counter(
    'judgment_executions_total', 
    'Total number of judgment executions',
    ['method', 'result', 'workflow_id']
)

judgment_execution_duration = Histogram(
    'judgment_execution_duration_seconds',
    'Duration of judgment execution',
    ['method']
)

judgment_confidence_score = Gauge(
    'judgment_confidence_score',
    'Average confidence score of judgments',
    ['workflow_id']
)

# 시스템 메트릭
active_websocket_connections = Gauge(
    'dashboard_websocket_connections_active',
    'Number of active WebSocket connections for dashboards'
)
```

---

## 🎯 11. Ver2.0 Claude 개발 체크리스트

Claude가 개발시 **반드시 확인해야 할 체크리스트**:

### ✅ 아키텍처 준수사항
- [ ] 마이크로서비스별 독립적 배포 가능한 구조
- [ ] PostgreSQL + pgvector 활용한 RAG 구현  
- [ ] Redis 캐싱으로 성능 최적화
- [ ] JWT 기반 인증 및 RBAC 구현
- [ ] AST 기반 안전한 Rule Engine (eval 금지!)

### ✅ 핵심 기능 구현사항
- [ ] 하이브리드 판단 로직 (Rule → LLM 보완)
- [ ] 자연어 → React 대시보드 자동 생성
- [ ] 실시간 WebSocket 데이터 스트리밍
- [ ] pgvector 기반 유사 사례 검색
- [ ] Celery 비동기 외부 시스템 연동

### ✅ 품질 보증사항  
- [ ] 각 서비스별 유닛 테스트 90% 이상 커버리지
- [ ] Playwright E2E 테스트 시나리오 구현
- [ ] Prometheus 메트릭 및 Grafana 대시보드 구성
- [ ] 에러 처리 및 로깅 구조화
- [ ] API 문서 자동 생성 (OpenAPI/Swagger)

### ✅ 운영 준비사항
- [ ] Docker 컨테이너화 및 Kubernetes 배포 설정
- [ ] 헬스체크 엔드포인트 구현
- [ ] 환경별 설정 분리 (dev/staging/prod)
- [ ] 백업/복구 전략 수립
- [ ] 모니터링 및 알림 시스템 구축

### ✅ AI 에이전트 협업사항
- [ ] Phase 1 핵심 에이전트 8개 활성화
- [ ] 서비스별 담당 에이전트 명확한 역할 분담
- [ ] 에이전트 간 협업 워크플로우 구축
- [ ] MCP 도구 우선순위에 따른 단계적 도입
- [ ] 에이전트별 성과 측정 지표 설정

---

## 📖 12. Ver2.0 Quick Start for Claude

새로운 기능 개발시 Claude가 따라야 하는 **단계별 가이드**:

### 🚀 1단계: 아키텍처 및 팀 구성 이해
```bash
1. README.md 읽기 → 전체 프로젝트 파악
2. initial.md 읽기 → Ver2.0 요구사항 이해  
3. system-structure.md 읽기 → 마이크로서비스 구조 파악
4. 참고.txt 읽기 → 추가된 AI 에이전트 및 MCP 도구 현황
```

### 🚀 2단계: 에이전트 팀과 서비스별 설계 이해 (Ver2.0 Final)
```bash
# 핵심 서비스별 담당 에이전트 확인 (Ver2.0 Final - 9 services)
1. Judgment Service (8002) → ai-engineer, prompt-engineer 주도
2. Learning Service (8009) 🔥 → ai-engineer, mlops-engineer 주도 (혁신!)
3. Workflow Service (8001) → task-decomposition-expert, graphql-architect + frontend-architect (Visual Builder)
4. BI Service (8007) → ai-engineer, business-analyst 주도 (MCP 기반 BI)
5. Chat Interface Service (8008) → prompt-engineer, frontend-architect 주도 (마스터 컨트롤러)
6. Data Visualization Service (8006) → frontend-architect, data-engineer 주도

# 상세 설계 문서 확인
7. docs/services/judgment_engine.md → 하이브리드 판단 로직
8. docs/services/learning_service.md → 자동학습 시스템 (신규! ML 대체)
9. docs/algorithms/auto_rule_extraction.md → 3개 알고리즘 설계 (신규!)
10. docs/algorithms/data_aggregation.md → 데이터 집계 알고리즘 (신규!)
11. docs/services/workflow_editor.md → Visual Workflow Builder (n8n 스타일)
12. docs/services/bi_service.md → MCP 기반 BI (컴포넌트 조립)
13. docs/services/chat_interface_service.md → 통합 AI 채팅
14. docs/architecture/database_design.md → DB 스키마 (Learning 테이블 포함)
```

### 🚀 3단계: MCP 도구 설정 및 개발 시작 (Ver2.0 Final)
```bash
# 1. 핵심 MCP 도구 설치 (우선순위)
1. postgresql-integration → 데이터베이스 연결
2. filesystem-access → 프로젝트 파일 관리
3. github-integration → 코드 관리
4. memory-integration → 컨텍스트 관리
5. playwright-mcp-server → 테스트 자동화

# 2. 개발 우선순위 (에이전트 협업) - Ver2.0 Final (9 services)
1. Judgment Service (8002) → ai-engineer + prompt-engineer
2. Learning Service (8009) 🔥 → ai-engineer + mlops-engineer (혁신 기능!)
3. BI Service (8007) → ai-engineer + business-analyst (MCP 기반 BI)
4. Chat Interface Service (8008) → prompt-engineer + frontend-architect (마스터 컨트롤러)
5. Workflow Service (8001) → task-decomposition-expert + frontend-architect (Visual Builder)
6. Data Visualization Service (8006) → frontend-architect + data-engineer
7. 기타 지원 서비스들 → 각 담당 에이전트
```

### 🚀 4단계: 품질 검증 및 에이전트 성과 평가
```bash
# 기술적 품질 검증
1. 유닛 테스트 작성 및 실행 (performance-engineer 지원)
2. E2E 테스트 시나리오 구현 (playwright MCP 활용)  
3. API 문서 자동 생성 확인 (technical-writer 검토)
4. Docker 컨테이너 빌드/실행 테스트 (devops-engineer 주도)

# 에이전트 협업 성과 검증
5. 각 에이전트별 담당 영역 완료도 확인
6. 서비스별 에이전트 협업 효율성 측정
7. Phase별 에이전트 확장 계획 검토
```

---

---

## 🌟 13. Ver2.0 AI 에이전트 활용 가이드

### 13.1 에이전트별 핵심 역할 및 활용법

#### 🧠 **AI/ML 전문 에이전트**
```bash
# ai-engineer
- 하이브리드 판단 로직 아키텍처 설계
- AST Rule Engine + LLM 통합 구현
- 판단 성능 최적화 및 메트릭 설계

# prompt-engineer  
- LLM 판단용 프롬프트 템플릿 설계
- Few-shot 학습 데이터 구성
- 프롬프트 성능 A/B 테스트

# search-specialist
- pgvector 기반 RAG 시스템 구현
- 유사 사례 검색 알고리즘 최적화
- 임베딩 모델 선택 및 튜닝
```

#### 📊 **데이터/백엔드 전문 에이전트**
```bash
# data-engineer
- ETL 파이프라인 설계 및 구현
- 실시간 데이터 스트리밍 구축
- 데이터 품질 관리 및 검증

# database-optimization
- PostgreSQL 성능 튜닝
- 인덱스 전략 및 쿼리 최적화
- pgvector 벡터 검색 최적화

# graphql-architect
- 마이크로서비스 간 API 설계
- GraphQL 스키마 최적화
- API Gateway 라우팅 전략
```

#### 🎨 **Frontend/UX 전문 에이전트**
```bash
# frontend-architect
- 자동 대시보드 생성 UI/UX 설계
- React 컴포넌트 자동 생성 로직
- 실시간 데이터 시각화 최적화

# business-analyst
- 비즈니스 메트릭 및 KPI 설계
- 사용자 요구사항 분석
- 대시보드 효과성 측정
```

### 13.2 MCP 도구 활용 우선순위

#### 🥇 **1단계: 핵심 도구 (즉시 도입)**
```bash
1. postgresql-integration → database-optimization 에이전트와 협업
2. filesystem-access → 모든 에이전트 공통 활용
3. github-integration → devops-engineer 주도 활용
4. memory-integration → ai-engineer, prompt-engineer 활용
5. playwright-mcp-server → performance-engineer 테스트 자동화
```

#### 🥈 **2단계: 확장 도구 (단계적 도입)**
```bash
6. context7 → academic-researcher 최신 기술 동향 파악
7. circleci → devops-engineer CI/CD 파이프라인 구축
8. slack → customer-support 사용자 소통 채널
9. notion → technical-writer 문서 관리
10. terminal → devops-engineer 배포 자동화
```

### 13.3 에이전트 간 협업 워크플로우

#### 🔄 **Judgment Service 개발 워크플로우**
```mermaid
workflow TD
    A[ai-engineer: 판단 로직 설계] --> B[prompt-engineer: 프롬프트 최적화]
    B --> C[search-specialist: RAG 구현]
    C --> D[mlops-engineer: 모델 배포]
    D --> E[performance-engineer: 성능 테스트]
```

#### 🔄 **BI Service + Data Visualization 개발 워크플로우**
```mermaid
workflow TD
    A[business-analyst: 요구사항 분석] --> B[frontend-architect: UI 설계]
    B --> C[data-engineer: 데이터 파이프라인]
    C --> D[technical-writer: 사용자 가이드]
    D --> E[customer-support: 피드백 수집]
```

### 13.4 에이전트 성과 측정 지표

| 에이전트 | 핵심 KPI | 측정 방법 |
|----------|----------|----------|
| **ai-engineer** | 판단 정확도, 응답 시간 | 95% 정확도, <500ms 응답 |
| **prompt-engineer** | LLM 성능, 비용 효율성 | F1-score >0.9, 30% 비용 절감 |
| **frontend-architect** | 대시보드 생성 시간, 사용성 | <30초 생성, 4.5/5 사용자 만족도 |
| **devops-engineer** | 배포 성공률, 서비스 가용성 | 99.9% 배포 성공, 99.5% 가용성 |
| **performance-engineer** | 시스템 성능, 확장성 | <100ms API 응답, 10x 트래픽 대응 |

---

**🎯 Ver2.0 Final 최종 성공 지표**:
1. **사용자 경험**:
   - "지난 주 불량률 분석해줘" → 30초 내 AI 인사이트 + 자동 대시보드 생성
   - 채팅으로 "품질 검사 워크플로우 실행해줘" → 즉시 실행 및 결과 표시
   - Settings에서 MCP 서버 연결 상태 실시간 확인
   - n8n 스타일 Visual Builder로 워크플로우 드래그앤드롭 생성
2. **기술적 성과**: 18개 에이전트가 협력하여 **9개 마이크로서비스 (Ver2.0 Final)** 완성
3. **비즈니스 가치**: 하이브리드 판단으로 95% 정확도, 50% 비용 절감 달성
4. **혁신 기능**: 자동학습 시스템 (ML 대체) + 데이터 집계 알고리즘으로 토큰 최적화

---

**Happy Coding with 9 Services + AI Agents + Auto-Learning, Claude! 🤖⚡🚀🔥**