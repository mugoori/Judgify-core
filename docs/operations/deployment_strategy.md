# Judgify-core Ver2.0 배포 및 운영 전략

**문서 버전**: v2.0  
**작성일**: 2024.08.10  
**대상**: DevOps 엔지니어, SRE, 플랫폼 엔지니어  
**목적**: 마이크로서비스 기반 AI 판단 플랫폼의 운영 환경 배포 및 관리 전략

---

## 📋 1. 배포 전략 개요

### 1.1 배포 아키텍처 원칙

- **마이크로서비스 독립 배포**: 각 서비스별 독립적인 배포 파이프라인
- **컨테이너 우선**: Docker 기반 컨테이너화로 환경 일관성 보장
- **Infrastructure as Code**: 모든 인프라 구성을 코드로 관리
- **점진적 배포**: Blue-Green, Canary 배포를 통한 무중단 서비스
- **자동화 우선**: 수동 개입 최소화로 인적 오류 방지

### 1.2 서비스 포트 매핑 및 구성

| 서비스 | 포트 | 역할 | 의존성 |
|--------|------|------|--------|
| **API Gateway** | 8000 | JWT 인증 + 라우팅 | Kong/Nginx, Redis |
| **Workflow Service** | 8001 | 워크플로우 CRUD | PostgreSQL, Redis |
| **Judgment Service** | 8002 | 하이브리드 판단 엔진 | PostgreSQL, Redis, OpenAI |
| **Action Service** | 8003 | 외부 시스템 연동 | PostgreSQL, Celery, MCP |
| **Logging Service** | 8005 | 중앙집중 로그 관리 | PostgreSQL, ELK Stack |
| **Dashboard Service** | 8006 | React 자동 생성 | PostgreSQL, Redis, LLM |

---

## 🐳 2. Docker 컨테이너화 전략

### 2.1 멀티스테이지 빌드 전략

```dockerfile
# 공통 베이스 이미지 전략
FROM python:3.11-slim as base
RUN apt-get update && apt-get install -y \
    curl \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# 빌드 스테이지
FROM base as builder
COPY requirements.txt .
RUN pip wheel --no-cache-dir --no-deps --wheel-dir /usr/src/app/wheels -r requirements.txt

# 런타임 스테이지  
FROM base as runtime
COPY --from=builder /usr/src/app/wheels /wheels
RUN pip install --no-cache /wheels/*
WORKDIR /app
COPY . .
EXPOSE 8000
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1
```

### 2.2 서비스별 최적화 전략

#### API Gateway (Kong 기반)
- **이미지**: `kong:3.4-alpine`
- **최적화**: 플러그인 선택적 로딩, 메모리 사용량 최소화
- **헬스체크**: `/status` 엔드포인트 활용

#### FastAPI 서비스들 (Workflow, Judgment, Action, Logging, Dashboard)
- **베이스 이미지**: `python:3.11-alpine`  
- **최적화**: 
  - 멀티스테이지 빌드로 이미지 크기 60% 절약
  - Alpine Linux로 보안 취약점 최소화
  - 비루트 유저로 실행 (보안 강화)
- **헬스체크**: `/health` 엔드포인트 통합

#### Frontend (Next.js 14)
- **베이스 이미지**: `node:18-alpine`
- **최적화**: 
  - 정적 빌드 최적화 (빌드 시간 40% 단축)
  - 멀티스테이지로 dev dependencies 제거
  - Nginx 프록시와 통합

### 2.3 Docker Compose 개발 환경

```yaml
# docker-compose.dev.yml 핵심 구조
version: '3.8'

services:
  # 인프라 서비스
  postgres:
    image: pgvector/pgvector:pg15
    environment:
      POSTGRES_DB: judgify
      POSTGRES_USER: judgify
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./sql/init.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "5432:5432"
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U judgify -d judgify"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes

  # 마이크로서비스
  api-gateway:
    build: 
      context: ./docker/services/api-gateway
      dockerfile: Dockerfile
    ports:
      - "8000:8000"
    depends_on:
      - redis
    environment:
      - REDIS_URL=redis://redis:6379/0
    volumes:
      - ./kong/kong.conf:/etc/kong/kong.conf

  workflow-service:
    build:
      context: ./services/workflow
      dockerfile: Dockerfile
    ports:
      - "8001:8001" 
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_started
    environment:
      - DATABASE_URL=postgresql://judgify:${POSTGRES_PASSWORD}@postgres:5432/judgify
      - REDIS_URL=redis://redis:6379/1

  # ... 기타 서비스들
```

---

## ☸️ 3. Kubernetes 배포 전략

### 3.1 클러스터 아키텍처

```yaml
# 네임스페이스 전략
apiVersion: v1
kind: Namespace
metadata:
  name: judgify-prod
  labels:
    environment: production
    project: judgify-core
---
apiVersion: v1  
kind: Namespace
metadata:
  name: judgify-staging
  labels:
    environment: staging
    project: judgify-core
```

### 3.2 배포 전략별 구성

#### Blue-Green 배포 전략
- **적용 대상**: Judgment Service (핵심 서비스)
- **이유**: 무중단 서비스가 가장 중요한 핵심 판단 로직
- **구현**: Kubernetes Service의 selector 변경을 통한 트래픽 전환

```yaml
# Blue-Green 배포 예시
apiVersion: apps/v1
kind: Deployment
metadata:
  name: judgment-service-blue
  namespace: judgify-prod
spec:
  replicas: 3
  selector:
    matchLabels:
      app: judgment-service
      version: blue
  template:
    metadata:
      labels:
        app: judgment-service
        version: blue
    spec:
      containers:
      - name: judgment-service
        image: judgify/judgment-service:v2.0.0
        ports:
        - containerPort: 8002
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
        readinessProbe:
          httpGet:
            path: /ready
            port: 8002
          initialDelaySeconds: 5
          periodSeconds: 5
```

