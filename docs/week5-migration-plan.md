# Week 5 Visual Workflow Builder - Migration Plan

**생성일**: 2025-11-06
**브랜치**: `feature/week5-visual-workflow-builder`
**백업 브랜치**: `backup/workflow-v1-2025-11-06`

---

## 📋 목적

Week 5 (Visual Workflow Builder) 작업을 안전하게 진행하기 위한 마이그레이션 계획 문서입니다. 현재 WorkflowBuilder.tsx (85% 완성)에서 n8n 스타일 고급 Visual Builder로 업그레이드하는 과정의 충돌 지점과 해결 전략을 정의합니다.

---

## 🎯 마이그레이션 목표

### 현재 상태 (v1)
- **노드 타입**: 4가지 (input, decision, action, output)
- **AI 생성**: 패턴 기반 간단한 생성 (`workflow-generator.ts`)
- **Validation**: 기본 JSON 구조 검증
- **시뮬레이션**: 단계별 하이라이트 (기본 구현)
- **React Flow**: v11.x 기본 설정

### Week 5 목표 (v2)
- **노드 타입**: 7가지로 확장 (data_input, rule_judgment, llm_judgment, action_execution, notification, data_aggregation, output)
- **AI 생성**: LLM 기반 고급 워크플로우 생성 (Claude API 연동)
- **Validation**: AST 기반 Rule Expression 검증 (Rhai 엔진 심화)
- **시뮬레이션**: 실시간 디버깅 + 변수 추적
- **React Flow**: 성능 최적화 유지 (1,000+ 노드 지원)

---

## 📂 변경 예상 파일 목록

### 1️⃣ Frontend (TypeScript/React)

| 파일 | 현재 줄 수 | 변경 유형 | 충돌 확률 | 영향도 |
|------|----------|----------|-----------|--------|
| **src/pages/WorkflowBuilder.tsx** | ~1,000 | 구조 변경 | 🔴 70% | High |
| **src/components/workflow/CustomNode.tsx** | ~150 | 대규모 리팩토링 | 🔴 80% | High |
| **src/lib/workflow-generator.ts** | ~200 | 로직 변경 | 🟡 60% | Medium |
| **src/components/workflow/NodeEditPanel.tsx** | ~300 | 필드 확장 | 🟡 50% | Medium |
| **src/components/workflow/SimulationPanel.tsx** | ~250 | 기능 확장 | 🟡 40% | Medium |

### 2️⃣ Backend (Rust/Tauri)

| 파일 | 현재 줄 수 | 변경 유형 | 충돌 확률 | 영향도 |
|------|----------|----------|-----------|--------|
| **src-tauri/src/services/workflow_service.rs** | 119 | 메서드 추가 | 🟢 20% | Low |
| **src-tauri/src/commands/workflow.rs** | 182 | 새 Command 추가 | 🟢 15% | Low |
| **src-tauri/src/models/workflow.rs** | ~80 | 필드 확장 | 🟡 30% | Medium |

### 3️⃣ 문서

| 파일 | 변경 유형 |
|------|----------|
| **docs/PERFORMANCE_OPTIMIZATION.md** | 업데이트 (새 최적화 기법 추가) |
| **docs/development/plan.md** | Week 5 진행 상황 업데이트 |

---

## 🔴 충돌 지점 및 해결 전략

### **1. 노드 타입 재정의 (High Risk - 80%)**

#### 🔹 문제점
- **현재**: Union Type 사용 (`'input' | 'decision' | 'action' | 'output'`)
- **목표**: 7가지 타입으로 확장 + Enum 전환 (타입 안전성 향상)
- **영향**: `CustomNode.tsx`의 조건부 렌더링 로직 전면 수정

#### 🔹 해결 전략

