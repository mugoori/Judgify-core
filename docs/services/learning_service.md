# Learning Service 상세 설계 (Port 8009, Ver2.0 Final) 🔥

**완료율**: 100% ✅ (2025-10-30 완성)
**상태**: ✅ Rule 저장 기능 추가 완료, 3개 알고리즘 구현 완료 (빈도 분석 + LLM 패턴 발견), 테스트 25개 통과

## 1. 개요

### 1.1 서비스 목적
Learning Service는 **전통적 ML 알고리즘으로 ML 모델을 완전 대체**하는 혁신적인 자동학습 시스템입니다. 사용자 피드백 데이터를 분석하여 자동으로 Rule을 추출하고, Few-shot 학습을 통해 판단 정확도를 향상시킵니다.

### 1.2 핵심 기능
1. **3가지 Rule 추출 알고리즘**: 빈도 분석, 결정 트리, LLM 패턴 발견
2. **Few-shot 학습 관리**: pgvector 기반 유사 샘플 10-20개 자동 검색
3. **피드백 수집**: 👍👎, LOG 리뷰, 채팅 피드백 통합 관리
4. **성능 검증**: 추출된 Rule의 정확도 및 신뢰도 자동 검증

### 1.3 ML 모델 미사용 이유
- **해석 가능성**: Rule은 명확한 조건식으로 사람이 이해 가능
- **유지보수성**: 모델 재학습 없이 Rule 수정 가능
- **비용 효율성**: 고가의 GPU 서버 불필요, 단순 CPU로 처리
- **즉시 적용성**: 학습 완료 즉시 Rule로 변환하여 워크플로우에 적용

---

## 2. 시스템 아키텍처

### 2.1 서비스 구조
```
Learning Service (Port 8009)
├── Feedback Collection Engine    ← 👍👎, LOG, 채팅 피드백 수집
├── Rule Extraction Engine         ← 3가지 알고리즘 병렬 실행
│   ├── Frequency Analysis         (빈도 분석)
│   ├── Decision Tree Converter    (결정 트리)
│   └── LLM Pattern Discovery      (LLM 패턴 발견)
├── Few-shot Sample Manager        ← pgvector 유사도 검색 (10-20개)
├── Rule Validation Engine         ← 추출 Rule 정확도 검증
└── REST API (FastAPI)             ← 외부 서비스 연동
```

### 2.2 데이터 흐름
```
사용자 피드백 (👍👎/LOG/채팅)
    ↓
Feedback Collection Engine (최소 50개 수집)
    ↓
3가지 Rule 추출 알고리즘 병렬 실행
    ├── Frequency Analysis → Rule 후보 1
    ├── Decision Tree → Rule 후보 2
    └── LLM Pattern → Rule 후보 3
    ↓
Rule Validation (정확도 검증, 신뢰도 비교)
    ↓
최적 Rule 선택 (신뢰도 가장 높은 Rule)
    ↓
Workflow Service에 Rule 등록
    ↓
Few-shot Sample Manager (판단시 유사 샘플 10-20개 제공)
```

### 2.3 외부 서비스 연동
```yaml
입력 의존성:
  - Judgment Service (8002): 판단 실행 결과 + 피드백 데이터
  - Chat Interface (8008): 채팅 기반 피드백 수집
  - PostgreSQL: raw_data, judgment_executions 테이블

출력 의존성:
  - Workflow Service (8001): 추출된 Rule 자동 등록
  - Judgment Service (8002): Few-shot 샘플 제공 (유사도 검색)
  - BI Service (8007): 학습 성능 메트릭 시각화
```

---

## 3. 3가지 Rule 추출 알고리즘

### 3.1 알고리즘 1: 빈도 분석 (Frequency Analysis)
**원리**: 사용자 피드백에서 자주 발생하는 패턴을 찾아 Rule로 변환

