# MVP 로드맵 및 단계적 출시 전략 (Ver2.0 Final)

이 문서는 Judgify-core Ver2.0 Final의 **10주 개발 계획** 및 **단계적 출시 전략**을 정의합니다.

---

## 📅 전체 일정 개요

```
Week 1-2: 인프라 및 설계
Week 3-6: Phase 1 MVP (핵심 4개 서비스)
Week 7-8: Phase 2 확장 (나머지 5개 서비스)
Week 9-10: 통합 테스트 및 배포
```

---

## 🚀 Phase 1: MVP (Minimum Viable Product) - Week 1-6

### 목표
- 하이브리드 판단 엔진 검증
- 자동학습 시스템 작동 확인
- MCP 기반 BI 생성 데모

### 구현 서비스 (4개)
1. **API Gateway (8000)**
2. **Judgment Service (8002)**
3. **Learning Service (8009)**
4. **BI Service (8007)**

---

## 📆 Week 1-2: 인프라 및 핵심 설계

### Week 1: 개발 환경 구축

#### Day 1-2: 프로젝트 초기화
- [ ] Git 저장소 초기화
- [ ] 프로젝트 디렉토리 구조 생성
- [ ] Docker Compose 개발 환경 설정
- [ ] PostgreSQL + pgvector 설치 및 설정
- [ ] Redis 설치 및 설정

**산출물**:
```
Judgify-core/
├── services/
│   ├── api-gateway/
│   ├── judgment/
│   ├── learning/
│   └── bi/
├── docker-compose.dev.yml
├── .env.development
└── README.md
```

#### Day 3-4: MCP 서버 설치 및 테스트
- [ ] PostgreSQL MCP 서버 설치
- [ ] Memory MCP 서버 설치
- [ ] GitHub MCP 서버 연결 확인
- [ ] MCP 서버 상태 테스트 스크립트 작성

**MCP 설치 명령**:
```bash
# .mcp.json 생성
npx -y @modelcontextprotocol/server-postgres
npx -y @modelcontextprotocol/server-memory
npx -y @modelcontextprotocol/server-github
```

#### Day 5: 데이터베이스 스키마 설계 및 생성
- [ ] PostgreSQL 스키마 생성 (initial.md 기반)
- [ ] pgvector extension 활성화
- [ ] 테이블 생성 스크립트 작성
- [ ] 초기 데이터 마이그레이션 스크립트

**핵심 테이블**:
- workflows
- judgment_executions
- predictions
- user_feedback
- training_samples
- extracted_rules

---

### Week 2: 핵심 서비스 설계

#### Day 1-2: API Gateway 구현
- [ ] Kong/Nginx 선택 및 설정
- [ ] JWT 인증 미들웨어 구현
- [ ] Rate Limiting 설정
- [ ] 라우팅 규칙 정의

**라우팅 규칙**:
```yaml
# Kong 라우팅 설정
routes:
  - path: /api/v2/judgment/*
    service: judgment-service:8002
  - path: /api/v2/learning/*
    service: learning-service:8009
  - path: /api/v2/bi/*
    service: bi-service:8007
```

#### Day 3-5: Judgment Service 기본 구조
- [ ] FastAPI 프로젝트 초기화
- [ ] AST 기반 Rule Engine 설계
- [ ] LLM 통합 준비 (OpenAI API)
- [ ] PostgreSQL 연결 설정
- [ ] Redis 캐싱 설정

**디렉토리 구조**:
```
services/judgment/
├── app/
│   ├── api/
│   │   └── v2/
│   │       └── judgment.py
│   ├── core/
│   │   ├── rule_engine.py
│   │   ├── llm_engine.py
│   │   └── hybrid_logic.py
│   ├── models/
│   │   └── judgment.py
│   └── main.py
├── tests/
├── Dockerfile
└── requirements.txt
```

---

## 📆 Week 3-4: Judgment Service 및 Learning Service 구현

### Week 3: Judgment Service 핵심 로직