#### Canary 배포 전략
- **적용 대상**: Dashboard Service, Workflow Service
- **이유**: 신기능의 점진적 검증이 중요한 서비스
- **구현**: Istio Service Mesh를 통한 트래픽 비율 제어

```yaml
# Canary 배포 설정 (Istio)
apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: dashboard-service
spec:
  http:
  - match:
    - headers:
        canary:
          exact: "true"
    route:
    - destination:
        host: dashboard-service
        subset: canary
  - route:
    - destination:
        host: dashboard-service
        subset: stable
      weight: 90
    - destination:
        host: dashboard-service  
        subset: canary
      weight: 10
```

#### Rolling Update 배포 전략  
- **적용 대상**: API Gateway, Action Service, Logging Service
- **이유**: 상대적으로 안정적인 서비스들
- **구현**: Kubernetes 기본 Rolling Update

### 3.3 리소스 할당 전략

| 서비스 | CPU Request | CPU Limit | Memory Request | Memory Limit | 복제본 수 |
|--------|-------------|-----------|----------------|--------------|-----------|
| **API Gateway** | 100m | 200m | 128Mi | 256Mi | 3 |
| **Workflow Service** | 200m | 400m | 256Mi | 512Mi | 3 |
| **Judgment Service** | 300m | 600m | 512Mi | 1Gi | 5 |
| **Action Service** | 200m | 400m | 256Mi | 512Mi | 3 |
| **Logging Service** | 150m | 300m | 256Mi | 512Mi | 3 |
| **Dashboard Service** | 250m | 500m | 384Mi | 768Mi | 3 |

### 3.4 Persistent Volume 전략

```yaml
# PostgreSQL Persistent Volume
apiVersion: v1
kind: PersistentVolumeClaim  
metadata:
  name: postgres-pvc
  namespace: judgify-prod
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: fast-ssd
  resources:
    requests:
      storage: 100Gi
---
# Redis Persistent Volume
apiVersion: v1  
kind: PersistentVolumeClaim
metadata:
  name: redis-pvc
  namespace: judgify-prod
spec:
  accessModes:
    - ReadWriteOnce
  storageClassName: fast-ssd
  resources:
    requests:
      storage: 20Gi
```

---

## 🎛️ 4. 환경 관리 전략

### 4.1 환경별 구성

#### 개발 환경 (Development)
- **목적**: 개발자 로컬 개발 및 단위 테스트
- **구성**: Docker Compose 기반
- **데이터**: 합성 데이터 (Faker 라이브러리 활용)
- **외부 연동**: 모든 외부 연동 Mock 처리
- **리소스**: 최소 사양 (8GB RAM, 4 CPU 권장)

#### 스테이징 환경 (Staging)  
- **목적**: 통합 테스트 및 UAT (User Acceptance Test)
- **구성**: Kubernetes 클러스터 (단일 노드)
- **데이터**: 운영 데이터의 익명화된 복사본
- **외부 연동**: 일부 실제 연동 (테스트 API 키 사용)
- **리소스**: 운영 환경의 50% 규모

#### 운영 환경 (Production)
- **목적**: 실제 서비스 제공
- **구성**: 고가용성 Kubernetes 클러스터 (Multi-AZ)
- **데이터**: 실제 운영 데이터
- **외부 연동**: 모든 실제 연동
- **리소스**: 고가용성 및 확장성 고려한 사양

### 4.2 설정 관리 전략

#### ConfigMap을 통한 애플리케이션 설정

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
  namespace: judgify-prod
data:
  # 판단 엔진 설정
  judgment.yaml: |
    engine:
      rule_confidence_threshold: 0.7
      llm_fallback_enabled: true
      max_judgment_time_seconds: 30
      
  # 대시보드 설정  
  dashboard.yaml: |
    auto_generation:
      default_chart_types: ["bar", "line", "pie"]
      max_components_per_dashboard: 12
      cache_ttl_minutes: 30
      
  # 외부 연동 설정
  integrations.yaml: |
    mcp:
      timeout_seconds: 10
      retry_count: 3
    openai:
      model: "gpt-4"
      max_tokens: 1000
      temperature: 0.3
```

#### Secret을 통한 민감 정보 관리

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
  namespace: judgify-prod
type: Opaque
stringData:
  postgres-connection-string: "postgresql://judgify:${POSTGRES_PASSWORD}@postgres-service:5432/judgify"
  redis-url: "redis://redis-service:6379"
  openai-api-key: "${OPENAI_API_KEY}"
  jwt-secret-key: "${JWT_SECRET_KEY}"
  slack-webhook-url: "${SLACK_WEBHOOK_URL}"
```

---

## 🔄 5. CI/CD 파이프라인 전략

### 5.1 GitHub Actions Workflow 구조

#### 빌드 및 테스트 파이프라인

