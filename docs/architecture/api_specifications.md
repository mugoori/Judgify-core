# API 명세서 및 통신 프로토콜 정의

**문서 버전**: v2.0  
**작성일**: 2024.08.10  
**대상**: 백엔드 개발자, 프론트엔드 개발자, API 테스터  
**목적**: 마이크로서비스별 API 상세 명세 및 통신 프로토콜 정의

## 📋 1. API 설계 원칙

### 1.1 RESTful API 설계 원칙
```yaml
Principles:
  - Resource-oriented URLs
  - HTTP methods semantic usage
  - Stateless communication
  - HATEOAS (Hypermedia as the Engine of Application State)
  - Consistent error handling
  - API versioning strategy

URL_Patterns:
  Collections: "/api/v1/workflows"
  Resources: "/api/v1/workflows/{id}"
  Sub-resources: "/api/v1/workflows/{id}/executions"
  Actions: "/api/v1/workflows/{id}/simulate"

HTTP_Methods:
  GET: Retrieve resources (safe, idempotent)
  POST: Create resources (unsafe, non-idempotent)
  PUT: Update resources (unsafe, idempotent)
  PATCH: Partial update (unsafe, non-idempotent)
  DELETE: Remove resources (unsafe, idempotent)
```

### 1.2 공통 응답 형식
```json
{
  "success": true,
  "data": {
    // 실제 응답 데이터
  },
  "meta": {
    "timestamp": "2024-08-10T12:00:00Z",
    "request_id": "req_123456789",
    "version": "v1",
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 100,
      "has_next": true
    }
  },
  "errors": null
}

// 에러 응답 형식
{
  "success": false,
  "data": null,
  "meta": {
    "timestamp": "2024-08-10T12:00:00Z",
    "request_id": "req_123456789",
    "version": "v1"
  },
  "errors": [
    {
      "code": "VALIDATION_ERROR",
      "message": "Invalid input data",
      "field": "workflow_id",
      "details": "Workflow ID must be a valid UUID"
    }
  ]
}
```

### 1.3 공통 헤더
```http
# 요청 헤더
Content-Type: application/json
Accept: application/json
Authorization: Bearer {jwt_token}
X-Request-ID: {unique_request_id}
X-Service-Call: true  # 내부 서비스 호출시
X-Client-Version: 1.0.0

# 응답 헤더
Content-Type: application/json
X-Request-ID: {same_as_request}
X-Response-Time: 150ms
X-Rate-Limit-Remaining: 95
X-Rate-Limit-Reset: 1691664000
```

## 🔧 2. API Gateway (Port 8000)

### 2.1 인증 및 인가 API
```yaml
# POST /api/v1/auth/login
Authentication:
  Description: "사용자 로그인"
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        email:
          type: string
          format: email
          example: "user@example.com"
        password:
          type: string
          minLength: 8
          example: "securepassword"
        tenant_id:
          type: string
          format: uuid
          example: "550e8400-e29b-41d4-a716-446655440000"
      required: [email, password]
  Response:
    200:
      description: "Login successful"
      schema:
        type: object
        properties:
          access_token:
            type: string
            example: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
          refresh_token:
            type: string
          token_type:
            type: string
            example: "bearer"
          expires_in:
            type: integer
            example: 3600
          user:
            type: object
            properties:
              id:
                type: string
                format: uuid
              name:
                type: string
              roles:
                type: array
                items:
                  type: string
    401:
      description: "Invalid credentials"
    429:
      description: "Too many login attempts"

# POST /api/v1/auth/refresh
Token_Refresh:
  Description: "액세스 토큰 갱신"
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        refresh_token:
          type: string
      required: [refresh_token]
  Response:
    200:
      description: "Token refreshed successfully"
      schema:
        type: object
        properties:
          access_token:
            type: string
          expires_in:
            type: integer

# GET /api/v1/auth/me
User_Profile:
  Description: "현재 사용자 정보 조회"
  Security:
    - BearerAuth: []
  Response:
    200:
      description: "User profile"
      schema:
        type: object
        properties:
          id:
            type: string
            format: uuid
          name:
            type: string
          email:
            type: string
          roles:
            type: array
            items:
              type: string
          permissions:
            type: array
            items:
              type: string
          tenant_id:
            type: string
            format: uuid
```

### 2.2 라우팅 및 프록시 규칙
```python
# Kong Gateway 설정 예시
ROUTING_CONFIG = {
    "routes": [
        {
            "name": "workflow-api",
            "paths": ["/api/v1/workflows"],
            "methods": ["GET", "POST", "PUT", "DELETE", "PATCH"],
            "service": "workflow-service",
            "upstream_url": "http://workflow-service:8001",
            "plugins": [
                {
                    "name": "rate-limiting",
                    "config": {"minute": 100, "hour": 1000}
                },
                {
                    "name": "jwt",
                    "config": {"claims_to_verify": ["exp", "sub"]}
                }
            ]
        },
        {
            "name": "judgment-api",
            "paths": ["/api/v1/judgment"],
            "methods": ["POST", "GET"],
            "service": "judgment-service", 
            "upstream_url": "http://judgment-service:8002",
            "plugins": [
                {
                    "name": "rate-limiting",
                    "config": {"minute": 200, "hour": 2000}
                },
                {
                    "name": "response-transformer",
                    "config": {
                        "add": {
                            "headers": ["X-Service:judgment-service"]
                        }
                    }
                }
            ]
        }
    ]
}
```

## 📋 3. Workflow Service API (Port 8001)