#### Day 1-2: AST 기반 Rule Engine 구현
- [ ] AST 파서 구현 (Python ast 모듈)
- [ ] 안전성 검증 로직 (whitelist 방식)
- [ ] Rule 평가 엔진
- [ ] 유닛 테스트 (90% 커버리지)

**Rule Engine 예시**:
```python
class ASTRuleEngine:
    def evaluate(self, rule_expression: str, input_data: dict) -> RuleResult:
        """
        AST 기반 안전한 Rule 평가
        """
        tree = ast.parse(rule_expression, mode='eval')
        # whitelist 검증
        validated_tree = self.validate_ast(tree)
        # 평가 실행
        result = self.execute_ast(validated_tree, input_data)
        return RuleResult(success=True, value=result, confidence=1.0)
```

#### Day 3-4: LLM 판단 엔진 구현
- [ ] OpenAI API 통합
- [ ] Prompt 템플릿 구현 (prompt-guide.md 기반)
- [ ] Few-shot 학습 통합 (Learning Service 연동)
- [ ] 신뢰도 점수 계산

#### Day 5: 하이브리드 판단 로직
- [ ] Rule → LLM fallback 로직
- [ ] 신뢰도 임계값 설정 (0.7)
- [ ] 결과 종합 알고리즘
- [ ] 통합 테스트

---

### Week 4: Learning Service 자동학습 시스템

#### Day 1-2: 피드백 수집 시스템
- [ ] 피드백 API 엔드포인트 구현
- [ ] PostgreSQL 피드백 저장 로직
- [ ] 피드백 UI (판단 직후 팝업)
- [ ] 유닛 테스트

**피드백 API**:
```python
@app.post("/api/v2/learning/feedback")
async def submit_feedback(
    judgment_id: UUID,
    feedback_type: Literal["thumbs_up", "thumbs_down", "chat", "log_review"],
    feedback_value: int,  # -1, 0, 1
    feedback_text: Optional[str] = None
):
    """사용자 피드백 수집"""
    await save_feedback(judgment_id, feedback_type, feedback_value, feedback_text)
    await update_few_shot_samples(judgment_id, feedback_value)
    return {"status": "success"}
```

#### Day 3-4: Few-shot 학습 관리
- [ ] pgvector 임베딩 생성 (OpenAI)
- [ ] 유사도 검색 알고리즘
- [ ] 동적 샘플 개수 조정 (10-20개)
- [ ] Few-shot 샘플 반환 API

**Few-shot 검색**:
```python
async def get_few_shot_samples(input_data: dict, count: int = 15) -> List[dict]:
    """
    pgvector로 유사한 Few-shot 샘플 검색
    """
    # 임베딩 생성
    embedding = await openai.create_embedding(input_data)

    # pgvector 유사도 검색
    samples = await db.query(
        "SELECT * FROM training_samples ORDER BY sample_embedding <-> $1 LIMIT $2",
        embedding, count
    )

    return samples
```

#### Day 5: 자동 Rule 추출 알고리즘 (빈도 분석)
- [ ] 빈도 분석 알고리즘 구현
- [ ] 80% 임계값 패턴 추출
- [ ] Rule 표현식 자동 생성
- [ ] 유닛 테스트

**빈도 분석 예시**:
```python
async def frequency_analysis(feedback_data: List[dict]) -> ExtractedRule:
    """
    빈도 분석 기반 Rule 추출
    """
    # 패턴 카운팅
    patterns = defaultdict(int)
    for sample in feedback_data:
        pattern = extract_pattern(sample)
        patterns[pattern] += 1

    # 80% 이상 빈도 패턴 선택
    total = len(feedback_data)
    high_frequency_patterns = {
        pattern: count for pattern, count in patterns.items()
        if count / total >= 0.8
    }

    # Rule 생성
    best_pattern = max(high_frequency_patterns, key=high_frequency_patterns.get)
    rule_expression = generate_rule_expression(best_pattern)

    return ExtractedRule(
        rule_expression=rule_expression,
        confidence=high_frequency_patterns[best_pattern] / total,
        method="frequency_analysis",
        sample_count=high_frequency_patterns[best_pattern]
    )
```

---

## 📆 Week 5-6: BI Service 및 MVP 통합