```yaml
# .github/workflows/ci.yml
name: CI Pipeline
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: pgvector/pgvector:pg15
        env:
          POSTGRES_PASSWORD: test
          POSTGRES_DB: judgify_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      
      redis:
        image: redis:7-alpine
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    strategy:
      matrix:
        service: [workflow, judgment, action, logging, dashboard]

    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.11'
        
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: ~/.cache/pip
        key: ${{ runner.os }}-pip-${{ hashFiles('services/${{ matrix.service }}/requirements.txt') }}
        
    - name: Install dependencies
      run: |
        cd services/${{ matrix.service }}
        pip install -r requirements.txt
        pip install -r requirements-dev.txt
        
    - name: Run lint
      run: |
        cd services/${{ matrix.service }}
        flake8 .
        black --check .
        mypy .
        
    - name: Run unit tests
      run: |
        cd services/${{ matrix.service }}
        pytest tests/unit --cov=. --cov-report=xml --cov-fail-under=80
        
    - name: Run integration tests
      run: |
        cd services/${{ matrix.service }}
        pytest tests/integration --cov=. --cov-report=xml
        
    - name: Upload coverage reports
      uses: codecov/codecov-action@v3
      with:
        file: ./services/${{ matrix.service }}/coverage.xml
        flags: ${{ matrix.service }}
```

#### 배포 파이프라인

```yaml
# .github/workflows/deploy.yml
name: Deploy Pipeline
on:
  push:
    branches: [main]
    tags: ['v*']

jobs:
  build-and-push:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        service: [api-gateway, workflow, judgment, action, logging, dashboard, frontend]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v3
      
    - name: Login to Container Registry
      uses: docker/login-action@v3
      with:
        registry: ${{ secrets.CONTAINER_REGISTRY }}
        username: ${{ secrets.REGISTRY_USERNAME }}
        password: ${{ secrets.REGISTRY_PASSWORD }}
        
    - name: Build and push Docker image
      uses: docker/build-push-action@v5
      with:
        context: ./docker/services/${{ matrix.service }}
        platforms: linux/amd64,linux/arm64
        push: true
        tags: |
          ${{ secrets.CONTAINER_REGISTRY }}/judgify/${{ matrix.service }}:${{ github.sha }}
          ${{ secrets.CONTAINER_REGISTRY }}/judgify/${{ matrix.service }}:latest
        cache-from: type=gha
        cache-to: type=gha,mode=max

  deploy-staging:
    needs: build-and-push
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup kubectl
      uses: azure/setup-kubectl@v3
      with:
        version: 'v1.28.0'
        
    - name: Deploy to staging
      run: |
        kubectl config use-context staging-cluster
        helm upgrade --install judgify-staging ./deployments/helm/judgify-core \
          --namespace judgify-staging \
          --values ./deployments/helm/judgify-core/values-staging.yaml \
          --set image.tag=${{ github.sha }}
          
  deploy-production:
    needs: [build-and-push, deploy-staging]
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    environment: production
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup kubectl
      uses: azure/setup-kubectl@v3
      with:
        version: 'v1.28.0'
        
    - name: Blue-Green deployment to production
      run: |
        kubectl config use-context production-cluster
        
        # Blue-Green 배포 스크립트 실행
        ./scripts/blue-green-deploy.sh \
          --namespace judgify-prod \
          --image-tag ${{ github.sha }} \
          --service judgment-service
          
        # 기타 서비스는 Rolling Update
        helm upgrade --install judgify-prod ./deployments/helm/judgify-core \
          --namespace judgify-prod \
          --values ./deployments/helm/judgify-core/values-prod.yaml \
          --set image.tag=${{ github.sha }}
```

### 5.2 보안 및 품질 게이트

#### 보안 스캔 통합

```yaml
# 보안 스캔 작업 추가
  security-scan:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Run Trivy vulnerability scanner
      uses: aquasecurity/trivy-action@master
      with:
        image-ref: ${{ secrets.CONTAINER_REGISTRY }}/judgify/${{ matrix.service }}:${{ github.sha }}
        format: 'sarif'
        output: 'trivy-results.sarif'
        
    - name: Upload Trivy scan results
      uses: github/codeql-action/upload-sarif@v2
      with:
        sarif_file: 'trivy-results.sarif'
        
    - name: Run SAST scan
      uses: github/codeql-action/analyze@v2
      with:
        languages: python, javascript
```

#### 품질 게이트 설정

```yaml
  quality-gate:
    runs-on: ubuntu-latest
    steps:
    - name: SonarQube Scan
      uses: sonarqube-quality-gate-action@master
      env:
        SONAR_TOKEN: ${{ secrets.SONAR_TOKEN }}
      with:
        scanMetadataReportFile: target/sonar/report-task.txt
        
    - name: Quality Gate check
      run: |
        # 코드 커버리지 80% 이상 확인
        # 코드 중복률 3% 이하 확인  
        # 보안 취약점 Critical/High 0개 확인
        ./scripts/quality-gate-check.sh
```

---

## 📊 6. 모니터링 및 관찰가능성

### 6.1 Prometheus + Grafana 모니터링

#### 메트릭 수집 전략

```yaml
# prometheus-config.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
      evaluation_interval: 15s
      
    rule_files:
      - "judgment_engine_rules.yml"
      - "system_rules.yml"
      
    scrape_configs:
    # 각 마이크로서비스 메트릭 수집
    - job_name: 'judgment-service'
      static_configs:
      - targets: ['judgment-service:8002']
      scrape_interval: 10s
      metrics_path: /metrics
      
    - job_name: 'workflow-service'  
      static_configs:
      - targets: ['workflow-service:8001']
      
    - job_name: 'dashboard-service'
      static_configs:
      - targets: ['dashboard-service:8006']
      
    # 인프라 메트릭
    - job_name: 'postgres-exporter'
      static_configs:
      - targets: ['postgres-exporter:9187']
      
    - job_name: 'redis-exporter'
      static_configs:
      - targets: ['redis-exporter:9121']
```

#### 핵심 비즈니스 메트릭 정의

