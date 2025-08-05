# initial.md

이 문서는 Claude 판단 시스템에서 활용되는 초기 요구사항 정의 파일입니다. 각 프로젝트의 기능적 요구사항, 예외 조건, 주요 흐름, 의존 모듈 등을 명확히 정의함으로써, Claude가 일관된 PRP(Plan, Reason, Predict) 기반 판단을 수행할 수 있도록 컨텍스트를 제공합니다.


## 1. 프로젝트 개요

- **프로젝트명**: Core Judgement Engine
- **목표**: 자연어 기반의 판단 요청에 대해 PRP 기반 DSL로 평가하고, Supabase에 기록하며, MCP 도구를 통해 후속 조치까지 연결하는 통합 판단 시스템 구축
- **플랫폼**: Supabase (DB 및 Edge Functions), MCP (도구 실행 프레임워크), Claude LLM (판단 엔진)
- **대상 사용자**: 제조업 품질 관리자, 설비 담당자, 문서 작업자 등


## 2. 핵심 기능 요구사항

### FR-001. 판단 입력 처리
- 자연어 또는 JSON 형식의 판단 요청 수신
- 입력값: 조건, 센서 값, 기준 등

### FR-002. PRP 실행 흐름 구성
- 입력 기반으로 PRP 단계 자동 구성
- Claude가 내부적으로 목적-이유-조건화 처리 수행

### FR-003. DSL 평가 로직 실행
- DSL 문법에 맞춘 조건식 생성 및 평가
- 예: `(온도 > 80) AND (습도 < 30)`

### FR-004. 판단 결과 기록
- 판단 결과를 Supabase `judgements` 테이블에 기록
- LLM 요약 결과는 `judgement_logs`에 기록

### FR-005. 후속 조치 실행
- MCP 도구(`slack`, `notion`, `terminal`, `git`)를 통해 자동화된 후속 실행
- 판단에 따라 알림 전송, 문서 생성, Git 커밋 등 수행


## 3. 예외 처리 조건

- **입력 불완전**: 주요 입력값 누락 시, Claude는 `Missing key: {변수명}` 메시지 반환
- **DSL 실패**: 판단 조건 생성 실패 시, `DSL Parse Error` 반환 및 재PRP 수행 유도
- **MCP 실행 실패**: MCP에서 오류 발생 시 오류 메시지 수집 및 Slack 보고


## 4. 데이터베이스 스키마 요약

### judgements
| 필드 | 타입 | 설명 |
|------|------|------|
| id | UUID | 고유 식별자 |
| input_json | JSONB | 판단 입력 원본 |
| result | TEXT | 판단 결과 (예: CALL_OPERATOR) |
| created_at | TIMESTAMP | 생성 시간 |

### judgement_logs
| 필드 | 타입 | 설명 |
|------|------|------|
| judgement_id | UUID | 연동 판단 ID |
| summary | TEXT | 판단 요약 |
| engine | TEXT | 처리 방식 (DSL / Claude 등) |
| embedding | VECTOR | 유사 검색용 벡터 |
| timestamp | TIMESTAMP | 기록 시각 |


## 5. MCP 연동 설정

| MCP 도구 | 설명 | 사용 조건 |
|----------|------|------------|
| supabase | 판단 결과 기록/조회 | 자동 연결됨 |
| slack | 판단 결과 보고 | result = CALL_OPERATOR 또는 ALERT 포함 시 |
| notion | 문서화 작업 | judgement_logs에 summary 포함 시 |
| terminal | 테스트 및 배포 | 사용자가 `/deploy` 명령 요청 시 |
| git | 소스 코드 추적 | 판단 기반 코드 리팩터링 후 사용 |


## 6. 초기 설정 값

- 판단 기준 임계값:
  - 온도 기준: 85도
  - 습도 기준: 20%
- Slack 채널: `#judgement-alerts`
- Notion 템플릿 링크: `<프로젝트 별도 문서 참조>`


## 7. 문맥 삽입 지침 (Claude 기준)

- `initial.md`는 항상 첫 문맥으로 삽입됨
- 이후 PRP, prompt-guide는 판단 흐름에 따라 동적으로 추가됨
- 사용자 명령 `/reset-context` 시, 이 파일이 재삽입됨