### 3.1 워크플로우 관리 API
```yaml
# GET /api/v1/workflows
List_Workflows:
  Description: "워크플로우 목록 조회"
  Parameters:
    - name: page
      in: query
      type: integer
      default: 1
      minimum: 1
    - name: limit
      in: query
      type: integer
      default: 20
      maximum: 100
    - name: status
      in: query
      type: string
      enum: [active, inactive, archived]
    - name: created_by
      in: query
      type: string
      format: uuid
    - name: search
      in: query
      type: string
      description: "워크플로우 이름 검색"
    - name: sort
      in: query
      type: string
      enum: [created_at, updated_at, name]
      default: created_at
    - name: order
      in: query
      type: string
      enum: [asc, desc]
      default: desc
  Responses:
    200:
      description: "Workflows retrieved successfully"
      schema:
        type: object
        properties:
          workflows:
            type: array
            items:
              $ref: '#/definitions/Workflow'
          pagination:
            $ref: '#/definitions/Pagination'

# POST /api/v1/workflows
Create_Workflow:
  Description: "새 워크플로우 생성"
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        name:
          type: string
          minLength: 3
          maxLength: 100
          pattern: "^[a-zA-Z0-9_\\-\\s]+$"
          example: "Temperature Monitoring Workflow"
        description:
          type: string
          maxLength: 500
          example: "Monitors temperature sensors and triggers alerts"
        definition:
          type: object
          properties:
            nodes:
              type: array
              items:
                type: object
                properties:
                  id:
                    type: string
                  type:
                    type: string
                    enum: [input, condition, action, output]
                  position:
                    type: object
                    properties:
                      x:
                        type: number
                      y:
                        type: number
                  config:
                    type: object
            edges:
              type: array
              items:
                type: object
                properties:
                  id:
                    type: string
                  source:
                    type: string
                  target:
                    type: string
                  type:
                    type: string
            startNode:
              type: string
        rule_expression:
          type: string
          example: "temperature > 85 and vibration > 40"
        llm_criteria:
          type: string
          example: "Assess if maintenance is required based on sensor data"
        hybrid_strategy:
          type: string
          enum: [rule_first, llm_first, parallel, consensus]
          default: rule_first
        required_context:
          type: array
          items:
            type: object
            properties:
              type:
                type: string
                enum: [machine_status, historical_data, policy_documents]
              config:
                type: object
        tags:
          type: array
          items:
            type: string
          maxItems: 10
      required: [name, definition]
  Response:
    201:
      description: "Workflow created successfully"
      schema:
        $ref: '#/definitions/Workflow'
    400:
      description: "Invalid request data"
    409:
      description: "Workflow with same name already exists"

# GET /api/v1/workflows/{id}
Get_Workflow:
  Description: "특정 워크플로우 조회"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
    - name: include_executions
      in: query
      type: boolean
      default: false
      description: "최근 실행 이력 포함 여부"
    - name: execution_limit
      in: query
      type: integer
      default: 10
      maximum: 100
      description: "포함할 실행 이력 수"
  Responses:
    200:
      description: "Workflow retrieved successfully"
      schema:
        allOf:
          - $ref: '#/definitions/Workflow'
          - type: object
            properties:
              recent_executions:
                type: array
                items:
                  $ref: '#/definitions/JudgmentExecution'
    404:
      description: "Workflow not found"

# PUT /api/v1/workflows/{id}
Update_Workflow:
  Description: "워크플로우 전체 업데이트"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
  Request:
    Content-Type: application/json
    Body:
      $ref: '#/definitions/WorkflowUpdateRequest'
  Response:
    200:
      description: "Workflow updated successfully"
      schema:
        $ref: '#/definitions/Workflow'
    400:
      description: "Invalid request data"
    404:
      description: "Workflow not found"
    409:
      description: "Version conflict"

# PATCH /api/v1/workflows/{id}
Partial_Update_Workflow:
  Description: "워크플로우 부분 업데이트"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        name:
          type: string
        description:
          type: string
        is_active:
          type: boolean
        tags:
          type: array
          items:
            type: string
  Response:
    200:
      description: "Workflow updated successfully"
    400:
      description: "Invalid request data"
    404:
      description: "Workflow not found"

# DELETE /api/v1/workflows/{id}
Delete_Workflow:
  Description: "워크플로우 삭제 (soft delete)"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
    - name: force
      in: query
      type: boolean
      default: false
      description: "강제 삭제 (hard delete)"
  Response:
    204:
      description: "Workflow deleted successfully"
    404:
      description: "Workflow not found"
    409:
      description: "Cannot delete workflow with active executions"

# POST /api/v1/workflows/{id}/simulate
Simulate_Workflow_V2:
  Description: "워크플로우 시뮬레이션 실행 (Ver2.0 - 6개 NodeType 지원 + 실행 이력 저장)"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        input_data:
          type: object
          example:
            temperature: 90
            vibration: 45
            machine_id: "M001"
        simulation_config:
          type: object
          properties:
            dry_run:
              type: boolean
              default: true
            include_explanations:
              type: boolean
              default: true
            timeout_seconds:
              type: integer
              default: 30
              maximum: 300
      required: [input_data]
  Response:
    200:
      description: "Simulation completed successfully (execution history saved)"
      schema:
        type: object
        properties:
          simulation_id:
            type: string
            format: uuid
          workflow_id:
            type: string
            format: uuid
          input_data:
            type: object
          final_result:
            type: object
            description: "최종 판단 결과 (JUDGMENT 노드 결과 or null)"
          steps_executed:
            type: array
            items:
              type: object
              properties:
                step_id:
                  type: string
                step_type:
                  type: string
                  enum: [TRIGGER, QUERY, CALC, JUDGMENT, APPROVAL, ALERT]
                label:
                  type: string
                input:
                  type: object
                output:
                  type: object
                  description: "단계별 실행 결과 (step_type 키 포함)"
                execution_time_ms:
                  type: integer
          total_execution_time_ms:
            type: integer
          status:
            type: string
            enum: [success, failed, partial]
            description: "success: 모든 단계 성공, failed: 에러 발생, partial: 일부 성공"
          execution_id:
            type: string
            description: "workflow_executions 테이블에 저장된 실행 이력 ID"
          confidence_score:
            type: number
            format: float
            description: "JUDGMENT 노드 신뢰도 (존재하는 경우)"
          explanation:
            type: string
            description: "실행 요약 설명"
    400:
      description: "Invalid simulation request"
    404:
      description: "Workflow not found"
    408:
      description: "Simulation timeout"
    500:
      description: "Simulation execution failed"
      schema:
        type: object
        properties:
          error:
            type: string
          failed_step:
            type: string
          execution_id:
            type: string
            description: "부분 실행 이력 ID (저장된 경우)"

# GET /api/v1/workflows/{id}/executions
Get_Workflow_Executions:
  Description: "워크플로우 실행 이력 목록 조회 (Ver2.0)"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
      description: "워크플로우 ID"
    - name: limit
      in: query
      type: integer
      default: 10
      maximum: 100
      description: "조회할 최대 이력 수"
    - name: offset
      in: query
      type: integer
      default: 0
      description: "건너뛸 이력 수"
    - name: status
      in: query
      type: string
      enum: [success, failed, partial]
      description: "실행 상태 필터"
    - name: date_from
      in: query
      type: string
      format: date
      description: "시작 날짜 (YYYY-MM-DD)"
    - name: date_to
      in: query
      type: string
      format: date
      description: "종료 날짜 (YYYY-MM-DD)"
    - name: sort
      in: query
      type: string
      enum: [created_at, execution_time_ms]
      default: created_at
      description: "정렬 기준"
    - name: order
      in: query
      type: string
      enum: [asc, desc]
      default: desc
      description: "정렬 순서"
  Response:
    200:
      description: "Execution history retrieved successfully"
      schema:
        type: object
        properties:
          workflow_id:
            type: string
            format: uuid
          total_count:
            type: integer
            description: "전체 실행 이력 수"
          executions:
            type: array
            items:
              type: object
              properties:
                id:
                  type: string
                  description: "실행 이력 ID"
                workflow_id:
                  type: string
                  format: uuid
                status:
                  type: string
                  enum: [success, failed, partial]
                execution_time_ms:
                  type: integer
                created_at:
                  type: string
                  format: date-time
                steps_count:
                  type: integer
                  description: "실행된 단계 수"
                has_judgment:
                  type: boolean
                  description: "JUDGMENT 노드 존재 여부"
          pagination:
            type: object
            properties:
              limit:
                type: integer
              offset:
                type: integer
              has_more:
                type: boolean
    404:
      description: "Workflow not found"

# GET /api/v1/workflows/executions/{execution_id}
Get_Workflow_Execution_Detail:
  Description: "워크플로우 실행 이력 상세 조회 (Ver2.0)"
  Parameters:
    - name: execution_id
      in: path
      type: string
      required: true
      description: "실행 이력 ID"
  Response:
    200:
      description: "Execution detail retrieved successfully"
      schema:
        type: object
        properties:
          id:
            type: string
            description: "실행 이력 ID"
          workflow_id:
            type: string
            format: uuid
          status:
            type: string
            enum: [success, failed, partial]
          steps_executed:
            type: array
            description: "실행된 단계 목록 (JSON)"
            items:
              type: object
              properties:
                step_id:
                  type: string
                step_type:
                  type: string
                label:
                  type: string
                input:
                  type: object
                output:
                  type: object
                execution_time_ms:
                  type: integer
          final_result:
            type: object
            description: "최종 판단 결과 (JUDGMENT 노드 결과 or null)"
          execution_time_ms:
            type: integer
            description: "총 실행 시간"
          created_at:
            type: string
            format: date-time
            description: "실행 시각"
    404:
      description: "Execution not found"
```

