# Judgify-core Ver2.0 배포 및 운영 가이드 📚

**문서 버전**: v2.0
**작성일**: 2024.08.10
**최종 업데이트**: 2024-11-XX
**대상**: DevOps 엔지니어, SRE, 플랫폼 엔지니어, 운영팀

이 가이드는 Judgify-core Ver2.0 마이크로서비스 기반 AI 판단 플랫폼의 배포 전략, 운영 환경 구성, 그리고 일상적인 배포 절차를 통합적으로 다룹니다.

---

## 📑 문서 구성

1. **배포 전략 및 아키텍처** - 전체 배포 전략, 인프라 구성, CI/CD 파이프라인
2. **배포 런북** - 자동/수동 배포 절차, Blue-Green 배포, 롤백 절차
3. **모니터링 및 관찰가능성** - Prometheus/Grafana, ELK Stack, 알림 시스템
4. **보안 및 컴플라이언스** - 컨테이너 보안, 네트워크 정책, 비밀 정보 관리
5. **백업 및 재해 복구** - 데이터베이스 백업, 재해 복구 계획
6. **운영 체크리스트** - 배포 전/중/후 체크리스트, 정기 운영 작업

---

# 1. 배포 전략 및 아키텍처

## 1.1 배포 전략 개요

### 1.1.1 배포 아키텍처 원칙
- **마이크로서비스 독립 배포**: 각 서비스별 독립적인 배포 파이프라인
- **컨테이너 우선**: Docker 기반 컨테이너화로 환경 일관성 보장
- **Infrastructure as Code**: 모든 인프라 구성을 코드로 관리
- **점진적 배포**: Blue-Green, Canary 배포를 통한 무중단 서비스
- **자동화 우선**: 수동 개입 최소화로 인적 오류 방지

### 1.1.2 배포 방식 및 주기
- **기본 전략**: Blue-Green 배포 (무중단 배포)
- **롤백 전략**: 즉시 Blue 환경으로 트래픽 복귀
- **배포 주기**: 주 1회 정기 배포 (화요일 02:00-06:00)
- **긴급 배포**: 필요시 언제든 가능

### 1.1.3 환경별 배포 순서
```
1. Development (자동) →
2. Staging (자동) →
3. Production (수동 승인)
```

### 1.1.4 서비스 포트 매핑 및 구성 (Ver2.0 Final - 9 services)

| 서비스 | 포트 | 역할 | 의존성 | 배포 우선순위 |
|--------|------|------|--------|--------------|
| **API Gateway** | 8000 | JWT 인증 + 라우팅 | Kong/Nginx, Redis | Critical |
| **Workflow Service** | 8001 | Visual Workflow Builder | PostgreSQL, Redis | Important |
| **Judgment Service** | 8002 | 하이브리드 판단 엔진 | PostgreSQL, Redis, OpenAI | Critical |
| **Action Service** | 8003 | 외부 시스템 연동 | PostgreSQL, Celery, MCP | Supporting |
| **Notification Service** | 8004 | Slack/Teams/Email | PostgreSQL, Message Queue | Supporting |
| **Logging Service** | 8005 | 중앙집중 로그 관리 | PostgreSQL, ELK Stack | Supporting |
| **Data Visualization Service** | 8006 | 단순 데이터 대시보드 | PostgreSQL, Redis | Important |
| **BI Service** | 8007 | MCP 기반 BI | PostgreSQL, LLM, MCP | Important |
| **Chat Interface Service** | 8008 | 통합 AI 채팅 | PostgreSQL, LLM, WebSocket | Important |
| **Learning Service** | 8009 | 자동학습 + Rule 추출 | PostgreSQL, pgvector, sklearn | Important |

### 1.1.5 핵심 서비스 우선순위
```yaml
Critical:     # 장애시 즉시 롤백
  - API Gateway (8000)
  - Judgment Service (8002)

Important:    # 모니터링 후 판단
  - Workflow Service (8001)
  - Learning Service (8009)
  - BI Service (8007)
  - Chat Interface Service (8008)
  - Data Visualization Service (8006)

Supporting:   # 서비스 지속 가능
  - Action Service (8003)
  - Notification Service (8004)
  - Logging Service (8005)
```

---

## 1.2 Docker 컨테이너화 전략

### 1.2.1 멀티스테이지 빌드 전략

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

### 1.2.2 서비스별 최적화 전략

#### API Gateway (Kong 기반)
- **이미지**: `kong:3.4-alpine`
- **최적화**: 플러그인 선택적 로딩, 메모리 사용량 최소화
- **헬스체크**: `/status` 엔드포인트 활용

#### FastAPI 서비스들 (Ver2.0 Final - 9개 서비스)
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

### 1.2.3 Docker Compose 개발 환경

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

  # 마이크로서비스 (Ver2.0 Final - 9개)
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

  # ... 기타 7개 서비스들 (8002-8009)
