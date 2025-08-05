# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Judgify-core is currently an empty repository. As development progresses, this file should be updated with:

1. Build and development commands
2. Project architecture and structure
3. Testing procedures
4. Key design decisions and patterns

## Getting Started

Since this is a new project, the first steps will likely involve:
- Initializing the project with appropriate package management (npm, pip, cargo, etc.)
- Setting up the basic project structure
- Configuring development tools (linters, formatters, testing frameworks)

## Development Notes

As the project develops, update this file with:
- Specific commands for building, testing, and running the application
- Important architectural decisions and patterns
- Dependencies and their purposes
- Any project-specific conventions or practices

# CLAUDE.md

이 문서는 Claude Code (claude.ai/code)를 위한 **컨텍스트 엔지니어링 가이드**입니다. 본 저장소는 Supabase 기반의 Core Judgement Engine(CJE)을 구현하며, Claude는 이 문서를 기준으로 모든 개발과 판단 로직 작성을 수행합니다.

---

## 1. 프로젝트 개요

본 프로젝트는 **제조업의 판단 자동화를 위한 코어 판단 엔진(Core Judgement Engine)** 으로, 규칙 기반 + LLM 기반 추론을 결합한 하이브리드 구조를 채택합니다.

### 주요 기능
- 조건 기반 DSL 처리
- 판단 로직의 노드 워크플로우 실행
- LLM 기반 설명 및 예외적 판단 처리
- MCP 기반 외부 명령 연동 (Slack, Notion, Terminal 등)
- Supabase 기반 RAG 벡터 저장 및 자동 임베딩

---

## 2. 사용 기술 및 구조

| 영역          | 기술 스택                            |
|---------------|--------------------------------------|
| 데이터베이스  | Supabase PostgreSQL 16 + pgvector   |
| 서버리스 함수| Supabase Edge Functions (Deno + TS) |
| 프론트엔드    | React 18 + Vite + Tailwind CSS      |
| 벡터검색      | pgvector (768 dim, HNSW)            |
| LLM 연동      | Claude via prompt templates          |
| 로그 저장     | Supabase table + RLS                 |
| MCP 연결      | Supabase MCP Server + 기타 MCPs      |

---

## 3. 프로젝트 구조

```text
.
├─ supabase/                  # Supabase 프로젝트 스키마 및 함수
│  ├─ migrations/             # DB 마이그레이션 기록
│  ├─ functions/              # Edge Functions 디렉터리
│  │   └─ judge-temp/         # 온도 판단 함수
│  │       └─ index.ts
│  └─ .env                    # 환경변수 (Access Token 등)
├─ frontend/                  # 클라이언트 앱
├─ docs/                      # 아키텍처 및 템플릿 문서
└─ tests/                     # 유닛 및 통합 테스트
```

---

## 4. 빠른 시작 (로컬 실행)

```bash
npm install -g supabase            # CLI 설치
supabase login                     # 토큰 발급 및 로그인
supabase init                      # 프로젝트 초기화
supabase start                     # 로컬 스택 실행 (Postgres + Auth + Edge)
```

---

## 5. Edge Function 예제

```ts
// supabase/functions/judge-temp/index.ts
import { serve } from "https://deno.land/x/sift@0.6.0/mod.ts";
import { createClient } from "@supabase/supabase-js";

serve(async (req) => {
  const { SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY } = Deno.env.toObject();
  const supabase = createClient(SUPABASE_URL, SUPABASE_SERVICE_ROLE_KEY);

  const { temp, humidity } = await req.json();
  const result = temp > 85 && humidity < 20 ? "CALL_OPERATOR" : "NONE";

  await supabase.from("judgements").insert({ temp, humidity, result });

  return new Response(JSON.stringify({ result }), {
    headers: { "Content-Type": "application/json" },
  });
});
```

---

## 6. 데이터베이스 스키마 예시

```sql
create extension if not exists vector;

create table judgements (
  id bigserial primary key,
  temp numeric,
  humidity numeric,
  result text,
  embedding vector(768),
  user_id uuid references auth.users,
  created_at timestamptz default now()
);

-- 자동 임베딩 트리거
create trigger embeddings
before insert on judgements
for each row
execute procedure supabase_ai.embed(column_name := 'result', embedding_column := 'embedding');

-- RLS 설정
alter table judgements enable row level security;
create policy "owner" on judgements for select using (auth.uid() = user_id);
```