### 3.2 워크플로우 버전 관리 API
```yaml
# GET /api/v1/workflows/{id}/versions
Get_Workflow_Versions:
  Description: "워크플로우 버전 이력 조회"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
    - name: limit
      in: query
      type: integer
      default: 10
      maximum: 50
  Response:
    200:
      description: "Workflow versions retrieved"
      schema:
        type: object
        properties:
          versions:
            type: array
            items:
              type: object
              properties:
                version:
                  type: integer
                created_at:
                  type: string
                  format: date-time
                created_by:
                  type: string
                  format: uuid
                changes:
                  type: array
                  items:
                    type: string
                definition:
                  type: object

# POST /api/v1/workflows/{id}/versions
Create_Workflow_Version:
  Description: "새 워크플로우 버전 생성"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        definition:
          type: object
        change_summary:
          type: string
          maxLength: 200
        major_change:
          type: boolean
          default: false
      required: [definition, change_summary]
  Response:
    201:
      description: "New version created"
      schema:
        $ref: '#/definitions/Workflow'

# PUT /api/v1/workflows/{id}/versions/{version}/activate
Activate_Workflow_Version:
  Description: "특정 버전을 활성 버전으로 설정"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
    - name: version
      in: path
      type: integer
      required: true
  Response:
    200:
      description: "Version activated successfully"
    404:
      description: "Workflow or version not found"
```

### 3.3 AI 워크플로우 생성 API (Phase 9-2)
```yaml
# POST /api/v2/workflows/generate-draft
Generate_Workflow_Draft:
  Description: "자연어 입력으로 워크플로우 자동 생성 (AI 기반)"
  Tags: [Phase 9-2, AI Generator]
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        user_prompt:
          type: string
          minLength: 10
          maxLength: 1000
          example: "1호선 불량률이 3% 초과하면 팀장에게 알림 보내기"
          description: "사용자 자연어 요청 (한글/영문)"
      required: [user_prompt]
  Response:
    200:
      description: "Workflow draft generated successfully"
      schema:
        type: object
        properties:
          steps:
            type: array
            description: "생성된 워크플로우 스텝 배열"
            items:
              type: object
              properties:
                id:
                  type: string
                  example: "trigger_1"
                  description: "스텝 고유 ID"
                type:
                  type: string
                  enum: [TRIGGER, QUERY, CALC, JUDGMENT, APPROVAL, ALERT]
                  example: "TRIGGER"
                  description: "노드 타입 (Manufacturing DSL 6종)"
                label:
                  type: string
                  example: "불량률 3% 초과 감지"
                  description: "사용자 친화적 레이블"
                config:
                  type: object
                  example:
                    triggerType: "threshold"
                    metric: "불량률"
                    condition: "> 3%"
                  description: "노드별 설정 (동적 JSON)"
          metadata:
            type: object
            properties:
              generated_at:
                type: string
                format: date-time
                example: "2025-11-21T10:30:00Z"
              model_used:
                type: string
                example: "claude-sonnet-4-5-20250929"
              prompt_tokens:
                type: integer
                example: 1523
              completion_tokens:
                type: integer
                example: 387
    400:
      description: "Invalid user prompt (너무 짧거나 명확하지 않음)"
      schema:
        type: object
        properties:
          error:
            type: string
            example: "Prompt must be at least 10 characters"
    500:
      description: "Claude API 호출 실패 또는 JSON 파싱 에러"
      schema:
        type: object
        properties:
          error:
            type: string
            example: "Failed to parse Claude response as valid JSON"

  Implementation_Notes:
    - Backend: Tauri 커맨드 `generate_workflow_draft` (src-tauri/src/commands/workflow_v2.rs)
    - Service: ChatService::generate_workflow_from_prompt (src-tauri/src/services/chat_service.rs)
    - LLM Model: Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)
    - Temperature: 0.3 (일관된 구조화 출력)
    - Max Tokens: 4096
    - System Prompt: Manufacturing DSL 가이드 + 5개 Few-shot 예시 포함
    - Response Processing: Markdown code block 자동 제거 (```json ... ```)
    - Validation: serde_json으로 WorkflowStep 배열 파싱 검증

  Manufacturing_DSL_NodeTypes:
    TRIGGER:
      description: "워크플로우 시작 조건"
      examples:
        - "일정 기반 (cron)"
        - "임계값 초과 감지"
        - "이벤트 수신"
    QUERY:
      description: "데이터베이스 조회"
      examples:
        - "MES 데이터 조회"
        - "센서 데이터 조회"
        - "불량 이력 조회"
    CALC:
      description: "계산 및 집계"
      examples:
        - "평균 계산"
        - "표준편차 계산"
        - "비율 계산"
    JUDGMENT:
      description: "규칙 기반 또는 AI 판단"
      examples:
        - "불량 여부 판정"
        - "품질 등급 분류"
        - "이상 탐지"
    APPROVAL:
      description: "사람 승인 대기"
      examples:
        - "팀장 승인"
        - "품질 책임자 승인"
        - "생산 책임자 승인"
    ALERT:
      description: "알림 전송"
      examples:
        - "Slack 메시지"
        - "이메일 전송"
        - "SMS 발송"

  Frontend_Integration:
    - Component: AiGenerator.tsx (src/components/workflow/v2/AiGenerator.tsx)
    - Usage: WorkflowBuilderV2.tsx에 통합
    - User Flow:
      1. 사용자가 자연어 입력 (예: "불량률 모니터링")
      2. AI 생성 버튼 클릭
      3. Claude API 호출 (loading indicator 표시)
      4. 생성된 WorkflowStep 배열 수신
      5. 워크플로우 빌더에 자동 추가 (드래그앤드롭 가능)

  Testing:
    - Unit Tests: src-tauri/src/commands/tests/workflow_ai_tests.rs
    - Test Coverage:
      - System prompt 검증 (6개 NodeType, 5개 Few-shot 포함)
      - JSON 파싱 검증 (단순/복잡 워크플로우)
      - Markdown code block 제거 로직
      - 유효하지 않은 JSON 에러 처리
    - Test Results: 22/22 passing ✅
```

## 🧠 4. Judgment Service API (Port 8002)