#### 처리 프로세스
```python
def frequency_analysis(feedback_data: List[FeedbackData]) -> List[Rule]:
    """
    빈도 분석 기반 Rule 추출

    입력: 최근 100개 판단 데이터 + 피드백 (👍👎)
    출력: 80% 이상 빈도 패턴 → Rule 후보
    """
    # 1. 패턴 카운팅
    pattern_counts = {}
    for data in feedback_data:
        if data.feedback == "👍":  # 긍정 피드백만 분석
            pattern = extract_condition_pattern(data.input_data)
            pattern_counts[pattern] = pattern_counts.get(pattern, 0) + 1

    # 2. 임계값 적용 (80% 이상)
    total_positive = sum(pattern_counts.values())
    threshold = total_positive * 0.80

    # 3. Rule 생성
    extracted_rules = []
    for pattern, count in pattern_counts.items():
        if count >= threshold:
            rule = Rule(
                expression=pattern_to_expression(pattern),
                frequency=count / total_positive,
                confidence=calculate_confidence(count, total_positive),
                sample_count=count,
                method="frequency_analysis"
            )
            extracted_rules.append(rule)

    return extracted_rules
```

#### 예시: 온도/진동 모니터링
```json
입력 데이터 (최근 100개):
[
  {"input": {"temp": 88, "vib": 42}, "result": true, "feedback": "👍"},
  {"input": {"temp": 87, "vib": 43}, "result": true, "feedback": "👍"},
  {"input": {"temp": 89, "vib": 41}, "result": true, "feedback": "👍"},
  ... (82개 유사 패턴)
  {"input": {"temp": 82, "vib": 38}, "result": false, "feedback": "👍"},
  ... (18개 다른 패턴)
]

분석 결과:
- temp > 85 AND vib > 40: 82회 (82%) ← Rule 후보!
- temp > 90 AND vib > 35: 15회 (15%) ← 임계값 미달

추출된 Rule:
{
  "rule_expression": "temp > 85 AND vib > 40",
  "frequency": 0.82,
  "confidence": 0.85,
  "sample_count": 82,
  "method": "frequency_analysis",
  "recommendation": "이 Rule을 워크플로우에 추가하면 82% 케이스를 자동 처리할 수 있습니다."
}
```

### 3.2 알고리즘 2: 결정 트리 (Decision Tree Conversion)
**원리**: sklearn DecisionTreeClassifier로 학습 → 트리를 Rule로 변환

#### 처리 프로세스
```python
from sklearn.tree import DecisionTreeClassifier, export_text

def decision_tree_extraction(feedback_data: List[FeedbackData]) -> List[Rule]:
    """
    결정 트리 기반 Rule 추출

    입력: 최근 100개 판단 데이터 + 피드백
    출력: 트리 경로 → Rule 조건식
    """
    # 1. 데이터 준비
    X = [extract_features(d.input_data) for d in feedback_data]
    y = [1 if d.feedback == "👍" else 0 for d in feedback_data]

    # 2. 결정 트리 학습
    clf = DecisionTreeClassifier(
        max_depth=3,           # 최대 깊이 제한 (해석 가능성)
        min_samples_split=10,  # 최소 분할 샘플
        random_state=42
    )
    clf.fit(X, y)

    # 3. 트리 경로 추출
    tree_rules = export_text(clf, feature_names=list(X[0].keys()))

    # 4. Rule 변환
    extracted_rules = []
    for leaf in get_leaf_nodes(clf):
        path = get_decision_path(clf, leaf)
        rule_expression = path_to_expression(path)

        rule = Rule(
            expression=rule_expression,
            confidence=leaf.confidence,
            sample_count=leaf.samples,
            method="decision_tree",
            tree_depth=clf.get_depth(),
            feature_importance=dict(zip(X[0].keys(), clf.feature_importances_))
        )
        extracted_rules.append(rule)

    return extracted_rules
```

#### 예시: 결정 트리 → Rule 변환
```
학습 완료된 결정 트리:
|--- temp <= 85.0
|   |--- class: False (samples=22, confidence=0.91)
|--- temp > 85.0
|   |--- vib <= 40.0
|   |   |--- class: False (samples=12, confidence=0.83)
|   |--- vib > 40.0
|   |   |--- class: True (samples=78, confidence=0.89)

변환된 Rule:
{
  "rule_expression": "temp > 85 AND vib > 40",
  "confidence": 0.89,
  "sample_count": 78,
  "method": "decision_tree",
  "tree_depth": 2,
  "feature_importance": {
    "temp": 0.62,  // 온도가 더 중요
    "vib": 0.38
  }
}
```

