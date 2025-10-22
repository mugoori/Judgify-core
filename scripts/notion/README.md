# Notion 자동 업무 일지 시스템

Notion API를 활용하여 업무 일지를 자동으로 작성하는 시스템입니다.

## 📋 기능

1. **일일 업무 일지 자동 작성** (`daily_logger.py`)
   - 오늘 수행한 업무를 Notion "일일 업무일지 DB"에 자동 추가
   - 날짜 자동 설정 (기본값: 오늘)
   - 대화형 모드 및 CLI 모드 지원

2. **긴급 업무 추가** (`urgent_task_manager.py`)
   - 긴급 업무를 Notion "To-do DB"에 추가
   - 목표일, 분류, 진행상황 자동 관리
   - 남은 기간 자동 계산

3. **주간 업무 일지 자동 생성** (`weekly_logger.py`)
   - 이번 주의 일일 업무일지를 AI로 자동 요약
   - OpenAI API를 활용한 주요 성과, 완료 작업, 진행 중 작업 분석
   - 주간 업무일지 페이지 자동 생성

## 🚀 설치 및 설정

### 1. 필요 패키지 설치

```bash
pip install notion-client python-dotenv openai
```

또는 프로젝트 루트에서:

```bash
pip install -r requirements.txt
```

### 2. Notion Integration 설정

