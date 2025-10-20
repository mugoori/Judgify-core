# Judgify-core Ver2.0 Final - Windows Desktop Application 개발 계획서

**프로젝트명**: Judgify Desktop (Windows 네이티브 AI 판단 플랫폼)
**아키텍처**: Tauri (Rust + React)
**개발 기간**: 8주
**목표**: Windows 기업용 솔루션 단일 실행 파일 (.exe) 배포

---

## 📋 1. 프로젝트 개요

### 1.1 비전
기업용 AI 기반 하이브리드 판단 플랫폼을 Windows 네이티브 데스크톱 앱으로 제공하여, 설치 및 관리가 간편하고 안정적인 솔루션 구현

### 1.2 핵심 가치
- **경량성**: 15~30MB 실행 파일, 메모리 200~400MB
- **안정성**: Rust의 메모리 안전성 + 타입 안전성
- **오프라인 동작**: 인터넷 없이도 Rule Engine 동작
- **자동 업데이트**: GitHub Releases 기반 무중단 업데이트
- **LLM 자동 생성**: React 기반 동적 UI/워크플로우 생성

---

## 🏗 2. 기술 스택

### 2.1 Frontend
```yaml
Core:
  - React 18.2+
  - TypeScript 5.0+
  - Vite 5.0+ (빌드 도구)

UI Framework:
  - shadcn/ui (컴포넌트 라이브러리)
  - Tailwind CSS 3.4+
  - Radix UI (Headless Components)

State Management:
  - Zustand (경량 상태 관리)
  - TanStack Query (서버 상태)

Visualization:
  - Recharts (차트)
  - React Flow (워크플로우 에디터)
  - TanStack Table (데이터 테이블)
```

### 2.2 Backend (Rust)
```yaml
Framework:
  - Tauri 1.5+
  - tokio (비동기 런타임)
  - serde (직렬화)

Database:
  - rusqlite (SQLite 바인딩)
  - faiss-rs (벡터 검색)

AI/ML:
  - reqwest (HTTP 클라이언트 - OpenAI API)
  - tiktoken-rs (토큰 카운팅)

Rule Engine:
  - rhai (안전한 스크립팅 엔진)
  - ast-parser (AST 파싱)
```

### 2.3 DevOps
```yaml
Build:
  - Rust 1.75+
  - Node.js 20+
  - pnpm 8+

Packaging:
  - tauri-bundler (Windows Installer)
  - NSIS (커스텀 인스톨러)

CI/CD:
  - GitHub Actions
  - electron-updater 패턴 적용
```

---

## 🎯 3. 핵심 기능 요구사항

### 3.1 하이브리드 판단 엔진 (Judgment Service)
```rust
기능:
  - Rule Engine 우선 실행 (AST 기반, 안전함)
  - Rule 실패시 LLM 보완 (OpenAI GPT-4)
  - 신뢰도 기반 결과 선택 (Confidence >= 0.7)
  - 판단 결과 SQLite 저장 + FAISS 임베딩

기술적 구현:
  - rhai 스크립팅 엔진으로 Rule 실행
  - reqwest로 OpenAI API 비동기 호출
  - serde_json으로 JSON 처리
  - rusqlite로 결과 저장
```

### 3.2 자동학습 시스템 (Learning Service)
```rust
기능:
  - 사용자 피드백 수집 (👍👎, LOG, 채팅)
  - Few-shot 학습 관리 (10-20개 유사 예시)
  - 자동 Rule 추출 (3개 알고리즘)
    1. 빈도 분석 (Frequency Analysis)
    2. 결정 트리 학습 (Decision Tree)
    3. LLM 패턴 발견 (Pattern Discovery)

기술적 구현:
  - FAISS로 유사 샘플 벡터 검색
  - 통계 알고리즘 (평균, 중앙값, 표준편차)
  - LLM으로 패턴 추출 후 Rule 변환
```

