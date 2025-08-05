# system-structure.md

이 문서는 Claude 판단 엔진을 포함한 전체 MCP 기반 시스템 구조를 명확히 설명하기 위한 기술 아키텍처 문서입니다.

목적은 다음과 같습니다:
- 각 MCP 모듈의 기능과 역할 정의
- Claude와 MCP 간 통신 방식 설명
- Supabase 및 외부 서비스와의 연동 구조 명시

---

## 1. 전체 아키텍처 개요

```
[사용자] ⇄ [Claude 판단 엔진]
               ⇅
         ┌──────────────┐
         │    MCP CORE    │
         └──────────────┘
               ↓
        [MCP 모듈 집합]
            ├─ supabase
            ├─ terminal
            ├─ notion
            ├─ slack
            ├─ playwright
            └─ 외부 API 등
```

- 사용자 입력 → Claude에 프롬프트 전달
- Claude는 판단 후 MCP CLI 명령 또는 명시된 API로 실행 요청
- MCP는 내부 모듈 또는 외부 API로 실제 실행

---

## 2. 주요 구성 요소 설명

### 2.1 Claude 판단 엔진
- 역할: PRP 기반 판단 수행
- 입력: 자연어 또는 구조화된 프롬프트
- 출력: 결과 + 판단 로그 + 실행 지시
- 통신 방식: CLI 또는 RESTful 명령어

### 2.2 MCP Core
- 역할: 판단 결과를 기반으로 실행 처리 담당
- 기능: 
  - 판단 결과 로그 저장
  - MCP 모듈 호출
  - Slack/Notion/Terminal 등 연결

### 2.3 Supabase MCP
- 역할: DB 조회 및 업데이트 (읽기/쓰기)
- 프로토콜: REST API + RPC 함수 사용
- 예시: `GET /temperature_logs?gt=90`
- 응답: JSON / 테이블 데이터

### 2.4 Playwright MCP
- 역할: 브라우저 자동화 (테스트 시나리오, 데이터 수집 등)
- 명령어: 페이지 이동, 클릭, 데이터 추출 등

### 2.5 Notion MCP
- 역할: 작업 메모 기록, 프로젝트 워크스페이스 관리
- 방식: 페이지 생성, 텍스트 삽입, DB 연결

### 2.6 Slack MCP
- 역할: 판단 결과를 사용자에게 알림 또는 승인 요청
- 방식: 실시간 메시지 전송, 버튼 인터랙션

### 2.7 기타 MCP 모듈
- terminal: 터미널 명령 실행 (로컬 또는 원격)
- text-editor: 실시간 코드 수정 지원
- git: 코드 히스토리 판단 기반 메시지 생성
- word/hwp: 문서 자동작성
- youtube-data, googleSearch: 외부 콘텐츠 분석용

---

## 3. 데이터 흐름 시나리오 예시

### 예시: 실시간 센서 → 판단 → 작업자 호출

1. MCP가 Supabase로부터 온도/습도 로그 수집
2. Claude에게 판단 요청 (PRP 기반)
3. 판단 결과: `CALL_OPERATOR`
4. Slack MCP를 통해 작업자 호출 메시지 전송

---

## 4. 에러 처리 및 예외 대응

- Claude 응답 실패 시 → MCP는 fallback 메시지 생성
- MCP 모듈 연결 실패 시 → retry 및 로그 저장
- 판단 결과가 `UNKNOWN` → Notion에 기록 후 관리자 확인 요청

---

## 5. 확장 전략

- 모든 MCP는 모듈 단위로 설계되어 쉽게 교체/추가 가능
- Supabase 외에도 Firebase, PostgreSQL 등 추가 가능
- 판단 엔진은 Claude 외에도 GPT, local LLM으로 대체 가능

---

(다음 문서: `supabase-guide.md`)
