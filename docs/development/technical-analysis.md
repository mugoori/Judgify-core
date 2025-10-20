# 기술적 분석 및 위험 평가 (Ver2.0 Final)

이 문서는 Judgify-core Ver2.0 Final의 **아키텍처, 성능, 보안, 위험 요소**를 분석하고 대응 전략을 수립합니다.

---

## 📊 분석 개요

| 분석 영역 | 위험 수준 | 주요 발견사항 | 대응 전략 |
|----------|----------|-------------|----------|
| **아키텍처** | 🟡 중간 | 9개 마이크로서비스 복잡도 | 명확한 책임 분리, API Gateway |
| **성능** | 🟡 중간 | LLM 응답 시간, pgvector 검색 | 캐싱, 배치 처리, 인덱싱 |
| **보안** | 🟢 낮음 | AST 기반으로 안전 | JWT, RBAC, 입력 검증 철저 |
| **위험** | 🟡 중간 | 일정 지연 가능성 | 단계적 출시, MVP 우선 |

---

## 1. 아키텍처 분석

### 1.1 마이크로서비스 통신 패턴 분석

#### 분석 내용
```
사용자 요청
    ↓
API Gateway (8000)
    ↓
┌─────────┬──────────┬───────────┬──────────┐
│Judgment │ Learning │    BI     │   Chat   │
│ (8002)  │  (8009)  │  (8007)   │  (8008)  │
└─────────┴──────────┴───────────┴──────────┘
    ↓           ↓          ↓          ↓
┌────────────────────────────────────────────┐
│         PostgreSQL + pgvector + Redis       │
└────────────────────────────────────────────┘
```

**통신 방식**:
- **동기 통신**: REST API (Judgment ↔ Learning, BI ↔ Judgment)
- **비동기 통신**: Celery + Redis (Action Service)
- **실시간 통신**: WebSocket (Data Visualization)

#### 🟡 발견된 위험
1. **서비스 간 의존성**: Judgment Service가 Learning Service에 의존
   - Learning Service 장애 시 Few-shot 학습 불가
   - 연쇄 장애 가능성

2. **네트워크 지연**: 9개 서비스 간 네트워크 호출로 인한 지연
   - 최악의 경우: API Gateway → Judgment → Learning → OpenAI (4-hop)
   - 예상 총 응답 시간: 2-5초

3. **데이터 일관성**: 분산 트랜잭션 미지원
   - Judgment 저장 성공 + Learning 저장 실패 시나리오
   - Eventually Consistent 방식 채택 필요

#### ✅ 대응 전략

**전략 1: Circuit Breaker 패턴**
```python
from circuitbreaker import circuit

@circuit(failure_threshold=5, recovery_timeout=60)
async def call_learning_service(input_data):
    """
    Learning Service 호출 시 Circuit Breaker 적용
    - 5회 연속 실패 시 Circuit Open (60초 동안 호출 차단)
    - Fallback: Few-shot 학습 없이 진행
    """
    try:
        return await learning_client.get_few_shot_samples(input_data)
    except Exception as e:
        logger.warning(f"Learning Service unavailable: {e}")
        return []  # Fallback: 빈 Few-shot 샘플
```

**전략 2: Saga 패턴 (분산 트랜잭션)**
```python
async def execute_judgment_saga(workflow_input):
    """
    Saga 패턴으로 분산 트랜잭션 구현
    """
    judgment_id = None

    try:
        # 1. Judgment 저장
        judgment_id = await judgment_repo.save(workflow_input)

        # 2. Learning Service에 예측 저장
        await learning_repo.save_prediction(judgment_id, result)

        return {"status": "success", "judgment_id": judgment_id}

    except Exception as e:
        # 롤백: Judgment 삭제
        if judgment_id:
            await judgment_repo.delete(judgment_id)

        return {"status": "failed", "error": str(e)}
```

**전략 3: API Gateway 레벨 타임아웃**
```yaml
# Kong 타임아웃 설정
routes:
  - path: /api/v2/judgment/*
    service: judgment-service:8002
    timeout: 5000ms  # 5초 타임아웃
    retries: 2       # 2회 재시도
```

---

### 1.2 데이터 일관성 전략 분석

#### 분석 내용
- **Eventually Consistent** 방식 채택
- PostgreSQL 단일 데이터베이스 사용 (ACID 보장)
- 서비스 간 데이터 동기화는 이벤트 기반

#### 🟢 장점
- PostgreSQL ACID 트랜잭션으로 데이터 무결성 보장
- 단일 DB로 인한 낮은 복잡도

#### 🟡 위험
- 9개 서비스가 동일 DB 접근 시 병목 가능성
- DB 장애 시 전체 시스템 다운