```

---

## 1.3 Kubernetes 배포 전략

### 1.3.1 클러스터 아키텍처

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

### 1.3.2 배포 전략별 구성

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
- **적용 대상**: BI Service, Chat Interface Service
- **이유**: 신기능의 점진적 검증이 중요한 서비스
- **구현**: Istio Service Mesh를 통한 트래픽 비율 제어

```yaml
# Canary 배포 설정 (Istio)
apiVersion: networking.istio.io/v1alpha3
kind: VirtualService
metadata:
  name: bi-service
spec:
  http:
  - match:
    - headers:
        canary:
          exact: "true"
    route:
    - destination:
        host: bi-service
        subset: canary
  - route:
    - destination:
        host: bi-service
        subset: stable
      weight: 90
    - destination:
        host: bi-service
        subset: canary
      weight: 10
```

#### Rolling Update 배포 전략
- **적용 대상**: API Gateway, Action Service, Logging Service, Notification Service
- **이유**: 상대적으로 안정적인 서비스들
- **구현**: Kubernetes 기본 Rolling Update

### 1.3.3 리소스 할당 전략 (Ver2.0 Final - 9 services)

| 서비스 | CPU Request | CPU Limit | Memory Request | Memory Limit | 복제본 수 |
|--------|-------------|-----------|----------------|--------------|-----------|
| **API Gateway** | 100m | 200m | 128Mi | 256Mi | 3 |
| **Workflow Service** | 200m | 400m | 256Mi | 512Mi | 3 |
| **Judgment Service** | 300m | 600m | 512Mi | 1Gi | 5 |
| **Action Service** | 200m | 400m | 256Mi | 512Mi | 3 |
| **Notification Service** | 100m | 200m | 128Mi | 256Mi | 2 |
| **Logging Service** | 150m | 300m | 256Mi | 512Mi | 3 |
| **Data Visualization Service** | 200m | 400m | 256Mi | 512Mi | 3 |
| **BI Service** | 250m | 500m | 384Mi | 768Mi | 3 |
| **Chat Interface Service** | 250m | 500m | 384Mi | 768Mi | 3 |
| **Learning Service** | 300m | 600m | 512Mi | 1Gi | 3 |

### 1.3.4 Persistent Volume 전략

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

## 1.4 환경 관리 전략

### 1.4.1 환경별 구성

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

### 1.4.2 설정 관리 전략

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

  # Learning Service 설정 (Ver2.0 Final)
  learning.yaml: |
    few_shot:
      min_samples: 10
      max_samples: 20
      complexity_threshold: [0.3, 0.7]
    rule_extraction:
      algorithms: ["frequency", "decision_tree", "llm_pattern"]
      parallel_execution: true

  # BI Service 설정 (Ver2.0 Final)
  bi.yaml: |
    mcp_components:
      chart_types: ["bar", "line", "pie", "gauge", "kpi"]
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

## 1.5 CI/CD 파이프라인 전략

### 1.5.1 GitHub Actions Workflow 구조

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
        service: [workflow, judgment, action, notification, logging, data-viz, bi, chat, learning]

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
        service: [api-gateway, workflow, judgment, action, notification, logging, data-viz, bi, chat, learning, frontend]

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

### 1.5.2 보안 및 품질 게이트

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

# 2. 배포 런북

## 2.1 자동 배포 가이드

### 2.1.1 GitHub Actions를 통한 자동 배포

#### 스테이징 환경 자동 배포
```bash
# develop 브랜치 푸시시 자동 실행
git push origin develop

# CI/CD 파이프라인 자동 실행:
# 1. CI 파이프라인 (코드 품질, 테스트, 빌드)
# 2. CD 파이프라인 (스테이징 배포)
```

#### 프로덕션 환경 배포 (수동 승인)
```bash
# main 브랜치 푸시 또는 수동 트리거
git push origin main

# 또는 GitHub Actions에서 수동 실행
# Repository → Actions → CD Pipeline → Run workflow
```

### 2.1.2 배포 상태 모니터링
```bash
# GitHub Actions 실시간 모니터링
https://github.com/your-org/judgify-core/actions

# Slack 알림 확인 (#deployment-alerts)
# 배포 성공/실패 알림 자동 수신
```

---

## 2.2 수동 배포 가이드

### 2.2.1 환경 준비
```bash
# 1. 로컬 환경 설정
export ENVIRONMENT=production
export KUBECONFIG=~/.kube/config-prod

# 2. 필수 도구 확인
kubectl version --client
docker version
helm version  # (사용시)

# 3. 네임스페이스 확인
kubectl get namespaces
kubectl config set-context --current --namespace=judgify-prod
```

### 2.2.2 이미지 빌드 및 푸시
```bash
# 1. 프로젝트 루트 디렉토리로 이동
cd /path/to/judgify-core

