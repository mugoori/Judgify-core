# 📊 Judgify-core Ver2.0 Final - 구현 상태 및 개발 계획

**분석 기준일**: 2025-01-17
**전체 진행도**: 약 **45%** 완료

---

## 🎯 1. 현재 구현 상태 요약

### 1.1 전체 구조 (✅ 100% 완료)
```
프로젝트 구조: ✅ 완료
  ├─ Tauri 기본 설정: ✅ 완료
  ├─ React + Vite 설정: ✅ 완료
  ├─ TypeScript 설정: ✅ 완료
  ├─ Tailwind CSS + shadcn/ui: ✅ 완료
  └─ 빌드/배포 스크립트: ✅ 완료

문서화: ✅ 완료 (7개 가이드 문서)
  ├─ CLAUDE.md: ✅ 개발 가이드 완료
  ├─ README-SETUP.md: ✅ 설치 가이드 완료
  ├─ RUN-LOCALLY.md: ✅ 실행 가이드 완료
  ├─ QUICKSTART.md: ✅ 빠른 시작 완료
  ├─ PROJECT-STATUS.md: ✅ 프로젝트 상태 완료
  ├─ EXECUTE-NOW.ps1: ✅ 자동 실행 스크립트 완료
  └─ EXECUTE-INSTRUCTIONS.md: ✅ 실행 설명서 완료
```

### 1.2 백엔드 (Rust) - 약 60% 완료
```
총 코드량: ~1,520 줄

✅ 완료된 서비스 (100%):
  ├─ Database Service (287줄)
  │   ├─ SQLite 연결 및 초기화
  │   ├─ 워크플로우 CRUD
  │   ├─ 판단 실행 기록 저장
  │   └─ 학습 데이터 관리
  │
  └─ Judgment Engine (124줄)
      ├─ 하이브리드 판단 로직 (Rule + LLM)
      ├─ Rule Engine 우선 실행
      ├─ LLM 보완 로직
      └─ 최종 결과 생성

⚠️ 기본 구현 완료 (60-70%):
  ├─ Rule Engine (75줄)
  │   ├─ ✅ rhai 스크립트 엔진 통합
  │   ├─ ✅ 기본 표현식 평가 (temperature > 90)
  │   ├─ ❌ 복잡한 조건 처리 (중첩 조건, 배열 등)
  │   └─ ❌ 에러 처리 고도화
  │
  └─ LLM Engine (97줄)
      ├─ ✅ OpenAI API 통합
      ├─ ✅ 기본 판단 요청/응답
      ├─ ❌ Few-shot 학습 통합
      ├─ ❌ 응답 파싱 개선 (JSON 구조화)
      └─ ❌ 프롬프트 템플릿 고도화

❌ 초기 단계 (20-40%):
  ├─ Learning Service (64줄) - 30% 완료
  │   ├─ ✅ 피드백 저장 구조
  │   ├─ ❌ 훈련 샘플 자동 생성
  │   ├─ ❌ Few-shot 샘플 검색 (유사도 기반)
  │   ├─ ❌ 알고리즘 1: 빈도 분석 Rule 추출
  │   ├─ ❌ 알고리즘 2: 결정 트리 학습
  │   └─ ❌ 알고리즘 3: LLM 패턴 발견
  │
  └─ BI Service (40줄) - 25% 완료
      ├─ ✅ 기본 API 구조
      ├─ ❌ 사용자 요청 분석 (LLM)
      ├─ ❌ Judgment Service 연동
      ├─ ❌ React 컴포넌트 자동 생성
      └─ ❌ 비즈니스 인사이트 생성
```