#### ✅ 대응 전략

**전략 1: PostgreSQL 읽기 복제본 (Read Replica)**
```yaml
# docker-compose.prod.yml
services:
  postgres-primary:
    image: pgvector/pgvector:pg15
    environment:
      POSTGRES_REPLICATION_MODE: master

  postgres-replica-1:
    image: pgvector/pgvector:pg15
    environment:
      POSTGRES_REPLICATION_MODE: slave
      POSTGRES_MASTER_HOST: postgres-primary
```

**전략 2: 연결 풀링 (Connection Pooling)**
```python
# SQLAlchemy 연결 풀 설정
engine = create_engine(
    DATABASE_URL,
    pool_size=20,        # 기본 연결 20개
    max_overflow=10,     # 최대 추가 연결 10개
    pool_pre_ping=True,  # 연결 사전 확인
    pool_recycle=3600    # 1시간마다 연결 재생성
)
```

**전략 3: DB 백업 및 복구 자동화**
```bash
#!/bin/bash
# 매일 자정 자동 백업
0 0 * * * pg_dump -U judgify -d judgify_core | gzip > /backups/backup_$(date +\%Y\%m\%d).sql.gz

# 7일 이상 오래된 백업 삭제
find /backups -name "backup_*.sql.gz" -mtime +7 -delete
```

---

## 2. 성능 분석

### 2.1 응답 시간 분석

#### 예상 응답 시간 (95 percentile)

| 엔드포인트 | 주요 처리 | 예상 시간 | 목표 시간 | 상태 |
|-----------|---------|----------|----------|------|
| **Judgment (Rule Only)** | AST 파싱 + 평가 | 50-100ms | <200ms | ✅ 양호 |
| **Judgment (LLM Only)** | OpenAI API 호출 | 2-4초 | <5초 | 🟡 주의 |
| **Judgment (Hybrid)** | Rule + LLM | 100ms-4초 | <5초 | ✅ 양호 |
| **Learning (Few-shot)** | pgvector 검색 | 200-500ms | <1초 | ✅ 양호 |
| **Learning (Rule 추출)** | 3개 알고리즘 병렬 | 5-10초 | <15초 | 🟡 주의 |
| **BI (컴포넌트 조립)** | MCP 검색 + LLM | 3-6초 | <8초 | 🟡 주의 |

#### 🟡 성능 병목 지점

1. **LLM API 호출** (OpenAI)
   - 평균 2-4초
   - 토큰 수에 따라 가변적
   - 월 비용 고려 필요

2. **pgvector 유사도 검색**
   - 10만 개 이상 샘플 시 느려질 가능성
   - 인덱싱 전략 필수

3. **Rule 추출 알고리즘**
   - 결정 트리 학습: 3-5초
   - LLM 패턴 발견: 2-3초
   - 병렬 실행으로 최적화 필요

#### ✅ 대응 전략

**전략 1: Redis 다층 캐싱**
```python
# 3단계 캐싱 전략
class CachingStrategy:
    async def get_judgment_result(self, input_hash: str):
        # Level 1: 인메모리 캐시 (LRU, 1000개)
        if result := self.memory_cache.get(input_hash):
            return result

        # Level 2: Redis 캐시 (TTL 5분)
        if result := await self.redis_cache.get(input_hash):
            self.memory_cache.set(input_hash, result)
            return result

        # Level 3: DB 조회
        result = await self.db.query_judgment(input_hash)
        await self.redis_cache.set(input_hash, result, ttl=300)
        self.memory_cache.set(input_hash, result)
        return result
```

**전략 2: pgvector 인덱싱 최적화**
```sql
-- HNSW 인덱스 (Hierarchical Navigable Small World)
CREATE INDEX ON training_samples USING hnsw (sample_embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);

-- IVFFlat 인덱스 (대용량 데이터용)
CREATE INDEX ON training_samples USING ivfflat (sample_embedding vector_cosine_ops)
WITH (lists = 100);

-- 인덱스 성능 비교 쿼리
EXPLAIN ANALYZE
SELECT * FROM training_samples
ORDER BY sample_embedding <-> $1
LIMIT 20;
```

**전략 3: LLM API 배치 처리**
```python
# 배치 요청으로 비용 및 시간 절감
async def batch_llm_requests(requests: List[dict]):
    """
    여러 판단 요청을 배치로 처리
    - 단일 요청: 2-4초 × 10회 = 20-40초
    - 배치 요청: 5-8초 (80% 시간 단축)
    """
    batch_prompt = "\n\n---\n\n".join([
        f"요청 {i+1}:\n{req}" for i, req in enumerate(requests)
    ])

    response = await openai.chat.completions.create(
        model="gpt-4",
        messages=[{"role": "user", "content": batch_prompt}]
    )

    return parse_batch_response(response)
```

