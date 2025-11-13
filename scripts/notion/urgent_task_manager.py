"""
긴급 업무 추가 관리 스크립트

이 스크립트는 Notion "To-do DB"에 긴급 업무를 추가합니다.
원래 템플릿의 "긴급하게 처리할 일" 테이블이 인라인 DB이므로
To-do DB를 활용하여 긴급 업무를 관리합니다.

사용법:
    # 기본 사용
    python scripts/notion/urgent_task_manager.py "고객 미팅 준비" --deadline "2025-10-25"

    # 카테고리 지정
    python scripts/notion/urgent_task_manager.py "블로그 포스팅" --deadline "2025-10-24" --category "컨텐츠"

    # 대화형 모드
    python scripts/notion/urgent_task_manager.py
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


def get_database_schema(notion: Client, database_id: str) -> dict:
    """데이터베이스 스키마 조회"""
    try:
        db = notion.databases.retrieve(database_id=database_id)
        return db.get("properties", {})
    except Exception as e:
        print(f"[ERROR] 데이터베이스 스키마 조회 실패: {e}")
        return {}


def calculate_remaining_days(deadline: str) -> int:
    """남은 기간 계산"""
    today = datetime.now().date()
    deadline_date = datetime.strptime(deadline, "%Y-%m-%d").date()
    delta = (deadline_date - today).days
    return delta


def add_urgent_task(
    task: str,
    deadline: str = None,
    category: str = "긴급",
    status: str = "진행 중"
) -> dict:
    """긴급 업무 추가

    Args:
        task: 할 일
        deadline: 목표일 (YYYY-MM-DD, 기본값: 오늘+3일)
        category: 분류 (기본값: "긴급")
        status: 진행상황 (기본값: "진행 중")

    Returns:
        생성된 페이지 정보
    """
    # Notion API 초기화
    api_token = os.getenv("NOTION_API_TOKEN")
    database_id = os.getenv("NOTION_TODO_DB_ID")

    if not api_token:
        raise ValueError("NOTION_API_TOKEN이 설정되지 않았습니다.")
    if not database_id:
        raise ValueError("NOTION_TODO_DB_ID가 설정되지 않았습니다. extract_notion_databases.py를 먼저 실행하세요.")

    # 목표일 설정 (기본값: 오늘+3일)
    if not deadline:
        deadline = (datetime.now() + timedelta(days=3)).strftime("%Y-%m-%d")

    notion = Client(auth=api_token)

    # 데이터베이스 스키마 확인
    print(f"[1/5] 데이터베이스 스키마 확인 중...")
    schema = get_database_schema(notion, database_id)

    if not schema:
        raise ValueError("데이터베이스 스키마를 가져올 수 없습니다.")

    print(f"[2/5] 발견된 필드: {', '.join(schema.keys())}")

    # 남은 기간 계산
    remaining_days = calculate_remaining_days(deadline)
    print(f"[3/5] 목표일까지 남은 기간: {remaining_days}일")

    # 페이지 속성 구성
    properties = {}

    # 제목 필드 찾기
    title_field = None
    for field_name, field_config in schema.items():
        if field_config.get("type") == "title":
            title_field = field_name
            break

    if title_field:
        properties[title_field] = {
            "title": [
                {
                    "text": {
                        "content": f"[긴급] {task}"
                    }
                }
            ]
        }

    # 날짜 필드 찾기 (목표일)
    for field_name, field_config in schema.items():
        if field_config.get("type") == "date" and "목표" in field_name.lower() or "deadline" in field_name.lower():
            properties[field_name] = {
                "date": {
                    "start": deadline
                }
            }
            break

    # Select 필드 찾기 (분류, 진행상황)
    for field_name, field_config in schema.items():
        field_type = field_config.get("type")

        if field_type == "select":
            # 분류 필드
            if "분류" in field_name or "category" in field_name.lower():
                properties[field_name] = {
                    "select": {
                        "name": category
                    }
                }
            # 진행상황 필드
            elif "진행" in field_name or "status" in field_name.lower():
                properties[field_name] = {
                    "select": {
                        "name": status
                    }
                }

    # Number 필드 찾기 (남은 기간)
    for field_name, field_config in schema.items():
        if field_config.get("type") == "number" and ("남은" in field_name or "remaining" in field_name.lower()):
            properties[field_name] = {
                "number": remaining_days
            }
            break

    # 페이지 생성
    print(f"[4/5] 긴급 업무 추가 중...")

    try:
        new_page = notion.pages.create(
            parent={"database_id": database_id},
            properties=properties
        )

        page_id = new_page["id"]
        page_url = new_page["url"]

        print(f"[5/5] 생성 완료!")
        print(f"\n[SUCCESS] 긴급 업무가 추가되었습니다!")
        print(f"          할 일: {task}")
        print(f"          목표일: {deadline}")
        print(f"          남은 기간: {remaining_days}일")
        print(f"          분류: {category}")
        print(f"          진행상황: {status}")
        print(f"          페이지 ID: {page_id}")
        print(f"          URL: {page_url}")

        return {
            "id": page_id,
            "url": page_url,
            "task": task,
            "deadline": deadline,
            "remaining_days": remaining_days,
            "category": category,
            "status": status
        }

    except Exception as e:
        print(f"[ERROR] 페이지 생성 실패: {e}")
        import traceback
        traceback.print_exc()
        raise


def main():
    """메인 함수"""
    parser = argparse.ArgumentParser(
        description="Notion 긴급 업무 추가 도구"
    )
    parser.add_argument(
        "task",
        nargs="?",
        help="할 일 (미제공시 대화형 모드)"
    )
    parser.add_argument(
        "--deadline",
        "-d",
        help="목표일 (YYYY-MM-DD 형식, 기본값: 오늘+3일)"
    )
    parser.add_argument(
        "--category",
        "-c",
        default="긴급",
        help="분류 (기본값: 긴급)"
    )
    parser.add_argument(
        "--status",
        "-s",
        default="진행 중",
        help="진행상황 (기본값: 진행 중)"
    )

    args = parser.parse_args()

    # 할 일 입력
    task = args.task
    if not task:
        print("=== 긴급 업무 추가 ===")
        task = input("할 일을 입력하세요: ").strip()

        if not task:
            print("[ERROR] 할 일을 입력해야 합니다.")
            sys.exit(1)

        # 목표일 입력 (선택)
        if not args.deadline:
            deadline_input = input("목표일 (YYYY-MM-DD, Enter=오늘+3일): ").strip()
            args.deadline = deadline_input if deadline_input else None

    # 긴급 업무 추가
    try:
        result = add_urgent_task(
            task=task,
            deadline=args.deadline,
            category=args.category,
            status=args.status
        )
        print(f"\n[+] 완료! Notion에서 확인하세요: {result['url']}")
    except Exception as e:
        print(f"\n[ERROR] 오류 발생: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
