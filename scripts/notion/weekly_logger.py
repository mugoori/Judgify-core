"""
주간 업무 일지 자동 생성 스크립트

이 스크립트는 이번 주의 일일 업무일지를 분석하여
주간 요약 페이지를 자동으로 생성합니다.

사용법:
    # 이번 주 자동 생성
    python scripts/notion/weekly_logger.py

    # 특정 주 생성 (해당 주의 월요일 날짜)
    python scripts/notion/weekly_logger.py --week "2025-10-20"
"""

import os
import sys
import argparse
from datetime import datetime, timedelta
from pathlib import Path
from notion_client import Client
from dotenv import load_dotenv

# 프로젝트 루트 경로 설정
PROJECT_ROOT = Path(__file__).parent.parent.parent
sys.path.insert(0, str(PROJECT_ROOT))

# .env 파일 로드
load_dotenv(PROJECT_ROOT / ".env")


def get_week_range(week_start: str = None) -> tuple:
    """주간 범위 계산 (월요일~일요일)

    Args:
        week_start: 주의 시작일 (월요일, YYYY-MM-DD)

    Returns:
        (월요일, 일요일) 튜플
    """
    if week_start:
        monday = datetime.strptime(week_start, "%Y-%m-%d").date()
    else:
        today = datetime.now().date()
        monday = today - timedelta(days=today.weekday())

    sunday = monday + timedelta(days=6)

    return monday.strftime("%Y-%m-%d"), sunday.strftime("%Y-%m-%d")


def query_daily_logs(notion: Client, database_id: str, start_date: str, end_date: str) -> list:
    """일일 업무일지 조회

    Args:
        notion: Notion 클라이언트
        database_id: 일일 업무일지 DB ID
        start_date: 시작 날짜 (YYYY-MM-DD)
        end_date: 종료 날짜 (YYYY-MM-DD)

    Returns:
        일일 업무일지 페이지 목록
    """
    try:
        # 날짜 필드 찾기
        db = notion.databases.retrieve(database_id=database_id)
        date_field = None

        for field_name, field_config in db.get("properties", {}).items():
            if field_config.get("type") == "date":
                date_field = field_name
                break

        if not date_field:
            print("[WARNING] 날짜 필드를 찾을 수 없습니다. 모든 페이지를 조회합니다.")

        # 데이터베이스 쿼리
        filter_obj = None
        if date_field:
            filter_obj = {
                "and": [
                    {
                        "property": date_field,
                        "date": {
                            "on_or_after": start_date
                        }
                    },
                    {
                        "property": date_field,
                        "date": {
                            "on_or_before": end_date
                        }
                    }
                ]
            }

        if filter_obj:
            response = notion.databases.query(
                database_id=database_id,
                filter=filter_obj,
                sorts=[
                    {
                        "property": date_field,
                        "direction": "ascending"
                    }
                ]
            )
        else:
            response = notion.databases.query(database_id=database_id)

        return response.get("results", [])

    except Exception as e:
        print(f"[ERROR] 일일 업무일지 조회 실패: {e}")
        import traceback
        traceback.print_exc()
        return []


def extract_page_content(notion: Client, page_id: str) -> str:
    """페이지 내용 추출

    Args:
        notion: Notion 클라이언트
        page_id: 페이지 ID

    Returns:
        페이지 내용 (텍스트)
    """
    try:
        blocks = notion.blocks.children.list(block_id=page_id)
        content_parts = []

        for block in blocks.get("results", []):
            block_type = block.get("type")

            if block_type in ["paragraph", "heading_1", "heading_2", "heading_3"]:
                rich_text = block.get(block_type, {}).get("rich_text", [])
                text = "".join([rt.get("plain_text", "") for rt in rich_text])
                if text:
                    content_parts.append(text)

        return "\n".join(content_parts)

    except Exception as e:
        print(f"[WARNING] 페이지 내용 추출 실패 ({page_id}): {e}")
        return ""


def summarize_with_openai(daily_logs_content: list) -> str:
    """OpenAI API로 주간 요약 생성

    Args:
        daily_logs_content: 일일 업무일지 내용 목록

    Returns:
        주간 요약 텍스트
    """
    try:
        from openai import OpenAI

        api_key = os.getenv("OPENAI_API_KEY")
        if not api_key:
            print("[WARNING] OPENAI_API_KEY가 설정되지 않았습니다. 간단한 요약을 생성합니다.")
            return "\n\n".join([f"- {content[:100]}..." for content in daily_logs_content])

        client = OpenAI(api_key=api_key)

        # 프롬프트 구성
        daily_logs_text = "\n\n".join([
            f"[일일 업무 {i+1}]\n{content}"
            for i, content in enumerate(daily_logs_content)
        ])

        prompt = f"""다음은 이번 주의 일일 업무 기록입니다. 이를 분석하여 주간 요약을 작성해주세요.

{daily_logs_text}

다음 형식으로 요약해주세요:

## 주요 성과
(이번 주의 주요 성과 3-5개를 bullet point로 작성)

## 완료된 작업
(완료된 작업 목록)

## 진행 중인 작업
(아직 진행 중인 작업 목록)

## 다음 주 계획
(다음 주에 진행할 예정인 작업)
"""

        response = client.chat.completions.create(
            model=os.getenv("OPENAI_MODEL", "gpt-4-turbo-preview"),
            messages=[
                {"role": "system", "content": "당신은 업무 일지를 분석하고 요약하는 전문가입니다."},
                {"role": "user", "content": prompt}
            ],
            temperature=0.7,
            max_tokens=1500
        )

        summary = response.choices[0].message.content
        return summary

    except ImportError:
        print("[WARNING] openai 패키지가 설치되지 않았습니다. 간단한 요약을 생성합니다.")
        return "\n\n".join([f"- {content[:100]}..." for content in daily_logs_content])
    except Exception as e:
        print(f"[WARNING] OpenAI 요약 생성 실패: {e}. 간단한 요약을 생성합니다.")
        return "\n\n".join([f"- {content[:100]}..." for content in daily_logs_content])