```python
# 각 서비스에 구현될 메트릭 (예시)
from prometheus_client import Counter, Histogram, Gauge

# Judgment Service 메트릭
judgment_requests_total = Counter(
    'judgment_requests_total',
    'Total number of judgment requests', 
    ['method', 'result', 'workflow_id']
)

judgment_duration_seconds = Histogram(
    'judgment_duration_seconds',
    'Duration of judgment execution',
    ['method']  # rule, llm, hybrid
)

judgment_confidence_score = Gauge(
    'judgment_confidence_score',
    'Average confidence score',
    ['workflow_id']
)

# Dashboard Service 메트릭
dashboard_generation_requests_total = Counter(
    'dashboard_generation_requests_total',
    'Total dashboard generation requests',
    ['status']
)

dashboard_generation_duration_seconds = Histogram(
    'dashboard_generation_duration_seconds', 
    'Dashboard generation time'
)

active_websocket_connections = Gauge(
    'active_websocket_connections',
    'Number of active WebSocket connections'
)
```

### 6.2 로깅 전략 (ELK Stack)

#### Elasticsearch 설정

```yaml
apiVersion: elasticsearch.k8s.elastic.co/v1
kind: Elasticsearch
metadata:
  name: judgify-elasticsearch
spec:
  version: 8.10.0
  nodeSets:
  - name: default
    count: 3
    config:
      node.store.allow_mmap: false
      xpack.security.enabled: true
    podTemplate:
      spec:
        containers:
        - name: elasticsearch
          resources:
            requests:
              memory: 2Gi
              cpu: 1000m
            limits:
              memory: 4Gi
              cpu: 2000m
    volumeClaimTemplates:
    - metadata:
        name: elasticsearch-data
      spec:
        accessModes:
        - ReadWriteOnce
        resources:
          requests:
            storage: 50Gi
        storageClassName: fast-ssd
```

#### Logstash 구조화 로그 파이프라인

```ruby
# logstash.conf
input {
  beats {
    port => 5044
  }
}

filter {
  if [fields][service] == "judgment-service" {
    json {
      source => "message"
    }
    
    # 판단 실행 로그 파싱
    if [event_type] == "judgment_executed" {
      mutate {
        add_field => { "[@metadata][index_prefix]" => "judgment-execution" }
      }
    }
    
    # 에러 로그 파싱
    if [level] == "ERROR" {
      mutate {
        add_field => { "[@metadata][index_prefix]" => "errors" }
      }
    }
  }
  
  # 타임스탬프 정규화
  date {
    match => [ "timestamp", "ISO8601" ]
  }
  
  # 민감 정보 마스킹
  mutate {
    gsub => [
      "message", "password=\w+", "password=***",
      "message", "api_key=\w+", "api_key=***"
    ]
  }
}

output {
  elasticsearch {
    hosts => ["elasticsearch-service:9200"]
    index => "%{[@metadata][index_prefix]}-%{+YYYY.MM.dd}"
    user => "${ELASTICSEARCH_USER}"
    password => "${ELASTICSEARCH_PASSWORD}"
  }
}
```

#### 구조화된 로깅 표준

```python
# 각 서비스에서 사용할 구조화된 로깅
import structlog
import logging

# 구조화된 로거 설정
structlog.configure(
    processors=[
        structlog.stdlib.filter_by_level,
        structlog.stdlib.add_logger_name,
        structlog.stdlib.add_log_level,
        structlog.stdlib.PositionalArgumentsFormatter(),
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.JSONRenderer()
    ],
    context_class=dict,
    logger_factory=structlog.stdlib.LoggerFactory(),
    wrapper_class=structlog.stdlib.BoundLogger,
    cache_logger_on_first_use=True,
)

logger = structlog.get_logger()

# 사용 예시
logger.info(
    "judgment_executed",
    workflow_id="wf-123",
    method="hybrid", 
    result=True,
    confidence=0.95,
    execution_time_ms=1250,
    user_id="user-456"
)
```

### 6.3 알림 및 인시던트 관리

#### Prometheus Alertmanager 설정

```yaml
# judgment_engine_rules.yml
groups:
- name: judgment_engine
  rules:
  
  # 판단 실패율 알림
  - alert: HighJudgmentFailureRate
    expr: rate(judgment_requests_total{result="error"}[5m]) / rate(judgment_requests_total[5m]) > 0.05
    for: 2m
    labels:
      severity: warning
      service: judgment-service
    annotations:
      summary: "High judgment failure rate detected"
      description: "Judgment failure rate is {{ $value }}% over the last 5 minutes"
      
  # 판단 응답 시간 알림  
  - alert: SlowJudgmentResponse
    expr: histogram_quantile(0.95, rate(judgment_duration_seconds_bucket[5m])) > 5
    for: 5m
    labels:
      severity: warning
      service: judgment-service
    annotations:
      summary: "Slow judgment response time"
      description: "95th percentile response time is {{ $value }}s"
      
  # 신뢰도 점수 하락 알림
  - alert: LowConfidenceScore
    expr: avg_over_time(judgment_confidence_score[10m]) < 0.6
    for: 3m
    labels:
      severity: critical
      service: judgment-service
    annotations:
      summary: "Low confidence score in judgments" 
      description: "Average confidence score dropped to {{ $value }}"

- name: system_resources
  rules:
  
  # 시스템 리소스 알림
  - alert: HighMemoryUsage
    expr: container_memory_usage_bytes / container_spec_memory_limit_bytes > 0.85
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High memory usage on {{ $labels.pod }}"
      
  - alert: HighCPUUsage  
    expr: rate(container_cpu_usage_seconds_total[5m]) > 0.8
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High CPU usage on {{ $labels.pod }}"
```