### 1.3 프론트엔드 (React) - 약 95% 완료
```
총 코드량: ~1,509 줄

✅ 완전히 완료된 페이지 (100%):
  ├─ Chat Interface (143줄)
  │   ├─ 메시지 입력/표시
  │   ├─ 대화 히스토리 관리
  │   ├─ Tauri IPC 통신
  │   └─ 의도 분류 표시
  │
  ├─ Workflow Builder (236줄)
  │   ├─ React Flow 드래그앤드롭
  │   ├─ 워크플로우 저장/로드
  │   ├─ Rule 표현식 입력
  │   └─ 노드 연결 관리
  │
  ├─ Dashboard (187줄)
  │   ├─ KPI 카드 (총 판단, 성공률, 평균 신뢰도)
  │   ├─ 판단 방법 분포 (Pie Chart)
  │   ├─ 신뢰도 트렌드 (Line Chart)
  │   └─ 최근 판단 기록 (Table)
  │
  └─ BI Insights (189줄)
      ├─ 자연어 요청 입력
      ├─ 자동 생성된 대시보드 표시
      ├─ 인사이트 및 권장사항 표시
      └─ Tauri IPC 통신

✅ Tauri API 레이어 (140줄) - 100% 완료
  ├─ Judgment API (execute, history)
  ├─ Learning API (feedback, samples, extract_rules)
  ├─ BI API (generate_insight)
  ├─ Chat API (send_message)
  ├─ Workflow API (save, load, list, delete)
  └─ System API (health, version)

⚠️ 개선 필요:
  └─ Settings 페이지
      ├─ ✅ 기본 설정 UI
      ├─ ❌ MCP 서버 상태 실시간 표시
      └─ ❌ OpenAI API Key 검증
```

### 1.4 마이크로서비스 설계 문서 - 약 80% 완료
```
docs/services/ (5개 문서)
  ├─ ✅ judgment_engine.md (100% - 상세 설계 완료)
  ├─ ✅ data_visualization_service.md (100% - 단순 대시보드 설계)
  ├─ ⚠️ bi_service.md (80% - MCP 컴포넌트 조립 설계)
  ├─ ⚠️ chat_interface_service.md (80% - 통합 AI 채팅 설계)
  └─ ⚠️ workflow_editor.md (70% - Visual Builder 설계)

docs/architecture/
  ├─ ✅ system_overview.md (100%)
  └─ ✅ database_design.md (100%)
```

---

## 🚀 2. 향후 개발 계획 (8주 로드맵)

### 📅 Phase 1: Week 2 (2025-01-20 ~ 01-24) - Judgment Engine 강화
**목표**: 하이브리드 판단 엔진 고도화

#### 우선순위 🔴 Critical
1. **Rule Engine 고도화** (3일)
   ```rust
   // 현재 (기본 조건만)
   temperature > 90 && vibration < 50

   // 목표 (복잡한 조건)
   (temperature > 90 || pressure > 120) &&
   (vibration < 50 && status in ["active", "running"]) &&
   data.history.avg() > threshold
   ```
   - [ ] 중첩 조건 처리 (AND, OR, NOT)
   - [ ] 배열/객체 접근 (data.sensors[0].value)
   - [ ] 함수 지원 (avg, sum, contains 등)
   - [ ] 에러 처리 고도화

2. **LLM Engine Few-shot 통합** (2일)
   ```rust
   // 목표: 유사한 10-20개 예시 자동 검색 및 프롬프트 포함
   pub async fn evaluate_with_fewshot(&self, input: &JudgmentInput) -> Result<JudgmentResult> {
       // 1. 유사 샘플 검색 (벡터 유사도)
       let similar_samples = self.learning_service
           .get_similar_samples(&input.data, 20).await?;

       // 2. Few-shot 프롬프트 구성
       let prompt = self.build_fewshot_prompt(input, similar_samples);

       // 3. LLM 호출
       let response = self.openai_client.chat_completion(prompt).await?;

       // 4. 구조화된 JSON 응답 파싱
       let result = serde_json::from_str::<JudgmentResult>(&response)?;
       Ok(result)
   }
   ```
   - [ ] Learning Service 연동
   - [ ] Few-shot 프롬프트 템플릿 작성
   - [ ] JSON 구조화 응답 파싱
   - [ ] 에러 처리 및 Fallback

#### 우선순위 🟡 Important
3. **Judgment History 개선** (1일)
   - [ ] 실행 시간 측정 (execution_time_ms)
   - [ ] 판단 근거 저장 (explanation)
   - [ ] 신뢰도 점수 계산 알고리즘

**Week 2 예상 결과**:
- ✅ 복잡한 Rule 표현식 처리 가능
- ✅ LLM 판단에 Few-shot 학습 자동 적용
- ✅ 판단 정확도 70% → 85% 향상 예상

---

### 📅 Phase 2: Week 3-4 (01-27 ~ 02-07) - Learning Service 완성
**목표**: 자동학습 시스템 (ML 대체) 완전 구현

