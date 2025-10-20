# Judgify-core Ver2.0 Final 🚀

**AI 판단 플랫폼 - 제조업 SME를 위한 하이브리드 자동화 시스템**

Ver2.0 Final은 마이크로서비스 아키텍처 기반으로 전면 재설계된 차세대 AI 판단 플랫폼입니다.

---

## 🎯 핵심 혁신 (Ver2.0 Final)

### 1️⃣ **9개 마이크로서비스 아키텍처**
```
API Gateway (8000) → 인증 및 라우팅
Workflow Service (8001) → n8n 스타일 워크플로우 관리
Judgment Service (8002) → 하이브리드 판단 엔진 (Rule + LLM)
Action Service (8003) → 외부 시스템 연동
Logging Service (8005) → 중앙집중 로깅
Dashboard Service (8006) → 데이터 시각화
BI Service (8007) 🔥 → MCP 컴포넌트 조립
Chat Interface (8008) 🔥 → 통합 마스터 컨트롤러
Learning Service (8009) 🔥 → 자동학습 (ML 모델 대체)
```

### 2️⃣ **ML 모델 없는 자동학습**
3가지 전통적 알고리즘으로 ML 모델 완전 대체:
- **빈도 분석**: 80% 패턴 → Rule 자동 추출
- **결정 트리**: sklearn → Rule 변환
- **LLM 패턴**: 통계 + LLM 분석

### 3️⃣ **MCP 컴포넌트 조립 (React 코드 생성 대체)**
사전 제작된 MCP 컴포넌트를 검색하고 조립하여 안정적인 BI 대시보드 생성

### 4️⃣ **LLM 할루시네이션 완벽 방지**
- 원시 데이터 영구 보관 (raw_data)
- LLM에는 집계 통계만 전달
- 3단계 집계: 통계 + 평가 + 트렌드

---

## 📚 문서 구조

### 핵심 가이드 (루트)
```
CLAUDE.md           ← Claude 개발 가이드 (18개 AI 에이전트 매핑)
initial.md          ← Ver2.0 Final 통합 요구사항
prompt-guide.md     ← LLM Prompt 설계 전략
system-structure.md ← 시스템 아키텍처 개요
```

### 상세 설계 (docs/)
```
docs/
├── algorithms/            ← 알고리즘 상세 설계
│   ├── auto_rule_extraction.md   (3가지 Rule 추출 알고리즘)
│   └── data_aggregation.md       (LLM 할루시네이션 방지)
│
├── services/              ← 마이크로서비스별 설계
│   ├── judgment_engine.md         (하이브리드 판단)
│   ├── dashboard_service.md       (데이터 시각화)
│   ├── workflow_editor.md         (워크플로우 관리)
│   ├── learning_service.md 🔥     (자동학습)
│   └── external_integration.md    (외부 연동)
│
├── architecture/          ← 시스템 아키텍처
│   ├── system_overview.md
│   ├── database_design.md
│   ├── api_specifications.md
│   └── security_architecture.md
│
└── operations/            ← 운영 관리
    ├── monitoring_guide.md
    ├── deployment_strategy.md
    └── incident_response_guide.md
```

---

## 🚀 빠른 시작

⚠️ **새 PC에서 시작하는 경우 반드시 읽어주세요!**

### 1단계: 레포지토리 클론
```bash
git clone https://github.com/mugoori/Judgify-core.git
cd Judgify-core
```

### 2단계: 필수 설정 파일 생성 ⚠️
**중요**: `.gitignore`에 포함된 파일들은 Git에 커밋되지 않으므로, 클론 후 반드시 생성해야 합니다.

#### 자동 생성 (권장)
```bash
# Mac/Linux
./scripts/setup-env.sh

# Windows (PowerShell)
.\scripts\setup-env.ps1

# Windows (Command Prompt)
scripts\setup-env.bat
```

#### 수동 생성
```bash
# Mac/Linux
cp .env.example .env
cp .mcp.template.json .mcp.json

# Windows
copy .env.example .env
copy .mcp.template.json .mcp.json
```

### 3단계: 환경 변수 설정
`.env` 파일을 열고 다음 값을 입력하세요:
```bash
# PostgreSQL 데이터베이스
DATABASE_URL=postgresql://user:pass@localhost:5432/judgify_prod

# Redis 캐시
REDIS_URL=redis://localhost:6379/0

# OpenAI API Key (AI 판단 엔진용)
OPENAI_API_KEY=sk-your-openai-api-key
```

