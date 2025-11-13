# MCP 통합 가이드 (Ver2.0 Final)

**목적**: Model Context Protocol (MCP) 도구를 활용한 마이크로서비스 개발 및 외부 시스템 연동

**관련 서비스**: 전체 9개 마이크로서비스

---

## 🎯 MCP 도구 우선순위

### 1단계: 핵심 MCP 도구 (즉시 필요)

| 도구 | 용도 | 관련 서비스 | 설치 우선순위 |
|------|------|------------|-------------|
| **postgresql-integration** | PostgreSQL 직접 연결 (Supabase 대체) | 전체 | 🔥 필수 |
| **filesystem-access** | 프로젝트 코드 관리 및 파일 처리 | 전체 | 🔥 필수 |
| **github-integration** | 코드 관리 및 CI/CD 파이프라인 | 전체 | 🔥 필수 |
| **memory-integration** | AI 판단 컨텍스트 및 세션 관리 | Judgment (8002), Chat (8008) | ⭐ 높음 |
| **playwright-mcp-server** | 마이크로서비스 E2E 테스트 자동화 | 전체 | ⭐ 높음 |

### 2단계: 확장 MCP 도구 (기능 확장)

| 도구 | 용도 | 관련 서비스 | 설치 우선순위 |
|------|------|------------|-------------|
| **context7** | 최신 라이브러리 문서 및 API 참조 | 개발 전반 | ✅ 중간 |
| **circleci** | CI/CD 파이프라인 자동화 | 배포 자동화 | ✅ 중간 |
| **deepgraph-typescript** | 코드 분석 및 아키텍처 검증 | Workflow (8001) | ✅ 중간 |
| **openai** | 하이브리드 판단 및 대시보드 생성용 LLM | Judgment, BI (8007) | ⭐ 높음 |
| **slack** | 판단 결과 알림 및 실시간 보고 | Action (8003) | ✅ 중간 |
| **notion** | 프로젝트 문서 및 설계 문서 관리 | 문서화 | ⬇️ 낮음 |
| **terminal** | Docker/Kubernetes 배포 명령 | 배포 자동화 | ✅ 중간 |
| **redis** | 캐시 및 세션 관리 | 전체 | ⭐ 높음 |

---

## 📋 Judgify-core 특화 MCP 활용 시나리오

### 시나리오 1: PostgreSQL MCP 활용

```bash
# 판단 실행 결과 조회
/query "SELECT * FROM judgment_executions WHERE confidence_score > 0.8"

# 워크플로우 성능 분석
/analyze-workflow-performance
# → workflow_id별 평균 실행 시간, 성공률 통계 반환

# 데이터베이스 스키마 최적화 제안
/optimize-database-schema
# → 인덱스 누락, 쿼리 성능 문제 자동 탐지
```

**사용 예시**:
```python
# PostgreSQL MCP를 통한 데이터 조회

import mcp_postgresql

# 고신뢰도 판단 결과 분석
high_confidence_results = mcp_postgresql.query(
    """
    SELECT
        workflow_id,
        AVG(confidence_score) as avg_confidence,
        COUNT(*) as total_executions
    FROM judgment_executions
    WHERE confidence_score >= 0.8
    GROUP BY workflow_id
    ORDER BY avg_confidence DESC
    LIMIT 10
    """
)

print(f"Top 10 workflows by confidence: {high_confidence_results}")
```

---

### 시나리오 2: Memory MCP 활용 (컨텍스트 관리)

```bash
# 하이브리드 판단 로직 개선사항 저장
/save-context "하이브리드 판단 로직 개선사항"
# 내용: Rule Engine 신뢰도 임계값을 0.7 → 0.75로 조정 (성능 개선)

# 마이크로서비스 아키텍처 설계 복원
/restore-context "마이크로서비스 아키텍처 설계"
# → Learning Service (8009) 추가 배경 및 설계 의도 복원
```

**사용 예시**:
```python
# Memory MCP를 통한 세션 관리

import mcp_memory

# Chat Interface에서 멀티턴 대화 컨텍스트 저장
session_id = "user-123-session-456"

mcp_memory.save_context(
    session_id=session_id,
    context={
        "last_workflow": "quality-check-workflow",
        "user_intent": "데이터 시각화",
        "conversation_history": [
            {"role": "user", "content": "품질 검사 워크플로우 실행해줘"},
            {"role": "assistant", "content": "워크플로우를 실행하겠습니다..."}
        ]
    }
)

# 이후 대화에서 컨텍스트 복원
restored_context = mcp_memory.restore_context(session_id)
print(f"Last workflow: {restored_context['last_workflow']}")
```

---

### 시나리오 3: GitHub MCP 활용 (CI/CD)

```bash
# Judgment Service 성능 최적화 이슈 생성
/create-issue "Judgment Service 성능 최적화"
# 내용: Rule Engine 실행 시간 200ms → 100ms 목표

# PR 리뷰
/review-pr 123
# → PR #123의 변경사항 분석 및 코멘트 추가

# 릴리스 노트 자동 생성
/generate-release-notes v2.0.0
# → v1.0.0 → v2.0.0 사이 커밋 기반 릴리스 노트
```