**전략 4: 비동기 Rule 추출 (Celery)**
```python
# Rule 추출을 백그라운드에서 실행
@celery_app.task
def extract_rules_async(workflow_id: UUID):
    """
    사용자 응답을 기다리지 않고 백그라운드에서 Rule 추출
    - 완료시 Notification Service로 알림
    """
    results = asyncio.run(extract_rules(workflow_id))
    notification_service.send(
        channel="#alerts",
        message=f"Rule 추출 완료: {results}"
    )
```

---

### 2.2 확장성 분석

#### 예상 트래픽

| 시나리오 | 예상 QPS | 동시 사용자 | 상태 |
|---------|---------|----------|------|
| **개발 환경** | 10 | 5 | ✅ 문제없음 |
| **스테이징** | 100 | 50 | ✅ 문제없음 |
| **프로덕션 (초기)** | 1000 | 500 | 🟡 모니터링 필요 |
| **프로덕션 (확장)** | 10000 | 5000 | 🔴 수평 확장 필수 |

#### ✅ 대응 전략

**전략 1: Kubernetes 수평 확장 (HPA)**
```yaml
# Horizontal Pod Autoscaler
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: judgment-service-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: judgment-service
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

**전략 2: 로드밸런싱 전략**
```yaml
# Nginx 로드밸런서
upstream judgment_service {
    least_conn;  # 최소 연결 알고리즘
    server judgment-1:8002 weight=3;
    server judgment-2:8002 weight=2;
    server judgment-3:8002 weight=1;
}
```

---

## 3. 보안 분석

### 3.1 AST 기반 Rule Engine 안전성 분석

#### 보안 위협 시나리오

| 위협 | 설명 | 위험 수준 | 대응 |
|------|------|----------|------|
| **코드 인젝션** | 악의적인 Rule 표현식 실행 | 🔴 높음 | AST whitelist |
| **DoS 공격** | 무한루프 Rule 표현식 | 🟡 중간 | 타임아웃 설정 |
| **데이터 탈취** | Rule에서 민감 데이터 접근 | 🟡 중간 | 변수 whitelist |

#### ✅ AST Whitelist 구현
```python
# AST 안전성 검증
ALLOWED_AST_NODES = {
    ast.Expression,
    ast.BoolOp,
    ast.BinOp,
    ast.UnaryOp,
    ast.Compare,
    ast.Name,
    ast.Constant,
    ast.And,
    ast.Or,
    ast.Not,
    ast.Eq,
    ast.NotEq,
    ast.Lt,
    ast.LtE,
    ast.Gt,
    ast.GtE,
    ast.Add,
    ast.Sub,
    ast.Mult,
    ast.Div,
}

class ASTValidator:
    def validate(self, tree: ast.AST) -> bool:
        """
        AST 트리의 모든 노드가 whitelist에 있는지 검증
        """
        for node in ast.walk(tree):
            if type(node) not in ALLOWED_AST_NODES:
                raise SecurityError(
                    f"Forbidden AST node: {type(node).__name__}"
                )

        return True

    def validate_variables(self, tree: ast.AST, allowed_vars: Set[str]):
        """
        Rule에서 사용하는 변수가 허용된 변수인지 검증
        """
        for node in ast.walk(tree):
            if isinstance(node, ast.Name):
                if node.id not in allowed_vars:
                    raise SecurityError(
                        f"Forbidden variable: {node.id}"
                    )
```

**예시: 악의적인 Rule 차단**
```python
# ❌ 차단되는 악의적 Rule 표현식들
evil_rules = [
    "__import__('os').system('rm -rf /')",  # 시스템 명령 실행
    "eval('malicious code')",                # eval 실행
    "[x for x in range(99999999)]",          # 메모리 소진
]

# ✅ 허용되는 안전한 Rule 표현식들
safe_rules = [
    "temperature > 85 and vibration > 40",
    "(temp + vib) / 2 > 60",
    "status == 'RUNNING' or status == 'IDLE'",
]
```

---

### 3.2 인증 및 인가 분석

#### JWT 인증 구현
```python
# JWT 토큰 생성 및 검증
from jose import jwt, JWTError
from datetime import datetime, timedelta

SECRET_KEY = os.getenv("JWT_SECRET_KEY")
ALGORITHM = "HS256"
ACCESS_TOKEN_EXPIRE_MINUTES = 30