#### 우선순위 🔴 Critical
1. **훈련 샘플 자동 생성** (2일)
   ```rust
   // 목표: 긍정 피드백 받은 판단 → 훈련 샘플 변환
   pub fn create_training_sample(&self, judgment_id: String) -> Result<TrainingSample> {
       let judgment = self.db.get_judgment(&judgment_id)?;

       Ok(TrainingSample {
           id: Uuid::new_v4().to_string(),
           workflow_id: judgment.workflow_id,
           input_data: judgment.input_data,
           expected_output: judgment.final_result,
           confidence: judgment.confidence_score,
           created_at: Utc::now(),
           embedding: self.create_embedding(&judgment.input_data).await?,
       })
   }
   ```
   - [ ] Judgment → TrainingSample 변환 로직
   - [ ] 임베딩 생성 (OpenAI Embeddings API)
   - [ ] 데이터베이스 저장

2. **알고리즘 1: 빈도 분석 Rule 추출** (3일)
   ```rust
   // 목표: 패턴 빈도 분석으로 Rule 자동 생성
   pub fn frequency_analysis(&self, workflow_id: String) -> Result<Vec<String>> {
       let samples = self.db.get_training_samples(&workflow_id, 1000)?;

       // 1. 조건별 빈도 계산
       let mut condition_freq = HashMap::new();
       for sample in samples {
           for (key, value) in sample.input_data.iter() {
               let condition = format!("{} > {}", key, value);
               *condition_freq.entry(condition).or_insert(0) += 1;
           }
       }

       // 2. 빈도 80% 이상 → Rule 추출
       let rules = condition_freq.iter()
           .filter(|(_, &freq)| freq as f32 / samples.len() as f32 > 0.8)
           .map(|(cond, _)| cond.clone())
           .collect();

       Ok(rules)
   }
   ```
   - [ ] 조건 패턴 추출 알고리즘
   - [ ] 빈도 계산 및 임계값 설정
   - [ ] Rule 문자열 생성

3. **알고리즘 2: 결정 트리 학습** (3일)
   ```rust
   // 목표: sklearn 스타일 결정 트리로 Rule 생성
   // Rust에서는 linfa 라이브러리 사용
   use linfa::prelude::*;
   use linfa_trees::DecisionTree;

   pub fn decision_tree_learning(&self, workflow_id: String) -> Result<Vec<String>> {
       let samples = self.db.get_training_samples(&workflow_id, 500)?;

       // 1. 데이터 준비 (특징 행렬 + 레이블)
       let (features, labels) = self.prepare_training_data(samples)?;

       // 2. 결정 트리 학습
       let tree = DecisionTree::params()
           .max_depth(5)
           .fit(&DatasetBase::new(features, labels))?;

       // 3. 트리 → Rule 변환
       let rules = self.tree_to_rules(&tree)?;

       Ok(rules)
   }
   ```
   - [ ] linfa 라이브러리 통합
   - [ ] 훈련 데이터 준비 (특징 추출)
   - [ ] 결정 트리 학습 및 Rule 변환

4. **알고리즘 3: LLM 패턴 발견** (2일)
   ```rust
   // 목표: LLM으로 패턴 분석 및 Rule 추천
   pub async fn llm_pattern_discovery(&self, workflow_id: String) -> Result<Vec<String>> {
       let samples = self.db.get_training_samples(&workflow_id, 50)?;

       let prompt = format!(
           "다음 판단 샘플들을 분석하여 공통 패턴을 찾아 Rule을 추출하세요:\n\
            샘플:\n{}\n\
            Rule 형식: temperature > 90 && vibration < 50",
           serde_json::to_string_pretty(&samples)?
       );

       let response = self.openai_client
           .chat_completion(&prompt)
           .await?;

       // JSON 파싱 → Vec<String>
       let rules = serde_json::from_str(&response)?;
       Ok(rules)
   }
   ```
   - [ ] 샘플 분석 프롬프트 설계
   - [ ] LLM 호출 및 Rule 추출
   - [ ] 응답 파싱 및 검증

