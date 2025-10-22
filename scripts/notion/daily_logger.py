"""
일일 업무 일지 자동 작성 스크립트

이 스크립트는 Notion "일일 업무일지 DB"에 오늘 날짜의 업무 일지를 자동으로 생성합니다.

사용법:
    # 기본 사용
    python scripts/notion/daily_logger.py "오늘은 Notion 자동화 시스템을 구축했습니다."

    # 대화형 모드
    python scripts/notion/daily_logger.py
"""

import os
import sys
import argparse
from datetime import datetime
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


def extract_summary_from_content(content: str, max_length: int = 50) -> str:
    """
    업무 내용에서 요약 키워드 추출

    추출 규칙:
    1. 콜론(:) 앞 부분이 있으면 그것을 제목으로 사용
    2. 콜론이 없으면 첫 번째 쉼표(,) 앞 부분 사용
    3. 둘 다 없으면 전체 내용 사용 (최대 max_length까지)
    4. max_length 초과 시 자르고 "..." 추가
    5. 빈 내용이면 "업무 일지" 기본값 반환

    Args:
        content: 업무 내용 텍스트
        max_length: 최대 제목 길이 (기본값: 50자)

    Returns:
        추출된 요약 키워드

    예시:
        "Notion 가독성 개선: 블록 파싱, 테스트" → "Notion 가독성 개선"
        "Database ID 추출, 스크립트 작성" → "Database ID 추출"
        "회의 참석" → "회의 참석"
    """
    if not content or not content.strip():
        return "업무 일지"

    content = content.strip()
    summary = ""

    # 1. 콜론(:) 앞 부분 우선 사용
    if ':' in content:
        summary = content.split(':', 1)[0].strip()
    # 2. 콜론 없으면 첫 번째 쉼표(,) 앞 부분 사용
    elif ',' in content:
        summary = content.split(',', 1)[0].strip()
    # 3. 둘 다 없으면 전체 내용 사용
    else:
        summary = content

    # 4. 길이 제한 (max_length 초과 시 자르고 "..." 추가)
    if len(summary) > max_length:
        summary = summary[:max_length].strip() + "..."

    # 5. 빈 결과면 기본값 반환
    if not summary:
        return "업무 일지"

    return summary


def parse_content_to_blocks(content: str) -> list:
    """
    업무 내용을 구조화된 Notion 블록으로 변환

    파싱 규칙:
    - 콜론(:) → heading_2 (섹션 제목)
    - 쉼표(,) → numbered_list_item (주요 항목)
    - 괄호() → bulleted_list_item (하위 항목)

    Args:
        content: 업무 내용 텍스트

    Returns:
        Notion 블록 리스트
    """
    blocks = []

    # 1. "제목: 내용" 형식 분리
    if ':' in content:
        parts = content.split(':', 1)
        title = parts[0].strip()
        body = parts[1].strip()

        # 제목 블록 (heading_2)
        blocks.append({
            "object": "block",
            "type": "heading_2",
            "heading_2": {
                "rich_text": [{"type": "text", "text": {"content": title}}]
            }
        })
    else:
        # 콜론이 없으면 전체를 본문으로 처리
        body = content

    # 2. 쉼표로 주요 항목 분리
    items = [item.strip() for item in body.split(',')]

    for item in items:
        if not item:
            continue

        # 3. 괄호로 하위 항목 추출
        if '(' in item and ')' in item:
            # 주요 항목과 하위 항목 분리
            main_item = item[:item.index('(')].strip()
            sub_items_str = item[item.index('(')+1:item.index(')')].strip()
            sub_items = [s.strip() for s in sub_items_str.split(',')]

            # 주요 항목 블록 (numbered_list)
            blocks.append({
                "object": "block",
                "type": "numbered_list_item",
                "numbered_list_item": {
                    "rich_text": [{"type": "text", "text": {"content": main_item}}]
                }
            })

            # 하위 항목 블록들 (bulleted_list)
            for sub_item in sub_items:
                if sub_item:
                    blocks.append({
                        "object": "block",
                        "type": "bulleted_list_item",
                        "bulleted_list_item": {
                            "rich_text": [{"type": "text", "text": {"content": sub_item}}]
                        }
                    })
        else:
            # 단순 항목 (numbered_list)
            blocks.append({
                "object": "block",
                "type": "numbered_list_item",
                "numbered_list_item": {
                    "rich_text": [{"type": "text", "text": {"content": item}}]
                }
            })

    # 블록이 없으면 원본 텍스트를 paragraph로 반환 (fallback)
    if not blocks:
        blocks.append({
            "object": "block",
            "type": "paragraph",
            "paragraph": {
                "rich_text": [{"type": "text", "text": {"content": content}}]
            }
        })

    return blocks


