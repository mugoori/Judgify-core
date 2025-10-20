# ✅ Judgify-core Ver2.0 개발 체크리스트

**빠른 참조용 체크리스트**

---

## 📦 Phase 1: 프로젝트 구조 (100% ✅)
- [x] Tauri 기본 설정
- [x] React + Vite 설정
- [x] TypeScript 설정
- [x] Tailwind CSS + shadcn/ui
- [x] 빌드/배포 스크립트
- [x] 문서화 (7개 가이드)

---

## 🔧 Backend (Rust) - 60% 완료

### ✅ Database Service (100%)
- [x] SQLite 연결 및 초기화
- [x] 워크플로우 CRUD
- [x] 판단 실행 기록 저장
- [x] 학습 데이터 관리 구조

### ✅ Judgment Engine (100%)
- [x] 하이브리드 판단 로직 (Rule + LLM)
- [x] Rule Engine 우선 실행
- [x] LLM 보완 로직
- [x] 최종 결과 생성

### ⚠️ Rule Engine (70%)
- [x] rhai 스크립트 엔진 통합
- [x] 기본 표현식 평가 (temperature > 90)
- [ ] **복잡한 조건 처리 (중첩 조건, 배열) ← Week 2**
- [ ] **에러 처리 고도화 ← Week 2**

### ⚠️ LLM Engine (60%)
- [x] OpenAI API 통합
- [x] 기본 판단 요청/응답
- [ ] **Few-shot 학습 통합 ← Week 2**
- [ ] **응답 파싱 개선 (JSON 구조화) ← Week 2**
- [ ] **프롬프트 템플릿 고도화 ← Week 2**

### ❌ Learning Service (30%)
- [x] 피드백 저장 구조
- [ ] **훈련 샘플 자동 생성 ← Week 3**
- [ ] **Few-shot 샘플 검색 (유사도) ← Week 3**
- [ ] **알고리즘 1: 빈도 분석 Rule 추출 ← Week 3**
- [ ] **알고리즘 2: 결정 트리 학습 ← Week 4**
- [ ] **알고리즘 3: LLM 패턴 발견 ← Week 4**

### ❌ BI Service (25%)
- [x] 기본 API 구조
- [ ] **사용자 요청 분석 (LLM) ← Week 5**
- [ ] **Judgment Service 연동 ← Week 5**
- [ ] **React 컴포넌트 자동 생성 ← Week 5**
- [ ] **비즈니스 인사이트 생성 ← Week 6**

---

## 🎨 Frontend (React) - 95% 완료

### ✅ Chat Interface (100%)
- [x] 메시지 입력/표시
- [x] 대화 히스토리 관리
- [x] Tauri IPC 통신
- [x] 의도 분류 표시

### ✅ Workflow Builder (100%)
- [x] React Flow 드래그앤드롭
- [x] 워크플로우 저장/로드
- [x] Rule 표현식 입력
- [x] 노드 연결 관리

### ✅ Dashboard (100%)
- [x] KPI 카드 (총 판단, 성공률, 평균 신뢰도)
- [x] 판단 방법 분포 (Pie Chart)
- [x] 신뢰도 트렌드 (Line Chart)
- [x] 최근 판단 기록 (Table)

### ✅ BI Insights (100%)
- [x] 자연어 요청 입력
- [x] 자동 생성된 대시보드 표시
- [x] 인사이트 및 권장사항 표시
- [x] Tauri IPC 통신

### ✅ Tauri API Layer (100%)
- [x] Judgment API (execute, history)
- [x] Learning API (feedback, samples, extract_rules)
- [x] BI API (generate_insight)
- [x] Chat API (send_message)
- [x] Workflow API (save, load, list, delete)
- [x] System API (health, version)

### ⚠️ Settings Page (80%)
- [x] 기본 설정 UI
- [ ] **MCP 서버 상태 실시간 표시 ← Week 6**
- [ ] **OpenAI API Key 검증 ← Week 6**

---

## 📅 주차별 우선순위 작업

### Week 2 (01-20 ~ 01-24) - Judgment Engine 강화
#### 🔴 Critical
- [ ] Rule Engine 복잡한 조건 처리 (3일)
- [ ] LLM Engine Few-shot 통합 (2일)

#### 🟡 Important
- [ ] Judgment History 개선 (1일)

**목표**: 판단 정확도 70% → 85%

---

