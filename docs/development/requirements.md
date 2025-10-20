# 명확화된 요구사항 정리 (Ver2.0 Final)

이 문서는 `/speckit.clarify` 단계에서 명확화한 요구사항을 정리합니다.
모든 구현은 이 문서의 결정사항을 기준으로 진행됩니다.

---

## 📋 1. Learning Service (8009) 자동학습 시스템

### 1.1 Few-shot 샘플 개수 전략
**결정사항**: **동적 조정 방식**

```python
# Few-shot 샘플 개수 결정 로직
def determine_few_shot_count(input_data: dict) -> int:
    """
    입력 데이터 복잡도에 따라 Few-shot 샘플 개수를 동적으로 결정

    반환값:
    - 단순한 판단: 10개 (예: 단일 센서, 명확한 조건)
    - 보통 판단: 15개 (기본값)
    - 복잡한 판단: 20개 (예: 다중 센서, 복잡한 컨텍스트)
    """
    complexity_score = calculate_complexity(input_data)

    if complexity_score < 0.3:
        return 10  # 단순한 케이스
    elif complexity_score < 0.7:
        return 15  # 보통 케이스
    else:
        return 20  # 복잡한 케이스

def calculate_complexity(input_data: dict) -> float:
    """
    복잡도 계산:
    - 입력 변수 개수
    - 데이터 타입 다양성 (숫자, 문자열, boolean 등)
    - 중첩 구조 깊이

    반환값: 0.0 (단순) ~ 1.0 (복잡)
    """
    num_fields = len(input_data.keys())
    type_diversity = len(set(type(v).__name__ for v in input_data.values()))
    nesting_depth = get_max_nesting_depth(input_data)

    complexity = (
        min(num_fields / 10, 1.0) * 0.4 +
        min(type_diversity / 5, 1.0) * 0.3 +
        min(nesting_depth / 3, 1.0) * 0.3
    )

    return complexity
```

**이점**:
- LLM 토큰 최적화 (단순한 케이스는 10개만 사용)
- 복잡한 케이스는 충분한 컨텍스트 제공 (20개)
- 정확도와 비용의 균형

---

### 1.2 자동 Rule 추출 3가지 알고리즘 실행 전략
**결정사항**: **3개 알고리즘 동시 실행 후 최고 신뢰도 선택**

```python
# 3가지 알고리즘 병렬 실행
async def extract_rules(workflow_id: UUID, feedback_data: List[dict]) -> ExtractedRule:
    """
    3가지 알고리즘을 동시 실행하고 최고 신뢰도 Rule 선택

    알고리즘:
    1. 빈도 분석 (Frequency Analysis)
    2. 결정 트리 학습 (Decision Tree Learning)
    3. LLM 패턴 발견 (LLM Pattern Discovery)
    """
    # 병렬 실행
    results = await asyncio.gather(
        frequency_analysis(feedback_data),
        decision_tree_learning(feedback_data),
        llm_pattern_discovery(feedback_data)
    )

    # 최고 신뢰도 Rule 선택
    best_rule = max(results, key=lambda r: r.confidence)

    # 로깅: 3가지 결과 모두 기록 (비교 분석용)
    await log_rule_extraction_results(
        workflow_id=workflow_id,
        all_results=results,
        selected_rule=best_rule
    )

    return best_rule

# 각 알고리즘별 신뢰도 계산
class FrequencyAnalysisRule:
    confidence: float  # 빈도율 기반 (0.8 이상이면 높음)

class DecisionTreeRule:
    confidence: float  # Gini impurity 기반 (낮을수록 높음)

class LLMPatternRule:
    confidence: float  # LLM이 제시한 신뢰도
```

**이점**:
- 3가지 알고리즘의 강점을 모두 활용
- 최고 신뢰도 Rule 자동 선택
- 비교 분석 데이터 축적 (향후 개선용)

**비용 최적화**:
- Redis 캐싱: 동일한 피드백 데이터로 재추출 방지
- 배치 처리: 주기적 실행 (매일 1회)

---

### 1.3 사용자 피드백 수집 UI
**결정사항**: **판단 직후 팝업 + Chat Interface 메시지 옆**