# 2. Docker 이미지 빌드
./scripts/deploy/env-setup.sh --build

# 3. 레지스트리에 푸시 (GitHub Container Registry)
docker login ghcr.io
docker push ghcr.io/judgify/api-gateway-service:v2.0.0
docker push ghcr.io/judgify/judgment-service:v2.0.0
docker push ghcr.io/judgify/learning-service:v2.0.0
docker push ghcr.io/judgify/bi-service:v2.0.0
docker push ghcr.io/judgify/chat-interface-service:v2.0.0
# ... 기타 서비스들
```

### 2.2.3 Kubernetes 배포
```bash
# 1. 시크릿 설정 (최초 1회)
./scripts/deploy/env-setup.sh --env production --setup-secrets

# 2. ConfigMap 적용
kubectl apply -f k8s/configmaps/ -n judgify-prod

# 3. Blue-Green 배포 실행
kubectl apply -f k8s/services/ -n judgify-prod

# 4. 배포 상태 확인
kubectl rollout status deployment -n judgify-prod --timeout=300s
```

---

## 2.3 Blue-Green 배포 상세 절차

### 2.3.1 Blue-Green 배포 아키텍처
```
[Load Balancer]
       |
   [Service]  ←→ selector: version=blue/green
       |
┌─────────────┬─────────────┐
│    Blue     │    Green    │
│ (Current)   │   (New)     │
│  v1.9.0     │   v2.0.0    │
└─────────────┴─────────────┘
```

### 2.3.2 단계별 실행

#### Step 1: Green 환경 배포
```bash
# 1. Green 버전 배포 (Ver2.0 Final - 핵심 5개 서비스 우선)
for service in api-gateway judgment learning bi chat; do
  envsubst < k8s/services/${service}-service.yaml | \
    sed "s/${service}-service/${service}-service-green/g" | \
    sed "s/version: blue/version: green/g" | \
    kubectl apply -f - -n judgify-prod
done

# 2. Green 배포 완료 대기
kubectl rollout status deployment -n judgify-prod --timeout=600s
```

#### Step 2: Green 환경 검증
```bash
# 1. 헬스체크 (Green 환경 직접 테스트)
kubectl port-forward svc/api-gateway-service-green 8080:8000 -n judgify-prod &
curl http://localhost:8080/health

# 2. 스모크 테스트 실행
cd tests/smoke
python production_smoke_tests.py --base-url http://localhost:8080

# 3. 핵심 기능 테스트
python critical_path_tests.py --base-url http://localhost:8080

# Port-forward 종료
pkill -f "kubectl port-forward"
```

#### Step 3: 트래픽 전환 (Blue → Green)
```bash
# 1. 서비스 셀렉터를 Green으로 변경
for service in api-gateway judgment learning bi chat; do
  kubectl patch service ${service}-service -n judgify-prod \
    -p '{"spec":{"selector":{"version":"green"}}}'
done

# 2. 트래픽 전환 확인 (30초 대기 후)
sleep 30
curl -f https://api.judgify.ai/health

# 3. 실시간 메트릭 확인 (5분간)
kubectl top pods -n judgify-prod
```

#### Step 4: Blue 환경 정리
```bash
# 1. Blue 환경 제거 (트래픽 전환 완료 후 1시간 대기)
for service in api-gateway judgment learning bi chat; do
  kubectl delete deployment ${service}-service -n judgify-prod --ignore-not-found=true
done

# 2. Green을 새로운 Blue로 변경
for service in api-gateway judgment learning bi chat; do
  kubectl patch deployment ${service}-service-green -n judgify-prod \
    --type='merge' -p='{"metadata":{"name":"'${service}'-service"}}'

  kubectl patch deployment ${service}-service -n judgify-prod \
    --type='merge' -p='{"spec":{"template":{"metadata":{"labels":{"version":"blue"}}}}}'
done
```

---

## 2.4 배포 후 검증 절차

### 2.4.1 즉시 검증 (배포 후 10분 이내)

#### 시스템 헬스체크
```bash
# 1. 모든 Pod 상태 확인
kubectl get pods -n judgify-prod
# 모든 Pod가 Running/Ready 상태여야 함

# 2. 서비스 엔드포인트 확인
kubectl get services -n judgify-prod
kubectl get ingress -n judgify-prod

# 3. API 엔드포인트 테스트 (Ver2.0 Final - 9 services)
curl -f https://api.judgify.ai/health
curl -f https://api.judgify.ai/api/v2/workflow/health
curl -f https://api.judgify.ai/api/v2/judgment/health
curl -f https://api.judgify.ai/api/v2/learning/health
curl -f https://api.judgify.ai/api/v2/bi/health
curl -f https://api.judgify.ai/api/v2/chat/health
curl -f https://api.judgify.ai/api/v2/data-viz/health
```

#### 핵심 기능 검증
```bash
# 1. 자동 테스트 실행
cd tests/smoke
python production_smoke_tests.py --base-url https://api.judgify.ai --output-json /tmp/smoke_results.json