### 4.1 판단 실행 API
```yaml
# POST /api/v1/judgment/execute
Execute_Judgment:
  Description: "하이브리드 판단 실행"
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        workflow_id:
          type: string
          format: uuid
          example: "550e8400-e29b-41d4-a716-446655440000"
        input_data:
          type: object
          description: "판단에 사용될 입력 데이터"
          example:
            temperature: 90
            vibration: 45
            pressure: 120
            machine_id: "M001"
            timestamp: "2024-08-10T12:00:00Z"
        method:
          type: string
          enum: [rule, llm, hybrid]
          default: hybrid
          description: "판단 방식 선택"
        options:
          type: object
          properties:
            timeout_seconds:
              type: integer
              default: 30
              maximum: 300
            include_explanation:
              type: boolean
              default: true
            confidence_threshold:
              type: number
              format: float
              minimum: 0.0
              maximum: 1.0
              default: 0.7
            enable_caching:
              type: boolean
              default: true
            priority:
              type: string
              enum: [low, normal, high, urgent]
              default: normal
        context:
          type: object
          description: "추가 컨텍스트 데이터"
          properties:
            user_id:
              type: string
              format: uuid
            session_id:
              type: string
            external_data:
              type: object
      required: [workflow_id, input_data]
  Response:
    200:
      description: "Judgment executed successfully"
      schema:
        type: object
        properties:
          execution_id:
            type: string
            format: uuid
          workflow_id:
            type: string
            format: uuid
          result:
            description: "판단 결과 (boolean, string, number, object)"
          confidence_score:
            type: number
            format: float
            minimum: 0.0
            maximum: 1.0
          method_used:
            type: string
            enum: [rule, llm, hybrid]
          execution_time_ms:
            type: integer
          explanation:
            type: string
            description: "판단 근거 설명"
          metadata:
            type: object
            properties:
              rule_result:
                type: object
                properties:
                  success:
                    type: boolean
                  result:
                    description: "Rule 엔진 결과"
                  confidence:
                    type: number
                  error:
                    type: string
              llm_result:
                type: object
                properties:
                  success:
                    type: boolean
                  result:
                    description: "LLM 결과"
                  confidence:
                    type: number
                  model_used:
                    type: string
                  tokens_used:
                    type: integer
              context_used:
                type: object
              cache_hit:
                type: boolean
          recommended_actions:
            type: array
            items:
              type: object
              properties:
                action_type:
                  type: string
                target_system:
                  type: string
                command:
                  type: object
                priority:
                  type: string
                  enum: [low, medium, high, urgent]
          timestamp:
            type: string
            format: date-time
    400:
      description: "Invalid request data"
      schema:
        type: object
        properties:
          errors:
            type: array
            items:
              type: object
              properties:
                code:
                  type: string
                  example: "INVALID_WORKFLOW_ID"
                message:
                  type: string
                field:
                  type: string
    404:
      description: "Workflow not found"
    408:
      description: "Judgment execution timeout"
    422:
      description: "Judgment execution failed"
      schema:
        type: object
        properties:
          execution_id:
            type: string
            format: uuid
          error_code:
            type: string
            enum: [RULE_ENGINE_ERROR, LLM_API_ERROR, CONTEXT_ERROR, VALIDATION_ERROR]
          error_message:
            type: string
          retry_possible:
            type: boolean
    429:
      description: "Rate limit exceeded"
    503:
      description: "Service temporarily unavailable"

# GET /api/v1/judgment/executions/{id}
Get_Execution_Details:
  Description: "특정 판단 실행 결과 조회"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
    - name: include_metadata
      in: query
      type: boolean
      default: false
    - name: include_context
      in: query
      type: boolean
      default: false
  Response:
    200:
      description: "Execution details retrieved"
      schema:
        $ref: '#/definitions/JudgmentExecution'
    404:
      description: "Execution not found"

# GET /api/v1/judgment/executions/{id}/status
Get_Execution_Status:
  Description: "판단 실행 상태 조회 (비동기 실행용)"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
  Response:
    200:
      description: "Execution status retrieved"
      schema:
        type: object
        properties:
          execution_id:
            type: string
            format: uuid
          status:
            type: string
            enum: [pending, running, completed, failed, timeout]
          progress:
            type: integer
            minimum: 0
            maximum: 100
          started_at:
            type: string
            format: date-time
          completed_at:
            type: string
            format: date-time
          estimated_completion:
            type: string
            format: date-time
          current_step:
            type: string
          result:
            description: "완료된 경우에만 포함"
    404:
      description: "Execution not found"

# POST /api/v1/judgment/executions/{id}/cancel
Cancel_Execution:
  Description: "실행 중인 판단 작업 취소"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
  Response:
    200:
      description: "Execution cancelled successfully"
    404:
      description: "Execution not found"
    409:
      description: "Cannot cancel completed execution"
```

### 4.2 판단 이력 및 통계 API
```yaml
# GET /api/v1/judgment/executions
List_Executions:
  Description: "판단 실행 이력 조회"
  Parameters:
    - name: workflow_id
      in: query
      type: string
      format: uuid
    - name: status
      in: query
      type: string
      enum: [completed, failed, timeout]
    - name: method_used
      in: query
      type: string
      enum: [rule, llm, hybrid]
    - name: date_from
      in: query
      type: string
      format: date
    - name: date_to
      in: query
      type: string
      format: date
    - name: min_confidence
      in: query
      type: number
      format: float
    - name: max_confidence
      in: query
      type: number
      format: float
    - name: page
      in: query
      type: integer
      default: 1
    - name: limit
      in: query
      type: integer
      default: 20
      maximum: 100
    - name: sort
      in: query
      type: string
      enum: [created_at, execution_time_ms, confidence_score]
      default: created_at
    - name: order
      in: query
      type: string
      enum: [asc, desc]
      default: desc
  Response:
    200:
      description: "Executions retrieved successfully"
      schema:
        type: object
        properties:
          executions:
            type: array
            items:
              $ref: '#/definitions/JudgmentExecutionSummary'
          pagination:
            $ref: '#/definitions/Pagination'
          filters_applied:
            type: object

# GET /api/v1/judgment/statistics
Get_Judgment_Statistics:
  Description: "판단 통계 조회"
  Parameters:
    - name: time_range
      in: query
      type: string
      enum: [1h, 24h, 7d, 30d, 90d]
      default: 24h
    - name: workflow_ids
      in: query
      type: array
      items:
        type: string
        format: uuid
      description: "특정 워크플로우로 필터링"
    - name: group_by
      in: query
      type: string
      enum: [workflow, method, hour, day]
      default: workflow
  Response:
    200:
      description: "Statistics retrieved successfully"
      schema:
        type: object
        properties:
          summary:
            type: object
            properties:
              total_executions:
                type: integer
              successful_executions:
                type: integer
              failed_executions:
                type: integer
              success_rate:
                type: number
                format: float
              average_confidence:
                type: number
                format: float
              average_execution_time_ms:
                type: number
              method_distribution:
                type: object
                properties:
                  rule:
                    type: integer
                  llm:
                    type: integer
                  hybrid:
                    type: integer
          time_series:
            type: array
            items:
              type: object
              properties:
                timestamp:
                  type: string
                  format: date-time
                executions_count:
                  type: integer
                success_rate:
                  type: number
                average_confidence:
                  type: number
                average_execution_time_ms:
                  type: number
          breakdown:
            type: array
            items:
              type: object
              properties:
                group_key:
                  type: string
                group_value:
                  type: string
                statistics:
                  type: object
          period_start:
            type: string
            format: date-time
          period_end:
            type: string
            format: date-time

# POST /api/v1/judgment/feedback
Submit_Feedback:
  Description: "판단 결과에 대한 피드백 제출"
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        execution_id:
          type: string
          format: uuid
        feedback_type:
          type: string
          enum: [correct, incorrect, partially_correct]
        correct_result:
          description: "실제 정답 (incorrect인 경우)"
        explanation:
          type: string
          maxLength: 1000
        confidence_rating:
          type: integer
          minimum: 1
          maximum: 5
          description: "사용자가 평가한 판단의 신뢰도"
        metadata:
          type: object
          properties:
            user_id:
              type: string
              format: uuid
            context:
              type: string
      required: [execution_id, feedback_type]
  Response:
    201:
      description: "Feedback submitted successfully"
      schema:
        type: object
        properties:
          feedback_id:
            type: string
            format: uuid
          execution_id:
            type: string
            format: uuid
          status:
            type: string
            example: "received"
    404:
      description: "Execution not found"
```