### Week 5: BI Service MCP 컴포넌트 조립

#### Day 1-2: MCP Component Library 연동
- [ ] MCP Component Library 서버 설정
- [ ] 컴포넌트 검색 API 구현
- [ ] 컴포넌트 메타데이터 캐싱 (Redis)
- [ ] 유닛 테스트

**MCP 컴포넌트 검색**:
```python
async def search_mcp_components(
    search_query: str,
    filters: dict = {}
) -> List[MCPComponent]:
    """
    MCP Component Library에서 컴포넌트 검색
    """
    response = await mcp_client.call(
        "component_library",
        "search_components",
        {
            "query": search_query,
            "filters": filters,
            "limit": 10
        }
    )

    return [MCPComponent(**comp) for comp in response["components"]]
```

#### Day 3-4: LLM 기반 컴포넌트 선택 및 조립
- [ ] 사용자 요청 분석 Prompt (prompt-guide.md)
- [ ] 적합한 컴포넌트 선택 로직
- [ ] 데이터 바인딩 자동 생성
- [ ] 레이아웃 구성 알고리즘

**컴포넌트 조립 예시**:
```python
async def assemble_bi_components(user_request: str) -> BIInsight:
    """
    사용자 요청 기반 MCP 컴포넌트 조립
    """
    # 1. LLM으로 요청 분석
    analysis = await llm_analyzer.analyze_request(user_request)

    # 2. 적합한 컴포넌트 검색
    components = await search_mcp_components(
        search_query=analysis.search_keywords,
        filters={"domain": "manufacturing", "type": analysis.chart_type}
    )

    # 3. 컴포넌트 선택 및 데이터 바인딩
    selected_components = await select_optimal_components(components, analysis)

    # 4. 레이아웃 구성
    layout = generate_layout(selected_components)

    return BIInsight(
        selected_components=selected_components,
        layout=layout,
        insight_summary=analysis.insight_summary
    )
```

#### Day 5: AI 인사이트 생성 (RAG 기반)
- [ ] pgvector 유사 사례 검색
- [ ] 비즈니스 권장사항 생성 Prompt
- [ ] 인사이트 설명 생성
- [ ] 통합 테스트

---

### Week 6: MVP 통합 및 테스트

#### Day 1-2: 서비스 간 통합
- [ ] API Gateway 라우팅 테스트
- [ ] Judgment ↔ Learning 연동 테스트
- [ ] BI ↔ Judgment 연동 테스트
- [ ] E2E 테스트 시나리오 작성

**E2E 테스트 시나리오**:
```python
async def test_hybrid_judgment_with_learning():
    """
    하이브리드 판단 + Few-shot 학습 통합 테스트
    """
    # 1. 판단 실행
    judgment_result = await judgment_client.execute(
        workflow_id="test_workflow",
        input_data={"temperature": 88, "vibration": 42}
    )

    assert judgment_result.method_used in ["rule", "llm", "hybrid"]
    assert judgment_result.confidence >= 0.7

    # 2. 피드백 제공
    await learning_client.submit_feedback(
        judgment_id=judgment_result.id,
        feedback_type="thumbs_up",
        feedback_value=1
    )

    # 3. Few-shot 샘플 업데이트 확인
    samples = await learning_client.get_few_shot_samples(
        input_data={"temperature": 88, "vibration": 42},
        count=15
    )

    assert len(samples) >= 10
```

#### Day 3-4: 성능 최적화
- [ ] Redis 캐싱 전략 검증
- [ ] PostgreSQL 쿼리 최적화
- [ ] API 응답 시간 측정 (목표: <2초)
- [ ] 부하 테스트 (1000 req/min)

#### Day 5: MVP 데모 준비
- [ ] 데모 시나리오 작성
- [ ] 샘플 데이터 준비
- [ ] 데모 발표 자료 작성
- [ ] MVP 검증 체크리스트 확인

**MVP 성공 지표**:
- [ ] 하이브리드 판단 정확도 90% 이상
- [ ] Few-shot 학습 효과 +15%p 정확도 향상
- [ ] Rule 자동 추출 성공률 80% 이상
- [ ] BI 컴포넌트 조립 성공률 90% 이상