#### Slack/Teams 통합 알림

```yaml
# alertmanager.yml
global:
  slack_api_url: '${SLACK_WEBHOOK_URL}'

route:
  group_by: ['alertname', 'service']
  group_wait: 10s
  group_interval: 5m
  repeat_interval: 1h
  receiver: 'slack-notifications'
  routes:
  - match:
      severity: critical
    receiver: 'pagerduty-critical'
  - match:
      service: judgment-service
    receiver: 'judgment-team'

receivers:
- name: 'slack-notifications'
  slack_configs:
  - channel: '#alerts'
    color: '{{ if eq .Status "firing" }}danger{{ else }}good{{ end }}'
    title: 'Judgify Alert - {{ .GroupLabels.alertname }}'
    text: |
      {{ range .Alerts }}
      *Alert:* {{ .Annotations.summary }}
      *Description:* {{ .Annotations.description }}
      *Severity:* {{ .Labels.severity }}
      *Service:* {{ .Labels.service }}
      {{ end }}

- name: 'judgment-team'
  slack_configs:
  - channel: '#judgment-alerts'
    color: 'warning'
    title: 'Judgment Service Alert'
    text: 'Judgment service requires attention'

- name: 'pagerduty-critical'
  pagerduty_configs:
  - routing_key: '${PAGERDUTY_INTEGRATION_KEY}'
    description: 'Critical alert in Judgify system'
```

---

## 🔒 7. 보안 및 컴플라이언스

### 7.1 컨테이너 보안

#### 보안 강화된 Dockerfile 패턴

```dockerfile
# 보안 강화 Dockerfile 예시
FROM python:3.11-slim as base

# 보안 패키지 업데이트
RUN apt-get update && apt-get install -y \
    --no-install-recommends \
    curl \
    postgresql-client \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /tmp/*

# 비루트 유저 생성
RUN groupadd -r appuser && useradd -r -g appuser appuser

# 빌드 스테이지
FROM base as builder
COPY requirements.txt .
RUN pip install --no-cache-dir --user -r requirements.txt

# 런타임 스테이지
FROM base as runtime

# 비루트 유저로 전환
USER appuser
WORKDIR /app

# 컨테이너 내 쓰기 권한 최소화
COPY --chown=appuser:appuser --from=builder /root/.local /home/appuser/.local
COPY --chown=appuser:appuser . .

# PATH 환경변수 설정
ENV PATH=/home/appuser/.local/bin:$PATH

# 비특권 포트 사용
EXPOSE 8002

# 헬스체크 (비루트 유저로 실행)
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8002/health || exit 1

# 읽기 전용 루트 파일시스템
CMD ["python", "-m", "uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8002"]
```

#### Pod Security Standards

```yaml
# Pod Security Policy
apiVersion: policy/v1beta1
kind: PodSecurityPolicy
metadata:
  name: judgify-restricted
spec:
  privileged: false
  allowPrivilegeEscalation: false
  requiredDropCapabilities:
    - ALL
  volumes:
    - 'configMap'
    - 'emptyDir'
    - 'projected'
    - 'secret'
    - 'downwardAPI'
    - 'persistentVolumeClaim'
  runAsUser:
    rule: 'MustRunAsNonRoot'
  seLinux:
    rule: 'RunAsAny'
  fsGroup:
    rule: 'RunAsAny'
  readOnlyRootFilesystem: true
```

### 7.2 네트워크 보안

#### Network Policies

```yaml
# 네트워크 정책 예시
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: judgment-service-netpol
  namespace: judgify-prod
spec:
  podSelector:
    matchLabels:
      app: judgment-service
  policyTypes:
  - Ingress
  - Egress
  
  # 인그레스 규칙: API Gateway에서만 접근 허용
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: api-gateway
    ports:
    - protocol: TCP
      port: 8002
      
  # 이그레스 규칙: 필요한 외부 연결만 허용  
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
  # OpenAI API 호출
  - to: []
    ports:
    - protocol: TCP
      port: 443
```

### 7.3 비밀 정보 관리

#### External Secrets Operator를 통한 비밀 관리

```yaml
# external-secrets.yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: judgify-secrets
  namespace: judgify-prod
spec:
  refreshInterval: 1m
  secretStoreRef:
    name: vault-backend
    kind: SecretStore
  target:
    name: app-secrets
    creationPolicy: Owner
  data:
  - secretKey: postgres-password
    remoteRef:
      key: database/judgify/prod
      property: password
  - secretKey: openai-api-key
    remoteRef:
      key: apis/openai
      property: api_key
  - secretKey: jwt-secret-key  
    remoteRef:
      key: auth/jwt
      property: secret_key
```

---

## 🔄 8. 백업 및 재해 복구

### 8.1 데이터베이스 백업 전략

#### PostgreSQL 백업

