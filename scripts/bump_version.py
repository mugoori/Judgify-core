#!/usr/bin/env python3
"""
버전 증가 스크립트

사용법:
  python scripts/bump_version.py minor  # 0.1.0 → 0.2.0
  python scripts/bump_version.py patch  # 0.1.0 → 0.1.1

자동 동기화 파일:
  - version.py (원본)
  - package.json
  - src-tauri/Cargo.toml
"""

import re
import sys
from pathlib import Path
from datetime import datetime


def read_current_version():
    """version.py에서 현재 버전 읽기"""
    version_file = Path(__file__).parent.parent / "version.py"

    if not version_file.exists():
        print("Error: version.py not found!")
        sys.exit(1)

    content = version_file.read_text(encoding='utf-8')
    match = re.search(r'__version__ = "([\d\.]+)"', content)

    if not match:
        print("Error: Version not found in version.py!")
        sys.exit(1)

    return match.group(1)


def parse_version(version_string):
    """버전 문자열 파싱 (0.1.0 → (0, 1, 0))"""
    parts = version_string.split('.')
    if len(parts) != 3:
        print(f"Error: Invalid version format: {version_string}")
        sys.exit(1)

    try:
        return tuple(map(int, parts))
    except ValueError:
        print(f"Error: Version parts must be integers: {version_string}")
        sys.exit(1)


def bump_version(current, part):
    """버전 증가 로직"""
    major, minor, patch = parse_version(current)

    if part == 'major':
        major += 1
        minor = 0
        patch = 0
    elif part == 'minor':
        minor += 1
        patch = 0
    elif part == 'patch':
        patch += 1
    else:
        print(f"Error: Invalid part '{part}'. Use: major, minor, or patch")
        sys.exit(1)

    return f"{major}.{minor}.{patch}"


def update_version_py(new_version):
    """version.py 업데이트"""
    version_file = Path(__file__).parent.parent / "version.py"
    content = version_file.read_text(encoding='utf-8')

    # __version__ 업데이트
    content = re.sub(
        r'__version__ = "[\d\.]+"',
        f'__version__ = "{new_version}"',
        content
    )

    # __release_date__ 업데이트
    today = datetime.now().strftime("%Y-%m-%d")
    content = re.sub(
        r'__release_date__ = "[\d-]+"',
        f'__release_date__ = "{today}"',
        content
    )

    version_file.write_text(content, encoding='utf-8')
    print(f"  version.py → {new_version}")


def update_package_json(new_version):
    """package.json 업데이트"""
    package_file = Path(__file__).parent.parent / "package.json"

    if not package_file.exists():
        print("  package.json not found (skipping)")
        return

    content = package_file.read_text(encoding='utf-8')
    content = re.sub(
        r'"version": "[\d\.]+"',
        f'"version": "{new_version}"',
        content
    )

    package_file.write_text(content, encoding='utf-8')
    print(f"  package.json → {new_version}")


def update_cargo_toml(new_version):
    """src-tauri/Cargo.toml 업데이트"""
    cargo_file = Path(__file__).parent.parent / "src-tauri" / "Cargo.toml"

    if not cargo_file.exists():
        print("  Cargo.toml not found (skipping)")
        return

    content = cargo_file.read_text(encoding='utf-8')

    # [package] 섹션의 첫 번째 version만 변경
    content = re.sub(
        r'(^\[package\].*?^version = )"[\d\.]+"',
        rf'\1"{new_version}"',
        content,
        count=1,
        flags=re.MULTILINE | re.DOTALL
    )

    cargo_file.write_text(content, encoding='utf-8')
    print(f"  Cargo.toml → {new_version}")


def update_tauri_conf_json(new_version):
    """src-tauri/tauri.conf.json 업데이트"""
    import json

    tauri_conf_file = Path(__file__).parent.parent / "src-tauri" / "tauri.conf.json"

    if not tauri_conf_file.exists():
        print("  tauri.conf.json not found (skipping)")
        return

    content = tauri_conf_file.read_text(encoding='utf-8')
    data = json.loads(content)

    # package.version 업데이트
    if 'package' in data and 'version' in data['package']:
        data['package']['version'] = new_version

    # JSON 파일 쓰기 (2-space indent, 한글 유지)
    tauri_conf_file.write_text(
        json.dumps(data, indent=2, ensure_ascii=False) + '\n',
        encoding='utf-8'
    )
    print(f"  tauri.conf.json → {new_version}")


def main():
    """메인 실행 함수"""
    # 인자 확인
    if len(sys.argv) < 2:
        print("Usage: python scripts/bump_version.py <major|minor|patch>")
        print("\nExamples:")
        print("  python scripts/bump_version.py minor  # 0.1.0 → 0.2.0")
        print("  python scripts/bump_version.py patch  # 0.1.0 → 0.1.1")
        sys.exit(1)

    part = sys.argv[1].lower()

    # 현재 버전 읽기
    current_version = read_current_version()
    print(f"\nCurrent version: {current_version}")

    # 새 버전 계산
    new_version = bump_version(current_version, part)
    print(f"New version: {new_version}\n")

    # 확인 요청
    confirm = input(f"Bump version {current_version} → {new_version}? (y/N): ")
    if confirm.lower() != 'y':
        print("Cancelled")
        sys.exit(0)

    print("\nUpdating files...")

    # 파일 업데이트
    update_version_py(new_version)
    update_package_json(new_version)
    update_cargo_toml(new_version)
    update_tauri_conf_json(new_version)

    print(f"\nVersion bumped successfully: {current_version} → {new_version}")
    print("\n다음 명령 실행:")
    print(f"  git add version.py package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json")
    print(f"  git commit -m 'chore: Bump version to {new_version}'")
    print(f"  git tag -a v{new_version} -m 'Release v{new_version}'")
    print(f"  git push origin develop --tags")


if __name__ == "__main__":
    main()
