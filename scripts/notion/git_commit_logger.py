"""
Git 커밋 메시지를 Notion 업무 일지에 자동 기록

이 스크립트는 Git push 후 post-push hook을 통해 자동 실행되며,
최근 커밋 메시지를 파싱하여 Notion에 추가합니다.

사용법:
    # Git push 후 자동 실행 (post-push hook)
    python scripts/notion/git_commit_logger.py

    # 수동 실행 (마지막 커밋만)
    python scripts/notion/git_commit_logger.py --manual

    # 특정 커밋 개수 지정
    python scripts/notion/git_commit_logger.py --count 3
"""

import os
import sys
import json
import argparse
import subprocess
from datetime import datetime
from pathlib import Path
from typing import List, Dict
from notion_client import Client
from dotenv import load_dotenv

# 프로젝트 루트 경로 설정
PROJECT_ROOT = Path(__file__).parent.parent.parent
sys.path.insert(0, str(PROJECT_ROOT))

# .env 파일 로드
load_dotenv(PROJECT_ROOT / ".env")

# daily_logger.py의 함수들 import
from scripts.notion.daily_logger import (
    parse_content_to_blocks,
    find_or_create_today_page,
    append_blocks_to_page,
    get_database_schema
)


def get_last_commits(repo_path: Path, count: int = None) -> List[Dict]:
    """
    마지막 push된 커밋 메시지들 가져오기

    Args:
        repo_path: Git 저장소 경로
        count: 가져올 커밋 개수 (None이면 origin/main..HEAD 범위)

    Returns:
        커밋 정보 리스트 [{"hash": "...", "title": "...", "body": "...", "time": "..."}, ...]
    """
    try:
        os.chdir(repo_path)

        # 커밋 범위 결정
        if count:
            # 마지막 N개 커밋
            commit_range = f"-{count}"
        else:
            # origin/main..HEAD (아직 push되지 않은 커밋들)
            # 이미 push된 경우라면 마지막 1개 커밋만
            result = subprocess.run(
                ["git", "rev-list", "--count", "origin/main..HEAD"],
                capture_output=True,
                text=True,
                encoding="utf-8",
                check=True
            )
            unpushed_count = int(result.stdout.strip())
            commit_range = f"-{unpushed_count}" if unpushed_count > 0 else "-1"

        # 커밋 메시지 가져오기
        # 포맷: 커밋해시|||제목|||본문|||타임스탬프
        result = subprocess.run(
            ["git", "log", commit_range, "--format=%H|||%s|||%b|||%ai"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            check=True
        )

        if not result.stdout.strip():
            print("[INFO] 새로운 커밋이 없습니다.")
            return []

        commits = []

        # 각 커밋은 빈 줄로 구분됨 (본문에도 빈 줄이 있을 수 있음)
        # 따라서 구분자(|||)를 기준으로 파싱
        raw_output = result.stdout.strip()

        # 커밋들을 분리 (각 커밋은 해시로 시작)
        import re
        commit_pattern = re.compile(r'^([0-9a-f]{40})\|\|\|(.+?)\|\|\|(.*?)\|\|\|(\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2})', re.MULTILINE | re.DOTALL)

        for match in commit_pattern.finditer(raw_output):
            commit_hash = match.group(1)
            title = match.group(2)
            body = match.group(3).strip()
            timestamp = match.group(4)

            # 시간 포맷팅 (예: "2025-10-22 14:30:15" → "2:30 PM")
            try:
                dt = datetime.strptime(timestamp, "%Y-%m-%d %H:%M:%S")
                time_str = dt.strftime("%-I:%M %p") if os.name != 'nt' else dt.strftime("%I:%M %p").lstrip("0")
            except:
                time_str = timestamp.split()[1][:5]  # fallback: HH:MM

            commits.append({
                "hash": commit_hash[:7],  # 짧은 해시
                "title": title,
                "body": body,
                "time": time_str,
                "timestamp": timestamp
            })

        return commits

    except subprocess.CalledProcessError as e:
        print(f"[ERROR] Git 명령 실패: {e}")
        return []
    except Exception as e:
        print(f"[ERROR] 커밋 가져오기 실패: {e}")
        return []


def parse_commit_message(commit: Dict) -> Dict:
    """
    커밋 메시지 파싱하여 Notion 블록으로 변환

    Args:
        commit: 커밋 정보 dict (hash, title, body, time)

    Returns:
        {"title": "...", "blocks": [...]}
    """
    # 제목에서 Conventional Commits 타입 제거 (feat:, fix:, docs: 등)
    title = commit["title"]
    for prefix in ["feat:", "fix:", "docs:", "style:", "refactor:", "test:", "chore:", "perf:"]:
        if title.lower().startswith(prefix):
            title = title[len(prefix):].strip()
            break

    # 본문이 있으면 Quote Block으로 처리 (가독성 우선)
    # parse_content_to_blocks() 대신 간단한 Quote 사용
    content_blocks = []
    if commit["body"]:
        content_blocks.append({
            "object": "block",
            "type": "quote",
            "quote": {
                "rich_text": [{
                    "type": "text",
                    "text": {"content": commit["body"]},
                    "annotations": {
                        "color": "default"
                    }
                }],
                "color": "gray_background"
            }
        })

    # 커밋 헤더 블록 생성
    blocks = []

    # 1. 구분선
    blocks.append({
        "object": "block",
        "type": "divider",
        "divider": {}
    })

    # 2. 시간 정보 (작은 글씨)
    blocks.append({
        "object": "block",
        "type": "paragraph",
        "paragraph": {
            "rich_text": [{
                "type": "text",
                "text": {"content": f"⏰ {commit['time']}  •  #{commit['hash']}"},
                "annotations": {
                    "color": "gray",
                    "code": False
                }
            }]
        }
    })

    # 3. 커밋 제목 (heading_3)
    blocks.append({
        "object": "block",
        "type": "heading_3",
        "heading_3": {
            "rich_text": [{
                "type": "text",
                "text": {"content": title},
                "annotations": {"bold": True}
            }]
        }
    })

    # 4. 본문 블록들 추가
    blocks.extend(content_blocks)

    return {
        "title": title,
        "blocks": blocks
    }


def save_backup(commits: List[Dict], backup_dir: Path):
    """
    로컬 백업 파일 생성 (Notion 업로드 실패시 사용)

    Args:
        commits: 커밋 정보 리스트
        backup_dir: 백업 디렉토리 경로
    """
    try:
        backup_dir.mkdir(parents=True, exist_ok=True)

        # 파일명: YYYY-MM-DD_HHMMSS.json
        timestamp = datetime.now().strftime("%Y-%m-%d_%H%M%S")
        backup_file = backup_dir / f"{timestamp}.json"

        # 백업 데이터 저장
        with open(backup_file, "w", encoding="utf-8") as f:
            json.dump(commits, f, ensure_ascii=False, indent=2)

        print(f"[INFO] 백업 파일 생성: {backup_file}")
        return backup_file

    except Exception as e:
        print(f"[ERROR] 백업 파일 생성 실패: {e}")
        return None


def main():
    """메인 함수"""
    parser = argparse.ArgumentParser(
        description="Git 커밋 메시지를 Notion 업무 일지에 자동 기록"
    )
    parser.add_argument(
        "--manual",
        action="store_true",
        help="수동 실행 모드 (마지막 커밋만 처리)"
    )
    parser.add_argument(
        "--count",
        type=int,
        help="처리할 커밋 개수 (기본값: origin/main..HEAD 범위)"
    )

    args = parser.parse_args()

    print("=" * 60)
    print("Git → Notion 자동 업무 일지 시스템")
    print("=" * 60)

    # Notion API 초기화
    api_token = os.getenv("NOTION_API_TOKEN")
    database_id = os.getenv("NOTION_DAILY_LOG_DB_ID")

    if not api_token:
        print("[ERROR] NOTION_API_TOKEN이 설정되지 않았습니다.")
        print("        .env 파일을 확인하세요.")
        return 1

    if not database_id:
        print("[ERROR] NOTION_DAILY_LOG_DB_ID가 설정되지 않았습니다.")
        print("        python scripts/notion/extract_notion_databases.py 실행 필요")
        return 1

    notion = Client(auth=api_token)
    backup_dir = PROJECT_ROOT / "scripts" / "notion" / ".commit_backup"

    # 1. 커밋 메시지 가져오기
    print("\n[1/4] 커밋 메시지 가져오기...")
    commits = get_last_commits(PROJECT_ROOT, count=args.count)

    if not commits:
        print("[INFO] 처리할 커밋이 없습니다.")
        return 0

    print(f"[INFO] {len(commits)}개 커밋 발견")
    for i, commit in enumerate(commits, 1):
        print(f"      {i}. [{commit['hash']}] {commit['title']}")

    # 2. 오늘 날짜 페이지 찾기 또는 생성
    print("\n[2/4] Notion 페이지 준비 중...")
    try:
        page_id = find_or_create_today_page(notion, database_id)
    except Exception as e:
        print(f"[ERROR] 페이지 준비 실패: {e}")
        print("[INFO] 백업 파일 생성 중...")
        save_backup(commits, backup_dir)
        return 1

    # 3. 각 커밋을 Notion에 추가
    print("\n[3/4] 커밋 내용 Notion에 추가 중...")
    success_count = 0
    failed_commits = []

    for i, commit in enumerate(commits, 1):
        try:
            print(f"      [{i}/{len(commits)}] {commit['title'][:50]}...")

            # 커밋 메시지 파싱
            parsed = parse_commit_message(commit)

            # Notion 페이지에 추가
            append_blocks_to_page(notion, page_id, parsed["blocks"])

            success_count += 1

        except Exception as e:
            print(f"      [ERROR] 추가 실패: {e}")
            failed_commits.append(commit)

    # 4. 결과 요약
    print("\n[4/4] 완료!")
    print(f"      성공: {success_count}/{len(commits)}개")

    if failed_commits:
        print(f"      실패: {len(failed_commits)}개")
        print("[INFO] 실패한 커밋 백업 중...")
        save_backup(failed_commits, backup_dir)

    # Notion 페이지 URL 가져오기
    try:
        page = notion.pages.retrieve(page_id=page_id)
        page_url = page["url"]
        print(f"\n[SUCCESS] Notion 업무 일지가 업데이트되었습니다!")
        print(f"          URL: {page_url}")
    except:
        print(f"\n[SUCCESS] Notion 업무 일지가 업데이트되었습니다!")

    return 0 if success_count == len(commits) else 1


if __name__ == "__main__":
    try:
        exit_code = main()
        sys.exit(exit_code)
    except KeyboardInterrupt:
        print("\n[INFO] 사용자에 의해 중단되었습니다.")
        sys.exit(1)
    except Exception as e:
        print(f"\n[ERROR] 예상치 못한 오류: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