---

## 7. Claude 사용 시 프롬프트 템플릿

### 판단 요청 프롬프트
```
입력: 온도 88.2도, 습도 18%
기준: 온도 > 85 && 습도 < 20 → 작업자 호출 필요
→ 지금 작업자 호출이 필요한가?
```

### 설명 요청 프롬프트
```
입력: 온도 88.2, 진동 40
결과: 작업자 호출
기준: 온도 > 85 && 진동 > 40
→ 이 판단은 왜 내려졌는지 간결하게 설명해줘.
```

---

## 8. MCP 서버 설정 (Supabase 및 기타)

### MCP 서버 리스트 (설치됨)

- notion: Notion 워크스페이스 관리
- slack: Slack 커뮤니케이션
- word-document-server: MS Word 문서 작업
- hwp: 한글 문서 작업 (로컬 설치)
- terminal: 터미널 명령 실행
- text-editor: 텍스트 편집기
- git: Git 버전 관리
- playwright: 브라우저 자동화 및 테스팅
- playwright-mcp: Microsoft Playwright MCP (Smithery)
- youtube-data-mcp-server: YouTube 데이터 API
- googleSearch: Google 검색
- context7-mcp: 문서 검색 (Smithery)
- supabase: Supabase 데이터베이스 (읽기 전용)

### Supabase MCP 설정
```json
{
  "mcpServers": {
    "supabase": {
      "command": "npx",
      "args": ["-y", "@supabase/mcp-server-supabase@latest", "--read-only", "--project-ref=ebmbematwyfrylgkrmnh"],
      "env": {
        "SUPABASE_ACCESS_TOKEN": "<YOUR_TOKEN>"
      }
    }
  }
}
```

### Claude 활용 예시
```
use supabase
query: select * from judgements where result = 'CALL_OPERATOR'
```

---

## 9. 테스트 및 배포

```bash
supabase db diff                 # 변경 사항 확인
supabase db push                # 스키마 반영
supabase functions deploy judge-temp
```

---

## 10. 체크리스트 (Claude 전용)

- [ ] 모든 입력값이 정의되었는가?
- [ ] 워크플로우 JSON이 구조적으로 유효한가?
- [ ] 조건 DSL이 파싱 가능하고 안전한가?
- [ ] 판단 결과가 judgements 테이블에 기록되었는가?
- [ ] pgvector가 정상적으로 embedding을 생성했는가?

---

## 11. 보안 및 운영 주의사항

| 항목         | 설명                                      |
|--------------|-------------------------------------------|
| Access Token | 공개 금지, .env 파일에 저장 및 gitignore |
| Row Level Security | auth.uid() 기준 사용자 권한 분리 적용      |
| Logs         | Studio 대시보드에서 function 쿼리 확인 가능 |
| Rate Limit   | Pro 플랜 기준 2M 함수 호출/월              |

---

## 12. 유지보수 및 확장 계획

- [ ] 판단 워크플로우 시각화 UI 연동
- [ ] Slack 알림 연동 기능 추가 (MCP 활용)
- [ ] context7-mcp 기반 유사도 검색 및 대화형 판단 흐름 도입
- [ ] CLI로 자동 judgement diff 생성기 탑재
- [ ] GPT/Claude 간 판단 품질 비교 및 confidence score 기반 다중 에이전트 구조 실험

---

## 13. Contributor

| 역할               | 담당자 ID          |
|--------------------|--------------------|
| Claude LLM 설계자  | @llm_prompter      |
| Backend 운영       | @engine_master     |
| UI/UX 설계         | @judgement_designer|
| Supabase 관리자     | @supabase_admin     |

---

> 본 문서는 2025-08-01 기준 최신 Supabase 및 MCP 서버 구성에 따라 작성되었으며, 모든 프로젝트 구성원은 변경 시 반드시 CLAUDE.md를 갱신하고 `CHANGELOG.md`에 반영해야 합니다.