### Week 3-4 (01-27 ~ 02-07) - Learning Service 완성
#### 🔴 Critical
- [ ] 훈련 샘플 자동 생성 (2일)
- [ ] 알고리즘 1: 빈도 분석 Rule 추출 (3일)
- [ ] 알고리즘 2: 결정 트리 학습 (3일)
- [ ] 알고리즘 3: LLM 패턴 발견 (2일)

#### 🟡 Important
- [ ] Few-shot 샘플 검색 (벡터 유사도) (2일)

**목표**: 자동 Rule 추출 성공률 60% 이상

---

### Week 5-6 (02-10 ~ 02-21) - BI Service + Chat 고도화
#### 🔴 Critical
- [ ] BI Service - 사용자 요청 분석 (LLM) (3일)
- [ ] BI Service - Judgment Service 연동 (2일)
- [ ] BI Service - React 컴포넌트 자동 생성 (3일)

#### 🟡 Important
- [ ] Chat Interface - 의도 분류 고도화 (2일)

**목표**: "지난 주 불량률 분석해줘" → 30초 내 자동 대시보드

---

### Week 7 (02-24 ~ 02-28) - Visual Workflow 고도화
#### 🟡 Important
- [ ] Workflow 노드 타입 확장 (2일)
- [ ] Workflow 실행 엔진 (2일)

#### 🟢 Enhancement
- [ ] Workflow 템플릿 라이브러리 (1일)

**목표**: 복잡한 워크플로우 (5+ 노드) 정상 실행

---

### Week 8 (03-03 ~ 03-07) - 테스트 + 프로덕션 빌드
#### 🔴 Critical
- [ ] 통합 테스트 (E2E, 유닛, 성능) (2일)
- [ ] 프로덕션 빌드 최적화 (2일)

#### 🟡 Important
- [ ] 문서화 완성 (사용자 가이드, API 문서) (1일)

**목표**: Windows 설치 프로그램 배포 준비

---

## 🎯 진행도 추적

| 항목 | 현재 | Week 2 | Week 3-4 | Week 5-6 | Week 7 | Week 8 |
|------|------|--------|----------|----------|--------|--------|
| **전체** | 45% | 55% | 75% | 90% | 95% | 100% |
| Backend | 60% | 70% | 85% | 90% | 92% | 95% |
| Frontend | 95% | 95% | 95% | 98% | 99% | 100% |
| 테스트 | 10% | 15% | 30% | 50% | 70% | 100% |
| 문서 | 80% | 85% | 88% | 92% | 95% | 100% |

---

## 🚀 즉시 시작 가능한 작업 (Week 2 Day 1)

### 1. Rule Engine 고도화
**파일**: `src-tauri/src/services/rule_engine.rs`
```rust
// TODO: rhai 엔진에 커스텀 함수 등록
engine.register_fn("avg", |arr: Vec<f64>| -> f64 { /* ... */ });
engine.register_fn("sum", |arr: Vec<f64>| -> f64 { /* ... */ });
engine.register_fn("contains", |arr: Vec<String>, val: String| -> bool { /* ... */ });

// TODO: 중첩 조건 테스트
// (temperature > 90 || pressure > 120) && status == "active"
```

### 2. Few-shot 프롬프트 템플릿
**파일**: `src-tauri/src/services/llm_engine.rs`
```rust
// TODO: Few-shot 프롬프트 템플릿 작성
const FEWSHOT_TEMPLATE: &str = r#"
다음 유사한 사례들을 참고하여 판단하세요:

{few_shot_samples}

현재 입력:
{current_input}

출력 형식 (JSON):
{
  "result": true/false,
  "confidence": 0.95,
  "explanation": "판단 근거"
}
"#;
```

### 3. Learning Service 테이블 추가
**파일**: `src-tauri/src/services/database.rs`
```sql
-- TODO: training_samples 테이블 추가
CREATE TABLE IF NOT EXISTS training_samples (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    input_data TEXT NOT NULL,
    expected_output TEXT NOT NULL,
    confidence REAL,
    embedding BLOB,  -- 임베딩 저장 (향후 pgvector)
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
```

---

## 📝 참고 링크

- **상세 개발 계획**: [IMPLEMENTATION-STATUS.md](IMPLEMENTATION-STATUS.md)
- **프로젝트 상태**: [PROJECT-STATUS.md](PROJECT-STATUS.md)
- **실행 가이드**: [RUN-LOCALLY.md](RUN-LOCALLY.md)
- **개발 가이드**: [CLAUDE.md](CLAUDE.md)

---

**업데이트**: 2025-01-17