---

## 🚀 Phase 2: 확장 - Week 7-8

### 목표
- 전체 9개 서비스 통합
- Visual Workflow Builder 사용성 검증
- Chat Interface 마스터 컨트롤러 작동

### 구현 서비스 (5개)
5. **Workflow Service (8001)**
6. **Chat Interface Service (8008)**
7. **Data Visualization Service (8006)**
8. **Action Service (8003)**
9. **Notification Service (8004)**
10. **Logging Service (8005)**

---

## 📆 Week 7: Workflow Service 및 Chat Interface

### Week 7 Day 1-3: Workflow Service (Visual Builder)

#### Visual Workflow Builder UI
- [ ] Next.js 14 프로젝트 초기화
- [ ] React Flow 또는 n8n-editor 라이브러리 통합
- [ ] 7가지 노드 타입 컴포넌트 구현 (clarified-requirements.md 기반)
- [ ] 드래그앤드롭 UI 구현

**노드 컴포넌트 예시**:
```typescript
const TriggerNodeComponent: React.FC<{ node: TriggerNode }> = ({ node }) => {
  return (
    <div className="node-container">
      <div className="node-header">
        <span>⏰ Trigger</span>
      </div>
      <div className="node-body">
        <select value={node.config.triggerType}>
          <option value="rest_api">REST API</option>
          <option value="schedule">Schedule</option>
          <option value="webhook">Webhook</option>
          <option value="sensor">Sensor</option>
        </select>
      </div>
      <div className="node-footer">
        <span className="output-port">→</span>
      </div>
    </div>
  );
};
```

#### Workflow CRUD API
- [ ] 워크플로우 생성/조회/수정/삭제 API
- [ ] JSON 기반 워크플로우 정의 저장
- [ ] 버전 관리 시스템
- [ ] 유닛 테스트

---

### Week 7 Day 4-5: Chat Interface Service

#### 의도 분석 및 라우팅
- [ ] NLP 기반 의도 분류 (prompt-guide.md)
- [ ] 9개 서비스 라우팅 로직
- [ ] 멀티턴 대화 컨텍스트 관리 (Memory MCP)
- [ ] 유닛 테스트

**의도 분류 예시**:
```python
async def classify_intent(user_message: str, conversation_history: List[dict]) -> Intent:
    """
    사용자 메시지를 분석하여 의도 분류
    """
    prompt = f"""
    사용자 메시지: "{user_message}"
    대화 히스토리: {conversation_history}

    의도를 분류하세요:
    1. workflow_execution
    2. bi_insight_generation
    3. judgment_explanation
    4. settings_change
    5. feedback_submission
    """

    response = await openai.chat.completions.create(
        model="gpt-4",
        messages=[{"role": "user", "content": prompt}]
    )

    intent = parse_intent(response.choices[0].message.content)

    return Intent(
        type=intent.type,
        target_service=intent.target_service,
        parameters=intent.parameters,
        confidence=intent.confidence
    )
```

#### MCP 서버 상태 표시 (Settings)
- [ ] MCP ping 방식 상태 확인 (clarified-requirements.md)
- [ ] Settings 화면 UI 구현
- [ ] 실시간 상태 업데이트 (WebSocket)
- [ ] 연결 테스트 및 로그 조회 기능

---

## 📆 Week 8: Data Visualization, Action, Notification, Logging

### Week 8 Day 1-2: Data Visualization Service

#### 단순 데이터 대시보드
- [ ] 미리 정의된 대시보드 템플릿 구현
- [ ] PostgreSQL 데이터 직접 조회
- [ ] WebSocket 실시간 데이터 스트리밍
- [ ] 드래그앤드롭 차트 배치 변경 기능