### 4단계: MCP 토큰 설정
`.mcp.json` 파일을 열고 GitHub Personal Access Token을 입력하세요:
```json
{
  "mcpServers": {
    "github": {
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_your_github_token"
      }
    }
  }
}
```

💡 **상세 설정 가이드**: [SETUP.md](SETUP.md) 참조

---

### 5단계: 문서 확인
```bash
# 1. 전체 아키텍처 이해
cat CLAUDE.md              # Claude 개발 가이드
cat initial.md             # Ver2.0 Final 요구사항
cat system-structure.md    # 시스템 구조도

# 2. 서비스별 상세 설계
cat docs/services/learning_service.md        # Learning Service
cat docs/algorithms/auto_rule_extraction.md  # Rule 추출 알고리즘
cat docs/algorithms/data_aggregation.md      # 데이터 집계
```

### 6단계: 개발 우선순위
```
Priority 1: Learning Service (8009)
  - 3가지 Rule 추출 알고리즘 구현
  - Few-shot 학습 관리 (pgvector)

Priority 2: Judgment Service (8002)
  - 하이브리드 판단 로직 (Rule → LLM)
  - Few-shot 샘플 활용

Priority 3: BI Service (8007)
  - MCP 컴포넌트 검색 및 조립
  - 자동 대시보드 생성
```

---

## 🤖 AI 에이전트 팀 (18개)

### Phase 1: 핵심 기능 구현 (8개)
```
ai-engineer            → 하이브리드 판단 로직
prompt-engineer        → LLM 프롬프트 최적화
database-optimization  → PostgreSQL + pgvector
data-engineer          → ETL 파이프라인
graphql-architect      → 마이크로서비스 API
business-analyst       → KPI 설계
task-decomposition     → 워크플로우 분해
search-specialist      → RAG 시스템
```

### Phase 2: 확장 및 연동 (6개)
```
devops-engineer        → Docker/Kubernetes
security-engineer      → JWT, RBAC
performance-engineer   → 성능 테스트
mlops-engineer         → AI 모델 배포
customer-support       → 사용자 가이드
risk-manager           → 시스템 안정성
```

### Phase 3: 고급 기능 (4개)
```
technical-writer       → 문서화
observability-engineer → 모니터링
frontend-architect     → UI/UX
academic-researcher    → 최신 기술 동향
```

---

## 🛠 기술 스택

### Backend
- **Framework**: FastAPI + Python 3.11+
- **Database**: PostgreSQL 15+ with pgvector
- **Cache**: Redis 7.0+ (5min TTL)
- **Queue**: Celery with Redis broker
- **ML Alternative**: sklearn (결정 트리만)

### Frontend
- **Framework**: Next.js 14 + TypeScript
- **UI Components**: MCP 사전 제작 컴포넌트
- **Workflow Editor**: n8n 스타일 (React Flow)
- **State**: React Context API

### Infrastructure
- **Deployment**: Docker + Kubernetes
- **Monitoring**: Prometheus + Grafana
- **Logging**: ELK Stack
- **CI/CD**: GitHub Actions

---

## 📊 성능 목표

```yaml
Rule 추출 성능:
  - 빈도 분석: < 1초
  - 결정 트리: < 2초
  - LLM 패턴: < 3초

판단 성능:
  - Rule Engine: < 100ms
  - LLM Fallback: < 2초
  - Hybrid: < 2.5초

데이터 집계:
  - 통계 집계: < 1초 (10K 샘플)
  - 전체 파이프라인: < 3초

정확도 목표:
  - Rule 추출 정확도: 85% 이상
  - Few-shot 효과성: +15%p 향상
  - 의도 분류 정확도: 92% 이상
```

---

## 📖 추가 리소스

- **CLAUDE.md**: Claude Code 개발 가이드 (AI 에이전트 협업)
- **initial.md**: Ver2.0 Final 전체 요구사항
- **prompt-guide.md**: 9개 서비스 LLM Prompt 템플릿
- **docs/**: 상세 설계 문서 (알고리즘, 서비스, 아키텍처)

---

## 📝 라이센스

Proprietary - Judgify-core Ver2.0 Final

---

**Ver2.0 Final - 깨끗한 시작, 강력한 혁신! 🚀**
