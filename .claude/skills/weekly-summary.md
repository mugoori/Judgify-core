---
name: weekly-summary
description: 이번 주 업무를 자동으로 요약하여 주간 업무일지를 생성합니다
---

# 주간 업무 일지 생성

이번 주의 일일 업무일지를 분석하여 주간 요약 페이지를 자동 생성합니다.

## 실행 단계

1. **이번 주 범위 계산**
   - 월요일~일요일 범위 자동 계산
   - 사용자가 특정 주를 지정한 경우, 해당 주의 월요일 날짜 사용

2. **일일 업무일지 조회**
   - Notion "일일 업무일지 DB"에서 이번 주 데이터 조회
   - 각 페이지의 내용 추출

3. **AI 기반 요약 생성**
   - OpenAI API를 활용하여 주간 요약 자동 생성
   - 주요 성과, 완료된 작업, 진행 중인 작업, 다음 주 계획 포함

4. **주간 업무일지 페이지 생성**
   ```bash
   python scripts/notion/weekly_logger.py
   ```

5. **결과 확인 및 피드백**
   - 생성된 주간 업무일지 정보 표시
   - 분석된 일일 업무일지 개수
   - Notion 페이지 URL 제공

## 예시

**사용자**: "이번 주 업무 요약해줘"

**Claude**:
```
이번 주 일일 업무일지를 분석하여 주간 요약을 생성합니다...
```

```bash
python scripts/notion/weekly_logger.py
```

주간 업무 일지가 생성되었습니다!
- 주간 범위: 2025-10-21 ~ 2025-10-27
- 분석된 일일 업무일지: 5개

주요 성과:
- Notion 자동화 시스템 구축 완료
- Database ID 자동 추출 기능 구현
- Claude Code Skill 3개 생성

URL: https://www.notion.so/...

## 특정 주 요약

**사용자**: "지난 주 업무 요약해줘"

**Claude**:
```bash
python scripts/notion/weekly_logger.py --week "2025-10-14"
```

2025년 10월 3주차 업무 일지가 생성되었습니다!

## 주의사항

- 이 스킬은 `scripts/notion/weekly_logger.py`에 의존합니다.
- `.env` 파일에 다음이 설정되어 있어야 합니다:
  - `NOTION_API_TOKEN`
  - `NOTION_DAILY_LOG_DB_ID`
  - `NOTION_WEEKLY_LOG_DB_ID`
  - `OPENAI_API_KEY` (선택, AI 요약 생성용)
- OPENAI_API_KEY가 없으면 간단한 요약을 생성합니다.
- 이번 주 일일 업무일지가 없으면 오류가 발생합니다.

## 날짜 계산

- "이번 주" → 현재 주의 월요일~일요일
- "지난 주" → 지난 주의 월요일 날짜 계산
- "다음 주" → 다음 주의 월요일 날짜 계산