**사용 예시**:
```python
# GitHub MCP를 통한 자동 이슈 생성

import mcp_github

# 성능 테스트 실패시 자동 이슈 생성
performance_issue = mcp_github.create_issue(
    repo="mugoori/Judgify-core",
    title="[Performance] Judgment Service 응답 시간 초과",
    body="""
## 문제
- 평균 응답 시간: 520ms (목표: 500ms 이하)
- P99 응답 시간: 1200ms (목표: 1000ms 이하)

## 원인 분석 필요
- Rule Engine 실행 시간 증가
- pgvector 유사 사례 검색 최적화

## 관련 서비스
- Judgment Service (8002)
    """,
    labels=["performance", "judgment-service"],
    assignees=["mugoori"]
)

print(f"Issue created: {performance_issue['html_url']}")
```

---

### 시나리오 4: Context7 MCP 활용 (최신 문서)

```bash
# FastAPI 비동기 패턴 문서 조회
/get-docs "fastapi async patterns"
# → FastAPI 최신 비동기 프로그래밍 가이드 반환

# PostgreSQL pgvector 통합 예제 검색
/search-examples "postgresql pgvector integration"
# → pgvector 설치, 임베딩 생성, 유사도 검색 코드 예제
```

**사용 예시**:
```python
# Context7 MCP를 통한 라이브러리 문서 조회

import mcp_context7

# RAG 구현시 pgvector 활용법 조회
pgvector_docs = mcp_context7.get_docs("pgvector similarity search")

print(f"pgvector documentation:\n{pgvector_docs}")

# 예상 출력:
# - pgvector 설치 방법
# - 임베딩 벡터 저장 SQL
# - 코사인 유사도 검색 쿼리
```

---

## 🔧 외부 시스템 연동 패턴

### Action Service (8003) 통합

**역할**: Judgment Service 판단 결과 기반 외부 시스템 자동 제어

#### 패턴 1: Slack 알림

```python
# Claude가 구현해야 하는 Action Service 패턴

from typing import List
from slack_sdk import WebClient
from slack_sdk.errors import SlackApiError

class ActionExecutor:
    def __init__(self):
        self.slack_client = WebClient(token=os.getenv("SLACK_BOT_TOKEN"))
        self.mcp_client = MCPClient()

    async def execute_action(
        self,
        judgment_result: JudgmentResult
    ) -> ActionResult:
        """판단 결과 기반 액션 실행"""

        actions = judgment_result.recommended_actions
        results = []

        for action in actions:
            # Slack 알림
            if action.type == "slack_notification":
                result = await self._send_slack_alert(
                    channel=action.channel or "#alerts",
                    message=f"⚠️ 판단 완료: {judgment_result.result}",
                    confidence=judgment_result.confidence,
                    details=judgment_result.explanation
                )
                results.append(result)

            # MCP 시스템 제어 (예: PostgreSQL 자동 스케일링)
            elif action.type == "mcp_control":
                result = await self._execute_mcp_command(
                    system=action.target_system,
                    command=action.command,
                    parameters=action.parameters
                )
                results.append(result)

        return ActionResult(executed_actions=results)

    async def _send_slack_alert(
        self,
        channel: str,
        message: str,
        confidence: float,
        details: str
    ) -> dict:
        """Slack 알림 전송"""

        try:
            response = self.slack_client.chat_postMessage(
                channel=channel,
                blocks=[
                    {
                        "type": "header",
                        "text": {
                            "type": "plain_text",
                            "text": message
                        }
                    },
                    {
                        "type": "section",
                        "fields": [
                            {
                                "type": "mrkdwn",
                                "text": f"*신뢰도:* {confidence:.2%}"
                            },
                            {
                                "type": "mrkdwn",
                                "text": f"*상세:* {details}"
                            }
                        ]
                    }
                ]
            )
            return {"success": True, "ts": response["ts"]}

        except SlackApiError as e:
            return {"success": False, "error": str(e)}

    async def _execute_mcp_command(
        self,
        system: str,
        command: str,
        parameters: dict
    ) -> dict:
        """MCP 시스템 제어 명령 실행"""

        # PostgreSQL 자동 스케일링 예시
        if system == "postgresql" and command == "scale_up":
            response = await self.mcp_client.execute(
                tool="postgresql-integration",
                command="scale-replicas",
                params={"replicas": parameters.get("replicas", 3)}
            )
            return response

        # Redis 캐시 무효화 예시
        elif system == "redis" and command == "invalidate_cache":
            response = await self.mcp_client.execute(
                tool="redis",
                command="del",
                params={"key": parameters.get("cache_key")}
            )
            return response

        return {"success": False, "error": "Unknown system or command"}
```

---

#### 패턴 2: 자동 워크플로우 트리거