### 3.3 BI 서비스 (MCP 기반 컴포넌트 조립)
```typescript
기능:
  - 자연어 요청 분석 (LLM)
  - 적절한 React 컴포넌트 자동 선택
  - shadcn/ui 컴포넌트 조립
  - 실시간 데이터 바인딩

기술적 구현:
  - LLM으로 요청 의도 분석
  - JSX 코드 생성 및 동적 렌더링
  - Recharts로 차트 생성
  - WebSocket으로 실시간 업데이트
```

### 3.4 Chat Interface (통합 AI 어시스턴트)
```typescript
기능:
  - 멀티턴 대화 컨텍스트 유지
  - 의도 분류 (워크플로우 실행, BI 요청, 설정 변경)
  - 마스터 컨트롤러 역할 (모든 서비스 통합)

기술적 구현:
  - Zustand로 대화 세션 관리
  - Tauri IPC로 Rust 백엔드 호출
  - Markdown 렌더링 (react-markdown)
```

### 3.5 Visual Workflow Builder (n8n 스타일)
```typescript
기능:
  - 드래그앤드롭 노드 에디터
  - 자연어 → 워크플로우 자동 생성
  - Rule 표현식 시각적 편집
  - 실시간 실행 및 디버깅

기술적 구현:
  - React Flow 라이브러리
  - LLM으로 워크플로우 생성
  - Rust로 워크플로우 실행
```

### 3.6 Data Visualization (단순 대시보드)
```typescript
기능:
  - 미리 정의된 차트 렌더링
  - 드래그앤드롭 레이아웃 편집
  - 실시간 데이터 업데이트

기술적 구현:
  - Recharts 차트 라이브러리
  - react-grid-layout
  - TanStack Query로 데이터 페칭
```

---

## 📐 4. 시스템 아키텍처

