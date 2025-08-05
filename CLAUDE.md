# CLAUDE.md (v4)

이 문서는 Claude Code 또는 유사한 LLM 기반 AI 에이전트가 이 리포지토리의 전체 개발/운영 흐름을 이해하고, 안정적인 판단 기반 시스템을 구현할 수 있도록 설계된 **포괄적 컨텍스트 엔지니어링 가이드**입니다. 기존의 단순한 프롬프트 엔지니어링이 아닌, 문맥 창을 설계하고 운용하는 **고도화된 컨텍스트 전략**을 중심으로 작성되었습니다.

---

## 0. 문서 목적 및 범위

Claude는 단순히 코드 작성 보조 도구가 아니라, 판단 로직 구성, Supabase 연동, Edge Function 배포, MCP 툴 실행, PRP 흐름 설계까지 관여하는 **상태 지향적 에이전트**로 작동해야 합니다. 본 문서는 그 전 과정을 포괄하며, 다음 파일들과 함께 문맥 세트를 구성합니다:

- `initial.md`: 프로젝트/기능 단위 초기 요구사항 문서
- `prp-example.md`: PRP 실행을 위한 Claude 내부 플래너 설계 템플릿
- `prompt-guide.md`: 사용자 프롬프트 설계 가이드 및 예시

---

## 1. Claude의 역할 정의

Claude는 다음과 같은 역할을 수행해야 합니다:

1. 명확한 컨텍스트 기반 판단
2. DSL 조건 및 워크플로우 노드 실행
3. Supabase DB + RAG 시스템과 상호작용
4. 판단 로그 및 임베딩 데이터 기록
5. 주기적 리팩토링 및 테스트 검증
6. MCP 도구를 통해 CI/CD, 문서 작업, 커뮤니케이션 자동화

Claude는 "바이브 코딩"이 아닌, **PRP 설계 → 코드 생성 → 테스트 생성 → 설명 생성 → 재검토 및 반복 개선**이라는 흐름에 따라 작동해야 합니다.

---

## 2. 파일 기반 컨텍스트 구조

Claude는 항상 다음 파일을 우선 참조해야 합니다:

| 파일명 | 설명 | 포함 목적 |
|--------|------|------------|
| `CLAUDE.md` | 전역 규칙, 명명 표준, 판단 흐름, 도구 연동법 등 | 상수 규칙 기반 컨텍스트 |
| `initial.md` | 프로젝트별 요구사항 및 예외조건 | 판단 입력 컨텍스트 |
| `prp-example.md` | PRP 구조 및 실행 흐름 예시 | 판단 설계 도구 |
| `prompt-guide.md` | 사용자 질의 흐름 및 포맷 | 사용자 입력 가이드 |

이러한 구조는 **LangChain의 문맥 선택(Selecting Context)** 전략을 반영합니다.

---

## 3. 컨텍스트 엔지니어링 전략 (LangChain 4전략 기반)

### 3.1 문맥 작성 (Writing Context)

- 판단 과정에서 `judgements` 테이블, `judgement_logs`, `scratchpad`에 단계별 메모를 남김
- Claude는 명시적으로 문맥을 다음 위치에 기록함:
  - 판단 결과: `judgements`
  - LLM 응답 요약: `judgement_logs`
  - 피드백 수렴/토론: `scratchpad`

### 3.2 문맥 선택 (Selecting Context)

- Claude는 다음의 기준으로 판단 전 문맥을 준비함:
  - 시스템 규칙: `CLAUDE.md`
  - 기능 정의: `initial.md`
  - 도구 설명: MCP 서버 Tool 등록 리스트
  - 유사 판단 사례: `judgement_logs`의 vector 검색 결과

### 3.3 문맥 압축 (Compressing Context)

- `summarize latest 5 judgement_logs` 등의 명령으로 문맥 요약 수행
- 이전 결과가 너무 길면 요약본만 재삽입
- 자동 요약은 MCP `context7-mcp`를 호출해 수행 가능

### 3.4 문맥 격리 (Isolating Context)

- MCP 도구 사용 시 `use {tool}` 명령으로 격리 시작
- 멀티 에이전트 동시 판단 시 `context: agent_a`, `context: agent_b`를 명시
- 사용자 요청이 충돌할 경우에는 `context clash detected` 메시지와 함께 판단 보류

---

## 4. Claude 판단 엔진 흐름

1. 판단 요청 수신 → JSON or 자연어
2. `initial.md` 기반으로 판단 로직 PRP 생성 → DSL 조건 생성
3. 조건 평가: DSL → 판단 결과
4. 결과 기록: `judgements` 테이블 insert
5. 결과 설명 생성: LLM → `judgement_logs` 기록
6. 벡터 임베딩: Supabase Automatic Embeddings 트리거
7. 재사용을 위한 유사 판단 조회 등록
8. MCP 통해 후속 조치 (Slack 알림, Git commit 등)

