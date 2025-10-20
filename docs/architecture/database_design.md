# 데이터베이스 스키마 및 API 설계서

**문서 버전**: v2.0  
**작성일**: 2024.08.05  
**대상**: 백엔드 개발자, 데이터베이스 관리자, API 개발자  
**목적**: PostgreSQL 데이터베이스 스키마와 RESTful API 명세 정의

## 📊 1. 데이터베이스 설계 원칙

### 1.1 설계 철학
- **단일 진실 원천**: PostgreSQL을 메인 데이터 저장소로 사용
- **정규화**: 3NF까지 정규화하여 데이터 일관성 보장
- **성능 최적화**: 인덱스 전략과 파티셔닝 적용
- **확장성**: 멀티 테넌트 지원을 위한 tenant_id 필드 포함
- **감사 추적**: 모든 테이블에 생성/수정 시간 및 사용자 정보 포함

### 1.2 기술 스택
```sql
-- PostgreSQL 15+ with Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "vector";  -- pgvector for embeddings
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";
```

## 🗃 2. 핵심 테이블 스키마

### 2.1 사용자 및 인증 테이블
```sql
-- 테넌트 (회사/조직)
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    settings JSONB DEFAULT '{}',
    subscription_plan VARCHAR(50) DEFAULT 'free',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- 사용자
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    role VARCHAR(50) DEFAULT 'user',
    is_active BOOLEAN DEFAULT true,
    last_login TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- API 키 (시스템 간 인증용)
CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    permissions JSONB DEFAULT '[]',
    expires_at TIMESTAMP,
    last_used TIMESTAMP,
    is_active BOOLEAN DEFAULT true,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW()
);

-- 인덱스
CREATE INDEX idx_users_tenant_id ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_api_keys_tenant_id ON api_keys(tenant_id);
CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);
```

### 2.2 워크플로우 관련 테이블
```sql
-- 워크플로우 정의
CREATE TABLE workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    definition JSONB NOT NULL,
    version INTEGER DEFAULT 1,
    status VARCHAR(20) DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'inactive', 'archived')),
    tags TEXT[] DEFAULT '{}',
    created_by UUID NOT NULL REFERENCES users(id),
    updated_by UUID REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    
    UNIQUE(tenant_id, name, version)
);

-- 워크플로우 버전 관리
CREATE TABLE workflow_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    definition JSONB NOT NULL,
    change_summary TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW(),
    
    UNIQUE(workflow_id, version)
);

-- 워크플로우 실행 스케줄
CREATE TABLE workflow_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    cron_expression VARCHAR(100) NOT NULL,
    timezone VARCHAR(50) DEFAULT 'UTC',
    is_active BOOLEAN DEFAULT true,
    next_run TIMESTAMP,
    last_run TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- 인덱스
CREATE INDEX idx_workflows_tenant_id ON workflows(tenant_id);
CREATE INDEX idx_workflows_status ON workflows(status);
CREATE INDEX idx_workflows_tags ON workflows USING GIN(tags);
CREATE INDEX idx_workflow_versions_workflow_id ON workflow_versions(workflow_id);
```

### 2.3 판단 실행 관련 테이블
```sql
-- 판단 실행 이력
CREATE TABLE judgment_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES workflows(id),
    trigger_source VARCHAR(50) NOT NULL, -- 'api', 'schedule', 'webhook', 'manual'
    trigger_data JSONB,
    input_data JSONB NOT NULL,
    context_data JSONB DEFAULT '{}',
    
    -- 판단 결과
    rule_result JSONB,
    llm_result JSONB,
    final_result JSONB NOT NULL,
    method_used VARCHAR(20) NOT NULL CHECK (method_used IN ('rule', 'llm', 'hybrid')),
    confidence_score DECIMAL(5,4) CHECK (confidence_score >= 0 AND confidence_score <= 1),
    
    -- 실행 메타데이터
    execution_time_ms INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL CHECK (status IN ('success', 'partial_success', 'failed')),
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    
    -- 추적 정보
    trace_id UUID DEFAULT gen_random_uuid(),
    parent_execution_id UUID REFERENCES judgment_executions(id),
    
    created_at TIMESTAMP DEFAULT NOW()
);

-- 판단 실행 단계별 로그
CREATE TABLE judgment_execution_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL REFERENCES judgment_executions(id) ON DELETE CASCADE,
    step_name VARCHAR(100) NOT NULL,
    step_type VARCHAR(50) NOT NULL, -- 'validation', 'context_gathering', 'rule_execution', 'llm_call', 'action_execution'
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    status VARCHAR(20) NOT NULL,
    input_data JSONB,
    output_data JSONB,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- 파티셔닝 (월별)
CREATE TABLE judgment_executions_y2024m01 PARTITION OF judgment_executions
FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

-- 인덱스
CREATE INDEX idx_judgment_executions_tenant_id ON judgment_executions(tenant_id);
CREATE INDEX idx_judgment_executions_workflow_id ON judgment_executions(workflow_id);
CREATE INDEX idx_judgment_executions_created_at ON judgment_executions(created_at);
CREATE INDEX idx_judgment_executions_status ON judgment_executions(status);
CREATE INDEX idx_judgment_executions_trace_id ON judgment_executions(trace_id);
```