### 4.1 전체 구조
```
┌─────────────────────────────────────────────────┐
│  Judgify Desktop (Windows .exe)                 │
│                                                  │
│  ┌────────────────────────────────────────────┐ │
│  │  Frontend (React + Vite)                   │ │
│  │  - Chat Interface                          │ │
│  │  - Dashboard                               │ │
│  │  - Workflow Builder                        │ │
│  │  - Settings                                │ │
│  └────────────────────────────────────────────┘ │
│           ▲ Tauri IPC                            │
│           ▼                                      │
│  ┌────────────────────────────────────────────┐ │
│  │  Backend (Rust)                            │ │
│  │  ├─ Judgment Engine                        │ │
│  │  ├─ Learning Service                       │ │
│  │  ├─ BI Service                             │ │
│  │  ├─ Chat Service                           │ │
│  │  ├─ Workflow Service                       │ │
│  │  └─ Database Layer                         │ │
│  └────────────────────────────────────────────┘ │
│           ▲                                      │
│           ▼                                      │
│  ┌────────────────────────────────────────────┐ │
│  │  SQLite + FAISS                            │ │
│  │  - judgments.db (SQLite)                   │ │
│  │  - vectors.index (FAISS)                   │ │
│  └────────────────────────────────────────────┘ │
│                                                  │
│  ┌────────────────────────────────────────────┐ │
│  │  System Integration                        │ │
│  │  - System Tray                             │ │
│  │  - Auto Update                             │ │
│  │  - File System Access                      │ │
│  └────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### 4.2 Tauri IPC 통신 구조
```typescript
// Frontend (TypeScript)
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke<JudgmentResult>('execute_judgment', {
  input: { temperature: 90, vibration: 45 }
});
```

```rust
// Backend (Rust)
#[tauri::command]
async fn execute_judgment(input: JudgmentInput) -> Result<JudgmentResult, String> {
    // 하이브리드 판단 로직
}
```

---

## 🗂 5. 프로젝트 디렉토리 구조

```
judgify-desktop/
├── src-tauri/                      # Rust 백엔드
│   ├── src/
│   │   ├── main.rs                # Tauri 엔트리포인트
│   │   ├── commands/              # Tauri Commands
│   │   │   ├── mod.rs
│   │   │   ├── judgment.rs       # 판단 엔진 Command
│   │   │   ├── learning.rs       # 학습 서비스 Command
│   │   │   ├── bi.rs             # BI 서비스 Command
│   │   │   ├── chat.rs           # 채팅 서비스 Command
│   │   │   └── workflow.rs       # 워크플로우 Command
│   │   ├── services/              # 비즈니스 로직
│   │   │   ├── mod.rs
│   │   │   ├── judgment_engine.rs
│   │   │   ├── rule_engine.rs
│   │   │   ├── llm_engine.rs
│   │   │   ├── learning_service.rs
│   │   │   ├── bi_service.rs
│   │   │   └── workflow_service.rs
│   │   ├── database/              # DB 레이어
│   │   │   ├── mod.rs
│   │   │   ├── sqlite.rs
│   │   │   ├── faiss.rs
│   │   │   └── models.rs
│   │   └── utils/                 # 유틸리티
│   │       ├── mod.rs
│   │       ├── openai.rs
│   │       └── embeddings.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/
│
├── src/                            # React Frontend
│   ├── main.tsx                   # React 엔트리포인트
│   ├── App.tsx
│   ├── pages/                     # 페이지 컴포넌트
│   │   ├── ChatInterface.tsx
│   │   ├── Dashboard.tsx
│   │   ├── WorkflowBuilder.tsx
│   │   ├── BiInsights.tsx
│   │   └── Settings.tsx
│   ├── components/                # 재사용 컴포넌트
│   │   ├── ui/                   # shadcn/ui 컴포넌트
│   │   ├── charts/               # 차트 컴포넌트
│   │   ├── workflow/             # 워크플로우 노드
│   │   └── layout/               # 레이아웃
│   ├── lib/                       # 유틸리티
│   │   ├── tauri-api.ts          # Tauri IPC 래퍼
│   │   ├── utils.ts
│   │   └── constants.ts
│   ├── hooks/                     # Custom Hooks
│   │   ├── useJudgment.ts
│   │   ├── useLearning.ts
│   │   └── useWorkflow.ts
│   ├── store/                     # Zustand Store
│   │   ├── chatStore.ts
│   │   ├── workflowStore.ts
│   │   └── settingsStore.ts
│   └── styles/
│       └── globals.css
│
├── public/                         # 정적 리소스
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
├── components.json                # shadcn/ui 설정
└── README.md
```

---

## 📅 6. 개발 일정 (8주)

### Week 1: 프로젝트 초기화 및 기본 구조
**목표**: Tauri 프로젝트 생성, 기본 UI 구조, DB 초기화

```yaml
Day 1-2:
  - Tauri 프로젝트 생성
  - React + TypeScript + Vite 설정
  - shadcn/ui 초기화
  - Git 저장소 설정

Day 3-4:
  - SQLite 데이터베이스 스키마 설계
  - rusqlite 통합
  - FAISS 벡터 인덱스 설정
  - 기본 CRUD 함수

Day 5:
  - Tauri IPC 통신 테스트
  - 기본 UI 레이아웃 (Header, Sidebar, Main)
  - 라우팅 설정 (React Router)
```

### Week 2: Judgment Engine 핵심 로직
**목표**: 하이브리드 판단 엔진 완성

```yaml
Day 1-2:
  - rhai Rule Engine 통합
  - AST 파싱 및 안전한 Rule 실행
  - Rule 평가 함수 구현

Day 3-4:
  - OpenAI API 클라이언트 구현
  - LLM 판단 로직
  - 프롬프트 템플릿 관리

Day 5:
  - 하이브리드 판단 로직 통합
  - 신뢰도 계산 알고리즘
  - 결과 저장 및 임베딩 생성
```

### Week 3: Learning Service (자동학습)
**목표**: 자동학습 시스템 완성

```yaml
Day 1-2:
  - 피드백 수집 시스템
  - Few-shot 샘플 관리
  - FAISS 유사도 검색

Day 3-4:
  - Rule 추출 알고리즘 3개 구현
    1. 빈도 분석
    2. 결정 트리
    3. LLM 패턴 발견