#### 우선순위 🟡 Important
5. **Few-shot 샘플 검색 (벡터 유사도)** (2일)
   ```rust
   // 목표: pgvector 스타일 유사도 검색 (SQLite에서는 수동 계산)
   pub async fn get_similar_samples(
       &self,
       input_data: &Value,
       limit: usize
   ) -> Result<Vec<TrainingSample>> {
       // 1. 입력 데이터 임베딩 생성
       let query_embedding = self.create_embedding(input_data).await?;

       // 2. 모든 훈련 샘플 로드
       let all_samples = self.db.get_all_training_samples()?;

       // 3. 코사인 유사도 계산
       let mut similarities: Vec<(TrainingSample, f32)> = all_samples
           .into_iter()
           .map(|sample| {
               let sim = cosine_similarity(&query_embedding, &sample.embedding);
               (sample, sim)
           })
           .collect();

       // 4. 유사도 순 정렬 후 상위 limit개 반환
       similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
       Ok(similarities.into_iter().take(limit).map(|(s, _)| s).collect())
   }
   ```
   - [ ] 임베딩 생성 (OpenAI API)
   - [ ] 코사인 유사도 계산 함수
   - [ ] 상위 N개 샘플 반환

6. **MCP 조건부 활성화 시스템** (2일) 🔥 **신규!**
   ```rust
   // 목표: 판단 복잡도에 따라 필요한 MCP만 선택적 활성화
   pub struct AdaptiveMCPSelector {
       mcp_costs: HashMap<String, i32>,      // 토큰 비용
       mcp_benefits: HashMap<String, f32>,   // 정확도 향상
   }

   impl AdaptiveMCPSelector {
       pub fn analyze_complexity(&self, input_data: &Value, workflow: &Workflow) -> Complexity {
           // 규칙 1: Rule이 명확하게 정의되어 있으면 simple
           if workflow.rule.is_some() && self.is_deterministic_rule(&workflow.rule) {
               return Complexity::Simple;
           }

           // 규칙 2: 입력 필드가 5개 이하 + Rule 있으면 medium
           if input_data.as_object().unwrap().len() <= 5 && workflow.rule.is_some() {
               return Complexity::Medium;
           }

           // 규칙 3: 자연어 입력이 포함되면 complex
           if self.has_natural_language_input(input_data) {
               return Complexity::Complex;
           }

           Complexity::Medium
       }

       pub async fn execute_with_adaptive_mcp(
           &self,
           input_data: &Value,
           workflow: &Workflow
       ) -> Result<JudgmentResult> {
           let complexity = self.analyze_complexity(input_data, workflow);

           match complexity {
               Complexity::Simple => {
                   // MCP 사용 안 함 (Rule Engine만)
                   self.rule_only_judgment(workflow, input_data).await
               }
               Complexity::Medium => {
                   // Memory MCP만 사용 (과거 사례 참조)
                   self.rule_with_memory_judgment(workflow, input_data).await
               }
               Complexity::Complex => {
                   // 세 MCP 모두 사용 (Sequential Thinking + Memory + Context7)
                   self.full_hybrid_judgment(workflow, input_data).await
               }
           }
       }
   }
   ```
   - [ ] 복잡도 분석 로직 (Rule 기반)
   - [ ] 3-Tier MCP 활성화 전략 (simple/medium/complex)
   - [ ] 토큰 사용량 추적 시스템
   - [ ] Redis 캐싱으로 Context7 문서 재사용 (70-90% 절감)
   - [ ] 비용 모니터링 대시보드
   - [ ] 워크플로우별 MCP 설정 UI

   **예상 효과**:
   - 💰 **비용 절감**: 월 $3,420 → $1,200 (65% 절감)
   - ⚡ **성능 향상**: 간단한 판단 응답 시간 5초 → 0.5초
   - 📊 **투명성**: 사용자가 비용 대비 정확도 선택 가능

**Week 3-4 예상 결과**:
- ✅ 3개 알고리즘 모두 작동
- ✅ 자동 Rule 추출 성공률 60% 이상
- ✅ Few-shot 학습으로 LLM 판단 정확도 85% → 92% 향상
- ✅ MCP 조건부 활성화로 비용 65% 절감 ($3,420 → $1,200/월)

---

### 📅 MCP 서버 재평가 시점 (Phase별 계획)

#### 🔄 현재 비활성화된 MCP 서버 재추가 계획

1. **DeepGraph TypeScript MCP** ⚠️
   ```
   재활성화 조건:
   - Phase 3 완료 후 (Week 7-8 이후)
   - 코드베이스 규모: 10,000+ 줄
   - 리팩토링 계획 수립 시
   - 아키텍처 문서 자동 생성 필요 시

   예상 시점: Phase 4 (배포 후 3개월)

   활용 목적:
   - 대규모 코드베이스 의존성 분석
   - 리팩토링 임팩트 분석
   - 아키텍처 다이어그램 자동 생성
   ```