### 4.3 모델 관리 API
```yaml
# GET /api/v1/judgment/models
List_Available_Models:
  Description: "사용 가능한 LLM 모델 목록"
  Response:
    200:
      description: "Models retrieved successfully"
      schema:
        type: object
        properties:
          models:
            type: array
            items:
              type: object
              properties:
                model_id:
                  type: string
                  example: "gpt-4"
                model_name:
                  type: string
                  example: "GPT-4"
                provider:
                  type: string
                  example: "openai"
                capabilities:
                  type: array
                  items:
                    type: string
                max_tokens:
                  type: integer
                cost_per_token:
                  type: number
                  format: float
                availability:
                  type: string
                  enum: [available, limited, unavailable]
                recommended_use_cases:
                  type: array
                  items:
                    type: string

# PUT /api/v1/judgment/models/default
Set_Default_Model:
  Description: "기본 LLM 모델 설정"
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        model_id:
          type: string
        workflow_id:
          type: string
          format: uuid
          description: "특정 워크플로우용 (선택사항)"
      required: [model_id]
  Response:
    200:
      description: "Default model updated successfully"
```

## ⚡ 5. Action Service API (Port 8003)

### 5.1 액션 실행 API
```yaml
# POST /api/v1/actions/execute
Execute_Action:
  Description: "외부 시스템 액션 실행"
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        judgment_execution_id:
          type: string
          format: uuid
          description: "관련된 판단 실행 ID"
        action_type:
          type: string
          enum: [slack_notification, mcp_command, webhook_call, email_notification, sms_notification]
        target_system:
          type: string
          example: "slack-channel-alerts"
        command:
          type: object
          description: "액션별 특화된 명령 데이터"
          example:
            channel: "#alerts"
            message: "Temperature alert: 90°C detected"
            severity: "high"
        options:
          type: object
          properties:
            retry_policy:
              type: object
              properties:
                max_retries:
                  type: integer
                  default: 3
                backoff_strategy:
                  type: string
                  enum: [linear, exponential]
                  default: exponential
                retry_delay_seconds:
                  type: integer
                  default: 1
            timeout_seconds:
              type: integer
              default: 30
              maximum: 300
            priority:
              type: string
              enum: [low, normal, high, urgent]
              default: normal
            async_execution:
              type: boolean
              default: true
            callback_url:
              type: string
              format: uri
              description: "비동기 실행 완료 알림 URL"
        metadata:
          type: object
          properties:
            correlation_id:
              type: string
            user_id:
              type: string
              format: uuid
            additional_context:
              type: object
      required: [action_type, target_system, command]
  Response:
    202:
      description: "Action queued for execution (async)"
      schema:
        type: object
        properties:
          action_id:
            type: string
            format: uuid
          status:
            type: string
            enum: [queued, running, completed, failed]
          estimated_completion:
            type: string
            format: date-time
          queue_position:
            type: integer
    200:
      description: "Action completed immediately (sync)"
      schema:
        type: object
        properties:
          action_id:
            type: string
            format: uuid
          status:
            type: string
            value: completed
          result:
            type: object
          execution_time_ms:
            type: integer
    400:
      description: "Invalid action request"
    422:
      description: "Action validation failed"

# GET /api/v1/actions/{id}
Get_Action_Status:
  Description: "액션 실행 상태 및 결과 조회"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
    - name: include_logs
      in: query
      type: boolean
      default: false
  Response:
    200:
      description: "Action details retrieved"
      schema:
        type: object
        properties:
          action_id:
            type: string
            format: uuid
          judgment_execution_id:
            type: string
            format: uuid
          action_type:
            type: string
          target_system:
            type: string
          command:
            type: object
          status:
            type: string
            enum: [queued, running, completed, failed, cancelled]
          result:
            type: object
            description: "완료된 경우에만 포함"
          error:
            type: object
            properties:
              error_code:
                type: string
              error_message:
                type: string
              retry_count:
                type: integer
              next_retry_at:
                type: string
                format: date-time
          execution_history:
            type: array
            items:
              type: object
              properties:
                attempt:
                  type: integer
                started_at:
                  type: string
                  format: date-time
                completed_at:
                  type: string
                  format: date-time
                status:
                  type: string
                error:
                  type: string
          created_at:
            type: string
            format: date-time
          updated_at:
            type: string
            format: date-time
          logs:
            type: array
            items:
              type: object
              properties:
                timestamp:
                  type: string
                  format: date-time
                level:
                  type: string
                message:
                  type: string
    404:
      description: "Action not found"

# POST /api/v1/actions/{id}/retry
Retry_Action:
  Description: "실패한 액션 재시도"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        reset_retry_count:
          type: boolean
          default: false
        override_command:
          type: object
          description: "명령 재정의 (선택사항)"
  Response:
    202:
      description: "Action retry queued"
    404:
      description: "Action not found"
    409:
      description: "Cannot retry action in current status"

# POST /api/v1/actions/{id}/cancel
Cancel_Action:
  Description: "실행 중인 액션 취소"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
  Response:
    200:
      description: "Action cancelled successfully"
    404:
      description: "Action not found"
    409:
      description: "Cannot cancel action in current status"
```