Day 5-6: (신규 추가!) 🔥
  - MCP 조건부 활성화 시스템
  - 판단 복잡도 분석 로직 (Rule 기반)
  - 3-Tier MCP 활성화 전략 (simple/medium/complex)
  - 토큰 사용량 추적 시스템
  - Redis 캐싱으로 Context7 문서 재사용 (30분 TTL)
  - 비용 모니터링 대시보드 (일일/월별 토큰 사용량)
  - 워크플로우 UI에서 MCP 설정 체크박스
```

### Week 4: BI Service + Chat Interface
**목표**: LLM 기반 동적 UI 생성

```yaml
Day 1-2:
  - BI Service: LLM 요청 분석
  - React 컴포넌트 자동 선택
  - shadcn/ui 컴포넌트 조립

Day 3-4:
  - Chat Interface UI
  - 멀티턴 대화 컨텍스트
  - 의도 분류 시스템

Day 5:
  - Markdown 렌더링
  - 코드 하이라이팅
  - 채팅 히스토리 저장
```

### Week 5: Visual Workflow Builder
**목표**: n8n 스타일 워크플로우 에디터

```yaml
Day 1-2:
  - React Flow 통합
  - 드래그앤드롭 노드 에디터
  - 커스텀 노드 타입 정의

Day 3-4:
  - LLM 기반 워크플로우 자동 생성
  - 자연어 → 노드 변환
  - Rule 표현식 시각적 편집

Day 5:
  - 워크플로우 실행 엔진
  - 실시간 디버깅 UI
  - 저장/불러오기
```

### Week 6: Data Visualization + Settings
**목표**: 대시보드 및 설정 화면

```yaml
Day 1-2:
  - Recharts 차트 통합
  - 미리 정의된 차트 컴포넌트
  - 드래그앤드롭 레이아웃

Day 3-4:
  - Settings 화면
  - OpenAI API 키 관리
  - MCP 서버 상태 표시
  - 테마 설정 (다크 모드)

Day 5:
  - 실시간 데이터 업데이트
  - TanStack Query 캐싱
```

### Week 7: Windows Integration + Installer
**목표**: Windows 전용 기능 및 배포 준비

```yaml
Day 1-2:
  - System Tray 통합
  - 백그라운드 실행
  - 자동 시작 옵션

Day 3-4:
  - Auto Update 구현
  - GitHub Releases 연동
  - 업데이트 다운로드/설치

Day 5:
  - Windows Installer (NSIS)
  - 코드 사이닝 (선택)
  - 배포 스크립트 작성
```

### Week 8: 테스트 + 문서화 + 배포
**목표**: 안정성 확보 및 최종 배포

```yaml
Day 1-2:
  - 유닛 테스트 (Rust)
  - 통합 테스트
  - E2E 테스트 (Playwright)

Day 3-4:
  - 성능 최적화
  - 메모리 누수 체크
  - 보안 취약점 점검

Day 5:
  - 사용자 매뉴얼 작성
  - API 문서 생성
  - GitHub Release 배포
```

---

## 📅 Phase 4: 배포 후 MCP 재평가 계획 (신규 추가!)

### 배포 후 1개월 (M14: MCP 1차 재평가)
**목표**: 사용 패턴 분석 및 최적화

```yaml
Week 1-2:
  - 토큰 사용량 모니터링 및 분석
  - 사용자 피드백 수집
  - MCP 서버별 활용도 측정

Week 3-4:
  - 신규 MCP 서버 검토
    1. Slack MCP (Notification Service 연동)
    2. Redis MCP (캐싱 시스템 강화)
  - 비용 대비 효과 분석
```

**재평가 체크리스트**:
- [ ] 월간 토큰 사용량: 목표 2,500,000 토큰 이하 유지
- [ ] 평균 응답 시간: 2.3초 이하 유지
- [ ] 사용자 만족도: 4.0/5 이상
- [ ] Slack 알림 필요성 평가
- [ ] Redis 캐싱 효과 측정 (히트율 70% 이상 목표)

---

### 배포 후 3개월 (M15: MCP 2차 재평가)
**목표**: 확장 가능성 및 고급 기능 검토

```yaml
Week 1-2:
  - 코드베이스 규모 측정 (10,000+ 줄?)
  - DeepGraph TypeScript MCP 재활성화 검토
  - 아키텍처 복잡도 분석