### 3.3 알고리즘 3: LLM 패턴 발견 (LLM Pattern Discovery)
**원리**: 데이터 집계 통계를 LLM이 분석 → 숨겨진 패턴 발견 → Rule 제안

#### 처리 프로세스
```python
async def llm_pattern_discovery(feedback_data: List[FeedbackData]) -> List[Rule]:
    """
    LLM 기반 패턴 발견 Rule 추출

    입력: 데이터 집계 요약 (통계)
    출력: LLM이 제안한 Rule 후보
    """
    # 1. 데이터 집계
    summary = {
        "total_samples": len(feedback_data),
        "positive_feedback": sum(1 for d in feedback_data if d.feedback == "👍"),
        "negative_feedback": sum(1 for d in feedback_data if d.feedback == "👎"),
        "statistical_summary": calculate_statistics(feedback_data)
    }

    # 2. LLM Prompt 생성
    prompt = f"""
    너는 데이터 패턴 발견 전문가야.
    아래 통계 요약을 분석해서 숨겨진 Rule을 제안해줘.

    데이터 집계 요약:
    {json.dumps(summary, indent=2)}

    분석 프로세스:
    1. 긍정/부정 피드백 간 변수 차이 발견
    2. 상관관계 분석
    3. 패턴 제안
    4. Rule 생성

    요구 응답 형식: JSON
    """

    # 3. LLM 호출 (OpenAI)
    response = await openai.ChatCompletion.create(
        model="gpt-4o",
        messages=[{"role": "user", "content": prompt}],
        temperature=0.3  # 일관성 있는 분석
    )

    # 4. Rule 파싱
    llm_rules = parse_llm_response(response.choices[0].message.content)

    return llm_rules
```

#### 예시: LLM 패턴 발견
```json
입력 (데이터 집계 요약):
{
  "total_samples": 100,
  "positive_feedback": 85,
  "negative_feedback": 15,
  "statistical_summary": {
    "temp_avg_positive": 87.5,
    "temp_avg_negative": 82.3,
    "vib_avg_positive": 43.2,
    "vib_avg_negative": 38.7,
    "correlation_temp_vib": 0.72  // 강한 양의 상관관계
  }
}

LLM 분석 결과:
"긍정 피드백 케이스에서 온도 평균 87.5도, 진동 평균 43.2로
부정 케이스보다 각각 5.2도, 4.5 높음.
온도와 진동의 상관관계 0.72로 두 변수가 함께 움직임.
→ 두 변수가 모두 높을 때 긍정 피드백 발생"

추출된 Rule:
{
  "rule_expression": "temp > 85 AND vib > 40",
  "reasoning": "긍정 피드백 케이스에서 temp 평균 87.5, vib 평균 43.2로 부정 케이스보다 각각 5.2, 4.5 높음. 상관관계 0.72로 두 변수가 함께 움직임.",
  "confidence": 0.83,
  "method": "llm_pattern_discovery"
}
```

### 3.4 3가지 알고리즘 결과 통합
```python
def integrate_rules(
    freq_rules: List[Rule],
    tree_rules: List[Rule],
    llm_rules: List[Rule]
) -> Rule:
    """
    3가지 알고리즘 결과를 통합하여 최적 Rule 선택

    통합 전략:
    1. 동일 Rule 표현식 찾기
    2. 신뢰도 평균 계산
    3. 샘플 수 합산
    4. 최종 신뢰도 가장 높은 Rule 선택
    """
    all_rules = freq_rules + tree_rules + llm_rules

    # Rule 표현식별 그룹화
    rule_groups = {}
    for rule in all_rules:
        expr = rule.expression
        if expr not in rule_groups:
            rule_groups[expr] = []
        rule_groups[expr].append(rule)

    # 각 그룹별 통합 Rule 생성
    integrated_rules = []
    for expr, rules in rule_groups.items():
        avg_confidence = sum(r.confidence for r in rules) / len(rules)
        total_samples = sum(r.sample_count for r in rules)
        methods_used = [r.method for r in rules]

        integrated_rule = Rule(
            expression=expr,
            confidence=avg_confidence,
            sample_count=total_samples,
            method="integrated",
            methods_used=methods_used,
            agreement_level=len(rules) / 3.0  # 3개 알고리즘 중 일치율
        )
        integrated_rules.append(integrated_rule)

    # 최고 신뢰도 Rule 반환
    best_rule = max(integrated_rules, key=lambda r: r.confidence)
    return best_rule
```