```yaml
# postgres-backup-cronjob.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-backup
  namespace: judgify-prod
spec:
  schedule: "0 2 * * *"  # 매일 새벽 2시
  jobTemplate:
    spec:
      template:
        spec:
          restartPolicy: OnFailure
          containers:
          - name: postgres-backup
            image: postgres:15
            command:
            - /bin/bash
            - -c
            - |
              # 백업 파일 생성
              pg_dump -h postgres-service -U judgify -d judgify \
                --verbose --clean --no-owner --no-acl \
                --format=custom \
                > /backup/judgify_$(date +%Y%m%d_%H%M%S).dump
                
              # S3에 업로드
              aws s3 cp /backup/judgify_*.dump \
                s3://judgify-backups/postgres/$(date +%Y/%m/%d)/
                
              # 로컬 백업 정리 (7일 이상된 파일 삭제)
              find /backup -name "*.dump" -mtime +7 -delete
            env:
            - name: PGPASSWORD
              valueFrom:
                secretKeyRef:
                  name: app-secrets
                  key: postgres-password
            - name: AWS_ACCESS_KEY_ID
              valueFrom:
                secretKeyRef:
                  name: aws-secrets
                  key: access-key
            - name: AWS_SECRET_ACCESS_KEY
              valueFrom:
                secretKeyRef:
                  name: aws-secrets
                  key: secret-key
            volumeMounts:
            - name: backup-storage
              mountPath: /backup
          volumes:
          - name: backup-storage
            persistentVolumeClaim:
              claimName: backup-pvc
```

#### Redis 백업

```bash
#!/bin/bash
# redis-backup.sh
BACKUP_DIR="/backup/redis"
DATE=$(date +%Y%m%d_%H%M%S)

# Redis 데이터 덤프
redis-cli -h redis-service -p 6379 --rdb ${BACKUP_DIR}/redis_${DATE}.rdb

# 압축
gzip ${BACKUP_DIR}/redis_${DATE}.rdb

# S3 업로드
aws s3 cp ${BACKUP_DIR}/redis_${DATE}.rdb.gz \
  s3://judgify-backups/redis/$(date +%Y/%m/%d)/

# 로컬 정리 (3일 이상된 파일 삭제)
find ${BACKUP_DIR} -name "*.rdb.gz" -mtime +3 -delete
```

### 8.2 재해 복구 계획

#### RTO/RPO 목표

| 서비스 | RTO (Recovery Time Objective) | RPO (Recovery Point Objective) |
|--------|-------------------------------|--------------------------------|
| **Judgment Service** | 15분 | 5분 |
| **Workflow Service** | 30분 | 15분 |
| **Dashboard Service** | 1시간 | 30분 |
| **기타 서비스** | 1시간 | 30분 |

#### 복구 절차

```bash
#!/bin/bash
# disaster-recovery.sh

echo "=== Judgify 재해 복구 스크립트 ==="

# 1. 클러스터 상태 확인
kubectl cluster-info
kubectl get nodes

# 2. 네임스페이스 생성
kubectl create namespace judgify-recovery

# 3. 시크릿 복구
kubectl apply -f ./recovery/secrets/

# 4. 데이터베이스 복구 (가장 최근 백업)
LATEST_BACKUP=$(aws s3 ls s3://judgify-backups/postgres/ --recursive | sort | tail -1 | awk '{print $4}')
echo "복구할 백업: $LATEST_BACKUP"

# PostgreSQL 인스턴스 생성
kubectl apply -f ./recovery/postgres-recovery.yaml

# 백업 복구 Job 실행
kubectl apply -f - <<EOF
apiVersion: batch/v1
kind: Job
metadata:
  name: postgres-restore
  namespace: judgify-recovery
spec:
  template:
    spec:
      containers:
      - name: restore
        image: postgres:15
        command: ["/bin/bash", "-c"]
        args:
        - |
          # S3에서 백업 다운로드
          aws s3 cp s3://judgify-backups/$LATEST_BACKUP /tmp/backup.dump
          
          # 데이터베이스 복구
          pg_restore -h postgres-service -U judgify -d judgify \
            --verbose --clean --if-exists /tmp/backup.dump
        env:
        - name: PGPASSWORD
          valueFrom:
            secretKeyRef:
              name: app-secrets
              key: postgres-password
      restartPolicy: Never
EOF

# 5. 서비스 복구 (우선순위 순)
echo "서비스 복구 시작..."

# 우선순위 1: Judgment Service
kubectl apply -f ./recovery/judgment-service.yaml
kubectl wait --for=condition=ready pod -l app=judgment-service --timeout=300s

# 우선순위 2: API Gateway
kubectl apply -f ./recovery/api-gateway.yaml
kubectl wait --for=condition=ready pod -l app=api-gateway --timeout=300s

# 우선순위 3: 나머지 서비스들
for service in workflow action dashboard logging; do
  kubectl apply -f ./recovery/${service}-service.yaml
  kubectl wait --for=condition=ready pod -l app=${service}-service --timeout=300s
done

# 6. 헬스체크 및 검증
./scripts/health-check.sh

echo "=== 재해 복구 완료 ==="
```

---

## 📈 9. 확장성 및 성능 최적화

### 9.1 Horizontal Pod Autoscaler (HPA)

```yaml
# judgment-service-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: judgment-service-hpa
  namespace: judgify-prod
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: judgment-service
  minReplicas: 3
  maxReplicas: 20
  metrics:
  # CPU 사용률 기반 스케일링
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
        
  # 메모리 사용률 기반 스케일링
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
        
  # 커스텀 메트릭: 판단 요청 큐 길이
  - type: Pods
    pods:
      metric:
        name: judgment_queue_length
      target:
        type: AverageValue
        averageValue: "10"
        
  # 커스텀 메트릭: 판단 응답 시간
  - type: Pods
    pods:
      metric:
        name: judgment_response_time_p95
      target:
        type: AverageValue
        averageValue: "3"

  behavior:
    scaleUp:
      stabilizationWindowSeconds: 60
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
      - type: Pods
        value: 2
        periodSeconds: 60
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 25
        periodSeconds: 60
```

### 9.2 Vertical Pod Autoscaler (VPA)