### 5.2 액션 템플릿 및 설정 API
```yaml
# GET /api/v1/actions/templates
List_Action_Templates:
  Description: "사용 가능한 액션 템플릿 목록"
  Parameters:
    - name: action_type
      in: query
      type: string
      enum: [slack_notification, mcp_command, webhook_call, email_notification]
  Response:
    200:
      description: "Action templates retrieved"
      schema:
        type: object
        properties:
          templates:
            type: array
            items:
              type: object
              properties:
                template_id:
                  type: string
                name:
                  type: string
                description:
                  type: string
                action_type:
                  type: string
                command_schema:
                  type: object
                  description: "JSON Schema for command validation"
                example_command:
                  type: object
                supported_systems:
                  type: array
                  items:
                    type: string

# GET /api/v1/actions/systems
List_Target_Systems:
  Description: "연동 가능한 외부 시스템 목록"
  Response:
    200:
      description: "Target systems retrieved"
      schema:
        type: object
        properties:
          systems:
            type: array
            items:
              type: object
              properties:
                system_id:
                  type: string
                system_name:
                  type: string
                system_type:
                  type: string
                status:
                  type: string
                  enum: [active, inactive, error]
                supported_actions:
                  type: array
                  items:
                    type: string
                configuration:
                  type: object
                last_health_check:
                  type: string
                  format: date-time

# POST /api/v1/actions/systems/{system_id}/health-check
Health_Check_System:
  Description: "외부 시스템 연결 상태 확인"
  Parameters:
    - name: system_id
      in: path
      type: string
      required: true
  Response:
    200:
      description: "Health check completed"
      schema:
        type: object
        properties:
          system_id:
            type: string
          status:
            type: string
            enum: [healthy, unhealthy, timeout]
          response_time_ms:
            type: integer
          last_error:
            type: string
          capabilities:
            type: array
            items:
              type: string
          metadata:
            type: object
```

## 📊 6. Dashboard Service API (Port 8006)

### 6.1 대시보드 자동 생성 API
```yaml
# POST /api/v1/dashboards/generate
Generate_Dashboard:
  Description: "자연어 요청으로 대시보드 자동 생성"
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        request:
          type: string
          minLength: 10
          maxLength: 500
          example: "지난 주 워크플로우별 성공률을 차트로 보여줘"
        options:
          type: object
          properties:
            dashboard_type:
              type: string
              enum: [summary, detailed, custom]
              default: summary
            time_range:
              type: string
              enum: [1h, 24h, 7d, 30d, 90d, custom]
              default: 7d
            custom_date_range:
              type: object
              properties:
                start_date:
                  type: string
                  format: date
                end_date:
                  type: string
                  format: date
            refresh_interval:
              type: integer
              default: 30
              description: "초 단위 자동 새로고침"
            chart_preferences:
              type: array
              items:
                type: string
                enum: [bar_chart, line_chart, pie_chart, metric_card, table, gauge]
            include_real_time:
              type: boolean
              default: true
            target_audience:
              type: string
              enum: [executive, manager, operator, analyst]
              default: manager
        context:
          type: object
          properties:
            user_id:
              type: string
              format: uuid
            workflow_ids:
              type: array
              items:
                type: string
                format: uuid
              description: "특정 워크플로우로 제한"
            department:
              type: string
            location:
              type: string
      required: [request]
  Response:
    201:
      description: "Dashboard generation started"
      schema:
        type: object
        properties:
          generation_id:
            type: string
            format: uuid
          status:
            type: string
            enum: [analyzing, generating, completed, failed]
          estimated_completion:
            type: string
            format: date-time
          progress:
            type: integer
            minimum: 0
            maximum: 100
          analysis:
            type: object
            properties:
              detected_intent:
                type: string
              required_data_sources:
                type: array
                items:
                  type: string
              suggested_chart_types:
                type: array
                items:
                  type: string
              complexity_score:
                type: number
                format: float
          websocket_url:
            type: string
            description: "실시간 진행상황 확인용"
    400:
      description: "Invalid generation request"
    422:
      description: "Cannot understand request"
      schema:
        type: object
        properties:
          error_code:
            type: string
            example: "AMBIGUOUS_REQUEST"
          suggestions:
            type: array
            items:
              type: string
          clarification_questions:
            type: array
            items:
              type: string

# GET /api/v1/dashboards/generation/{id}
Get_Generation_Status:
  Description: "대시보드 생성 상태 확인"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
  Response:
    200:
      description: "Generation status retrieved"
      schema:
        type: object
        properties:
          generation_id:
            type: string
            format: uuid
          status:
            type: string
            enum: [analyzing, generating, completed, failed]
          progress:
            type: integer
          current_step:
            type: string
          dashboard_id:
            type: string
            format: uuid
            description: "완료된 경우에만 포함"
          error:
            type: object
            properties:
              error_code:
                type: string
              error_message:
                type: string
          generated_components:
            type: array
            items:
              type: object
              properties:
                component_type:
                  type: string
                title:
                  type: string
                data_query:
                  type: string
                chart_config:
                  type: object
          estimated_completion:
            type: string
            format: date-time

# GET /api/v1/dashboards/{id}
Get_Dashboard:
  Description: "생성된 대시보드 조회"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
    - name: include_data
      in: query
      type: boolean
      default: true
      description: "실제 데이터 포함 여부"
  Response:
    200:
      description: "Dashboard retrieved successfully"
      schema:
        type: object
        properties:
          dashboard_id:
            type: string
            format: uuid
          title:
            type: string
          description:
            type: string
          created_at:
            type: string
            format: date-time
          created_by:
            type: string
            format: uuid
          last_updated:
            type: string
            format: date-time
          layout:
            type: object
            properties:
              grid_columns:
                type: integer
                default: 12
              components:
                type: array
                items:
                  type: object
                  properties:
                    id:
                      type: string
                    type:
                      type: string
                      enum: [bar_chart, line_chart, pie_chart, metric_card, table, gauge, text]
                    position:
                      type: object
                      properties:
                        x:
                          type: integer
                        y:
                          type: integer
                        width:
                          type: integer
                        height:
                          type: integer
                    title:
                      type: string
                    config:
                      type: object
                      description: "컴포넌트별 설정"
                    data_query:
                      type: object
                      description: "데이터 쿼리 정의"
                    data:
                      description: "실제 데이터 (include_data=true인 경우)"
          settings:
            type: object
            properties:
              refresh_interval:
                type: integer
              auto_refresh:
                type: boolean
              real_time_enabled:
                type: boolean
              theme:
                type: string
                enum: [light, dark, auto]
          metadata:
            type: object
            properties:
              original_request:
                type: string
              generation_method:
                type: string
              data_sources:
                type: array
                items:
                  type: string
              tags:
                type: array
                items:
                  type: string
    404:
      description: "Dashboard not found"

# PUT /api/v1/dashboards/{id}
Update_Dashboard:
  Description: "대시보드 수정"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        title:
          type: string
        description:
          type: string
        layout:
          type: object
        settings:
          type: object
        metadata:
          type: object
  Response:
    200:
      description: "Dashboard updated successfully"
    404:
      description: "Dashboard not found"
    400:
      description: "Invalid update data"
```

