---
name: log-today
description: 오늘 업무 일지를 Notion에 자동으로 작성합니다
---

# 오늘 업무 일지 작성

사용자로부터 오늘의 업무 내용을 받아 Notion "일일 업무일지 DB"에 추가합니다.

## 실행 단계

1. **사용자에게 오늘 수행한 업무 물어보기**
   - "오늘 수행한 업무를 간단히 설명해주세요."
   - 사용자가 이미 업무 내용을 제공한 경우, 이 단계는 건너뜁니다.

2. **스크립트 실행**
   ```bash
   python scripts/notion/daily_logger.py "사용자가 제공한 업무 내용"
   ```

3. **결과 확인 및 피드백**
   - 생성된 Notion 페이지 URL을 사용자에게 제공
   - 오늘 날짜와 함께 "일일 업무 일지가 생성되었습니다!" 메시지 표시

## 예시

**사용자**: "오늘 업무 일지 작성해줘"

**Claude**:
```
오늘 수행한 업무를 간단히 설명해주세요.
```

**사용자**: "Notion 자동화 시스템 구축 완료했어"

**Claude**:
```bash
python scripts/notion/daily_logger.py "Notion 자동화 시스템 구축 완료"
```

생성 완료! 오늘(2025-10-22) 업무 일지가 Notion에 추가되었습니다.
URL: https://www.notion.so/...

## 주의사항

- 이 스킬은 `scripts/notion/daily_logger.py`에 의존합니다.
- `.env` 파일에 `NOTION_API_TOKEN`과 `NOTION_DAILY_LOG_DB_ID`가 설정되어 있어야 합니다.
- 사용자가 이미 업무 내용을 제공한 경우 (예: "오늘 업무 일지 작성해줘: XXX 완료"), 바로 스크립트를 실행합니다.
