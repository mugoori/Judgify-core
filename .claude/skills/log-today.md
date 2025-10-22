---
name: log-today
description: 오늘 업무 일지를 Notion에 자동으로 상세하게 작성합니다
---

# 오늘 업무 일지 작성 (자동 상세 작성)

사용자의 간단한 요약 또는 오늘 대화 내용을 분석하여 **기술 스택, 구현 방법, 수정 파일**을 포함한 상세한 업무 일지를 Notion에 자동 생성합니다.

## 실행 단계

1. **오늘 대화 내용 자동 분석** (Claude가 자동 수행)
   - 오늘 수정된 파일 목록 추출
   - 사용된 기술 스택/라이브러리 감지 (예: notion-client, Python re, pytest)
   - 핵심 작업 내용 요약
   - 구현 방법 키워드 수집

2. **상세 내용 자동 생성** (Claude가 자동 수행)
   - 간단한 요약을 상세한 형식으로 확장
   - 형식: "작업명: 1. 세부작업1 - 기술스택, 구현방법, 2. 세부작업2 - 파일명, 설명"
   - 예시: "Notion 가독성 개선: 1. parse_content_to_blocks() 수정 - Python re로 정규식 구현, notion-client로 블록 생성, 2. 테스트 4건 - pytest 검증 완료"

3. **스크립트 실행**
   ```bash
   python scripts/notion/daily_logger.py "자동 생성된 상세 내용"
   ```

4. **결과 확인 및 피드백**
   - 생성된 Notion 페이지 URL을 사용자에게 제공
   - 오늘 날짜와 함께 "상세한 업무 일지가 생성되었습니다!" 메시지 표시

## 예시

### 예시 1: 간단한 요약 → 자동 상세 작성

**사용자**: "오늘 업무 일지 작성해줘"

**Claude** (대화 자동 분석):
```
오늘 작업 내용:
- 수정 파일: daily_logger.py, .env.example, install-all.bat
- 사용 라이브러리: notion-client 2.5.0, Python re
- 주요 작업: parse_content_to_blocks() 개선, 설치 스크립트 통합
- 테스트: 4건 통과

상세 내용 생성 중...
```

**Claude** (자동 실행):
```bash
python scripts/notion/daily_logger.py "Notion 업무일지 자동화 개선: 1. parse_content_to_blocks() 함수 수정 - Python re 라이브러리로 정규식 패턴 구현 (번호 감지), notion-client 2.5.0으로 구조화된 블록 생성, 2. 설치 스크립트 통합 - .env.example에 Notion 환경 변수 추가, install-all.bat 검증 섹션 개선, 3. 테스트 완료 - 4가지 패턴 검증"
```

**결과**:
```
생성 완료! 오늘(2025-10-22) 상세한 업무 일지가 Notion에 추가되었습니다.
URL: https://www.notion.so/...

Notion 출력:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Notion 업무일지 자동화 개선

1. parse_content_to_blocks() 함수 수정
   • Python re 라이브러리로 정규식 패턴 구현 (번호 감지)
   • notion-client 2.5.0으로 구조화된 블록 생성

2. 설치 스크립트 통합
   • .env.example에 Notion 환경 변수 추가
   • install-all.bat 검증 섹션 개선

3. 테스트 완료
   • 4가지 패턴 검증
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 예시 2: 이미 상세한 내용 제공시

**사용자**: "오늘 업무 일지 작성해줘: Notion 개선 완료 - Python re와 notion-client 사용"

**Claude** (이미 상세하므로 그대로 사용):
```bash
python scripts/notion/daily_logger.py "Notion 개선 완료 - Python re와 notion-client 사용"
```

## 주의사항

- 이 스킬은 `scripts/notion/daily_logger.py`에 의존합니다.
- `.env` 파일에 `NOTION_API_TOKEN`과 `NOTION_DAILY_LOG_DB_ID`가 설정되어 있어야 합니다.
- 사용자가 이미 업무 내용을 제공한 경우 (예: "오늘 업무 일지 작성해줘: XXX 완료"), 바로 스크립트를 실행합니다.