```yaml
# judgment-service-vpa.yaml
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: judgment-service-vpa
  namespace: judgify-prod
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: judgment-service
  updatePolicy:
    updateMode: "Auto"
  resourcePolicy:
    containerPolicies:
    - containerName: judgment-service
      maxAllowed:
        cpu: 2000m
        memory: 4Gi
      minAllowed:
        cpu: 100m
        memory: 256Mi
      controlledResources: ["cpu", "memory"]
```

### 9.3 클러스터 오토스케일러

```yaml
# cluster-autoscaler.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cluster-autoscaler
  namespace: kube-system
spec:
  selector:
    matchLabels:
      app: cluster-autoscaler
  template:
    metadata:
      labels:
        app: cluster-autoscaler
    spec:
      containers:
      - image: k8s.gcr.io/autoscaling/cluster-autoscaler:v1.21.0
        name: cluster-autoscaler
        resources:
          limits:
            cpu: 100m
            memory: 300Mi
          requests:
            cpu: 100m
            memory: 300Mi
        command:
        - ./cluster-autoscaler
        - --v=4
        - --stderrthreshold=info
        - --cloud-provider=aws
        - --skip-nodes-with-local-storage=false
        - --expander=least-waste
        - --node-group-auto-discovery=asg:tag=k8s.io/cluster-autoscaler/enabled,k8s.io/cluster-autoscaler/judgify-cluster
        - --balance-similar-node-groups
        - --scale-down-enabled=true
        - --scale-down-delay-after-add=2m
        - --scale-down-unneeded-time=5m
        - --max-node-provision-time=15m
```

---

## 🔍 10. 문제 해결 가이드

### 10.1 일반적인 문제와 해결책

#### 서비스 시작 실패

```bash
# 문제 진단
kubectl describe pod -l app=judgment-service
kubectl logs -l app=judgment-service --previous

# 일반적인 원인과 해결책
# 1. 이미지 풀링 실패
kubectl get events | grep "Failed to pull image"
# -> 이미지 태그 확인, 레지스트리 접근 권한 확인

# 2. ConfigMap/Secret 마운트 실패  
kubectl get configmap -n judgify-prod
kubectl get secret -n judgify-prod
# -> 누락된 설정 파일 확인

# 3. 리소스 부족
kubectl top nodes
kubectl top pods
# -> 노드 리소스 상황 확인, 필요시 스케일링
```

#### 데이터베이스 연결 실패

```bash
# PostgreSQL 연결 테스트
kubectl run psql-test --image=postgres:15 --rm -it --restart=Never -- \
  psql -h postgres-service -U judgify -d judgify

# Redis 연결 테스트  
kubectl run redis-test --image=redis:7-alpine --rm -it --restart=Never -- \
  redis-cli -h redis-service -p 6379 ping

# 네트워크 정책 확인
kubectl get networkpolicy -n judgify-prod
kubectl describe networkpolicy judgment-service-netpol
```

#### 메모리 부족 (OOMKilled)

```bash
# 메모리 사용량 분석
kubectl top pods | grep judgment-service
kubectl describe pod -l app=judgment-service | grep -A 10 "Limits:"

# 해결책
# 1. 메모리 제한 증가
kubectl patch deployment judgment-service -p='{"spec":{"template":{"spec":{"containers":[{"name":"judgment-service","resources":{"limits":{"memory":"2Gi"}}}]}}}}'

# 2. 메모리 누수 확인 (애플리케이션 수준)
kubectl exec -it deployment/judgment-service -- \
  python -c "import psutil; print(f'Memory: {psutil.virtual_memory().percent}%')"
```

### 10.2 성능 문제 해결

#### 응답 시간 증가

```bash
# 메트릭 확인
curl -s http://prometheus-service:9090/api/v1/query?query=histogram_quantile\(0.95,rate\(judgment_duration_seconds_bucket\[5m\]\)\)

# APM 도구를 통한 상세 분석
# Jaeger 추적
kubectl port-forward svc/jaeger-query 16686:16686
# http://localhost:16686 에서 분산 추적 분석

# 데이터베이스 성능 확인
kubectl exec -it postgres-0 -- \
  psql -U judgify -d judgify -c "
  SELECT query, calls, total_time, mean_time 
  FROM pg_stat_statements 
  ORDER BY mean_time DESC 
  LIMIT 10;"
```

#### 트래픽 급증 대응

```bash
# 현재 부하 상황 확인
kubectl top nodes
kubectl top pods -l app=judgment-service

# HPA 상태 확인
kubectl get hpa judgment-service-hpa -o yaml

# 수동 스케일링 (응급)
kubectl scale deployment judgment-service --replicas=10

# 트래픽 패턴 분석 (Prometheus)
curl "http://prometheus:9090/api/v1/query_range?query=rate(judgment_requests_total[5m])&start=$(date -d '1 hour ago' +%s)&end=$(date +%s)&step=60s"
```

### 10.3 보안 문제 대응

#### 보안 취약점 발견 시

```bash
# 컨테이너 이미지 스캔
trivy image judgify/judgment-service:v2.0.0

# 실행 중인 컨테이너 스캔  
kubectl get pods -l app=judgment-service -o jsonpath='{.items[0].spec.containers[0].image}' | xargs trivy image

# 보안 정책 확인
kubectl get podsecuritypolicy
kubectl auth can-i create pods --as=system:serviceaccount:judgify-prod:default

# 네트워크 트래픽 분석
kubectl exec -it deployment/judgment-service -- netstat -tuln
```

#### 비정상 트래픽 차단