2. **CircleCI MCP** ❌
   ```
   재활성화 조건:
   - 대규모 팀 협업 환경 전환 시
   - GitHub Actions 제한 초과 시 (빌드 시간, 동시 실행)
   - 복잡한 CI/CD 파이프라인 필요 시

   예상 시점: 기업용 배포 후 (10+ 개발자 팀)

   활용 목적:
   - 복잡한 빌드 파이프라인 관리
   - 병렬 테스트 실행
   - 고급 배포 전략 (Blue-Green, Canary)
   ```

3. **추가 고려 MCP 서버** 🆕
   ```
   Slack MCP:
   - 재활성화 시점: Notification Service (8004) 개발 시
   - 용도: 판단 결과 실시간 알림

   Redis MCP:
   - 재활성화 시점: MCP 캐싱 시스템 구현 시 (Week 4)
   - 용도: Context7 문서 캐싱 (30분 TTL)

   Notion MCP:
   - 재활성화 시점: 프로젝트 문서화 자동화 시
   - 용도: 설계 문서 자동 업데이트
   ```

#### 📊 MCP 서버 재평가 체크리스트

**Phase 3 종료 시 (Week 7-8)**:
- [ ] 코드베이스 라인 수 확인 (10,000+ 줄?)
- [ ] DeepGraph 필요성 재평가
- [ ] 아키텍처 복잡도 분석 필요 여부

**Phase 4 (배포 후 1개월)**:
- [ ] 팀 규모 확인 (5+ 개발자?)
- [ ] GitHub Actions 사용량 모니터링
- [ ] CircleCI 필요성 재평가

**Phase 4 (배포 후 3개월)**:
- [ ] 월간 토큰 사용량 분석
- [ ] 신규 MCP 서버 검토 (Slack, Redis, Notion)
- [ ] 비용 대비 효과 분석

---

### 📅 Phase 3: Week 5-6 (02-10 ~ 02-21) - BI Service + Chat 고도화
**목표**: AI 기반 BI 생성 및 통합 AI 채팅 완성

#### 우선순위 🔴 Critical
1. **BI Service - 사용자 요청 분석 (LLM)** (3일)
   ```rust
   // 목표: 자연어 요청 → 데이터 소스 + 차트 타입 분석
   pub async fn analyze_request(&self, user_request: String) -> Result<RequestAnalysis> {
       let prompt = format!(
           "사용자 요청: '{}'\n\
            다음 형식으로 분석하세요:\n\
            {{\n\
              \"data_sources\": [\"workflows\", \"judgment_executions\"],\n\
              \"chart_types\": [\"BarChart\", \"LineChart\"],\n\
              \"time_range\": \"last_7_days\",\n\
              \"filters\": {{\"confidence_score\": \">0.8\"}}\n\
            }}",
           user_request
       );

       let response = self.openai_client.chat_completion(&prompt).await?;
       let analysis: RequestAnalysis = serde_json::from_str(&response)?;
       Ok(analysis)
   }
   ```
   - [ ] 요청 분석 프롬프트 설계
   - [ ] LLM 응답 JSON 파싱
   - [ ] 데이터 소스 매핑

2. **BI Service - Judgment Service 연동** (2일)
   ```rust
   // 목표: 데이터 기반 판단 요청
   pub async fn generate_insight(&self, user_request: String) -> Result<BiInsight> {
       // 1. 요청 분석
       let analysis = self.analyze_request(user_request).await?;

       // 2. 데이터 조회
       let data = self.db.query_data(&analysis.data_sources, &analysis.filters)?;

       // 3. Judgment Service 호출
       let judgment_result = self.judgment_client.evaluate(JudgmentInput {
           workflow_id: "bi_analysis".to_string(),
           data: serde_json::to_value(&data)?,
           context: analysis.clone(),
       }).await?;

       // 4. 인사이트 생성
       let insights = self.generate_business_insights(&judgment_result).await?;

       Ok(BiInsight {
           title: analysis.suggested_title,
           insights,
           component_code: "...",
       })
   }
   ```
   - [ ] Judgment Service HTTP 클라이언트
   - [ ] 데이터 조회 로직
   - [ ] 인사이트 생성 프롬프트