**대시보드 템플릿 예시**:
```typescript
const InventoryDashboard: React.FC = () => {
  const { data, loading } = useRealTimeData({
    dataSource: "inventory_metrics",
    refreshInterval: 30000
  });

  return (
    <div className="dashboard-grid">
      <KPICard title="전체 재고" value={data.total_inventory} />
      <GaugeChart title="재고 회전율" value={data.turnover_rate} />
      <LineChart title="재고 추세" data={data.inventory_trend} />
      <BarChart title="품목별 재고" data={data.inventory_by_item} />
    </div>
  );
};
```

---

### Week 8 Day 3: Action Service

- [ ] MCP 프로토콜 기반 외부 시스템 연동
- [ ] Celery 비동기 처리
- [ ] 재시도 로직 (지수 백오프)
- [ ] 유닛 테스트

---

### Week 8 Day 4: Notification Service

- [ ] Slack/Teams/Email 통합
- [ ] 메시지 큐 기반 알림 발송
- [ ] 알림 템플릿 관리
- [ ] 유닛 테스트

---

### Week 8 Day 5: Logging Service

- [ ] ELK Stack 설정
- [ ] 구조화된 로그 수집
- [ ] 로그 검색 및 분석 API
- [ ] 유닛 테스트

---

## 📆 Week 9-10: 통합 테스트 및 배포

### Week 9: 통합 테스트

#### Day 1-2: E2E 테스트
- [ ] 9개 서비스 통합 E2E 테스트
- [ ] Playwright 자동화 테스트
- [ ] 성능 테스트 (1000 req/min)
- [ ] 부하 테스트 (10,000 동시 접속)

#### Day 3-4: 보안 및 성능 최적화
- [ ] JWT 인증 검증
- [ ] SQL Injection 방지 테스트
- [ ] AST Rule Engine 안전성 검증
- [ ] PostgreSQL 쿼리 최적화

#### Day 5: 문서화
- [ ] API 문서 자동 생성 (OpenAPI/Swagger)
- [ ] 사용자 가이드 작성
- [ ] 운영 매뉴얼 작성
- [ ] 아키텍처 다이어그램 최종 검토

---

### Week 10: 프로덕션 배포

#### Day 1-2: Docker/Kubernetes 배포
- [ ] Docker 이미지 빌드
- [ ] Kubernetes 배포 설정 (deployment.yaml)
- [ ] ConfigMap/Secret 설정
- [ ] Helm Chart 작성 (선택)

#### Day 3-4: 모니터링 및 알림 구축
- [ ] Prometheus 메트릭 수집
- [ ] Grafana 대시보드 구성
- [ ] 알림 규칙 설정 (Alertmanager)
- [ ] 로그 수집 파이프라인 (Fluentd → Elasticsearch)

#### Day 5: 프로덕션 배포 및 검증
- [ ] Staging 환경 배포
- [ ] Smoke 테스트
- [ ] Production 환경 배포
- [ ] 헬스체크 및 모니터링 확인

---

## 📊 성공 지표

### Phase 1 MVP (Week 6 종료 시점)
- [ ] 하이브리드 판단 정확도 90% 이상
- [ ] Few-shot 학습 효과 +15%p 정확도 향상
- [ ] Rule 자동 추출 성공률 80% 이상
- [ ] BI 컴포넌트 조립 성공률 90% 이상

### Phase 2 전체 (Week 8 종료 시점)
- [ ] 9개 서비스 정상 작동
- [ ] E2E 테스트 통과율 95% 이상
- [ ] API 응답 시간 < 2초 (95 percentile)
- [ ] Visual Workflow Builder 사용성 4.5/5 이상

### 프로덕션 배포 (Week 10 종료 시점)
- [ ] 서비스 가용성 99.5% 이상
- [ ] 배포 성공률 99.9%
- [ ] 모니터링 대시보드 100% 작동
- [ ] 문서화 완료 100%

---

## 🎯 다음 단계

이 로드맵을 기반으로 다음 단계를 진행합니다:

1. **/speckit.tasks** - 각 Week별 상세 작업 목록 생성 (약 100개)
2. **/speckit.analyze** - 기술적 위험 요소 및 대응 전략 분석
3. **/speckit.implement** - Context 관리하며 순차 구현 시작

---

**작성일**: 2025-10-20
**버전**: Ver2.0 Final
**상태**: 최종 확정