def create_access_token(data: dict) -> str:
    """JWT 액세스 토큰 생성"""
    to_encode = data.copy()
    expire = datetime.utcnow() + timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)
    to_encode.update({"exp": expire})
    return jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)

def verify_token(token: str) -> dict:
    """JWT 토큰 검증"""
    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        return payload
    except JWTError:
        raise HTTPException(status_code=401, detail="Invalid token")
```

#### RBAC (Role-Based Access Control)
```python
# 역할 기반 접근 제어
class Role(str, Enum):
    ADMIN = "admin"        # 모든 권한
    OPERATOR = "operator"  # 워크플로우 실행 및 조회
    VIEWER = "viewer"      # 조회만 가능

ROLE_PERMISSIONS = {
    Role.ADMIN: ["*"],
    Role.OPERATOR: [
        "workflow:execute",
        "workflow:read",
        "judgment:read",
    ],
    Role.VIEWER: [
        "workflow:read",
        "judgment:read",
        "dashboard:read",
    ],
}

def check_permission(user_role: Role, required_permission: str) -> bool:
    """권한 확인"""
    permissions = ROLE_PERMISSIONS[user_role]
    return "*" in permissions or required_permission in permissions
```

---

### 3.3 데이터 보안 분석

#### 민감 데이터 암호화
```python
# AES-256 암호화
from cryptography.fernet import Fernet

class DataEncryption:
    def __init__(self):
        self.key = os.getenv("ENCRYPTION_KEY")
        self.cipher = Fernet(self.key)

    def encrypt(self, data: str) -> str:
        """데이터 암호화"""
        return self.cipher.encrypt(data.encode()).decode()

    def decrypt(self, encrypted_data: str) -> str:
        """데이터 복호화"""
        return self.cipher.decrypt(encrypted_data.encode()).decode()

# 민감 데이터 필드
SENSITIVE_FIELDS = [
    "judgment_data.input_data.password",
    "judgment_data.input_data.api_key",
    "judgment_data.input_data.secret",
]
```

---

## 4. 위험 분석 및 대응 전략

### 4.1 일정 위험

| 위험 요소 | 확률 | 영향 | 심각도 | 대응 전략 |
|----------|------|------|--------|----------|
| **LLM API 변경** | 🟡 중간 | 🔴 높음 | 🟡 중간 | OpenAI API 버전 고정, 대체 API 준비 |
| **pgvector 성능 저하** | 🟡 중간 | 🟡 중간 | 🟡 중간 | 인덱싱 최적화, 샤딩 검토 |
| **서비스 간 의존성** | 🟢 낮음 | 🟡 중간 | 🟢 낮음 | Circuit Breaker, Fallback |
| **일정 지연** | 🟡 중간 | 🟡 중간 | 🟡 중간 | 단계적 출시, MVP 우선 |

#### 대응 전략: OpenAI API 버전 고정
```python
# OpenAI API 버전 고정
OPENAI_API_VERSION = "2023-05-15"  # 특정 버전 고정

client = OpenAI(
    api_key=os.getenv("OPENAI_API_KEY"),
    default_headers={"OpenAI-Version": OPENAI_API_VERSION}
)
```

---

### 4.2 기술 위험

| 기술 | 위험 | 확률 | 대응 |
|------|------|------|------|
| **FastAPI** | 버전 호환성 문제 | 🟢 낮음 | Poetry로 버전 고정 |
| **PostgreSQL** | 데이터 손실 | 🟢 낮음 | 자동 백업 + 복제본 |
| **pgvector** | 성능 저하 | 🟡 중간 | HNSW 인덱싱 |
| **Redis** | 메모리 부족 | 🟡 중간 | TTL 설정 + LRU 정책 |

---

### 4.3 비즈니스 위험

#### 비용 분석 (월 기준)

| 항목 | 예상 비용 | 비고 |
|------|----------|------|
| **OpenAI API** | $500-2000 | 사용량에 따라 가변 |
| **PostgreSQL (AWS RDS)** | $200-500 | db.t3.medium 기준 |
| **Redis (AWS ElastiCache)** | $100-200 | cache.t3.medium 기준 |
| **Kubernetes (AWS EKS)** | $300-600 | 3-6 노드 기준 |
| **총 예상 비용** | **$1100-3300** | 초기 단계 |

#### 대응 전략: 비용 최적화
```python
# LLM 호출 최적화
class CostOptimizer:
    async def should_use_llm(self, input_data: dict, rule_result: RuleResult) -> bool:
        """
        LLM 호출 여부 결정 (비용 절감)
        - Rule 성공 + 높은 신뢰도 → LLM 불필요
        - Rule 실패 또는 낮은 신뢰도 → LLM 필요
        """
        if rule_result.success and rule_result.confidence >= 0.8:
            return False  # LLM 생략 (비용 절감)

        return True  # LLM 호출

    async def use_cheaper_model(self, complexity: float) -> str:
        """
        복잡도에 따라 모델 선택
        - 단순한 케이스: gpt-3.5-turbo (저비용)
        - 복잡한 케이스: gpt-4 (고비용, 고정확도)
        """
        if complexity < 0.5:
            return "gpt-3.5-turbo"  # 20배 저렴
        else:
            return "gpt-4"