1. [Notion Integrations 페이지](https://www.notion.so/my-integrations) 접속
2. "New integration" 클릭
3. Integration 정보 입력:
   - 이름: "Judgify Auto Logger" (또는 원하는 이름)
   - Type: **Internal Integration**
   - Capabilities: Read content, Insert content, Update content 체크
4. Integration Token 복사 (secret_xxx 형태)

### 3. Notion 페이지에 Integration 연결

1. Notion에서 업무일지 페이지 열기
2. 우측 상단 "Share" 버튼 클릭
3. "Judgify Auto Logger" 검색 후 "Invite" 클릭

### 4. Database ID 자동 추출

```bash
cd scripts
python extract_notion_databases.py
```

이 스크립트는 자동으로 다음을 수행합니다:
- Notion API로 접근 가능한 모든 데이터베이스 조회
- 데이터베이스 이름으로 자동 매핑:
  - "일일 업무일지 DB" → NOTION_DAILY_LOG_DB_ID
  - "주간 업무일지 DB" → NOTION_WEEKLY_LOG_DB_ID
  - "To-do DB" → NOTION_TODO_DB_ID
- `.env` 파일에 ID 자동 저장

### 5. 환경 변수 확인

`.env` 파일에 다음이 설정되어 있는지 확인:

```env
NOTION_API_TOKEN=ntn_xxxxxxxxxxxxxxxxxxxxxxxxxxxxx
NOTION_DAILY_LOG_DB_ID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
NOTION_WEEKLY_LOG_DB_ID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
NOTION_TODO_DB_ID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
OPENAI_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx (선택사항)
```

## 💻 사용법

### 1. 일일 업무 일지 작성

**CLI 모드**:
```bash
python scripts/notion/daily_logger.py "오늘은 Notion 자동화 시스템을 구축했습니다."
```

**특정 날짜 지정**:
```bash
python scripts/notion/daily_logger.py "어제 한 일" --date "2025-10-21"
```

**대화형 모드**:
```bash
python scripts/notion/daily_logger.py
# 프롬프트에 따라 업무 내용 입력
```

**Claude Code Skill 사용** (추천):
```
/log-today
```

### 2. 긴급 업무 추가

**기본 사용**:
```bash
python scripts/notion/urgent_task_manager.py "고객 미팅 준비" --deadline "2025-10-25"
```

**카테고리 지정**:
```bash
python scripts/notion/urgent_task_manager.py "블로그 포스팅" --deadline "2025-10-24" --category "컨텐츠"
```

**대화형 모드**:
```bash
python scripts/notion/urgent_task_manager.py
# 프롬프트에 따라 정보 입력
```

**Claude Code Skill 사용** (추천):
```
/log-urgent
```

### 3. 주간 업무 일지 생성

**이번 주 요약**:
```bash
python scripts/notion/weekly_logger.py
```

**특정 주 요약** (해당 주의 월요일 날짜):
```bash
python scripts/notion/weekly_logger.py --week "2025-10-14"
```

**Claude Code Skill 사용** (추천):
```
/weekly-summary
```

## 🎯 Claude Code Skills

이 시스템은 Claude Code Skills와 통합되어 있습니다.

### 사용 가능한 Skills

1. `/log-today` - 오늘 업무 일지 작성
2. `/log-urgent` - 긴급 업무 추가
3. `/weekly-summary` - 주간 업무 요약 생성

### Skills 사용 예시

**시나리오 1: 일일 업무 일지 작성**
```
사용자: 오늘 업무 일지 작성해줘
Claude: /log-today 실행
        오늘 수행한 업무를 간단히 설명해주세요.
사용자: Notion 자동화 시스템 구축 완료
Claude: ✅ 2025-10-22 업무 일지가 생성되었습니다!
        https://notion.so/...
```

**시나리오 2: 긴급 업무 추가**
```
사용자: 긴급 업무 추가해줘: 고객 미팅 준비, 내일까지
Claude: /log-urgent 실행
        ✅ "고객 미팅 준비"가 추가되었습니다!
        목표일: 2025-10-23
        남은 기간: 1일
```

**시나리오 3: 주간 요약 생성**
```
사용자: 이번 주 업무 요약해줘
Claude: /weekly-summary 실행
        [이번 주 일일 업무일지 5개 분석 중...]
        ✅ 2025년 43주차 업무 일지가 생성되었습니다!
        주요 성과:
        - Notion 자동화 시스템 구축
        - Database ID 자동 추출 기능
        - Claude Code Skill 3개 생성
```

## 📁 파일 구조

```
Judgify-core/
├── .env (환경 변수)
├── requirements.txt (notion-client 포함)
├── scripts/
│   ├── extract_notion_databases.py (Database ID 추출)
│   └── notion/
│       ├── __init__.py
│       ├── README.md (이 파일)
│       ├── daily_logger.py (일일 업무 일지)
│       ├── urgent_task_manager.py (긴급 업무 추가)
│       └── weekly_logger.py (주간 요약 생성)
└── .claude/
    └── skills/
        ├── log-today.md (일일 업무 Skill)
        ├── log-urgent.md (긴급 업무 Skill)
        └── weekly-summary.md (주간 요약 Skill)
```

## 🔧 문제 해결

### Database ID를 찾을 수 없습니다

**원인**: Notion Integration이 페이지에 연결되지 않았습니다.

**해결**:
1. Notion 페이지에서 "Share" → Integration 선택 → "Invite"
2. `extract_notion_databases.py` 재실행

### OPENAI_API_KEY가 없습니다

**영향**: 주간 요약 생성 시 AI 요약 대신 간단한 요약이 생성됩니다.

**해결** (선택):
1. OpenAI API 키 발급: https://platform.openai.com/api-keys
2. `.env` 파일에 `OPENAI_API_KEY=sk-xxx` 추가

### 인코딩 오류 (UnicodeEncodeError)

**원인**: Windows 환경에서 cp949 인코딩 문제

**해결**: 스크립트에서 이모지를 제거했으므로 해결되었습니다.

## 📊 데이터베이스 필드 자동 감지

각 스크립트는 Notion 데이터베이스의 필드를 자동으로 감지합니다:

- **제목 필드**: `type: "title"` 자동 감지
- **날짜 필드**: `type: "date"` 자동 감지
- **Select 필드**: `type: "select"` 자동 감지 (분류, 진행상황)
- **Number 필드**: `type: "number"` 자동 감지 (남은 기간)

필드 이름이 다른 경우에도 필드 타입으로 자동 매핑됩니다.

## 🎉 완료!

이제 Claude Code에서 `/log-today`, `/log-urgent`, `/weekly-summary` 명령어로
Notion 업무 일지를 자동으로 작성할 수 있습니다!

---

**작성일**: 2025-10-22
**버전**: 0.1.0