Week 3-4:
  - 팀 규모 확인 (5+ 개발자?)
  - CircleCI MCP 재활성화 검토
  - GitHub Actions 사용량 모니터링
```

**DeepGraph 재활성화 조건**:
```yaml
조건 충족 시 활성화:
  - 코드베이스: 10,000+ 줄
  - 리팩토링 계획 수립 중
  - 아키텍처 문서 자동 생성 필요

활용 목적:
  - 대규모 코드베이스 의존성 분석
  - 리팩토링 임팩트 분석
  - 아키텍처 다이어그램 자동 생성

예상 비용 증가:
  - +5,000~20,000 토큰/호출
  - 월 사용 빈도: 4~8회
  - 추가 비용: $60~200/월
```

**CircleCI 재활성화 조건**:
```yaml
조건 충족 시 활성화:
  - 팀 규모: 10+ 개발자
  - GitHub Actions 제한 초과
  - 복잡한 CI/CD 파이프라인 필요

활용 목적:
  - 복잡한 빌드 파이프라인 관리
  - 병렬 테스트 실행
  - 고급 배포 전략 (Blue-Green, Canary)

예상 비용 증가:
  - CircleCI 구독료: $70~200/월
```

**추가 고려 MCP 서버**:
```yaml
Notion MCP:
  - 시점: 프로젝트 문서화 자동화 필요 시
  - 용도: 설계 문서 자동 업데이트
  - 비용: 토큰 증가 미미 (~1,000 토큰/호출)

Sentry MCP:
  - 시점: 프로덕션 에러 추적 필요 시
  - 용도: 자동 버그 리포트 생성
  - 비용: Sentry 구독료 $26/월

Datadog MCP:
  - 시점: 고급 모니터링 필요 시
  - 용도: 성능 메트릭 분석
  - 비용: Datadog 구독료 $15/월
```

---

### 📊 MCP 서버 재평가 의사결정 트리

```
배포 후 1개월:
├─ 토큰 사용량 > 3,000,000/월?
│  ├─ YES → MCP 최적화 강화 (캐시 TTL 연장, 복잡도 기준 조정)
│  └─ NO → 현상 유지
│
├─ 알림 기능 필요?
│  ├─ YES → Slack MCP 추가
│  └─ NO → 대기
│
└─ 캐시 히트율 < 70%?
   ├─ YES → Redis MCP 추가 검토
   └─ NO → 현상 유지

배포 후 3개월:
├─ 코드베이스 > 10,000 줄?
│  ├─ YES → DeepGraph MCP 추가
│  └─ NO → 대기
│
├─ 팀 규모 > 10명?
│  ├─ YES → CircleCI MCP 추가
│  └─ NO → GitHub Actions 유지
│
└─ 문서 자동화 필요?
   ├─ YES → Notion MCP 추가
   └─ NO → 수동 관리 유지
```

---

## 🔧 7. 핵심 구현 상세

### 7.1 Judgment Engine (하이브리드 판단)

#### Rust 구현
```rust
// src-tauri/src/services/judgment_engine.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct JudgmentInput {
    pub workflow_id: String,
    pub input_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JudgmentResult {
    pub result: bool,
    pub confidence: f64,
    pub method_used: String, // "rule" | "llm" | "hybrid"
    pub explanation: String,
}

pub struct JudgmentEngine {
    rule_engine: RuleEngine,
    llm_engine: LLMEngine,
    db: Database,
}