### 2.4 액션 실행 관련 테이블
```sql
-- 액션 실행 이력
CREATE TABLE action_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    judgment_execution_id UUID NOT NULL REFERENCES judgment_executions(id) ON DELETE CASCADE,
    action_type VARCHAR(50) NOT NULL, -- 'mcp_command', 'notification', 'webhook', 'database_update'
    action_name VARCHAR(100) NOT NULL,
    target_system VARCHAR(100),
    
    -- 액션 정의
    command JSONB NOT NULL,
    parameters JSONB DEFAULT '{}',
    
    -- 실행 결과
    result JSONB,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'executing', 'success', 'failed', 'timeout')),
    error_message TEXT,
    
    -- 재시도 로직
    retry_count INTEGER DEFAULT 0,
    max_retries INTEGER DEFAULT 3,
    next_retry_at TIMESTAMP,
    
    -- 실행 메타데이터
    execution_time_ms INTEGER,
    timeout_ms INTEGER DEFAULT 30000,
    
    created_at TIMESTAMP DEFAULT NOW(),
    started_at TIMESTAMP,
    completed_at TIMESTAMP
);

-- 액션 템플릿 (재사용 가능한 액션 정의)
CREATE TABLE action_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    action_type VARCHAR(50) NOT NULL,
    template JSONB NOT NULL,
    parameters_schema JSONB, -- JSON Schema for validation
    is_active BOOLEAN DEFAULT true,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    
    UNIQUE(tenant_id, name)
);

-- 인덱스
CREATE INDEX idx_action_executions_judgment_id ON action_executions(judgment_execution_id);
CREATE INDEX idx_action_executions_status ON action_executions(status);
CREATE INDEX idx_action_executions_action_type ON action_executions(action_type);
CREATE INDEX idx_action_templates_tenant_id ON action_templates(tenant_id);
```

### 2.5 피드백 및 설명 테이블
```sql
-- 판단 피드백
CREATE TABLE judgment_feedback (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL REFERENCES judgment_executions(id) ON DELETE CASCADE,
    feedback_type VARCHAR(20) NOT NULL CHECK (feedback_type IN ('positive', 'negative', 'neutral')),
    rating INTEGER CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    feedback_data JSONB DEFAULT '{}',
    
    -- 피드백 제공자
    provided_by UUID REFERENCES users(id),
    provided_via VARCHAR(50) DEFAULT 'web', -- 'web', 'api', 'slack', 'email'
    
    created_at TIMESTAMP DEFAULT NOW()
);

-- 판단 설명 (LLM 생성)
CREATE TABLE judgment_explanations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    execution_id UUID NOT NULL REFERENCES judgment_executions(id) ON DELETE CASCADE,
    explanation_type VARCHAR(20) NOT NULL, -- 'initial', 'enhanced', 'feedback_based'
    explanation_text TEXT NOT NULL,
    confidence_score DECIMAL(3,2),
    
    -- LLM 메타데이터
    llm_model VARCHAR(100),
    llm_tokens_used INTEGER,
    generation_time_ms INTEGER,
    
    created_at TIMESTAMP DEFAULT NOW()
);

-- 벡터 임베딩 (설명 검색용)
CREATE TABLE explanation_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    explanation_id UUID NOT NULL REFERENCES judgment_explanations(id) ON DELETE CASCADE,
    embedding vector(1536), -- OpenAI text-embedding-3-small 차원
    created_at TIMESTAMP DEFAULT NOW()
);

-- 인덱스
CREATE INDEX idx_judgment_feedback_execution_id ON judgment_feedback(execution_id);
CREATE INDEX idx_judgment_feedback_type ON judgment_feedback(feedback_type);
CREATE INDEX idx_explanation_embeddings_vector ON explanation_embeddings USING ivfflat (embedding vector_cosine_ops);
```