#### 옵션 1: 판단 결과 직후 팝업 (높은 응답률)
```typescript
// Judgment 실행 직후 자동으로 피드백 모달 표시
interface FeedbackModal {
  judgmentId: UUID;
  result: JudgmentResult;
  feedbackOptions: {
    thumbsUp: "👍 정확해요",
    thumbsDown: "👎 틀렸어요",
    neutral: "🤷 잘 모르겠어요",
    skip: "건너뛰기"
  };
  commentField?: string; // 선택적 코멘트
}

// 3초 후 자동 닫힘 (사용자가 응답하지 않으면)
const FEEDBACK_MODAL_TIMEOUT = 3000;
```

#### 옵션 2: Chat Interface 메시지 옆 (자연스러움)
```typescript
// 각 판단 결과 메시지 옆에 피드백 버튼 표시
interface ChatMessage {
  messageId: UUID;
  content: string;
  judgmentResult?: JudgmentResult;
  feedbackButtons: {
    thumbsUp: "👍",
    thumbsDown: "👎"
  };
  feedbackStatus: "pending" | "submitted"; // 중복 방지
}
```

**구현 우선순위**:
1. Phase 1: 판단 직후 팝업 (높은 응답률 확보)
2. Phase 2: Chat Interface 피드백 추가

---

## 📊 2. 데이터 집계 알고리즘 (할루시네이션 방지)

### 2.1 통계 집계 임계값 기준
**결정사항**: **워크플로우별 사용자 정의**

```python
# 워크플로우 테이블에 임계값 설정 추가
class Workflow(Base):
    id: UUID
    name: str
    definition: dict

    # 임계값 설정 (신규)
    aggregation_thresholds: dict = {
        "normal": {"operator": "<", "value": 80},
        "warning": {"operator": ">=", "value": 80, "and": "<", "value2": 90},
        "critical": {"operator": ">=", "value": 90}
    }

    # 사용자 정의 가능
    custom_thresholds: bool = True

# 데이터 집계시 임계값 적용
async def evaluate_aggregated_data(aggregated_value: float, workflow: Workflow) -> str:
    """
    통계값을 정상/경고/위험으로 평가
    """
    thresholds = workflow.aggregation_thresholds

    if aggregated_value < thresholds["normal"]["value"]:
        return "normal"
    elif aggregated_value >= thresholds["warning"]["value"] and aggregated_value < thresholds["warning"]["value2"]:
        return "warning"
    else:
        return "critical"
```

**기본 임계값 (제조업 표준)**:
- **온도**: 정상 < 80°C, 경고 80-90°C, 위험 > 90°C
- **진동**: 정상 < 40Hz, 경고 40-50Hz, 위험 > 50Hz
- **불량률**: 정상 < 3%, 경고 3-5%, 위험 > 5%

**사용자 커스터마이징**:
- Workflow Editor에서 임계값 설정 UI 제공
- 워크플로우별 독립적인 임계값 설정

---

### 2.2 데이터 집계 주기
**결정사항**: **1일 1회 (매일 자정) + 수동 트리거 옵션**

```python
# Celery 스케줄링 설정
from celery.schedules import crontab

app.conf.beat_schedule = {
    'aggregate-data-daily': {
        'task': 'learning_service.tasks.aggregate_data',
        'schedule': crontab(hour=0, minute=0),  # 매일 자정
        'args': ('all_workflows',)
    }
}

# 수동 트리거 API
@app.post("/api/v2/learning/aggregate-data")
async def manual_aggregate_data(
    workflow_id: Optional[UUID] = None,
    time_range: str = "last_7_days"
):
    """
    수동으로 데이터 집계 실행

    Parameters:
    - workflow_id: 특정 워크플로우만 집계 (None이면 전체)
    - time_range: 집계 기간 (last_7_days, last_30_days, custom)
    """
    await aggregate_data_task.delay(workflow_id, time_range)
    return {"status": "triggered", "workflow_id": workflow_id}
```

**집계 결과 저장**:
- `aggregated_data` 테이블에 저장
- 90일 이후 판단 데이터는 집계 형태로만 유지
- 원본 데이터는 `archived_judgments` 테이블로 이동