---

## 4. Few-shot 학습 관리

### 4.1 pgvector 기반 유사도 검색
```python
async def select_few_shot_samples(
    current_input: dict,
    limit: int = 20
) -> List[FewShotSample]:
    """
    현재 판단과 유사한 과거 샘플 10-20개 검색

    입력: 현재 판단 입력 데이터
    출력: 유사한 과거 샘플 10-20개
    """
    # 1. 입력 임베딩 생성
    embedding = await generate_embedding(current_input)

    # 2. pgvector 유사도 검색 (cosine similarity)
    query = f"""
    SELECT
        input_data,
        result,
        feedback,
        explanation,
        1 - (explanation_embedding <=> '{embedding}') AS similarity
    FROM judgment_executions
    WHERE feedback IS NOT NULL  -- 피드백 있는 것만
    ORDER BY explanation_embedding <=> '{embedding}'
    LIMIT {limit * 2}  -- 다양성 필터링 위해 2배 검색
    """

    raw_samples = await db.execute(query)

    # 3. 다양성 필터링 (너무 유사한 것 제거)
    filtered_samples = diversity_filter(raw_samples, threshold=0.95)

    # 4. 긍정/부정 균형 (긍정 15개, 부정 5개)
    balanced_samples = balance_feedback(
        filtered_samples,
        positive_count=15,
        negative_count=5
    )

    # 5. 최종 선택 (10-20개)
    final_samples = balanced_samples[:limit]

    return final_samples
```

### 4.2 Few-shot 효과성 측정
```python
async def measure_few_shot_effectiveness(workflow_id: str) -> dict:
    """
    Few-shot 샘플 사용 효과 측정

    비교:
    - Few-shot 사용시 정확도
    - Few-shot 미사용시 정확도
    → 향상률 계산 (목표: +15%p)
    """
    # 1. Few-shot 사용 판단 수집 (최근 100개)
    with_few_shot = await db.execute(f"""
        SELECT result, feedback
        FROM judgment_executions
        WHERE workflow_id = '{workflow_id}'
          AND few_shot_samples IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 100
    """)

    # 2. Few-shot 미사용 판단 수집 (최근 100개)
    without_few_shot = await db.execute(f"""
        SELECT result, feedback
        FROM judgment_executions
        WHERE workflow_id = '{workflow_id}'
          AND few_shot_samples IS NULL
        ORDER BY created_at DESC
        LIMIT 100
    """)

    # 3. 정확도 계산
    accuracy_with = calculate_accuracy(with_few_shot)
    accuracy_without = calculate_accuracy(without_few_shot)

    improvement = accuracy_with - accuracy_without

    return {
        "accuracy_with_few_shot": accuracy_with,
        "accuracy_without_few_shot": accuracy_without,
        "improvement": improvement,
        "meets_target": improvement >= 0.15  # 15%p 목표
    }
```

---

## 5. API 엔드포인트 설계

### 5.1 Rule 추출 API
```python
@app.post("/api/v2/learning/extract-rules")
async def extract_rules(
    workflow_id: str,
    algorithm: str = "all",  # all | frequency | decision_tree | llm_pattern
    min_samples: int = 50
) -> RuleExtractionResponse:
    """
    피드백 데이터로부터 자동 Rule 추출

    요청 예시:
    POST /api/v2/learning/extract-rules
    {
      "workflow_id": "temp_monitoring_v2",
      "algorithm": "all",
      "min_samples": 50
    }

    응답 예시:
    {
      "extracted_rules": [
        {
          "rule_expression": "temp > 85 AND vib > 40",
          "confidence": 0.87,
          "method": "integrated",
          "methods_used": ["frequency_analysis", "decision_tree", "llm_pattern"],
          "sample_count": 82,
          "recommendation": "이 Rule을 워크플로우에 추가하면 82% 케이스를 자동 처리할 수 있습니다."
        }
      ],
      "total_feedback_samples": 100,
      "processing_time_ms": 2340
    }
    """
    # 피드백 데이터 수집
    feedback_data = await collect_feedback(workflow_id)

    if len(feedback_data) < min_samples:
        raise HTTPException(400, f"샘플 부족: {len(feedback_data)}개 (최소 {min_samples}개)")

    # 알고리즘별 실행
    if algorithm == "all":
        freq_rules = await frequency_analysis(feedback_data)
        tree_rules = await decision_tree_extraction(feedback_data)
        llm_rules = await llm_pattern_discovery(feedback_data)

        best_rule = integrate_rules(freq_rules, tree_rules, llm_rules)
        return {"extracted_rules": [best_rule], ...}

    elif algorithm == "frequency":
        rules = await frequency_analysis(feedback_data)
        return {"extracted_rules": rules, ...}

    # ... 기타 알고리즘
```