### 2.6 시스템 모니터링 테이블
```sql
-- 시스템 메트릭
CREATE TABLE system_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES tenants(id),
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL(15,6) NOT NULL,
    metric_unit VARCHAR(20),
    dimensions JSONB DEFAULT '{}',
    timestamp TIMESTAMP DEFAULT NOW(),
    
    -- 파티셔닝을 위한 인덱스
    created_at TIMESTAMP DEFAULT NOW()
);

-- 알림 규칙
CREATE TABLE alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    metric_name VARCHAR(100) NOT NULL,
    condition_operator VARCHAR(10) NOT NULL, -- '>', '<', '>=', '<=', '==', '!='
    threshold_value DECIMAL(15,6) NOT NULL,
    severity VARCHAR(20) DEFAULT 'warning', -- 'info', 'warning', 'error', 'critical'
    notification_channels JSONB DEFAULT '[]',
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- 알림 이력
CREATE TABLE alert_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_rule_id UUID NOT NULL REFERENCES alert_rules(id),
    metric_value DECIMAL(15,6) NOT NULL,
    message TEXT NOT NULL,
    severity VARCHAR(20) NOT NULL,
    status VARCHAR(20) DEFAULT 'sent', -- 'sent', 'failed', 'acknowledged'
    sent_channels JSONB DEFAULT '[]',
    acknowledged_by UUID REFERENCES users(id),
    acknowledged_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW()
);
```

## 🔧 3. 데이터베이스 함수 및 트리거

### 3.1 자동 업데이트 트리거
```sql
-- updated_at 자동 업데이트 함수
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$ language 'plpgsql';

-- 트리거 생성 매크로
CREATE OR REPLACE FUNCTION create_updated_at_trigger(table_name text)
RETURNS void AS $
BEGIN
    EXECUTE format('CREATE TRIGGER update_%I_updated_at 
                    BEFORE UPDATE ON %I 
                    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column()', 
                   table_name, table_name);
END;
$ LANGUAGE plpgsql;

-- 트리거 적용
SELECT create_updated_at_trigger('tenants');
SELECT create_updated_at_trigger('users');
SELECT create_updated_at_trigger('workflows');
SELECT create_updated_at_trigger('action_templates');
```

### 3.2 비즈니스 로직 함수
```sql
-- 워크플로우 활성화 함수
CREATE OR REPLACE FUNCTION activate_workflow(
    p_workflow_id UUID,
    p_user_id UUID
) RETURNS BOOLEAN AS $
DECLARE
    v_tenant_id UUID;
BEGIN
    -- 권한 확인 및 워크플로우 활성화
    UPDATE workflows 
    SET status = 'active', 
        updated_by = p_user_id,
        updated_at = NOW()
    WHERE id = p_workflow_id 
    AND status IN ('draft', 'inactive')
    RETURNING tenant_id INTO v_tenant_id;
    
    IF FOUND THEN
        -- 로그 기록
        INSERT INTO system_metrics (tenant_id, metric_name, metric_value, dimensions)
        VALUES (v_tenant_id, 'workflow_activated', 1, 
                jsonb_build_object('workflow_id', p_workflow_id));
        
        RETURN TRUE;
    END IF;
    
    RETURN FALSE;
END;
$ LANGUAGE plpgsql;

-- 판단 실행 통계 함수
CREATE OR REPLACE FUNCTION get_judgment_stats(
    p_tenant_id UUID,
    p_start_date TIMESTAMP DEFAULT NOW() - INTERVAL '30 days',
    p_end_date TIMESTAMP DEFAULT NOW()
) RETURNS TABLE (
    total_executions BIGINT,
    success_rate DECIMAL(5,2),
    avg_execution_time_ms DECIMAL(10,2),
    method_distribution JSONB
) AS $
BEGIN
    RETURN QUERY
    SELECT 
        COUNT(*) as total_executions,
        ROUND(
            (COUNT(*) FILTER (WHERE status = 'success')::DECIMAL / COUNT(*)) * 100, 
            2
        ) as success_rate,
        ROUND(AVG(execution_time_ms), 2) as avg_execution_time_ms,
        jsonb_object_agg(method_used, method_count) as method_distribution
    FROM (
        SELECT 
            status,
            execution_time_ms,
            method_used,
            COUNT(*) OVER (PARTITION BY method_used) as method_count
        FROM judgment_executions
        WHERE tenant_id = p_tenant_id
        AND created_at BETWEEN p_start_date AND p_end_date
    ) stats;
END;
$ LANGUAGE plpgsql;
```