impl JudgmentEngine {
    pub async fn execute(&self, input: JudgmentInput) -> Result<JudgmentResult, String> {
        // 1. Rule Engine 시도
        let rule_result = self.rule_engine.evaluate(&input)?;

        if rule_result.confidence >= 0.7 {
            self.db.save_result(&rule_result).await?;
            return Ok(rule_result);
        }

        // 2. LLM 보완
        let llm_result = self.llm_engine.evaluate(&input).await?;

        // 3. 결과 결합
        let final_result = self.combine_results(rule_result, llm_result);
        self.db.save_result(&final_result).await?;

        Ok(final_result)
    }

    fn combine_results(&self, rule: JudgmentResult, llm: JudgmentResult) -> JudgmentResult {
        // 가중 평균 또는 최대값 선택
        if llm.confidence > rule.confidence {
            JudgmentResult {
                method_used: "hybrid".to_string(),
                ..llm
            }
        } else {
            rule
        }
    }
}
```

#### React 호출
```typescript
// src/hooks/useJudgment.ts

import { invoke } from '@tauri-apps/api/tauri';

export function useJudgment() {
  const executeJudgment = async (input: JudgmentInput) => {
    try {
      const result = await invoke<JudgmentResult>('execute_judgment', { input });
      return result;
    } catch (error) {
      console.error('Judgment failed:', error);
      throw error;
    }
  };

  return { executeJudgment };
}
```

### 7.2 Learning Service (자동학습)

#### Rust 구현
```rust
// src-tauri/src/services/learning_service.rs

pub struct LearningService {
    db: Database,
    faiss_index: FaissIndex,
}

impl LearningService {
    // Few-shot 샘플 검색
    pub async fn find_similar_samples(&self, input: &JudgmentInput, limit: usize) -> Vec<TrainingSample> {
        let embedding = self.generate_embedding(input).await?;
        let similar_ids = self.faiss_index.search(&embedding, limit)?;

        self.db.get_samples_by_ids(&similar_ids).await
    }

    // Rule 추출 (3개 알고리즘)
    pub async fn extract_rules(&self, workflow_id: &str) -> Vec<Rule> {
        let samples = self.db.get_workflow_samples(workflow_id).await?;

        // 1. 빈도 분석
        let freq_rules = self.frequency_analysis(&samples);

        // 2. 결정 트리
        let tree_rules = self.decision_tree_learning(&samples);

        // 3. LLM 패턴 발견
        let llm_rules = self.llm_pattern_discovery(&samples).await?;

        // 최적 Rule 선택
        vec![freq_rules, tree_rules, llm_rules]
            .into_iter()
            .max_by_key(|r| r.accuracy)
            .unwrap()
    }

    fn frequency_analysis(&self, samples: &[TrainingSample]) -> Rule {
        // 빈도 기반 패턴 추출
    }

    fn decision_tree_learning(&self, samples: &[TrainingSample]) -> Rule {
        // 결정 트리 알고리즘
    }
}
```

### 7.3 BI Service (동적 대시보드 생성)

#### TypeScript 구현
```typescript
// src/pages/BiInsights.tsx

import { invoke } from '@tauri-apps/api/tauri';

export function BiInsights() {
  const [dashboardCode, setDashboardCode] = useState<string>('');

  const generateInsight = async (userRequest: string) => {
    const result = await invoke<BiInsightResult>('generate_bi_insight', {
      request: userRequest
    });

    // LLM이 생성한 React 컴포넌트 코드
    setDashboardCode(result.componentCode);
  };

  return (
    <div>
      <ChatInput onSubmit={generateInsight} />
      <DynamicDashboard code={dashboardCode} />
    </div>
  );
}

function DynamicDashboard({ code }: { code: string }) {
  // JSX 코드를 안전하게 파싱 및 렌더링
  const Component = useMemo(() => parseJSX(code), [code]);
  return <Component />;
}
```

---

## 🧪 8. 테스트 전략

### 8.1 Rust 테스트
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_judgment_engine_rule_success() {
        let engine = JudgmentEngine::new();
        let input = JudgmentInput {
            workflow_id: "test-1".to_string(),
            input_data: json!({ "temperature": 90 }),
        };

        let result = engine.execute(input).await.unwrap();
        assert_eq!(result.method_used, "rule");
        assert!(result.confidence >= 0.7);
    }
}
```