### 5.2 Rule 저장 API (신규 추가! 🆕)
```python
@app.post("/api/v2/learning/save-rule")
async def save_extracted_rule(
    workflow_id: str,
    rule_expression: str,
    confidence: float
) -> SaveRuleResponse:
    """
    추출된 Rule을 Workflow에 자동 저장

    요청 예시:
    POST /api/v2/learning/save-rule
    {
      "workflow_id": "temp_monitoring_v2",
      "rule_expression": "temperature > 85 && vibration > 40",
      "confidence": 0.92
    }

    응답 예시:
    {
      "success": true,
      "workflow_id": "temp_monitoring_v2",
      "old_version": 1,
      "new_version": 2,
      "updated_at": "2025-10-30T14:23:45Z",
      "message": "Rule이 Workflow에 성공적으로 저장되었습니다."
    }

    에러 응답 (Workflow 없음):
    {
      "success": false,
      "error": "Workflow not found: temp_monitoring_v2"
    }
    """
    # Learning Service의 save_extracted_rule() 호출
    learning_service.save_extracted_rule(workflow_id, rule_expression, confidence)

    return {"success": True, "workflow_id": workflow_id, ...}
```

**자동 통합**: `extract_rules()` API는 내부적으로 `save_extracted_rule()`을 자동 호출하여 추출된 Rule을 즉시 Workflow에 저장합니다.

### 5.3 Few-shot 샘플 검색 API
```python
@app.post("/api/v2/learning/few-shot-samples")
async def get_few_shot_samples(
    input_data: dict,
    limit: int = 20
) -> FewShotResponse:
    """
    현재 판단과 유사한 Few-shot 샘플 검색

    요청 예시:
    POST /api/v2/learning/few-shot-samples
    {
      "input_data": {"temperature": 88, "vibration": 42},
      "limit": 15
    }

    응답 예시:
    {
      "selected_samples": [
        {
          "input": {"temp": 87, "vib": 43},
          "result": true,
          "feedback": "👍",
          "similarity": 0.92
        },
        ... (15개)
      ],
      "selection_summary": {
        "positive_count": 12,
        "negative_count": 3,
        "avg_similarity": 0.88,
        "diversity_score": 0.65
      }
    }
    """
    samples = await select_few_shot_samples(input_data, limit)

    return FewShotResponse(
        selected_samples=samples,
        selection_summary=calculate_summary(samples)
    )
```

### 5.3 피드백 수집 API
```python
@app.post("/api/v2/learning/feedback")
async def submit_feedback(
    execution_id: str,
    feedback_type: str,  # thumbs_up | thumbs_down | log_review | chat_feedback
    feedback_data: dict
) -> FeedbackResponse:
    """
    사용자 피드백 수집 및 저장

    요청 예시:
    POST /api/v2/learning/feedback
    {
      "execution_id": "exec-uuid-123",
      "feedback_type": "thumbs_up",
      "feedback_data": {
        "comment": "정확한 판단이었어요",
        "timestamp": "2025-10-16T10:30:00Z"
      }
    }

    응답 예시:
    {
      "feedback_id": "feedback-uuid-456",
      "status": "collected",
      "total_feedback_count": 52,
      "ready_for_extraction": false,
      "min_samples_needed": 50
    }
    """
    # 피드백 저장
    feedback = await db.insert("feedback_data", {
        "execution_id": execution_id,
        "feedback_type": feedback_type,
        "feedback_data": feedback_data,
        "created_at": datetime.now()
    })

    # 피드백 수 확인
    total_count = await db.count("feedback_data", {"workflow_id": ...})

    return FeedbackResponse(
        feedback_id=feedback.id,
        status="collected",
        total_feedback_count=total_count,
        ready_for_extraction=total_count >= 50
    )
```