## 📡 4. REST API 설계

### 4.1 API 설계 원칙
- **RESTful**: HTTP 메서드와 상태 코드를 적절히 활용
- **일관성**: 모든 엔드포인트에서 동일한 응답 구조 사용
- **버전 관리**: URL에 버전 정보 포함 (`/api/v1/`)
- **페이지네이션**: 리스트 API는 기본적으로 페이지네이션 지원
- **필터링**: 쿼리 파라미터로 데이터 필터링 지원

### 4.2 공통 응답 구조
```json
{
    "success": true,
    "data": {},
    "message": "Success",
    "errors": [],
    "meta": {
        "timestamp": "2024-08-05T10:30:00Z",
        "request_id": "uuid",
        "version": "v1"
    },
    "pagination": {
        "page": 1,
        "per_page": 20,
        "total": 100,
        "total_pages": 5,
        "next_page": 2,
        "prev_page": null
    }
}
```

### 4.3 인증 API
```yaml
# 로그인
POST /api/v1/auth/login
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "password123"
}

# 응답
{
    "success": true,
    "data": {
        "access_token": "eyJhbGciOiJIUz...",
        "refresh_token": "eyJhbGciOiJIUz...",
        "token_type": "bearer",
        "expires_in": 3600,
        "user": {
            "id": "uuid",
            "email": "user@example.com",
            "full_name": "John Doe",
            "role": "admin",
            "tenant_id": "uuid"
        }
    }
}

# 토큰 갱신
POST /api/v1/auth/refresh
Authorization: Bearer {refresh_token}

# 로그아웃
POST /api/v1/auth/logout
Authorization: Bearer {access_token}
```

### 4.4 워크플로우 API
```yaml
# 워크플로우 목록 조회
GET /api/v1/workflows
Authorization: Bearer {token}
Query Parameters:
  - page: int (default: 1)
  - per_page: int (default: 20, max: 100)
  - status: string (draft|active|inactive|archived)
  - search: string
  - tags: string (comma-separated)

# 워크플로우 생성
POST /api/v1/workflows
Authorization: Bearer {token}
Content-Type: application/json

{
    "name": "Temperature Monitor",
    "description": "Monitor machine temperature and alert operators",
    "definition": {
        "trigger": {
            "type": "sensor_data",
            "source": "temperature_sensor_01"
        },
        "conditions": {
            "rule_expression": "temperature > 85",
            "llm_criteria": "Determine if temperature is dangerous"
        },
        "actions": [
            {
                "type": "notification",
                "template_id": "high_temp_alert",
                "channels": ["slack", "email"]
            }
        ]
    },
    "tags": ["temperature", "safety", "critical"]
}

# 워크플로우 상세 조회
GET /api/v1/workflows/{workflow_id}
Authorization: Bearer {token}

# 워크플로우 업데이트
PUT /api/v1/workflows/{workflow_id}
Authorization: Bearer {token}
Content-Type: application/json

# 워크플로우 활성화/비활성화
PATCH /api/v1/workflows/{workflow_id}/status
Authorization: Bearer {token}
Content-Type: application/json

{
    "status": "active"
}

# 워크플로우 삭제
DELETE /api/v1/workflows/{workflow_id}
Authorization: Bearer {token}
```