def create_weekly_log(week_start: str = None) -> dict:
    """주간 업무 일지 생성

    Args:
        week_start: 주의 시작일 (월요일, YYYY-MM-DD)

    Returns:
        생성된 페이지 정보
    """
    # Notion API 초기화
    api_token = os.getenv("NOTION_API_TOKEN")
    daily_db_id = os.getenv("NOTION_DAILY_LOG_DB_ID")
    weekly_db_id = os.getenv("NOTION_WEEKLY_LOG_DB_ID")

    if not api_token:
        raise ValueError("NOTION_API_TOKEN이 설정되지 않았습니다.")
    if not daily_db_id:
        raise ValueError("NOTION_DAILY_LOG_DB_ID가 설정되지 않았습니다.")
    if not weekly_db_id:
        raise ValueError("NOTION_WEEKLY_LOG_DB_ID가 설정되지 않았습니다.")

    notion = Client(auth=api_token)

    # 주간 범위 계산
    monday, sunday = get_week_range(week_start)
    print(f"[1/6] 주간 범위: {monday} ~ {sunday}")

    # 일일 업무일지 조회
    print(f"[2/6] 일일 업무일지 조회 중...")
    daily_logs = query_daily_logs(notion, daily_db_id, monday, sunday)
    print(f"      {len(daily_logs)}개 발견")

    if not daily_logs:
        print("[WARNING] 이번 주 일일 업무일지가 없습니다.")
        raise ValueError("주간 요약을 생성할 일일 업무일지가 없습니다.")

    # 일일 업무일지 내용 추출
    print(f"[3/6] 일일 업무일지 내용 추출 중...")
    daily_contents = []
    for page in daily_logs:
        page_id = page["id"]
        content = extract_page_content(notion, page_id)
        if content:
            daily_contents.append(content)

    print(f"      {len(daily_contents)}개 페이지 내용 추출 완료")

    # OpenAI로 주간 요약 생성
    print(f"[4/6] AI로 주간 요약 생성 중...")
    summary = summarize_with_openai(daily_contents)

    # 주간 업무일지 페이지 생성
    print(f"[5/6] 주간 업무일지 페이지 생성 중...")

    # 주간 DB 스키마 확인
    weekly_db = notion.databases.retrieve(database_id=weekly_db_id)
    weekly_schema = weekly_db.get("properties", {})

    properties = {}

    # 제목 필드
    title_field = None
    for field_name, field_config in weekly_schema.items():
        if field_config.get("type") == "title":
            title_field = field_name
            break

    if title_field:
        week_num = datetime.strptime(monday, "%Y-%m-%d").isocalendar()[1]
        properties[title_field] = {
            "title": [
                {
                    "text": {
                        "content": f"2025년 {week_num}주차 업무 일지 ({monday} ~ {sunday})"
                    }
                }
            ]
        }

    # 날짜 필드 (주간 시작일)
    for field_name, field_config in weekly_schema.items():
        if field_config.get("type") == "date":
            properties[field_name] = {
                "date": {
                    "start": monday,
                    "end": sunday
                }
            }
            break

    # 페이지 내용 (본문)
    children = [
        {
            "object": "block",
            "type": "paragraph",
            "paragraph": {
                "rich_text": [{"type": "text", "text": {"content": summary}}]
            }
        }
    ]

    try:
        new_page = notion.pages.create(
            parent={"database_id": weekly_db_id},
            properties=properties,
            children=children
        )

        page_id = new_page["id"]
        page_url = new_page["url"]

        print(f"[6/6] 생성 완료!")
        print(f"\n[SUCCESS] 주간 업무 일지가 생성되었습니다!")
        print(f"          주간 범위: {monday} ~ {sunday}")
        print(f"          분석된 일일 업무일지: {len(daily_logs)}개")
        print(f"          페이지 ID: {page_id}")
        print(f"          URL: {page_url}")

        return {
            "id": page_id,
            "url": page_url,
            "week_start": monday,
            "week_end": sunday,
            "daily_logs_count": len(daily_logs)
        }

    except Exception as e:
        print(f"[ERROR] 페이지 생성 실패: {e}")
        import traceback
        traceback.print_exc()
        raise


def main():
    """메인 함수"""
    parser = argparse.ArgumentParser(
        description="Notion 주간 업무 일지 자동 생성 도구"
    )
    parser.add_argument(
        "--week",
        "-w",
        help="주의 시작일 (월요일, YYYY-MM-DD 형식, 기본값: 이번 주)"
    )

    args = parser.parse_args()

    # 주간 업무 일지 생성
    try:
        result = create_weekly_log(args.week)
        print(f"\n[+] 완료! Notion에서 확인하세요: {result['url']}")
    except Exception as e:
        print(f"\n[ERROR] 오류 발생: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