### 6.2 실시간 데이터 API
```yaml
# WebSocket /ws/dashboard/{id}/data
Dashboard_Data_Stream:
  Description: "대시보드 실시간 데이터 스트림"
  Connection:
    URL: "ws://dashboard-service:8006/ws/dashboard/{id}/data"
    Headers:
      Authorization: "Bearer {jwt_token}"
  Messages:
    # Client → Server
    Subscribe:
      type: object
      properties:
        action:
          type: string
          value: "subscribe"
        components:
          type: array
          items:
            type: string
          description: "구독할 컴포넌트 ID 목록"
        refresh_rate:
          type: integer
          default: 5
          description: "초 단위 업데이트 주기"
    
    Unsubscribe:
      type: object
      properties:
        action:
          type: string
          value: "unsubscribe"
        components:
          type: array
          items:
            type: string
    
    # Server → Client
    Data_Update:
      type: object
      properties:
        type:
          type: string
          value: "data_update"
        timestamp:
          type: string
          format: date-time
        component_id:
          type: string
        data:
          description: "새로운 데이터"
        metadata:
          type: object
          properties:
            data_source:
              type: string
            query_time_ms:
              type: integer
            cache_hit:
              type: boolean
    
    Error:
      type: object
      properties:
        type:
          type: string
          value: "error"
        error_code:
          type: string
        error_message:
          type: string
        component_id:
          type: string
    
    Connection_Status:
      type: object
      properties:
        type:
          type: string
          value: "connection_status"
        status:
          type: string
          enum: [connected, disconnected, reconnecting]
        client_count:
          type: integer

# GET /api/v1/dashboards/{id}/data
Get_Dashboard_Data:
  Description: "대시보드 데이터 일괄 조회"
  Parameters:
    - name: id
      in: path
      type: string
      format: uuid
      required: true
    - name: components
      in: query
      type: array
      items:
        type: string
      description: "특정 컴포넌트만 조회"
    - name: force_refresh
      in: query
      type: boolean
      default: false
      description: "캐시 무시하고 강제 새로고침"
  Response:
    200:
      description: "Dashboard data retrieved"
      schema:
        type: object
        properties:
          dashboard_id:
            type: string
            format: uuid
          timestamp:
            type: string
            format: date-time
          components:
            type: object
            additionalProperties:
              type: object
              properties:
                component_id:
                  type: string
                data:
                  description: "컴포넌트별 데이터"
                metadata:
                  type: object
                  properties:
                    last_updated:
                      type: string
                      format: date-time
                    data_source:
                      type: string
                    query_time_ms:
                      type: integer
                    record_count:
                      type: integer
                error:
                  type: object
                  properties:
                    error_code:
                      type: string
                    error_message:
                      type: string
```

## 📝 7. Logging Service API (Port 8005)

### 7.1 로그 수집 및 조회 API
```yaml
# POST /api/v1/logs/ingest
Ingest_Logs:
  Description: "구조화된 로그 수집"
  Request:
    Content-Type: application/json
    Body:
      type: object
      properties:
        logs:
          type: array
          items:
            type: object
            properties:
              timestamp:
                type: string
                format: date-time
              level:
                type: string
                enum: [DEBUG, INFO, WARN, ERROR, FATAL]
              message:
                type: string
              service:
                type: string
              component:
                type: string
              trace_id:
                type: string
              span_id:
                type: string
              user_id:
                type: string
                format: uuid
              session_id:
                type: string
              correlation_id:
                type: string
              fields:
                type: object
                description: "추가 구조화 데이터"
              tags:
                type: object
                additionalProperties:
                  type: string
            required: [timestamp, level, message, service]
      required: [logs]
  Response:
    202:
      description: "Logs accepted for processing"
      schema:
        type: object
        properties:
          accepted_count:
            type: integer
          rejected_count:
            type: integer
          batch_id:
            type: string

# GET /api/v1/logs/search
Search_Logs:
  Description: "로그 검색 및 필터링"
  Parameters:
    - name: query
      in: query
      type: string
      description: "검색 쿼리 (Lucene 문법 지원)"
    - name: services
      in: query
      type: array
      items:
        type: string
      description: "서비스 필터"
    - name: levels
      in: query
      type: array
      items:
        type: string
        enum: [DEBUG, INFO, WARN, ERROR, FATAL]
    - name: start_time
      in: query
      type: string
      format: date-time
    - name: end_time
      in: query
      type: string
      format: date-time
    - name: trace_id
      in: query
      type: string
    - name: user_id
      in: query
      type: string
      format: uuid
    - name: correlation_id
      in: query
      type: string
    - name: page
      in: query
      type: integer
      default: 1
    - name: limit
      in: query
      type: integer
      default: 100
      maximum: 1000
    - name: sort
      in: query
      type: string
      enum: [timestamp, level]
      default: timestamp
    - name: order
      in: query
      type: string
      enum: [asc, desc]
      default: desc
  Response:
    200:
      description: "Logs retrieved successfully"
      schema:
        type: object
        properties:
          logs:
            type: array
            items:
              type: object
              properties:
                id:
                  type: string
                timestamp:
                  type: string
                  format: date-time
                level:
                  type: string
                message:
                  type: string
                service:
                  type: string
                component:
                  type: string
                trace_id:
                  type: string
                fields:
                  type: object
                tags:
                  type: object
          pagination:
            $ref: '#/definitions/Pagination'
          aggregations:
            type: object
            properties:
              levels:
                type: object
                additionalProperties:
                  type: integer
              services:
                type: object
                additionalProperties:
                  type: integer
              time_histogram:
                type: array
                items:
                  type: object
                  properties:
                    timestamp:
                      type: string
                      format: date-time
                    count:
                      type: integer

# GET /api/v1/logs/statistics
Get_Log_Statistics:
  Description: "로그 통계 및 메트릭"
  Parameters:
    - name: time_range
      in: query
      type: string
      enum: [1h, 24h, 7d, 30d]
      default: 24h
    - name: services
      in: query
      type: array
      items:
        type: string
    - name: group_by
      in: query
      type: string
      enum: [service, level, hour, day]
      default: service
  Response:
    200:
      description: "Log statistics retrieved"
      schema:
        type: object
        properties:
          summary:
            type: object
            properties:
              total_logs:
                type: integer
              error_count:
                type: integer
              warn_count:
                type: integer
              error_rate:
                type: number
                format: float
              top_errors:
                type: array
                items:
                  type: object
                  properties:
                    error_message:
                      type: string
                    count:
                      type: integer
                    first_seen:
                      type: string
                      format: date-time
                    last_seen:
                      type: string
                      format: date-time
              services_breakdown:
                type: object
                additionalProperties:
                  type: object
                  properties:
                    total_logs:
                      type: integer
                    error_count:
                      type: integer
                    error_rate:
                      type: number
          time_series:
            type: array
            items:
              type: object
              properties:
                timestamp:
                  type: string
                  format: date-time
                total_count:
                  type: integer
                error_count:
                  type: integer
                levels:
                  type: object
                  additionalProperties:
                    type: integer
```

## 🔄 8. GraphQL Federation Schema