```python
# Judgment 결과 기반 후속 워크플로우 자동 실행

class WorkflowTrigger:
    async def trigger_on_judgment(
        self,
        judgment_result: JudgmentResult
    ):
        """판단 결과에 따른 워크플로우 트리거"""

        # 불량 판정 → 품질 검사 워크플로우 실행
        if judgment_result.result == "defect_detected":
            await self.workflow_service.execute(
                workflow_id="quality-inspection-workflow",
                input_data={
                    "item_id": judgment_result.input_data["item_id"],
                    "defect_reason": judgment_result.explanation
                }
            )

        # 고신뢰도 성공 → 자동 승인 워크플로우
        elif judgment_result.result is True and judgment_result.confidence >= 0.95:
            await self.workflow_service.execute(
                workflow_id="auto-approval-workflow",
                input_data=judgment_result.input_data
            )
```

---

## 🚀 MCP 도구 설치 가이드

### PostgreSQL MCP 설정

```bash
# 1. MCP 서버 설치
npm install -g @modelcontextprotocol/server-postgres

# 2. .mcp.json 설정
{
  "mcpServers": {
    "postgresql": {
      "command": "mcp-server-postgres",
      "args": [],
      "env": {
        "POSTGRES_CONNECTION": "postgresql://user:password@localhost:5432/judgify_core"
      }
    }
  }
}

# 3. 연결 테스트
/query "SELECT version()"
```

---

### GitHub MCP 설정

```bash
# 1. Personal Access Token 생성
# GitHub Settings > Developer settings > Personal access tokens
# 권한: repo, workflow, admin:org

# 2. .mcp.json 설정
{
  "mcpServers": {
    "github": {
      "command": "mcp-server-github",
      "args": [],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxxxxxxxxxxxx"
      }
    }
  }
}

# 3. 연결 테스트
/list-repos
```

---

### Memory MCP 설정

```bash
# 1. MCP 메모리 서버 설치
npm install -g @modelcontextprotocol/server-memory

# 2. .mcp.json 설정
{
  "mcpServers": {
    "memory": {
      "command": "mcp-server-memory",
      "args": ["--storage", "./mcp-memory"]
    }
  }
}

# 3. 연결 테스트
/save-context "test" "Hello, MCP!"
/restore-context "test"
```

---

## 💡 MCP 활용 모범 사례

### 1. PostgreSQL MCP 활용 (Database-Optimization Agent)

```python
# 자동 성능 분석 및 최적화 제안

async def optimize_judgment_queries():
    """Judgment Service 쿼리 최적화"""

    # 느린 쿼리 탐지
    slow_queries = await mcp_postgresql.query("""
        SELECT
            query,
            mean_exec_time,
            calls
        FROM pg_stat_statements
        WHERE mean_exec_time > 100
        ORDER BY mean_exec_time DESC
        LIMIT 10
    """)

    # 인덱스 제안
    for query in slow_queries:
        suggestions = await mcp_postgresql.suggest_index(query["query"])
        print(f"Optimize: {query['query']}")
        print(f"Suggestions: {suggestions}")
```

---

### 2. Playwright MCP 활용 (Performance-Engineer Agent)

```bash
# E2E 테스트 자동화 (docs/guides/e2e-testing.md 참조)

/playwright navigate "http://localhost:3000/dashboard"
/playwright fill '[data-testid="dashboard-request"]' "지난 주 워크플로우별 성공률"
/playwright click '[data-testid="generate-button"]'
/playwright screenshot "dashboard-test.png"
```

---

### 3. Context7 MCP 활용 (AI-Engineer Agent)

```bash
# 최신 라이브러리 문서 참조하며 개발

# FastAPI 최신 기능 확인
/get-docs "fastapi background tasks"

# pgvector 벡터 검색 최적화
/search-examples "pgvector cosine similarity index"

# OpenAI API 최신 가이드
/get-docs "openai embeddings api v2"
```

---

## 🔗 관련 문서

- [CLAUDE.md](../../CLAUDE.md) - 섹션 7 (MCP 및 외부 연동 전략)
- [docs/services/external_integration.md](../services/external_integration.md) - 외부 시스템 연동 상세 설계
- **MCP 공식 문서**: https://modelcontextprotocol.io

---

## 🆘 트러블슈팅

### 문제 1: PostgreSQL MCP 연결 실패

```bash
# 해결: 연결 문자열 확인
echo $POSTGRES_CONNECTION
# 예상: postgresql://user:password@localhost:5432/judgify_core

# 권한 확인
psql -c "GRANT ALL PRIVILEGES ON DATABASE judgify_core TO user;"
```

### 문제 2: GitHub MCP Personal Access Token 만료

```bash
# 해결: 토큰 재생성
# GitHub Settings > Developer settings > Personal access tokens
# Fine-grained token 사용 권장 (만료 기간 90일)
```

### 문제 3: Memory MCP 저장소 손상

```bash
# 해결: 저장소 초기화
rm -rf ./mcp-memory
mkdir ./mcp-memory
# MCP 메모리 서버 재시작
```