**Step 1**: 새 타입 정의 (기존 타입 유지, 하위 호환)
```typescript
// src/types/workflow.ts (신규 생성)
export enum NodeType {
  // 기존 타입 (v1 호환)
  INPUT = 'input',
  DECISION = 'decision',
  ACTION = 'action',
  OUTPUT = 'output',

  // 신규 타입 (Week 5)
  DATA_INPUT = 'data_input',
  RULE_JUDGMENT = 'rule_judgment',
  LLM_JUDGMENT = 'llm_judgment',
  ACTION_EXECUTION = 'action_execution',
  NOTIFICATION = 'notification',
  DATA_AGGREGATION = 'data_aggregation',
}

// 하위 호환성 타입 가드
export const isLegacyNodeType = (type: string): boolean => {
  return ['input', 'decision', 'action', 'output'].includes(type);
};
```

**Step 2**: `CustomNode.tsx` 리팩토링 (4-6시간)
- 조건문 → Switch Statement 전환
- 7가지 노드 타입별 렌더링 로직 분리
- 아이콘, 스타일, 동작 정의

**Step 3**: 기존 워크플로우 마이그레이션 테스트
- v1 노드 타입 → v2 Enum 자동 변환 함수 작성
- 샘플 워크플로우 로드 테스트 (최소 5개)

---

### **2. React Flow 구조 변경 (Medium-High Risk - 70%)**

#### 🔹 문제점
- **현재**: 12개 props 최적화 완료 (`WorkflowBuilder.tsx:500-600`)
- **목표**: 새 노드 타입 추가 → `nodeTypes` prop 재설정 필요
- **영향**: 성능 최적화 재적용 가능성

#### 🔹 해결 전략

**Step 1**: `nodeTypes` 객체 확장 (기존 유지)
```typescript
const nodeTypes = useMemo(
  () => ({
    // 기존 타입 (v1 호환)
    input: CustomNode,
    decision: CustomNode,
    action: CustomNode,
    output: CustomNode,

    // 신규 타입 (Week 5)
    data_input: CustomNode,
    rule_judgment: CustomNode,
    llm_judgment: CustomNode,
    action_execution: CustomNode,
    notification: CustomNode,
    data_aggregation: CustomNode,
  }),
  []
);
```

**Step 2**: 성능 벤치마크 실행 (Before/After)
- 1,000+ 노드 시나리오 렌더링 시간 측정
- FPS 60 유지 여부 확인
- 필요시 `React.memo`, `useMemo`, `useCallback` 재적용

---

### **3. AI 생성 로직 변경 (Medium Risk - 60%)**

#### 🔹 문제점
- **현재**: 패턴 기반 생성 (`workflow-generator.ts:testScenarios`)
- **목표**: **Claude API (Anthropic) 연동** LLM 기반 생성
- **영향**: 기존 샘플 시나리오 호환성 문제

#### 🔹 해결 전략

**하이브리드 접근** (패턴 + LLM)
```typescript
// src/lib/workflow-generator.ts
export const generateWorkflowFromDescription = async (
  description: string,
  mode: 'pattern' | 'llm' | 'hybrid' = 'hybrid'
): Promise<WorkflowDefinition> => {
  // Pattern 기반 시도 (빠름, 기존 샘플 유지)
  if (mode === 'pattern' || mode === 'hybrid') {
    const patternResult = tryPatternBasedGeneration(description);
    if (patternResult) return patternResult;
  }

  // Claude API 기반 폴백 (고급, 새로운 시나리오)
  if (mode === 'llm' || mode === 'hybrid') {
    return await generateWithClaude(description);
  }

  throw new Error('No generation strategy succeeded');
};

// Claude API 연동 함수 (신규)
async function generateWithClaude(description: string): Promise<WorkflowDefinition> {
  const apiKey = localStorage.getItem('claude_api_key');
  if (!apiKey) {
    throw new Error('Claude API key not found. Please set it in Settings.');
  }

  // Anthropic Messages API 호출
  const response = await fetch('https://api.anthropic.com/v1/messages', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'x-api-key': apiKey,
      'anthropic-version': '2023-06-01',
    },
    body: JSON.stringify({
      model: 'claude-3-5-sonnet-20241022', // 최신 Sonnet 4.5 모델
      max_tokens: 4096,
      messages: [
        {
          role: 'user',
          content: `Generate a workflow definition in JSON format based on this description: "${description}"`,
        },
      ],
    }),
  });

  const data = await response.json();
  return parseClaudeResponse(data.content[0].text);
}
```