```bash
# Ingress에서 IP 차단
kubectl patch ingress api-gateway --patch '
{
  "metadata": {
    "annotations": {
      "nginx.ingress.kubernetes.io/configuration-snippet": "
        deny 192.168.1.100;
        deny 10.0.0.0/8;
      "
    }
  }
}'

# Rate Limiting 적용
kubectl patch ingress api-gateway --patch '
{
  "metadata": {
    "annotations": {
      "nginx.ingress.kubernetes.io/rate-limit": "100",
      "nginx.ingress.kubernetes.io/rate-limit-window": "1m"  
    }
  }
}'
```

---

## 📝 11. 운영 체크리스트

### 11.1 배포 전 체크리스트

#### 개발 환경 검증
- [ ] 모든 서비스 로컬 Docker Compose 정상 실행
- [ ] 단위 테스트 90% 이상 커버리지 달성
- [ ] 통합 테스트 시나리오 통과
- [ ] API 문서 최신화 (OpenAPI/Swagger)
- [ ] 보안 스캔 Critical/High 이슈 0건

#### 스테이징 환경 검증
- [ ] 전체 워크플로우 E2E 테스트 통과
- [ ] 성능 테스트 기준 충족 (응답시간 < 5초)
- [ ] 부하 테스트 1000 concurrent users 처리
- [ ] 장애 시나리오 테스트 (카오스 엔지니어링)
- [ ] 모니터링 대시보드 정상 작동

#### 운영 환경 준비
- [ ] 백업 시스템 정상 작동 확인
- [ ] 롤백 계획 수립 및 검증
- [ ] 운영팀 배포 가이드 공유
- [ ] 장애 대응 매뉴얼 업데이트
- [ ] 사용자 공지사항 준비

### 11.2 배포 후 체크리스트

#### 즉시 확인 (배포 후 10분)
- [ ] 모든 서비스 Pod 정상 시작
- [ ] 헬스체크 엔드포인트 정상 응답
- [ ] 핵심 API 기능 테스트 통과
- [ ] 실시간 모니터링 메트릭 정상
- [ ] 에러 로그 확인 (Critical 없음)

#### 단기 확인 (배포 후 1시간)
- [ ] 전체 사용자 워크플로우 정상 작동
- [ ] 응답 시간 목표 달성 (95% < 5초)
- [ ] 메모리/CPU 사용량 정상 범위
- [ ] 데이터베이스 성능 지표 양호
- [ ] 외부 연동 시스템 정상 통신

#### 중기 확인 (배포 후 24시간)
- [ ] 비즈니스 메트릭 정상 (판단 성공률 등)
- [ ] 자동 스케일링 정상 작동
- [ ] 백업 작업 정상 수행
- [ ] 보안 이벤트 없음
- [ ] 사용자 피드백 수집

### 11.3 정기 운영 작업

#### 일간 작업
- [ ] 시스템 헬스 상태 점검
- [ ] 에러 로그 분석 및 대응  
- [ ] 백업 상태 확인
- [ ] 보안 이벤트 모니터링
- [ ] 성능 지표 리뷰

#### 주간 작업
- [ ] 보안 업데이트 적용
- [ ] 용량 계획 검토
- [ ] 성능 트렌드 분석
- [ ] 장애 대응 훈련
- [ ] 문서 업데이트

#### 월간 작업
- [ ] 재해 복구 테스트
- [ ] 보안 감사
- [ ] 용량 최적화
- [ ] SLA 리포트 작성
- [ ] 아키텍처 리뷰

---

## 🚀 12. 결론 및 다음 단계

### 12.1 핵심 성공 지표

| 지표 | 목표 | 측정 방법 |
|------|------|-----------|
| **가용성** | 99.5% | Prometheus 헬스체크 메트릭 |
| **응답 시간** | 95% < 5초 | API 응답 시간 히스토그램 |
| **판단 정확도** | 95% | 비즈니스 메트릭 대시보드 |
| **배포 성공률** | 99% | CI/CD 파이프라인 메트릭 |
| **보안 취약점** | Critical 0건 | 보안 스캔 도구 결과 |

### 12.2 지속적 개선 계획

#### Phase 1: 안정화 (1-3개월)
- 운영 환경 안정화 및 모니터링 고도화
- 성능 최적화 및 용량 계획 수립
- 보안 강화 및 컴플라이언스 준수

#### Phase 2: 확장 (3-6개월)  
- 멀티 리전 배포 및 CDN 연동
- AI/ML 모델 성능 향상
- 고급 관찰가능성 도구 도입 (분산 추적 등)

#### Phase 3: 혁신 (6-12개월)
- 서버리스 아키텍처로 일부 전환
- GitOps 기반 배포 자동화
- 카오스 엔지니어링 정규화

### 12.3 팀 역량 강화

#### 필수 스킬
- Kubernetes 운영 및 문제 해결
- Prometheus/Grafana 모니터링 시스템
- Docker 컨테이너 최적화
- CI/CD 파이프라인 관리
- 보안 취약점 분석 및 대응

#### 추천 교육 과정
- CKA (Certified Kubernetes Administrator)
- Prometheus 모니터링 전문가 과정
- Docker 컨테이너 보안 과정
- SRE (Site Reliability Engineering) 교육
- 클라우드 네이티브 보안 교육

---

**이 문서는 Judgify-core Ver2.0의 안정적이고 확장 가능한 운영을 위한 포괄적인 가이드입니다. 지속적인 업데이트와 개선을 통해 최고 수준의 서비스 품질을 달성해 나가겠습니다.**

---

**문서 히스토리**
- v1.0: 초기 배포 전략 수립 (2024.08.10)
- 향후 운영 경험을 바탕으로 지속적 업데이트 예정