def create_daily_log(content: str, date: str = None) -> dict:
    """일일 업무 일지 생성

    Args:
        content: 업무 내용
        date: 날짜 (기본값: 오늘, 형식: YYYY-MM-DD)

    Returns:
        생성된 페이지 정보
    """
    # Notion API 초기화
    api_token = os.getenv("NOTION_API_TOKEN")
    database_id = os.getenv("NOTION_DAILY_LOG_DB_ID")

    if not api_token:
        raise ValueError("NOTION_API_TOKEN이 설정되지 않았습니다.")
    if not database_id:
        raise ValueError("NOTION_DAILY_LOG_DB_ID가 설정되지 않았습니다. extract_notion_databases.py를 먼저 실행하세요.")

    # 날짜 설정 (기본값: 오늘)
    if not date:
        date = datetime.now().strftime("%Y-%m-%d")

    notion = Client(auth=api_token)

    # 데이터베이스 스키마 확인
    print(f"[1/4] 데이터베이스 스키마 확인 중...")
    schema = get_database_schema(notion, database_id)

    if not schema:
        raise ValueError("데이터베이스 스키마를 가져올 수 없습니다.")

    print(f"[2/4] 발견된 필드: {', '.join(schema.keys())}")

    # 페이지 속성 구성
    properties = {}

    # 제목 필드 찾기 (일반적으로 'title', 'Name', '이름' 등)
    title_field = None
    for field_name, field_config in schema.items():
        if field_config.get("type") == "title":
            title_field = field_name
            break

    if title_field:
        # 내용에서 요약 키워드 추출하여 제목으로 사용 (날짜 제거)
        summary = extract_summary_from_content(content)
        properties[title_field] = {
            "title": [
                {
                    "text": {
                        "content": summary
                    }
                }
            ]
        }

    # 날짜 필드 찾기
    date_field = None
    for field_name, field_config in schema.items():
        if field_config.get("type") == "date":
            date_field = field_name
            break

    if date_field:
        properties[date_field] = {
            "date": {
                "start": date
            }
        }

    # 페이지 생성
    print(f"[3/4] 일일 업무 일지 생성 중... (날짜: {date})")

    # 페이지 내용 (본문) - 구조화된 블록으로 생성
    title_block = {
        "object": "block",
        "type": "heading_2",
        "heading_2": {
            "rich_text": [{"type": "text", "text": {"content": "오늘의 주요 업무"}}]
        }
    }

    # 컨텐츠 파싱하여 구조화된 블록 생성
    content_blocks = parse_content_to_blocks(content)

    # 전체 블록 결합
    children = [title_block] + content_blocks

    try:
        new_page = notion.pages.create(
            parent={"database_id": database_id},
            properties=properties,
            children=children
        )

        page_id = new_page["id"]
        page_url = new_page["url"]

        print(f"[4/4] 생성 완료!")
        print(f"\n[SUCCESS] 일일 업무 일지가 생성되었습니다!")
        print(f"          날짜: {date}")
        print(f"          페이지 ID: {page_id}")
        print(f"          URL: {page_url}")

        return {
            "id": page_id,
            "url": page_url,
            "date": date,
            "content": content
        }

    except Exception as e:
        print(f"[ERROR] 페이지 생성 실패: {e}")
        import traceback
        traceback.print_exc()
        raise


def main():
    """메인 함수"""
    parser = argparse.ArgumentParser(
        description="Notion 일일 업무 일지 자동 작성 도구"
    )
    parser.add_argument(
        "content",
        nargs="?",
        help="업무 내용 (미제공시 대화형 모드)"
    )
    parser.add_argument(
        "--date",
        "-d",
        help="날짜 (YYYY-MM-DD 형식, 기본값: 오늘)"
    )

    args = parser.parse_args()

    # 업무 내용 입력
    content = args.content
    if not content:
        print("=== 일일 업무 일지 작성 ===")
        content = input("오늘 수행한 업무를 입력하세요: ").strip()

        if not content:
            print("[ERROR] 업무 내용을 입력해야 합니다.")
            sys.exit(1)

    # 일일 업무 일지 생성
    try:
        result = create_daily_log(content, args.date)
        print(f"\n[+] 완료! Notion에서 확인하세요: {result['url']}")
    except Exception as e:
        print(f"\n[ERROR] 오류 발생: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