---

## 🔌 3. MCP 통합 전략

### 3.1 PostgreSQL MCP 서버 설치 시점
**결정사항**: **지금 바로 설치**

```json
// .mcp.json 설정 추가
{
  "mcpServers": {
    "postgresql": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres"],
      "env": {
        "DATABASE_URL": "postgresql://judgify:password@localhost:5432/judgify_core"
      }
    },
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_TOKEN": "${GITHUB_TOKEN}"
      }
    },
    "memory": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-memory"]
    }
  }
}
```

**설치 순서**:
1. PostgreSQL MCP (최우선)
2. GitHub MCP (이미 설치됨)
3. Memory MCP (Chat Interface용)
4. Filesystem MCP (기본 제공)

---

### 3.2 Memory MCP 서버 컨텍스트 유지 기간
**결정사항**: **24시간 유지 + 중요 컨텍스트는 PostgreSQL에 별도 저장**

```python
# Memory MCP 설정
MEMORY_MCP_CONFIG = {
    "context_ttl": 86400,  # 24시간 (초 단위)
    "max_contexts": 1000,  # 최대 1000개 컨텍스트
    "cleanup_interval": 3600  # 1시간마다 만료된 컨텍스트 정리
}

# 중요 컨텍스트는 PostgreSQL에 영구 저장
class ImportantContext(Base):
    id: UUID
    user_id: UUID
    conversation_id: UUID
    context_data: dict
    created_at: datetime
    expires_at: datetime = None  # None이면 영구 보존

async def save_important_context(
    conversation_id: UUID,
    context_data: dict,
    permanent: bool = False
):
    """
    중요한 컨텍스트를 PostgreSQL에 저장

    Examples:
    - 워크플로우 생성 대화
    - 복잡한 BI 분석 요청
    - 사용자 선호도 설정
    """
    if permanent:
        expires_at = None
    else:
        expires_at = datetime.now() + timedelta(days=7)

    await db.save_context(
        conversation_id=conversation_id,
        context_data=context_data,
        expires_at=expires_at
    )
```

---

## 🎨 4. Visual Workflow Builder

### 4.1 n8n 스타일 노드 타입 상세 정의
**결정사항**: 7가지 노드 타입 + JSON 스키마

#### 노드 타입 1: Trigger
```typescript
interface TriggerNode {
  type: "trigger";
  id: UUID;
  name: string;
  config: {
    triggerType: "rest_api" | "schedule" | "webhook" | "sensor";

    // REST API 설정
    restApi?: {
      endpoint: string;
      method: "GET" | "POST" | "PUT" | "DELETE";
      headers?: Record<string, string>;
    };

    // 스케줄 설정
    schedule?: {
      cron: string;  // "0 */5 * * * *" (5분마다)
      timezone: string;
    };

    // Webhook 설정
    webhook?: {
      url: string;
      secret?: string;
    };

    // 센서 설정
    sensor?: {
      sensorId: string;
      pollingInterval: number;  // ms
    };
  };
}
```

#### 노드 타입 2: Condition
```typescript
interface ConditionNode {
  type: "condition";
  id: UUID;
  name: string;
  config: {
    conditionType: "if_else" | "switch_case";

    // IF-ELSE 설정
    ifElse?: {
      condition: string;  // "temperature > 85"
      trueOutput: UUID;   // 다음 노드 ID
      falseOutput: UUID;  // 다음 노드 ID
    };

    // Switch-Case 설정
    switchCase?: {
      variable: string;  // "status"
      cases: {
        value: any;
        output: UUID;  // 다음 노드 ID
      }[];
      defaultOutput: UUID;
    };
  };
}
```

#### 노드 타입 3: Judgment
```typescript
interface JudgmentNode {
  type: "judgment";
  id: UUID;
  name: string;
  config: {
    judgmentMethod: "rule_only" | "llm_only" | "hybrid";

    // Rule 설정
    ruleExpression?: string;  // "temp > 85 AND vib > 40"

    // LLM 설정
    llmPrompt?: string;
    fewShotEnabled: boolean;  // Few-shot 학습 활성화 여부

    // Hybrid 설정
    hybridStrategy?: {
      rulePriority: boolean;  // Rule 우선 실행
      confidenceThreshold: number;  // 0.7
    };
  };
}
```