# 2. 결과 확인
cat /tmp/smoke_results.json | jq '.success'
# true 반환되어야 함

# 3. 핵심 비즈니스 로직 테스트
python critical_path_tests.py --base-url https://api.judgify.ai
```

### 2.4.2 성능 검증 (배포 후 30분)

#### 응답 시간 확인
```bash
# 1. API 응답 시간 테스트 (10회 평균)
for i in {1..10}; do
  curl -w "Response time: %{time_total}s\n" -o /dev/null -s https://api.judgify.ai/health
  sleep 1
done

# 2. 판단 서비스 응답 시간 (모의 요청)
time curl -X POST https://api.judgify.ai/api/v2/judgment/execute \
  -H "Content-Type: application/json" \
  -d '{"workflow_id":"test","input_data":{"test":true},"method":"hybrid"}'
```

#### 리소스 사용률 확인
```bash
# 1. CPU/Memory 사용률
kubectl top pods -n judgify-prod

# 2. 노드 리소스 상태
kubectl top nodes

# 3. HPA 상태 확인
kubectl get hpa -n judgify-prod
```

### 2.4.3 모니터링 확인 (배포 후 1시간)

#### Grafana 대시보드 확인
```bash
# 주요 메트릭 확인:
# 1. API 요청 수/응답시간
# 2. 에러율 (< 1% 유지)
# 3. 데이터베이스 연결 수
# 4. Redis 캐시 히트율
# 5. 판단 실행 성공률
# 6. Learning Service Few-shot 성능
# 7. BI Service MCP 컴포넌트 생성 시간
```

#### 로그 확인
```bash
# 1. 에러 로그 확인 (Kibana 또는 kubectl)
kubectl logs -l app=api-gateway -n judgify-prod --tail=100 | grep -i error
kubectl logs -l app=judgment-service -n judgify-prod --tail=100 | grep -i error
kubectl logs -l app=learning-service -n judgify-prod --tail=100 | grep -i error

# 2. 경고 로그 확인
kubectl logs -l app.kubernetes.io/name=judgify -n judgify-prod --tail=500 | grep -i warn
```

---

## 2.5 롤백 절차

### 2.5.1 자동 롤백 (CI/CD)
```bash
# GitHub Actions에서 배포 실패시 자동 롤백
# 1. Green 환경 배포 실패 → Blue 환경 유지
# 2. 트래픽 전환 후 문제 감지 → 자동 Blue 환경으로 복귀
```

### 2.5.2 수동 롤백

#### 긴급 롤백 (5분 이내 복구)
```bash
# 1. 즉시 이전 버전으로 롤백 (Ver2.0 Final - 핵심 서비스)
kubectl rollout undo deployment/api-gateway-service -n judgify-prod
kubectl rollout undo deployment/judgment-service -n judgify-prod
kubectl rollout undo deployment/learning-service -n judgify-prod
kubectl rollout undo deployment/bi-service -n judgify-prod
kubectl rollout undo deployment/chat-interface-service -n judgify-prod

# 2. 롤백 상태 확인
kubectl rollout status deployment -n judgify-prod --timeout=300s

# 3. 서비스 상태 확인
curl -f https://api.judgify.ai/health
```

#### 완전 롤백 (이전 릴리즈)
```bash
# 1. 이전 이미지 태그로 완전 복구
kubectl set image deployment/judgment-service \
  judgment-service=ghcr.io/judgify/judgment-service:v1.9.0 \
  -n judgify-prod

# 2. 설정 변경 롤백 (필요시)
git checkout HEAD~1 -- k8s/configmaps/
kubectl apply -f k8s/configmaps/ -n judgify-prod

# 3. 데이터베이스 마이그레이션 롤백 (필요시)
# 별도 DB 롤백 절차 참조
```

### 2.5.3 롤백 후 검증
```bash
# 1. 시스템 상태 확인
kubectl get pods -n judgify-prod
kubectl get services -n judgify-prod

# 2. 기능 검증
python tests/smoke/smoke_tests.py --base-url https://api.judgify.ai