**Claude API 사용 이유**:
- ✅ Settings 페이지에서 이미 API 키 관리 (`localStorage.getItem('claude_api_key')`)
- ✅ 워크플로우 생성에 최적화된 추론 능력 (Sonnet 4.5)
- ✅ 한국어 프롬프트 지원 우수
- ✅ JSON 구조 생성 정확도 높음

**테스트 전략**:
- 기존 `testScenarios` 5개 → Pattern 모드 테스트 (통과 필수)
- 새로운 시나리오 3개 → LLM 모드 테스트
- 하이브리드 모드 통합 테스트

---

### **4. Validation 강화 (Medium Risk - 50%)**

#### 🔹 문제점
- **현재**: 기본 JSON 구조 검증 (`workflow_service.rs:validate_workflow`)
- **목표**: AST 기반 Rule Expression 검증 (Rhai 엔진 심화)
- **영향**: 기존 워크플로우 재검증 필요

#### 🔹 해결 전략

**옵션 검증** (기존 검증 유지 + AST 추가)
```rust
// src-tauri/src/services/workflow_service.rs
pub fn validate_workflow(&self, workflow: &Workflow) -> Result<ValidationResult> {
    // Step 1: 기본 검증 (v1 호환)
    self.validate_basic_structure(workflow)?;

    // Step 2: AST 검증 (옵션, Week 5 신규)
    if workflow.use_ast_validation.unwrap_or(false) {
        self.validate_rule_expressions_with_ast(workflow)?;
    }

    Ok(ValidationResult::success())
}
```

**마이그레이션 플래그**:
- 기존 워크플로우: `use_ast_validation: false` (기본 검증만)
- 신규 워크플로우: `use_ast_validation: true` (AST 검증 활성화)

---

### **5. 시뮬레이션 패널 확장 (Low-Medium Risk - 40%)**

#### 🔹 문제점
- **현재**: 단계별 하이라이트 (`SimulationPanel.tsx`)
- **목표**: 변수 추적 + 실시간 디버깅
- **영향**: UI 재설계 필요

#### 🔹 해결 전략

**점진적 기능 추가** (기존 UI 유지)
- Phase 1: 변수 추적 패널 추가 (하단 드로어)
- Phase 2: 브레이크포인트 설정 기능
- Phase 3: Step-by-step 디버깅

---

### **6. 백엔드 Service Layer 확장 (Low Risk - 20%)**

#### 🔹 문제점
- **현재**: CRUD + Soft Delete (`workflow_service.rs`)
- **목표**: 버전 관리 + A/B 테스트 기능
- **영향**: 새 메서드 추가만 필요 (기존 로직 보존)

#### 🔹 해결 전략

**새 메서드 추가** (기존 메서드 변경 없음)
```rust
impl WorkflowService {
    // 신규 메서드
    pub async fn create_workflow_version(&self, workflow_id: Uuid, version_data: VersionData) -> Result<WorkflowVersion>;
    pub async fn list_workflow_versions(&self, workflow_id: Uuid) -> Result<Vec<WorkflowVersion>>;
    pub async fn rollback_to_version(&self, workflow_id: Uuid, version_number: u32) -> Result<Workflow>;
    pub async fn enable_ab_test(&self, workflow_id: Uuid, test_config: ABTestConfig) -> Result<()>;
}
```

---

## 🔄 롤백 절차

### **조건**: 다음 상황 발생시 즉시 롤백
1. ❌ React Flow 라이브러리 버전 업그레이드 실패 (호환성 문제)
2. ❌ 노드 타입 리팩토링 후 기존 워크플로우 로드 실패 (마이그레이션 불가)
3. ❌ 성능 벤치마크 하락 (1,000 노드 → 500 노드 이하)