```

---

## 5. 모니터링 및 알림 전략

### 5.1 핵심 메트릭

#### 비즈니스 메트릭
```python
# Prometheus 메트릭 정의
from prometheus_client import Counter, Histogram, Gauge

# 판단 실행 카운터
judgment_executions_total = Counter(
    'judgment_executions_total',
    'Total number of judgment executions',
    ['method', 'result', 'workflow_id']
)

# 판단 실행 시간
judgment_execution_duration = Histogram(
    'judgment_execution_duration_seconds',
    'Duration of judgment execution',
    ['method']
)

# 판단 신뢰도 점수
judgment_confidence_score = Gauge(
    'judgment_confidence_score',
    'Average confidence score of judgments',
    ['workflow_id']
)

# LLM API 비용
llm_api_cost_total = Counter(
    'llm_api_cost_dollars',
    'Total LLM API cost in dollars'
)
```

#### 시스템 메트릭
- CPU 사용률
- 메모리 사용률
- API 응답 시간
- 에러율

---

### 5.2 알림 규칙

```yaml
# Prometheus Alertmanager 규칙
groups:
  - name: judgify_alerts
    rules:
      # 판단 실행 실패율 50% 이상
      - alert: HighJudgmentFailureRate
        expr: rate(judgment_executions_total{result="failed"}[5m]) > 0.5
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High judgment failure rate"

      # API 응답 시간 5초 이상
      - alert: SlowAPIResponse
        expr: histogram_quantile(0.95, rate(judgment_execution_duration_seconds_bucket[5m])) > 5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "API response time > 5 seconds"

      # LLM API 비용 $1000 초과
      - alert: HighLLMCost
        expr: llm_api_cost_dollars > 1000
        labels:
          severity: info
        annotations:
          summary: "LLM API cost exceeds $1000"
```

---

## 6. 재해 복구 계획 (Disaster Recovery)

### 6.1 백업 전략

| 대상 | 백업 주기 | 보관 기간 | 복구 목표 (RTO) |
|------|----------|----------|----------------|
| **PostgreSQL** | 매일 자정 | 30일 | 1시간 |
| **Redis** | 매 6시간 | 7일 | 30분 |
| **설정 파일** | Git 커밋시 | 영구 | 즉시 |
| **코드** | Git 푸시시 | 영구 | 즉시 |

### 6.2 장애 복구 절차

```bash
#!/bin/bash
# 재해 복구 스크립트

# 1. 최신 백업 확인
LATEST_BACKUP=$(ls -t /backups/*.sql.gz | head -1)
echo "Latest backup: $LATEST_BACKUP"

# 2. PostgreSQL 복구
gunzip -c $LATEST_BACKUP | psql -U judgify -d judgify_core

# 3. Redis 복구
redis-cli --rdb /backups/dump.rdb

# 4. Kubernetes 재배포
kubectl rollout restart deployment/judgment-service
kubectl rollout restart deployment/learning-service
kubectl rollout restart deployment/bi-service

# 5. 헬스체크
for service in judgment learning bi; do
    curl -f http://$service-service:800X/health || echo "$service health check failed"
done
```

---

## ✅ 분석 요약 및 권장사항

### 🎯 즉시 조치 필요 (P0)
1. ✅ **AST whitelist 구현** - 보안 최우선
2. ✅ **Redis 캐싱 전략** - 성능 최적화
3. ✅ **pgvector HNSW 인덱싱** - Few-shot 검색 성능
4. ✅ **Circuit Breaker 패턴** - 서비스 안정성

### 🟡 조기 구현 권장 (P1)
5. Prometheus + Grafana 모니터링
6. PostgreSQL 읽기 복제본
7. Kubernetes HPA 설정
8. LLM API 비용 최적화

### 🟢 점진적 개선 (P2)
9. ELK Stack 로그 분석
10. 재해 복구 자동화
11. A/B 테스트 프레임워크
12. 성능 벤치마킹 도구

---

**분석 완료일**: 2025-10-20
**분석자**: Claude (AI Engineer + Architect)
**다음 단계**: /speckit.implement - 실제 구현 시작