# 3. 사용자 영향도 확인
# Grafana에서 에러율, 응답시간 확인
```

---

# 3. 모니터링 및 관찰가능성

## 3.1 Prometheus + Grafana 모니터링

### 3.1.1 메트릭 수집 전략

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
      - "learning_service_rules.yml"
      - "system_rules.yml"

    scrape_configs:
    # Ver2.0 Final - 9 마이크로서비스 메트릭 수집
    - job_name: 'api-gateway'
      static_configs:
      - targets: ['api-gateway-service:8000']

    - job_name: 'judgment-service'
      static_configs:
      - targets: ['judgment-service:8002']
      scrape_interval: 10s
      metrics_path: /metrics

    - job_name: 'learning-service'
      static_configs:
      - targets: ['learning-service:8009']
      scrape_interval: 10s

    - job_name: 'bi-service'
      static_configs:
      - targets: ['bi-service:8007']

    - job_name: 'chat-interface-service'
      static_configs:
      - targets: ['chat-interface-service:8008']

    # ... 기타 5개 서비스들

    # 인프라 메트릭
    - job_name: 'postgres-exporter'
      static_configs:
      - targets: ['postgres-exporter:9187']

    - job_name: 'redis-exporter'
      static_configs:
      - targets: ['redis-exporter:9121']
```

### 3.1.2 핵심 비즈니스 메트릭 정의 (Ver2.0 Final)

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

# Learning Service 메트릭 (Ver2.0 Final)
few_shot_sample_count = Gauge(
    'few_shot_sample_count',
    'Number of few-shot samples used',
    ['complexity_level']  # low, medium, high
)

rule_extraction_duration_seconds = Histogram(
    'rule_extraction_duration_seconds',
    'Duration of rule extraction',
    ['algorithm']  # frequency, decision_tree, llm_pattern
)

# BI Service 메트릭 (Ver2.0 Final)
mcp_component_generation_requests_total = Counter(
    'mcp_component_generation_requests_total',
    'Total MCP component generation requests',
    ['component_type', 'status']
)

mcp_component_generation_duration_seconds = Histogram(
    'mcp_component_generation_duration_seconds',
    'MCP component generation time'
)

# Chat Interface Service 메트릭 (Ver2.0 Final)
chat_messages_total = Counter(
    'chat_messages_total',
    'Total chat messages',
    ['intent', 'service_routed']
)

active_websocket_connections = Gauge(
    'active_websocket_connections',
    'Number of active WebSocket connections'
)
```

---

## 3.2 로깅 전략 (ELK Stack)

### 3.2.1 Elasticsearch 설정

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

### 3.2.2 Logstash 구조화 로그 파이프라인

```ruby
# logstash.conf
input {
  beats {
    port => 5044
  }
}