### **롤백 명령어**
```bash
# 백업 브랜치로 복원
git checkout backup/workflow-v1-2025-11-06
git checkout -b feature/workflow-stable
git push origin feature/workflow-stable --force

# Week 5 작업 브랜치 보존 (나중 분석용)
git tag week5-failed-attempt-2025-11-06
git push origin week5-failed-attempt-2025-11-06
```

### **롤백 후 조치**
1. 실패 원인 분석 (로그, 에러 메시지 수집)
2. 이슈 생성 (GitHub): `[Week 5] Migration Failed - {원인}`
3. 대안 전략 수립 회의

---

## 📊 예상 영향도 종합

| 영역 | 충돌 확률 | 예상 재작업 시간 | 우선순위 |
|------|----------|-----------------|---------|
| **노드 타입 재정의** | 🔴 80% | 4-6시간 | 1 |
| **React Flow 구조** | 🔴 70% | 2-3시간 | 2 |
| **AI 생성 로직** | 🟡 60% | 1-2시간 | 3 |
| **Validation 강화** | 🟡 50% | 1-2시간 | 4 |
| **시뮬레이션 패널** | 🟡 40% | 2-3시간 | 5 |
| **백엔드 Service** | 🟢 20% | 1-2시간 | 6 |
| **총합** | 🟡 53% | **11-18시간** | - |

---

## 🎯 작업 우선순위 (Week 5 Day 1-5)

### **Day 1-2**: 노드 타입 재정의 + CustomNode 리팩토링
- [ ] `src/types/workflow.ts` 생성 (NodeType Enum)
- [ ] `CustomNode.tsx` 리팩토링 (7가지 노드 지원)
- [ ] 하위 호환성 테스트 (기존 워크플로우 로드)
- [ ] `WorkflowBuilder.test.tsx` 작성 (20개 테스트)

### **Day 3-4**: AI 생성 + Validation
- [ ] Claude API (Anthropic) 연동 (`generateWithClaude`)
- [ ] 하이브리드 생성 로직 구현 (Pattern + Claude)
- [ ] AST 기반 Validation 추가 (옵션)
- [ ] 통합 테스트 (Pattern + LLM 모드)

### **Day 5**: 시뮬레이션 + 성능 테스트
- [ ] 변수 추적 패널 UI 추가
- [ ] 성능 벤치마크 실행 (Before/After)
- [ ] 문서 업데이트 (`PERFORMANCE_OPTIMIZATION.md`)
- [ ] 최종 통합 테스트 (E2E 시나리오 3개)

---

## 📝 추가 고려사항

### **1. 테스트 커버리지 목표**
- **현재**: 0% (WorkflowBuilder.test.tsx 미작성)
- **Week 5 목표**: 85%
- **핵심 테스트**:
  - 7가지 노드 타입별 렌더링 (7 tests)
  - AI 생성 (Pattern + LLM, 5 tests)
  - Validation (기본 + AST, 4 tests)
  - 시뮬레이션 (변수 추적, 4 tests)

### **2. 성능 목표 유지**
- **렌더링 시간**: <100ms (1,000 노드)
- **FPS**: 60 유지 (드래그앤드롭 중)
- **메모리 사용량**: <200MB (대형 워크플로우)

### **3. 문서화 우선순위**
- [ ] 노드 타입별 사용 가이드 (`docs/workflow-node-types.md`)
- [ ] AI 생성 API 사용법 (`docs/workflow-ai-generation.md`)
- [ ] 마이그레이션 가이드 (v1 → v2, 이 문서)

---

## 🎉 완료 기준

Week 5 작업을 완료했다고 판단할 수 있는 조건:

- ✅ 7가지 노드 타입 모두 동작 확인
- ✅ LLM 기반 AI 생성 성공률 95% 이상
- ✅ 기존 워크플로우 (v1) 로드 성공률 100%
- ✅ 성능 벤치마크 유지 (1,000+ 노드 지원)
- ✅ 테스트 커버리지 85% 이상
- ✅ 문서화 3개 완성

---

**버전**: 1.0
**마지막 업데이트**: 2025-11-06
**작성자**: Claude Code (AI Assistant)