### 8.1 통합 GraphQL 스키마
```graphql
# Gateway Schema (통합)
schema {
  query: Query
  mutation: Mutation
  subscription: Subscription
}

type Query {
  # Workflow Service
  workflow(id: ID!): Workflow
  workflows(filter: WorkflowFilter, pagination: PaginationInput): WorkflowConnection!
  
  # Judgment Service
  judgmentExecution(id: ID!): JudgmentExecution
  judgmentExecutions(filter: JudgmentExecutionFilter, pagination: PaginationInput): JudgmentExecutionConnection!
  judgmentStatistics(filter: StatisticsFilter): JudgmentStatistics
  
  # Action Service
  action(id: ID!): Action
  actions(filter: ActionFilter, pagination: PaginationInput): ActionConnection!
  
  # Dashboard Service
  dashboard(id: ID!): Dashboard
  dashboards(filter: DashboardFilter, pagination: PaginationInput): DashboardConnection!
  
  # Logging Service
  logs(filter: LogFilter, pagination: PaginationInput): LogConnection!
  logStatistics(filter: LogStatisticsFilter): LogStatistics
  
  # 연합 쿼리 (Cross-service)
  workflowWithExecutions(id: ID!, executionLimit: Int): WorkflowWithExecutions
  userDashboard(userId: ID!): UserDashboard
}

type Mutation {
  # Workflow Service
  createWorkflow(input: CreateWorkflowInput!): Workflow!
  updateWorkflow(id: ID!, input: UpdateWorkflowInput!): Workflow!
  deleteWorkflow(id: ID!): DeleteResult!
  simulateWorkflow(id: ID!, input: SimulationInput!): SimulationResult!
  
  # Judgment Service
  executeJudgment(input: ExecuteJudgmentInput!): JudgmentExecution!
  submitJudgmentFeedback(input: FeedbackInput!): Feedback!
  
  # Action Service
  executeAction(input: ExecuteActionInput!): Action!
  retryAction(id: ID!): Action!
  cancelAction(id: ID!): Action!
  
  # Dashboard Service
  generateDashboard(input: GenerateDashboardInput!): DashboardGeneration!
  updateDashboard(id: ID!, input: UpdateDashboardInput!): Dashboard!
  deleteDashboard(id: ID!): DeleteResult!
}

type Subscription {
  # Real-time subscriptions
  judgmentExecutionUpdates(workflowId: ID): JudgmentExecution!
  actionStatusUpdates(actionId: ID): Action!
  dashboardDataUpdates(dashboardId: ID!): DashboardDataUpdate!
  systemLogs(filter: LogSubscriptionFilter): LogEntry!
}

# Core Types
type Workflow @key(fields: "id") {
  id: ID!
  name: String!
  description: String
  definition: JSON!
  version: Int!
  isActive: Boolean!
  createdAt: DateTime!
  updatedAt: DateTime!
  createdBy: User!
  
  # Federated fields
  executions(limit: Int, offset: Int): [JudgmentExecution!]! @requires(fields: "id")
  statistics: WorkflowStatistics @requires(fields: "id")
}

type JudgmentExecution @key(fields: "id") {
  id: ID!
  workflowId: ID!
  inputData: JSON!
  result: JSON
  confidenceScore: Float
  methodUsed: JudgmentMethod!
  executionTimeMs: Int!
  status: ExecutionStatus!
  createdAt: DateTime!
  
  # Federated fields
  workflow: Workflow! @provides(fields: "id name")
  actions: [Action!]! @requires(fields: "id")
  logs: [LogEntry!]! @requires(fields: "id")
}

type Action @key(fields: "id") {
  id: ID!
  judgmentExecutionId: ID
  actionType: ActionType!
  targetSystem: String!
  command: JSON!
  status: ActionStatus!
  result: JSON
  createdAt: DateTime!
  updatedAt: DateTime!
  
  # Federated fields
  judgmentExecution: JudgmentExecution @provides(fields: "id")
  logs: [LogEntry!]! @requires(fields: "id")
}

type Dashboard @key(fields: "id") {
  id: ID!
  title: String!
  description: String
  layout: DashboardLayout!
  settings: DashboardSettings!
  createdAt: DateTime!
  createdBy: User!
  
  # Real-time data
  data: DashboardData @requires(fields: "id")
}

type LogEntry {
  id: ID!
  timestamp: DateTime!
  level: LogLevel!
  message: String!
  service: String!
  component: String
  traceId: String
  fields: JSON
  tags: JSON
}

# Input Types
input WorkflowFilter {
  status: WorkflowStatus
  createdBy: ID
  search: String
  tags: [String!]
  dateRange: DateRangeInput
}

input JudgmentExecutionFilter {
  workflowIds: [ID!]
  status: ExecutionStatus
  methodUsed: JudgmentMethod
  confidenceRange: FloatRangeInput
  dateRange: DateRangeInput
}

input ExecuteJudgmentInput {
  workflowId: ID!
  inputData: JSON!
  method: JudgmentMethod = HYBRID
  options: JudgmentOptionsInput
  context: JSON
}

input GenerateDashboardInput {
  request: String!
  options: DashboardOptionsInput
  context: DashboardContextInput
}

# Enums
enum JudgmentMethod {
  RULE
  LLM
  HYBRID
}

enum ExecutionStatus {
  PENDING
  RUNNING
  COMPLETED
  FAILED
  TIMEOUT
  CANCELLED
}

enum ActionType {
  SLACK_NOTIFICATION
  MCP_COMMAND
  WEBHOOK_CALL
  EMAIL_NOTIFICATION
  SMS_NOTIFICATION
}

enum ActionStatus {
  QUEUED
  RUNNING
  COMPLETED
  FAILED
  CANCELLED
}

enum LogLevel {
  DEBUG
  INFO
  WARN
  ERROR
  FATAL
}

# Scalar Types
scalar JSON
scalar DateTime
scalar Upload
```

### 8.2 서비스별 Schema 확장
```graphql
# Workflow Service Schema Extension
extend type Query {
  workflow(id: ID!): Workflow @provides(fields: "id name definition version")
  workflows(filter: WorkflowFilter, pagination: PaginationInput): WorkflowConnection!
}

extend type Mutation {
  createWorkflow(input: CreateWorkflowInput!): Workflow!
  updateWorkflow(id: ID!, input: UpdateWorkflowInput!): Workflow!
}

# Judgment Service Schema Extension
extend type Query {
  judgmentExecution(id: ID!): JudgmentExecution @provides(fields: "id result confidenceScore")
  judgmentStatistics(filter: StatisticsFilter): JudgmentStatistics
}

extend type Workflow @key(fields: "id") {
  executions(limit: Int = 10): [JudgmentExecution!]!
  successRate: Float!
  averageExecutionTime: Int!
  totalExecutions: Int!
}

extend type Mutation {
  executeJudgment(input: ExecuteJudgmentInput!): JudgmentExecution!
}

# Action Service Schema Extension
extend type JudgmentExecution @key(fields: "id") {
  actions: [Action!]!
  triggeredActionsCount: Int!
  successfulActionsCount: Int!
}

# Dashboard Service Schema Extension
extend type Query {
  generateDashboard(input: GenerateDashboardInput!): DashboardGeneration!
}

type DashboardGeneration {
  id: ID!
  status: GenerationStatus!
  progress: Int!
  dashboard: Dashboard
  error: String
}

# Real-time Subscriptions
type Subscription {
  judgmentExecuted: JudgmentExecution!
  actionCompleted: Action!
  dashboardUpdated(dashboardId: ID!): DashboardDataUpdate!
}
```

이 API 명세서는 Judgify-core Ver2.0의 모든 마이크로서비스 간 통신을 위한 완전한 API 정의를 제공합니다. 각 서비스는 독립적으로 개발 및 배포 가능하면서도, GraphQL Federation을 통해 통합된 데이터 조회 경험을 제공합니다.