filter {
  # Judgment Service 로그
  if [fields][service] == "judgment-service" {
    json {
      source => "message"
    }

    if [event_type] == "judgment_executed" {
      mutate {
        add_field => { "[@metadata][index_prefix]" => "judgment-execution" }
      }
    }

    if [level] == "ERROR" {
      mutate {
        add_field => { "[@metadata][index_prefix]" => "errors" }
      }
    }
  }

  # Learning Service 로그 (Ver2.0 Final)
  if [fields][service] == "learning-service" {
    json {
      source => "message"
    }

    if [event_type] == "rule_extracted" {
      mutate {
        add_field => { "[@metadata][index_prefix]" => "learning-rule-extraction" }
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

### 3.2.3 구조화된 로깅 표준

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

# 사용 예시 (Judgment Service)
logger.info(
    "judgment_executed",
    workflow_id="wf-123",
    method="hybrid",
    result=True,
    confidence=0.95,
    execution_time_ms=1250,
    user_id="user-456"
)

# 사용 예시 (Learning Service - Ver2.0 Final)
logger.info(
    "rule_extracted",
    workflow_id="wf-123",
    algorithm="decision_tree",
    extracted_rule="temperature > 80 AND vibration > 50",
    confidence=0.92,
    sample_count=157
)
```

---

## 3.3 알림 및 인시던트 관리

### 3.3.1 Prometheus Alertmanager 설정 (Ver2.0 Final)

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

# learning_service_rules.yml (Ver2.0 Final)
- name: learning_service
  rules:

  # Few-shot 샘플 부족 알림
  - alert: LowFewShotSamples
    expr: few_shot_sample_count < 5
    for: 5m
    labels:
      severity: warning
      service: learning-service
    annotations:
      summary: "Low few-shot sample count"
      description: "Few-shot sample count is {{ $value }}"

  # Rule 추출 실패율 알림
  - alert: HighRuleExtractionFailureRate
    expr: rate(rule_extraction_requests_total{result="error"}[10m]) / rate(rule_extraction_requests_total[10m]) > 0.1
    for: 5m
    labels:
      severity: critical
      service: learning-service
    annotations:
      summary: "High rule extraction failure rate"
      description: "Rule extraction failure rate is {{ $value }}%"

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

### 3.3.2 Slack/Teams 통합 알림

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
  - match:
      service: learning-service
    receiver: 'learning-team'

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

- name: 'learning-team'
  slack_configs:
  - channel: '#learning-alerts'
    color: 'warning'
    title: 'Learning Service Alert'
    text: 'Learning service requires attention'

- name: 'pagerduty-critical'
  pagerduty_configs:
  - routing_key: '${PAGERDUTY_INTEGRATION_KEY}'
    description: 'Critical alert in Judgify system'
```

### 3.3.3 배포 중 모니터링

#### 실시간 메트릭 확인
```bash
# 1. Grafana 대시보드
https://grafana.company.com/d/judgify-overview

# 주요 확인 사항:
- API 응답 시간 (< 500ms 유지)
- 에러율 (< 1% 유지)
- 활성 연결 수
- 데이터베이스 성능
- 메모리/CPU 사용률
- Few-shot 샘플 개수 (Learning Service)
- MCP 컴포넌트 생성 시간 (BI Service)
```

#### 로그 모니터링
```bash
# 1. 실시간 로그 모니터링
kubectl logs -f deployment/api-gateway-service -n judgify-prod
kubectl logs -f deployment/learning-service -n judgify-prod

# 2. Kibana 대시보드
https://kibana.company.com/app/discover

# 주요 확인 사항:
- ERROR 레벨 로그 개수
- WARN 레벨 로그 패턴
- 느린 쿼리 로그
- 외부 API 호출 실패
- Rule 추출 실패 로그 (Learning Service)
```

### 3.3.4 알람 설정

#### Critical 알람 (즉시 대응)
- 서비스 Down (30초)
- API 에러율 > 5% (2분)
- 응답 시간 > 3초 (5분)
- 메모리 사용률 > 90% (5분)
- Rule 추출 실패율 > 10% (5분) - Learning Service
- Few-shot 샘플 < 5개 (5분) - Learning Service

#### Warning 알람 (모니터링)
- CPU 사용률 > 75% (10분)
- 디스크 사용률 > 80% (30분)
- 느린 쿼리 감지
- 외부 API 응답 지연
- MCP 컴포넌트 생성 시간 > 30초 (BI Service)

---

# 4. 보안 및 컴플라이언스

## 4.1 컨테이너 보안

### 4.1.1 보안 강화된 Dockerfile 패턴

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

### 4.1.2 Pod Security Standards

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

---

## 4.2 네트워크 보안

### 4.2.1 Network Policies

```yaml
# 네트워크 정책 예시 (Judgment Service)
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
  - to:
    - podSelector:
        matchLabels:
          app: learning-service  # Ver2.0 Final
    ports:
    - protocol: TCP
      port: 8009
  # OpenAI API 호출
  - to: []
    ports:
    - protocol: TCP
      port: 443
```

---

## 4.3 비밀 정보 관리

### 4.3.1 External Secrets Operator를 통한 비밀 관리

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

# 5. 백업 및 재해 복구

## 5.1 데이터베이스 백업 전략

### 5.1.1 PostgreSQL 백업

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

### 5.1.2 Redis 백업

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

---

## 5.2 재해 복구 계획

### 5.2.1 RTO/RPO 목표

| 서비스 | RTO (Recovery Time Objective) | RPO (Recovery Point Objective) |
|--------|-------------------------------|--------------------------------|
| **Judgment Service** | 15분 | 5분 |
| **Learning Service** | 30분 | 15분 |
| **Workflow Service** | 30분 | 15분 |
| **BI Service** | 1시간 | 30분 |
| **기타 서비스** | 1시간 | 30분 |

### 5.2.2 복구 절차

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

# 5. 서비스 복구 (우선순위 순 - Ver2.0 Final)
echo "서비스 복구 시작..."

# 우선순위 1: Judgment Service
kubectl apply -f ./recovery/judgment-service.yaml
kubectl wait --for=condition=ready pod -l app=judgment-service --timeout=300s

# 우선순위 2: API Gateway
kubectl apply -f ./recovery/api-gateway.yaml
kubectl wait --for=condition=ready pod -l app=api-gateway --timeout=300s

# 우선순위 3: Learning Service (Ver2.0 Final)
kubectl apply -f ./recovery/learning-service.yaml
kubectl wait --for=condition=ready pod -l app=learning-service --timeout=300s

# 우선순위 4: 나머지 서비스들
for service in workflow bi chat data-viz action notification logging; do
  kubectl apply -f ./recovery/${service}-service.yaml
  kubectl wait --for=condition=ready pod -l app=${service}-service --timeout=300s
done

# 6. 헬스체크 및 검증
./scripts/health-check.sh

echo "=== 재해 복구 완료 ==="
```

---

# 6. 확장성 및 성능 최적화

## 6.1 Horizontal Pod Autoscaler (HPA)

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

---

# 7. 트러블슈팅

## 7.1 일반적인 배포 문제

### 7.1.1 Pod 시작 실패
```bash
# 1. Pod 상태 확인
kubectl describe pod <pod-name> -n judgify-prod

# 2. 일반적인 원인:
- 이미지 Pull 실패 → 레지스트리 권한 확인
- 리소스 부족 → 노드 리소스 확인
- ConfigMap/Secret 오류 → 설정 값 확인
- Health check 실패 → 앱 로그 확인

# 3. 해결책:
kubectl logs <pod-name> -n judgify-prod
kubectl get events -n judgify-prod --sort-by='.lastTimestamp'
```

### 7.1.2 서비스 연결 실패
```bash
# 1. 서비스/엔드포인트 확인
kubectl get services -n judgify-prod
kubectl get endpoints -n judgify-prod

# 2. 네트워크 정책 확인
kubectl get networkpolicies -n judgify-prod

# 3. 포트/셀렉터 확인
kubectl describe service <service-name> -n judgify-prod
```

### 7.1.3 데이터베이스 연결 문제
```bash
# 1. 데이터베이스 상태 확인
kubectl exec -it <api-pod> -n judgify-prod -- nc -zv postgres-service 5432

# 2. 연결 문자열 확인
kubectl exec -it <api-pod> -n judgify-prod -- env | grep DATABASE_URL

# 3. 인증 정보 확인
kubectl get secret judgify-database-secret -n judgify-prod -o yaml
```

---

## 7.2 성능 문제 해결

### 7.2.1 높은 응답 시간
```bash
# 1. 병목 지점 확인
kubectl top pods -n judgify-prod
kubectl describe hpa -n judgify-prod

# 2. 로그 분석
kubectl logs -l app=api-gateway -n judgify-prod | grep -E "(slow|timeout|error)"
kubectl logs -l app=learning-service -n judgify-prod | grep "rule_extraction"

# 3. 데이터베이스 성능 확인
# PostgreSQL slow query 로그 확인
kubectl exec -it postgres-0 -n judgify-prod -- \
  psql -U judgify -d judgify -c "
  SELECT query, calls, total_time, mean_time
  FROM pg_stat_statements
  ORDER BY mean_time DESC
  LIMIT 10;"
```

### 7.2.2 메모리 부족
```bash
# 1. 메모리 사용량 확인
kubectl top pods -n judgify-prod --sort-by=memory

# 2. 리소스 제한 확인
kubectl describe pod <pod-name> -n judgify-prod | grep -A5 "Limits"

# 3. OOM 킬 확인
kubectl get events -n judgify-prod | grep OOMKilled
```

### 7.2.3 트래픽 급증 대응
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

---

## 7.3 보안 문제 대응

### 7.3.1 보안 취약점 발견 시
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

### 7.3.2 비정상 트래픽 차단
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

# 8. 운영 체크리스트

## 8.1 배포 전 체크리스트

### 8.1.1 개발 환경 검증
- [ ] 모든 서비스 로컬 Docker Compose 정상 실행 (Ver2.0 Final - 9 services)
- [ ] 단위 테스트 90% 이상 커버리지 달성
- [ ] 통합 테스트 시나리오 통과
- [ ] API 문서 최신화 (OpenAPI/Swagger)
- [ ] 보안 스캔 Critical/High 이슈 0건
- [ ] Learning Service Few-shot 로직 검증
- [ ] BI Service MCP 컴포넌트 조립 테스트

### 8.1.2 스테이징 환경 검증
- [ ] 전체 워크플로우 E2E 테스트 통과
- [ ] 성능 테스트 기준 충족 (응답시간 < 5초)
- [ ] 부하 테스트 1000 concurrent users 처리
- [ ] 장애 시나리오 테스트 (카오스 엔지니어링)
- [ ] 모니터링 대시보드 정상 작동
- [ ] Rule 추출 알고리즘 3개 모두 정상 작동 (Learning Service)

### 8.1.3 운영 환경 준비
- [ ] 백업 시스템 정상 작동 확인
- [ ] 롤백 계획 수립 및 검증
- [ ] 운영팀 배포 가이드 공유
- [ ] 장애 대응 매뉴얼 업데이트
- [ ] 사용자 공지사항 준비

---

## 8.2 배포 중 체크리스트
- [ ] 배포 시작 알림
- [ ] Blue-Green 배포 실행 (핵심 5개 서비스 우선)
- [ ] Green 환경 검증
- [ ] 트래픽 전환
- [ ] 모니터링 확인 (Grafana + Kibana)
- [ ] 성능 검증
- [ ] Blue 환경 정리

---

## 8.3 배포 후 체크리스트

### 8.3.1 즉시 확인 (배포 후 10분)
- [ ] 모든 서비스 Pod 정상 시작 (Ver2.0 Final - 9 services)
- [ ] 헬스체크 엔드포인트 정상 응답
- [ ] 핵심 API 기능 테스트 통과
- [ ] 실시간 모니터링 메트릭 정상
- [ ] 에러 로그 확인 (Critical 없음)

### 8.3.2 단기 확인 (배포 후 1시간)
- [ ] 전체 사용자 워크플로우 정상 작동
- [ ] 응답 시간 목표 달성 (95% < 5초)
- [ ] 메모리/CPU 사용량 정상 범위
- [ ] 데이터베이스 성능 지표 양호
- [ ] 외부 연동 시스템 정상 통신
- [ ] Learning Service Few-shot 샘플 정상 검색
- [ ] BI Service MCP 컴포넌트 정상 생성

### 8.3.3 중기 확인 (배포 후 24시간)
- [ ] 비즈니스 메트릭 정상 (판단 성공률 등)
- [ ] 자동 스케일링 정상 작동
- [ ] 백업 작업 정상 수행
- [ ] 보안 이벤트 없음
- [ ] 사용자 피드백 수집
- [ ] Rule 추출 성공률 > 90% (Learning Service)

---

## 8.4 정기 운영 작업

### 8.4.1 일간 작업
- [ ] 시스템 헬스 상태 점검
- [ ] 에러 로그 분석 및 대응
- [ ] 백업 상태 확인
- [ ] 보안 이벤트 모니터링
- [ ] 성능 지표 리뷰

### 8.4.2 주간 작업
- [ ] 보안 업데이트 적용
- [ ] 용량 계획 검토
- [ ] 성능 트렌드 분석
- [ ] 장애 대응 훈련
- [ ] 문서 업데이트

### 8.4.3 월간 작업
- [ ] 재해 복구 테스트
- [ ] 보안 감사
- [ ] 용량 최적화
- [ ] SLA 리포트 작성
- [ ] 아키텍처 리뷰

---

## 📞 9. 비상 연락망

### 9.1 배포 관련 연락처
| 역할 | 담당자 | 연락처 | 대응시간 |
|------|--------|--------|----------|
| 배포 책임자 | DevOps Lead | ext.2001 | 24/7 |
| 개발팀장 | Dev Manager | ext.2002 | 평일 9-18시 |
| 운영팀장 | Ops Manager | ext.2003 | 24/7 |
| 인프라 엔지니어 | Infra Eng | ext.2004 | 24/7 |

### 9.2 에스컬레이션 절차
1. **Level 1** (0-30분): 배포 담당자
2. **Level 2** (30-60분): 팀장급 대응
3. **Level 3** (60분+): 경영진 보고

---

## 🚀 10. 결론 및 다음 단계

### 10.1 핵심 성공 지표

| 지표 | 목표 | 측정 방법 |
|------|------|-----------|
| **가용성** | 99.5% | Prometheus 헬스체크 메트릭 |
| **응답 시간** | 95% < 5초 | API 응답 시간 히스토그램 |
| **판단 정확도** | 95% | 비즈니스 메트릭 대시보드 |
| **배포 성공률** | 99% | CI/CD 파이프라인 메트릭 |
| **보안 취약점** | Critical 0건 | 보안 스캔 도구 결과 |
| **Rule 추출 성공률** | 90% | Learning Service 메트릭 |

### 10.2 지속적 개선 계획

#### Phase 1: 안정화 (1-3개월)
- 운영 환경 안정화 및 모니터링 고도화
- 성능 최적화 및 용량 계획 수립
- 보안 강화 및 컴플라이언스 준수
- Learning Service 자동학습 성능 튜닝

#### Phase 2: 확장 (3-6개월)
- 멀티 리전 배포 및 CDN 연동
- AI/ML 모델 성능 향상
- 고급 관찰가능성 도구 도입 (분산 추적 등)
- MCP 컴포넌트 라이브러리 확장 (BI Service)

#### Phase 3: 혁신 (6-12개월)
- 서버리스 아키텍처로 일부 전환
- GitOps 기반 배포 자동화
- 카오스 엔지니어링 정규화
- 자동 Rule 최적화 (Learning Service)

### 10.3 팀 역량 강화

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

## 📚 11. 관련 문서

### 11.1 아키텍처 문서
- [시스템 아키텍처](../architecture/system_overview.md)
- [데이터베이스 설계](../architecture/database_design.md)
- [시스템 구조](../architecture/system_structure.md)

### 11.2 개발 문서
- [구현 계획](../development/implementation_plan.md)
- [브랜치 전략](../development/git-branch-strategy.md)

### 11.3 가이드 문서
- [프롬프트 엔지니어링 가이드](../guides/prompt_engineering.md)
- [설치 가이드](../../GETTING-STARTED.md)

---

**이 문서는 Judgify-core Ver2.0 Final의 안정적이고 확장 가능한 운영을 위한 포괄적인 가이드입니다. 지속적인 업데이트와 개선을 통해 최고 수준의 서비스 품질을 달성해 나가겠습니다.**

---

**📝 문서 버전**: v2.0.0
**최종 업데이트**: 2024-11-XX
**다음 리뷰 일정**: 2024-12-XX

**작성자**: DevOps Team
**검토자**: Architecture Team
**승인자**: Service Owner