#### 노드 타입 4: Action
```typescript
interface ActionNode {
  type: "action";
  id: UUID;
  name: string;
  config: {
    actionType: "slack" | "mcp" | "webhook" | "email";

    // Slack 설정
    slack?: {
      channel: string;
      message: string;
    };

    // MCP 설정
    mcp?: {
      system: string;  // "mes_system_a"
      command: string;
      parameters: Record<string, any>;
    };

    // Webhook 설정
    webhook?: {
      url: string;
      method: "GET" | "POST";
      body?: any;
    };

    // Email 설정
    email?: {
      to: string[];
      subject: string;
      body: string;
    };
  };
}
```

#### 노드 타입 5: Data Transform
```typescript
interface DataTransformNode {
  type: "data_transform";
  id: UUID;
  name: string;
  config: {
    transformType: "map" | "filter" | "aggregate" | "join";

    // Map 설정
    map?: {
      inputField: string;
      outputField: string;
      transformation: string;  // JavaScript 표현식
    };

    // Filter 설정
    filter?: {
      condition: string;  // "value > 100"
    };

    // Aggregate 설정
    aggregate?: {
      groupBy: string[];
      aggregations: {
        field: string;
        function: "sum" | "avg" | "count" | "min" | "max";
      }[];
    };
  };
}
```

#### 노드 타입 6: Loop
```typescript
interface LoopNode {
  type: "loop";
  id: UUID;
  name: string;
  config: {
    loopType: "for_each" | "while" | "until";

    // For Each 설정
    forEach?: {
      arrayField: string;  // "sensors"
      iterateOutput: UUID;  // 반복할 노드
      completeOutput: UUID;  // 완료 후 노드
    };

    // While 설정
    while?: {
      condition: string;  // "count < 10"
      maxIterations: number;  // 무한루프 방지
    };

    // Until 설정
    until?: {
      condition: string;  // "status == 'completed'"
      maxIterations: number;
    };
  };
}
```

#### 노드 타입 7: Merge
```typescript
interface MergeNode {
  type: "merge";
  id: UUID;
  name: string;
  config: {
    mergeType: "wait_all" | "first" | "any";

    inputs: UUID[];  // 입력 노드 ID 배열

    // Wait All: 모든 입력 대기
    waitAll?: {
      timeout: number;  // ms
    };

    // First: 첫 번째 입력만 사용
    // Any: 어떤 입력이든 도착하면 진행
  };
}
```

---

## 💬 5. Chat Interface Service

### 5.1 MCP 서버 상태 표시 방법
**결정사항**: **MCP ping 방식 (정확성 우선)**

```python
# MCP 서버 상태 확인 로직
class MCPServerStatus(BaseModel):
    server_name: str
    status: Literal["connected", "disconnected", "error"]
    last_ping: datetime
    response_time_ms: int
    version: str
    error_message: Optional[str]

async def check_mcp_server_status(server_name: str) -> MCPServerStatus:
    """
    MCP 서버 상태를 ping 방식으로 확인

    Process:
    1. MCP ping 명령 전송
    2. 응답 시간 측정
    3. 버전 정보 확인
    4. 상태 반환
    """
    try:
        start_time = time.time()

        # MCP ping 명령 (MCP 프로토콜 네이티브)
        response = await mcp_client.ping(server_name)

        response_time = (time.time() - start_time) * 1000

        return MCPServerStatus(
            server_name=server_name,
            status="connected",
            last_ping=datetime.now(),
            response_time_ms=int(response_time),
            version=response.get("version", "unknown"),
            error_message=None
        )

    except TimeoutError:
        return MCPServerStatus(
            server_name=server_name,
            status="disconnected",
            last_ping=datetime.now(),
            response_time_ms=0,
            version="unknown",
            error_message="Connection timeout"
        )

    except Exception as e:
        return MCPServerStatus(
            server_name=server_name,
            status="error",
            last_ping=datetime.now(),
            response_time_ms=0,
            version="unknown",
            error_message=str(e)
        )

# Settings 화면에서 실시간 상태 표시
@app.get("/api/v2/chat/mcp-status")
async def get_all_mcp_status():
    """
    모든 MCP 서버의 상태 조회
    """
    servers = ["postgresql", "github", "memory", "filesystem"]

    statuses = await asyncio.gather(*[
        check_mcp_server_status(server) for server in servers
    ])

    return {"mcp_servers": statuses}
```