3. **BI Service - React 컴포넌트 자동 생성** (3일)
   ```rust
   // 목표: 차트 타입 → React 코드 생성
   pub async fn generate_dashboard_code(&self, analysis: &RequestAnalysis) -> Result<String> {
       let prompt = format!(
           "다음 요구사항으로 React 대시보드 코드를 생성하세요:\n\
            차트 타입: {:?}\n\
            데이터 소스: {:?}\n\
            \n\
            요구사항:\n\
            - Recharts 라이브러리 사용\n\
            - Tailwind CSS 스타일링\n\
            - 반응형 레이아웃 (grid-cols-12)\n\
            \n\
            코드만 반환하세요 (설명 없이).",
           analysis.chart_types,
           analysis.data_sources
       );

       let code = self.openai_client.chat_completion(&prompt).await?;
       Ok(code)
   }
   ```
   - [ ] 코드 생성 프롬프트 설계
   - [ ] 템플릿 기반 코드 조립
   - [ ] 코드 검증 (TypeScript 컴파일 체크)

#### 우선순위 🟡 Important
4. **Chat Interface - 의도 분류 고도화** (2일)
   ```rust
   // 목표: 사용자 메시지 → 서비스 라우팅
   pub async fn classify_intent(&self, message: String) -> Result<Intent> {
       let prompt = format!(
           "사용자 메시지: '{}'\n\
            다음 중 하나로 분류하세요:\n\
            - workflow_execution: 워크플로우 실행 요청\n\
            - data_visualization: 데이터 시각화 요청\n\
            - settings_change: 설정 변경 요청\n\
            - general_question: 일반 질문\n\
            \n\
            JSON 형식: {{\"intent\": \"...\", \"confidence\": 0.95}}",
           message
       );

       let response = self.openai_client.chat_completion(&prompt).await?;
       let intent: Intent = serde_json::from_str(&response)?;
       Ok(intent)
   }
   ```
   - [ ] 의도 분류 프롬프트 설계
   - [ ] 서비스별 라우팅 로직
   - [ ] 멀티턴 대화 컨텍스트 관리

**Week 5-6 예상 결과**:
- ✅ "지난 주 불량률 분석해줘" → 30초 내 자동 대시보드 생성
- ✅ 채팅으로 "품질 검사 워크플로우 실행해줘" → 즉시 실행
- ✅ BI 인사이트 정확도 80% 이상

---

### 📅 Phase 4: Week 7 (02-24 ~ 02-28) - Visual Workflow 고도화
**목표**: n8n 스타일 드래그앤드롭 완성도 향상

#### 우선순위 🟡 Important
1. **Workflow 노드 타입 확장** (2일)
   - [ ] Trigger 노드 (스케줄, 이벤트)
   - [ ] Condition 노드 (분기 처리)
   - [ ] Action 노드 (외부 시스템 연동)
   - [ ] Data Transform 노드 (데이터 가공)

2. **Workflow 실행 엔진** (2일)
   ```rust
   // 목표: 노드 순서대로 실행
   pub async fn execute_workflow(&self, workflow_id: String, input: Value) -> Result<Value> {
       let workflow = self.db.get_workflow(&workflow_id)?;
       let nodes = self.parse_nodes(&workflow.definition)?;

       let mut context = ExecutionContext::new(input);

       for node in nodes {
           match node.node_type.as_str() {
               "judgment" => {
                   let result = self.judgment_service.evaluate(context.data).await?;
                   context.update(result);
               }
               "action" => {
                   self.action_service.execute(node.config, context.data).await?;
               }
               "condition" => {
                   if self.evaluate_condition(&node.condition, &context)? {
                       // true 분기
                   } else {
                       // false 분기
                   }
               }
               _ => {}
           }
       }

       Ok(context.output())
   }
   ```
   - [ ] 노드 순서 파싱 (topological sort)
   - [ ] 노드별 실행 로직
   - [ ] 분기 처리 (조건부 실행)

#### 우선순위 🟢 Enhancement
3. **Workflow 템플릿 라이브러리** (1일)
   - [ ] 품질 검사 템플릿
   - [ ] 이상 탐지 템플릿
   - [ ] 데이터 처리 템플릿

**Week 7 예상 결과**:
- ✅ 복잡한 워크플로우 (5+ 노드) 정상 실행
- ✅ 조건부 분기 처리 가능
- ✅ 템플릿으로 빠른 시작 지원

---

### 📅 Phase 5: Week 8 (03-03 ~ 03-07) - 테스트 + 프로덕션 빌드
**목표**: 완전한 프로덕션 준비