### 4.5 판단 실행 API
```yaml
# 판단 실행 (동기)
POST /api/v1/judgments/execute
Authorization: Bearer {token}
Content-Type: application/json

{
    "workflow_id": "uuid",
    "input_data": {
        "temperature": 90,
        "pressure": 85,
        "machine_id": "PRESS_01"
    },
    "context": {
        "shift": "day",
        "operator": "John Doe"
    },
    "force_method": "hybrid"  // optional: rule|llm|hybrid
}

# 응답
{
    "success": true,
    "data": {
        "execution_id": "uuid",
        "result": true,
        "confidence": 0.85,
        "method_used": "hybrid",
        "execution_time_ms": 1250,
        "explanation": "Temperature exceeds safety threshold...",
        "actions_executed": [
            {
                "action_id": "uuid",
                "type": "notification",
                "status": "success",
                "result": {
                    "slack_message_id": "1234567890.123456"
                }
            }
        ]
    }
}

# 판단 실행 (비동기)
POST /api/v1/judgments/execute-async
Authorization: Bearer {token}
Content-Type: application/json

# 응답
{
    "success": true,
    "data": {
        "execution_id": "uuid",
        "status": "pending",
        "status_url": "/api/v1/judgments/executions/{execution_id}/status"
    }
}

# 실행 상태 확인
GET /api/v1/judgments/executions/{execution_id}/status
Authorization: Bearer {token}

# 판단 실행 이력 조회
GET /api/v1/judgments/executions
Authorization: Bearer {token}
Query Parameters:
  - workflow_id: uuid
  - status: string (success|partial_success|failed)
  - method_used: string (rule|llm|hybrid)
  - start_date: ISO 8601 datetime 
  - end_date: ISO 8601 datetime
  - page: int
  - per_page: int

# 판단 실행 상세 조회
GET /api/v1/judgments/executions/{execution_id}
Authorization: Bearer {token}
```

### 4.6 피드백 API
```yaml
# 피드백 제출
POST /api/v1/judgments/executions/{execution_id}/feedback
Authorization: Bearer {token}
Content-Type: application/json

{
    "feedback_type": "negative",
    "rating": 2,
    "comment": "The judgment was incorrect because the machine was in maintenance mode",
    "feedback_data": {
        "expected_result": false,
        "suggested_improvement": "Consider maintenance schedule in context"
    }
}

# 피드백 목록 조회
GET /api/v1/judgments/executions/{execution_id}/feedback
Authorization: Bearer {token}

# 설명 요청 (LLM 기반 상세 설명)
POST /api/v1/judgments/executions/{execution_id}/explain
Authorization: Bearer {token}
Content-Type: application/json

{
    "explanation_type": "enhanced",
    "include_context": true,
    "target_audience": "operator"  // operator|manager|engineer
}
```

### 4.7 대시보드 및 분석 API
```yaml
# 대시보드 데이터 조회
GET /api/v1/analytics/dashboard
Authorization: Bearer {token}
Query Parameters:
  - timeframe: string (1h|6h|24h|7d|30d)
  - workflow_ids: string (comma-separated UUIDs)
  - metrics: string (comma-separated metric names)

# 응답
{
    "success": true,
    "data": {
        "summary": {
            "total_executions": 1250,
            "success_rate": 94.5,
            "avg_execution_time_ms": 850,
            "active_workflows": 15
        },
        "time_series": [
            {
                "timestamp": "2024-08-05T10:00:00Z",
                "executions": 45,
                "success_rate": 95.6,
                "avg_time_ms": 820
            }
        ],
        "method_distribution": {
            "rule": 60,
            "llm": 25,
            "hybrid": 15
        },
        "top_workflows": [
            {
                "workflow_id": "uuid",
                "name": "Temperature Monitor",
                "execution_count": 450,
                "success_rate": 98.2
            }
        ]
    }
}

# 워크플로우별 상세 분석
GET /api/v1/analytics/workflows/{workflow_id}
Authorization: Bearer {token}
Query Parameters:
  - start_date: ISO 8601 datetime
  - end_date: ISO 8601 datetime

# 시스템 성능 메트릭
GET /api/v1/analytics/performance
Authorization: Bearer {token}
```

## 🔒 5. 보안 고려사항

### 5.1 데이터 암호화
```sql
-- 민감 데이터 암호화 함수
CREATE OR REPLACE FUNCTION encrypt_sensitive_data(data TEXT)
RETURNS TEXT AS $
BEGIN
    RETURN encode(pgp_sym_encrypt(data, current_setting('app.encryption_key')), 'base64');
END;
$ LANGUAGE plpgsql SECURITY DEFINER;

-- 복호화 함수
CREATE OR REPLACE FUNCTION decrypt_sensitive_data(encrypted_data TEXT)
RETURNS TEXT AS $
BEGIN
    RETURN pgp_sym_decrypt(decode(encrypted_data, 'base64'), current_setting('app.encryption_key'));
END;
$ LANGUAGE plpgsql SECURITY DEFINER;
```