### 8.2 React 테스트
```typescript
// src/__tests__/ChatInterface.test.tsx

import { render, screen } from '@testing-library/react';
import { ChatInterface } from '../pages/ChatInterface';

test('sends message and displays response', async () => {
  render(<ChatInterface />);

  const input = screen.getByPlaceholderText('메시지 입력...');
  await userEvent.type(input, '워크플로우 실행');

  const button = screen.getByText('전송');
  await userEvent.click(button);

  expect(await screen.findByText(/실행 완료/)).toBeInTheDocument();
});
```

---

## 📦 9. 배포 전략

### 9.1 빌드 프로세스
```bash
# 개발 빌드
pnpm tauri dev

# 프로덕션 빌드 (Windows)
pnpm tauri build --target x86_64-pc-windows-msvc

# 생성 파일:
# - judgify-desktop_2.0.0_x64.msi (인스톨러)
# - judgify-desktop.exe (Portable)
```

### 9.2 Auto Update 설정
```json
// tauri.conf.json
{
  "tauri": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/your-org/judgify-desktop/releases/latest/download/latest.json"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

### 9.3 GitHub Actions CI/CD
```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags:
      - 'v*'

jobs:
  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build Tauri App
        run: pnpm tauri build
      - name: Upload Release
        uses: softprops/action-gh-release@v1
        with:
          files: src-tauri/target/release/bundle/msi/*.msi
```

---

## 🔒 10. 보안 고려사항

### 10.1 API 키 관리
```rust
// 암호화된 설정 저장
use aes_gcm::{Aes256Gcm, Key, Nonce};

pub fn save_api_key(key: &str) -> Result<()> {
    let encrypted = encrypt_api_key(key)?;
    // Windows Credential Manager 또는 암호화된 파일
    store_encrypted_key(encrypted)
}
```

### 10.2 Rule Engine 샌드박싱
```rust
// rhai 엔진으로 안전한 실행 (eval 금지)
use rhai::Engine;

let engine = Engine::new();
engine.set_max_operations(10000); // DOS 방지
let result = engine.eval::<bool>(rule_expression)?;
```

---

## 📊 11. 성공 지표

### 11.1 기술적 지표
```yaml
성능:
  - 앱 시작 시간: < 3초
  - 판단 실행 시간: < 500ms
  - 메모리 사용량: < 400MB
  - 실행 파일 크기: < 30MB

안정성:
  - 크래시율: < 0.1%
  - API 오류율: < 1%
  - 데이터 손실: 0%

사용성:
  - 대시보드 생성 시간: < 30초
  - 워크플로우 실행 성공률: > 95%
```

### 11.2 비즈니스 지표
```yaml
사용자 만족도:
  - NPS 점수: > 40
  - 사용자 피드백 긍정률: > 80%

도입률:
  - 기업 채택률: 10+ 기업 (6개월)
  - 월간 활성 사용자: 100+
```

---

## 🚀 12. 향후 확장 계획

### 12.1 Phase 2 (3개월 후)
- macOS 버전 출시
- Linux 버전 출시
- 다국어 지원 (한국어, 영어, 일본어)

### 12.2 Phase 3 (6개월 후)
- 클라우드 동기화 (선택적)
- 팀 협업 기능
- 엔터프라이즈 관리 콘솔

---

## 📞 13. 지원 및 문서

### 13.1 문서화
- README.md: 프로젝트 개요 및 설치 가이드
- CONTRIBUTING.md: 개발 기여 가이드
- API.md: Tauri Command API 문서
- USER_MANUAL.md: 사용자 매뉴얼

### 13.2 커뮤니티
- GitHub Issues: 버그 리포트
- GitHub Discussions: Q&A
- Discord: 실시간 지원

---

**작성일**: 2025-01-16
**버전**: 1.0.0
**작성자**: Claude (AI Assistant)
**승인자**: 프로젝트 관리자