#### 우선순위 🔴 Critical
1. **통합 테스트** (2일)
   - [ ] E2E 테스트 시나리오 (Playwright)
   - [ ] 각 서비스별 유닛 테스트
   - [ ] 성능 테스트 (응답 시간 <500ms)

2. **프로덕션 빌드** (2일)
   - [ ] Tauri 프로덕션 빌드 최적화
   - [ ] 실행 파일 서명 (코드 사이닝)
   - [ ] 설치 프로그램 생성 (MSI/EXE)

#### 우선순위 🟡 Important
3. **문서화 완성** (1일)
   - [ ] 사용자 가이드
   - [ ] API 문서 (OpenAPI)
   - [ ] 배포 가이드

**Week 8 예상 결과**:
- ✅ 전체 기능 테스트 완료
- ✅ Windows 설치 프로그램 배포 준비
- ✅ 사용자 문서 완성

---

## 🎯 3. 우선순위별 작업 분류

### 🔴 Critical (반드시 완료)
1. **Week 2**: Rule Engine 고도화, LLM Few-shot 통합
2. **Week 3-4**: Learning Service 3개 알고리즘 완성
3. **Week 5-6**: BI Service 핵심 기능 (요청 분석 + 자동 생성)
4. **Week 8**: 통합 테스트 + 프로덕션 빌드

### 🟡 Important (중요 기능)
1. **Week 2**: Judgment History 개선
2. **Week 3-4**: Few-shot 유사도 검색
3. **Week 5-6**: Chat Interface 의도 분류
4. **Week 7**: Workflow 실행 엔진
5. **Week 8**: 문서화 완성

### 🟢 Enhancement (부가 기능)
1. **Week 7**: Workflow 템플릿 라이브러리
2. Settings 페이지 MCP 서버 상태 표시
3. OpenAI API Key 검증 UI

---

## 📊 4. 예상 진행도 (주차별)

| 주차 | 핵심 작업 | 예상 완성도 |
|------|-----------|-------------|
| **현재** | 프로젝트 구조 + 기본 기능 | **45%** |
| Week 2 | Judgment Engine 강화 | **55%** (+10%) |
| Week 3-4 | Learning Service 완성 | **75%** (+20%) |
| Week 5-6 | BI + Chat 고도화 | **90%** (+15%) |
| Week 7 | Visual Workflow 고도화 | **95%** (+5%) |
| Week 8 | 테스트 + 프로덕션 빌드 | **100%** (+5%) |

---

## ✅ 5. 다음 단계 (즉시 시작 가능)

### 지금 바로 시작할 작업 (Week 2 Day 1):

1. **Rule Engine 고도화 시작**
   ```bash
   # 파일: src-tauri/src/services/rule_engine.rs

   # 작업 내용:
   - rhai 엔진에 커스텀 함수 등록 (avg, sum, contains)
   - 중첩 조건 테스트 케이스 작성
   - 배열/객체 접근 테스트
   ```

2. **Few-shot 프롬프트 템플릿 작성**
   ```bash
   # 파일: src-tauri/src/services/llm_engine.rs

   # 작업 내용:
   - Few-shot 샘플 포맷 설계
   - 프롬프트 템플릿 작성
   - JSON 응답 스키마 정의
   ```

3. **Learning Service 테이블 추가**
   ```bash
   # 파일: src-tauri/src/services/database.rs

   # 작업 내용:
   - training_samples 테이블 migration
   - 임베딩 저장 컬럼 추가
   - CRUD 메서드 구현
   ```

---

## 📝 6. 참고사항

### 기술 스택 확인
- **Backend**: Rust + Tauri + rhai (Rule Engine) + linfa (ML)
- **Frontend**: React + TypeScript + Tailwind CSS + shadcn/ui
- **Database**: SQLite (현재) → PostgreSQL + pgvector (향후)
- **AI**: OpenAI API (GPT-4 + Embeddings)
- **차트**: Recharts
- **워크플로우**: React Flow

### 에이전트 활용 권장
- **ai-engineer**: Judgment/Learning Service 개발
- **prompt-engineer**: LLM 프롬프트 최적화
- **database-optimization**: 데이터베이스 성능 튜닝
- **frontend-architect**: BI 대시보드 자동 생성
- **mlops-engineer**: Learning Service 알고리즘 구현

---

**🎯 최종 목표**: 8주 후 완전히 작동하는 하이브리드 AI 판단 플랫폼 완성!