---

## 5. MCP 도구 연동 규칙

Claude는 다음 MCP 서버와 연결되어 있으며, `use {tool}` 명령으로 활성화합니다:

| Tool ID | 용도 |
|---------|------|
| supabase | 데이터베이스 조회 및 실행 (read-only) |
| notion | 프로젝트 문서 작업 |
| slack | 판단 결과 알림 및 요약 보고 |
| terminal | CLI 명령 실행 (테스트, 배포, 마이그레이션 등) |
| git | 코드 저장소 상태 파악 및 커밋 메시지 자동화 |
| context7-mcp | RAG 기반 판단 사례 검색 |
| playwright | 브라우저 시나리오 테스트 및 오류 확인 |
| word-document-server, hwp | 문서 보고서 자동 작성 |

---

## 6. 판단 예시 포맷

### 판단 입력 (자연어 예시)
```text
작업자가 온도 89도, 습도 18% 조건에서 공정을 수행했을 때 판단해줘. 기준은 온도>85, 습도<20이면 작업자 호출이 필요해.
```

### Claude 내부 처리 흐름
```ts
if (temp > 85 && humidity < 20) {
  return "CALL_OPERATOR";
}
```

### 판단 로그 기록 예시 (judgement_logs)
```json
{
  "input": { "temp": 89, "humidity": 18 },
  "result": "CALL_OPERATOR",
  "engine": "DSL",
  "summary": "온도와 습도가 기준을 모두 초과하였기 때문에 작업자 호출이 필요함.",
  "embedding": "...vector...",
  "timestamp": "2025-08-04T12:00:00"
}
```

---

## 7. PRP 기반 판단 흐름 상세

### 7.1 Planning: 판단 계획 수립
- 판단 목적, 판단 조건, 고려 요소, 기준 규칙, 출력을 명시적으로 정리
- 예시:
  ```
  목적: 기준 조건 만족 여부 확인
  입력 변수: temp, humidity
  기준: temp > 85 AND humidity < 20 → CALL_OPERATOR
  출력: 판단 결과 (CALL_OPERATOR / NO_ACTION)
  ```

### 7.2 Reasoning: 판단 조건 평가
- DSL 또는 자연어 규칙 기반 평가
- 결과의 명확한 이유를 함께 기술 (explanation 생성)
- 판단 실패 시 `reasoning_error` 필드에 사유 기록

### 7.3 Prompting: 사용자 피드백 대응
- 판단 근거 요약 → `judgement_logs`에 기록
- 이후 유사 질의 시 `context7-mcp`를 통해 회수

---

## 8. 판단 검증 및 피드백 구조

### 8.1 Validation 절차
- 판단 결과는 Supabase Trigger 또는 MCP Agent가 후속 평가
- 조건 충족 여부, 판단 일관성, 에러 여부 등을 점검
- Validation 로그는 `judgement_validation_logs`에 저장 가능

### 8.2 Feedback 수렴 흐름
- 사용자 또는 Agent가 잘못된 판단에 대해 `feedback: incorrect` 메모 기록
- Claude는 향후 판단에서 해당 피드백을 `similar case penalty`로 반영

---

## 9. 컨텍스트 리셋 전략

### 9.1 문맥 충돌 감지
- `context clash detected` 메시지 발생 시:
  - 우선순위 설정: `priority: agent_a > user_request`
  - 중단 후 재계획

### 9.2 세션 리셋
- 세션 중간에 `/reset-context` 명령어로 문맥 초기화 가능
- `scratchpad`, `current_plan`, `prp_stack` 모두 초기화됨

### 9.3 문맥 크기 초과 대응
- 10000토큰 이상 초과 시:
  - judgement_logs 최신 5건만 요약 삽입
  - PRP 단계 중 Reasoning만 보존하고 Planning은 요약
  - MCP context7-mcp 통해 판단 근거 링크 대체

---

## 10. Claude 프롬프트 스타일 가이드

- **PRP 설계**: 항상 `## 목적`, `## 기준`, `## 예시` 항목 포함
- **판단 요청**: 명확한 입력 변수/조건과 함께 작성
- **후속 요청**: 이전 판단 로그 참조 유도 (`이전에 판단한 내용 기반으로`)
- **명령어 어투**: `please`, `analyze`, `evaluate`, `summarize` 등 명령형 사용
- **코드 요청**: `generate code for`, `refactor logic`, `test case for condition`

---

