---
name: log-urgent
description: 긴급하게 처리할 업무를 Notion에 추가합니다
---

# 긴급 업무 추가

사용자가 지시한 긴급 업무를 Notion "To-do DB"에 추가합니다.

## 실행 단계

1. **사용자로부터 정보 수집**
   - 할 일: 필수
   - 목표일: 선택 (기본값: 오늘+3일)
   - 분류: 선택 (기본값: "긴급")

2. **스크립트 실행**
   ```bash
   python scripts/notion/urgent_task_manager.py "할 일" --deadline "YYYY-MM-DD" --category "분류"
   ```

3. **결과 확인 및 피드백**
   - 추가된 항목 정보 표시 (할 일, 목표일, 남은 기간, 분류)
   - Notion 페이지 URL 제공

## 예시

**사용자**: "긴급 업무 추가해줘: 고객 미팅 준비, 내일까지"

**Claude**:
```bash
python scripts/notion/urgent_task_manager.py "고객 미팅 준비" --deadline "2025-10-23"
```

긴급 업무가 추가되었습니다!
- 할 일: 고객 미팅 준비
- 목표일: 2025-10-23
- 남은 기간: 1일
- 분류: 긴급
- 진행상황: 진행 중

URL: https://www.notion.so/...

## 날짜 파싱

사용자가 다양한 형식으로 날짜를 제공할 수 있습니다:
- "내일" → 오늘+1일
- "모레" → 오늘+2일
- "다음 주 월요일" → 다음 주 월요일 날짜
- "10월 25일" → 2025-10-25
- "2025-10-25" → 그대로 사용

날짜를 파싱한 후 YYYY-MM-DD 형식으로 변환하여 스크립트에 전달합니다.

## 주의사항

- 이 스킬은 `scripts/notion/urgent_task_manager.py`에 의존합니다.
- `.env` 파일에 `NOTION_API_TOKEN`과 `NOTION_TODO_DB_ID`가 설정되어 있어야 합니다.
- 목표일을 제공하지 않으면 자동으로 오늘+3일로 설정됩니다.