**UI 구현**:
```typescript
// Settings 화면 MCP 서버 상태 표시
interface MCPStatusDisplay {
  serverName: string;
  statusIcon: "🟢" | "🔴" | "🟡";  // connected / disconnected / error
  responseTime: string;  // "45ms"
  lastPing: string;  // "2초 전"
  version: string;
  actions: {
    reconnect: () => void;
    testConnection: () => void;
    viewLogs: () => void;
  };
}
```

---

## 📈 6. 개발 우선순위 및 MVP 범위

### 6.1 개발 기간
**결정사항**: **10주 확정**

- Week 1-2: 인프라 및 핵심 서비스 설계
- Week 3-6: 핵심 4개 서비스 구현 (MVP Phase 1)
- Week 7-8: 나머지 5개 서비스 구현 (Phase 2)
- Week 9-10: 통합 테스트 및 배포

---

### 6.2 MVP 범위
**결정사항**: **단계적 출시 (Phase 1: 핵심 4개 → Phase 2: 나머지 5개)**

#### Phase 1 (MVP) - Week 1-6
**핵심 4개 서비스**:
1. **API Gateway (8000)** - 인증/라우팅
2. **Judgment Service (8002)** - 하이브리드 판단 엔진 (최우선!)
3. **Learning Service (8009)** - 자동학습 시스템 (혁신 기능!)
4. **BI Service (8007)** - MCP 기반 컴포넌트 조립

**목표**:
- 하이브리드 판단 엔진 작동
- 자동학습 시스템 검증
- MCP 기반 BI 생성 데모

**성공 지표**:
- 판단 정확도 90% 이상
- Few-shot 학습 효과 검증 (+15%p 정확도 향상)
- Rule 자동 추출 성공률 80% 이상

#### Phase 2 (확장) - Week 7-8
**나머지 5개 서비스**:
5. **Workflow Service (8001)** - Visual Workflow Builder
6. **Chat Interface Service (8008)** - 통합 AI 어시스턴트
7. **Data Visualization Service (8006)** - 단순 대시보드
8. **Action Service (8003)** - 외부 시스템 연동
9. **Notification Service (8004)** - 알림 발송
10. **Logging Service (8005)** - 중앙 로그 관리

**목표**:
- 전체 9개 서비스 통합
- Visual Workflow Builder 사용성 검증
- Chat Interface 마스터 컨트롤러 작동

**성공 지표**:
- 9개 서비스 정상 작동
- E2E 테스트 통과율 95% 이상
- 사용자 만족도 4.5/5 이상

---

## 📊 우선순위 요약

### 🔴 P0 (최우선) - Phase 1 MVP
1. **Judgment Service** - 하이브리드 판단 엔진
2. **Learning Service** - 자동학습 시스템
3. **BI Service** - MCP 컴포넌트 조립
4. **API Gateway** - 인증/라우팅

### 🟡 P1 (Phase 2)
5. **Workflow Service** - Visual Builder
6. **Chat Interface** - 통합 AI 어시스턴트
7. **Data Visualization** - 단순 대시보드

### 🟢 P2 (Phase 2)
8. **Action Service** - 외부 연동
9. **Notification Service** - 알림
10. **Logging Service** - 중앙 로그

---

## ✅ 다음 단계

이 명확화된 요구사항을 기반으로 다음 단계를 진행합니다:

1. **/speckit.plan** - 10주 개발 계획 수립
2. **/speckit.tasks** - 약 100개 작업 목록 생성
3. **/speckit.analyze** - 아키텍처/성능/보안/위험 분석
4. **/speckit.implement** - Context 관리하며 순차 구현

---

**작성일**: 2025-10-20
**버전**: Ver2.0 Final
**상태**: 최종 확정