### 5.2 Row Level Security (RLS)
```sql
-- 테넌트 격리를 위한 RLS 정책
ALTER TABLE workflows ENABLE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation_workflows ON workflows
    FOR ALL TO authenticated_users
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- 사용자 역할 기반 접근 제어
CREATE POLICY workflow_read_policy ON workflows
    FOR SELECT TO authenticated_users
    USING (
        tenant_id = current_setting('app.current_tenant_id')::UUID
        AND (
            current_setting('app.user_role') = 'admin'
            OR created_by = current_setting('app.user_id')::UUID
            OR status = 'active'
        )
    );
```

### 5.3 API 보안 헤더
```python
# FastAPI 보안 설정
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.trustedhost import TrustedHostMiddleware

app = FastAPI()

# CORS 설정
app.add_middleware(
    CORSMiddleware,
    allow_origins=["https://yourdomain.com"],
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE"],
    allow_headers=["*"],
)

# 신뢰할 수 있는 호스트만 허용
app.add_middleware(
    TrustedHostMiddleware, 
    allowed_hosts=["yourdomain.com", "*.yourdomain.com"]
)

# 보안 헤더 미들웨어
@app.middleware("http")
async def add_security_headers(request, call_next):
    response = await call_next(request)
    response.headers["X-Content-Type-Options"] = "nosniff"
    response.headers["X-Frame-Options"] = "DENY"
    response.headers["X-XSS-Protection"] = "1; mode=block"
    response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
    return response
```

## 📊 6. 성능 최적화

### 6.1 인덱스 전략
```sql
-- 복합 인덱스 (자주 함께 사용되는 조건)
CREATE INDEX idx_judgment_executions_tenant_workflow_date 
ON judgment_executions(tenant_id, workflow_id, created_at DESC);

-- 부분 인덱스 (특정 조건의 데이터만)
CREATE INDEX idx_judgment_executions_failed 
ON judgment_executions(tenant_id, created_at DESC) 
WHERE status = 'failed';

-- 함수 기반 인덱스
CREATE INDEX idx_judgment_executions_date_trunc 
ON judgment_executions(tenant_id, date_trunc('hour', created_at));

-- GIN 인덱스 (JSONB 데이터)
CREATE INDEX idx_judgment_executions_input_data 
ON judgment_executions USING GIN(input_data);
```

### 6.2 파티셔닝 전략
```sql
-- 월별 파티셔닝을 위한 함수
CREATE OR REPLACE FUNCTION create_monthly_partition(
    table_name text,
    year int,
    month int
) RETURNS void AS $
DECLARE
    partition_name text;
    start_date date;
    end_date date;
BEGIN
    partition_name := format('%s_y%sm%02d', table_name, year, month);
    start_date := make_date(year, month, 1);
    end_date := start_date + interval '1 month';
    
    EXECUTE format('CREATE TABLE %I PARTITION OF %I 
                    FOR VALUES FROM (%L) TO (%L)',
                   partition_name, table_name, start_date, end_date);
                   
    EXECUTE format('CREATE INDEX idx_%s_created_at ON %I(created_at)',
                   partition_name, partition_name);
END;
$ LANGUAGE plpgsql;

-- 자동 파티션 생성 (크론 작업으로 실행)
SELECT create_monthly_partition('judgment_executions', 2024, 9);
SELECT create_monthly_partition('judgment_executions', 2024, 10);
```

## 🔄 7. 다음 문서 연결

이 데이터베이스 및 API 설계서를 기반으로 다음 문서들이 작성됩니다:

1. **워크플로우 편집기 구현 명세서**: React Flow 기반 UI와 워크플로우 JSON 스키마 연동
2. **외부 시스템 연동 가이드**: MCP 및 산업제어시스템 API 연동 방법
3. **모니터링 및 운영 가이드**: 시스템 메트릭 수집 및 알림 설정

각 문서는 이 데이터 모델과 API 명세를 기반으로 구체적인 구현 방법을 제시합니다.