---

## 6. 성능 최적화 및 모니터링

### 6.1 성능 목표
```yaml
Rule 추출 성능:
  - 알고리즘별 실행 시간: < 2초
  - 3가지 알고리즘 병렬 실행 시간: < 3초
  - Rule 통합 시간: < 500ms
  - 전체 처리 시간: < 5초

Few-shot 검색 성능:
  - pgvector 유사도 검색: < 200ms
  - 다양성 필터링: < 100ms
  - 전체 샘플 선택 시간: < 500ms

목표 메트릭:
  - Rule 추출 정확도: 85% 이상
  - Few-shot 효과성: +15%p 향상
  - 의도 분류 정확도: 92% 이상
```

### 6.2 모니터링 메트릭
```python
# Prometheus 메트릭 정의

from prometheus_client import Counter, Histogram, Gauge

# Rule 추출 메트릭
rule_extractions_total = Counter(
    'learning_rule_extractions_total',
    'Total number of rule extractions',
    ['algorithm', 'success']
)

rule_extraction_duration = Histogram(
    'learning_rule_extraction_duration_seconds',
    'Duration of rule extraction',
    ['algorithm']
)

rule_confidence_score = Gauge(
    'learning_rule_confidence_score',
    'Confidence score of extracted rules',
    ['workflow_id', 'algorithm']
)

# Few-shot 메트릭
few_shot_searches_total = Counter(
    'learning_few_shot_searches_total',
    'Total number of few-shot sample searches'
)

few_shot_effectiveness = Gauge(
    'learning_few_shot_effectiveness',
    'Few-shot learning effectiveness (accuracy improvement)',
    ['workflow_id']
)

# 피드백 수집 메트릭
feedback_collected_total = Counter(
    'learning_feedback_collected_total',
    'Total feedback collected',
    ['feedback_type']
)

feedback_count_by_workflow = Gauge(
    'learning_feedback_count',
    'Current feedback count by workflow',
    ['workflow_id']
)
```

---

## 7. 배포 및 운영 가이드

### 7.1 Docker 설정
```yaml
# docker-compose.yml 발췌
learning-service:
  image: judgify/learning-service:2.0.0
  container_name: learning-service
  ports:
    - "8009:8009"
  environment:
    DATABASE_URL: ${DATABASE_URL}
    REDIS_URL: ${REDIS_URL}
    OPENAI_API_KEY: ${OPENAI_API_KEY}
    MIN_FEEDBACK_SAMPLES: 50
    RULE_CONFIDENCE_THRESHOLD: 0.70
    FEW_SHOT_SAMPLE_LIMIT: 20
  depends_on:
    - postgres
    - redis
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:8009/health"]
    interval: 30s
    timeout: 10s
    retries: 3
```

### 7.2 환경 변수
```bash
# .env.production
DATABASE_URL=postgresql://user:pass@postgres:5432/judgify_prod
REDIS_URL=redis://redis:6379/0
OPENAI_API_KEY=sk-...

# Learning Service 설정
MIN_FEEDBACK_SAMPLES=50          # Rule 추출 최소 샘플 수
RULE_CONFIDENCE_THRESHOLD=0.70   # Rule 신뢰도 임계값
FEW_SHOT_SAMPLE_LIMIT=20         # Few-shot 샘플 최대 개수
EMBEDDING_MODEL=text-embedding-3-small  # OpenAI 임베딩 모델
```

---

## 8. 추가 참조 문서

- **`docs/algorithms/auto_rule_extraction.md`**: 3가지 Rule 추출 알고리즘 상세 설계
- **`docs/algorithms/data_aggregation.md`**: 데이터 집계 알고리즘 (LLM 할루시네이션 방지)
- **`docs/architecture/database_design.md`**: feedback_data, judgment_executions 테이블 스키마
- **`docs/services/judgment_engine.md`**: Few-shot 샘플 활용한 판단 로직

---

**Ver2.0 Final 핵심 혁신**: ML 모델 없이 전통적 알고리즘 + LLM으로 자동학습 구현! 🔥
