"""
Judgify-core 버전 관리 (Single Source of Truth)

이 파일이 프로젝트 전체 버전의 유일한 진실의 원천입니다.
버전 변경시 이 파일만 수정하고, scripts/bump_version.py를 실행하세요.
"""

__version__ = "0.1.0"
__stage__ = "alpha"  # alpha → beta → rc → stable
__release_date__ = "2025-10-22"
__description__ = "Desktop App 프로토타입 개발 중"

# 9개 마이크로서비스 구현 상태 추적
MICROSERVICES_STATUS = {
    # 포트: (서비스명, 상태, 완료율)
    8000: ("API Gateway", "planned", 0),
    8001: ("Workflow Service", "planned", 0),
    8002: ("Judgment Service", "planned", 0),
    8003: ("Action Service", "planned", 0),
    8004: ("Notification Service", "planned", 0),
    8005: ("Logging Service", "planned", 0),
    8006: ("Data Visualization Service", "planned", 0),
    8007: ("BI Service", "planned", 0),
    8008: ("Chat Interface Service", "planned", 0),
    8009: ("Learning Service", "planned", 0),
}

# Desktop App 구현 상태
DESKTOP_APP_STATUS = {
    "frontend": ("React + TypeScript", "in_progress", 60),
    "backend": ("Tauri + Rust", "in_progress", 75),  # Week 2 Phase 1 완료: Judgment Engine 고도화
    "database": ("SQLite", "in_progress", 80),  # Feedback, TrainingSample 테이블 활용
}

# 문서화 상태
DOCUMENTATION_STATUS = {
    "architecture": ("시스템 아키텍처", "completed", 100),
    "services": ("서비스별 설계", "completed", 100),
    "algorithms": ("알고리즘 설계", "completed", 100),
    "operations": ("운영 가이드", "completed", 100),
    "development": ("개발 계획", "completed", 100),
}


def get_version_info() -> dict:
    """버전 정보 전체 조회"""
    return {
        "version": __version__,
        "stage": __stage__,
        "release_date": __release_date__,
        "description": __description__,
        "microservices": MICROSERVICES_STATUS,
        "desktop_app": DESKTOP_APP_STATUS,
        "documentation": DOCUMENTATION_STATUS,
    }


def get_overall_completion() -> float:
    """전체 프로젝트 완료율 계산"""
    # 마이크로서비스 평균 (40% 가중치)
    ms_completion = sum(status[2] for status in MICROSERVICES_STATUS.values()) / len(MICROSERVICES_STATUS)

    # Desktop App 평균 (40% 가중치)
    app_completion = sum(status[2] for status in DESKTOP_APP_STATUS.values()) / len(DESKTOP_APP_STATUS)

    # 문서화 평균 (20% 가중치)
    doc_completion = sum(status[2] for status in DOCUMENTATION_STATUS.values()) / len(DOCUMENTATION_STATUS)

    # 가중 평균
    overall = (ms_completion * 0.4) + (app_completion * 0.4) + (doc_completion * 0.2)

    return round(overall, 1)


def print_status():
    """현재 프로젝트 상태 출력 (개발용)"""
    info = get_version_info()

    print("=" * 60)
    print(f"  Judgify-core {info['version']} ({info['stage']})")
    print(f"  {info['description']}")
    print("=" * 60)
    print(f"  전체 완료율: {get_overall_completion()}%")
    print("=" * 60)

    print(f"\n[Desktop App]")
    for name, (desc, status, pct) in info['desktop_app'].items():
        status_mark = "[OK]" if status == "completed" else "[->]" if status == "in_progress" else "[ ]"
        print(f"  {status_mark} {desc}: {pct}%")

    print(f"\n[마이크로서비스 (9개)]")
    for port, (name, status, pct) in info['microservices'].items():
        status_mark = "[OK]" if status == "completed" else "[->]" if status == "in_progress" else "[ ]"
        print(f"  {status_mark} {name} ({port}): {pct}%")

    print(f"\n[문서화]")
    for name, (desc, status, pct) in info['documentation'].items():
        status_mark = "[OK]" if status == "completed" else "[->]" if status == "in_progress" else "[ ]"
        print(f"  {status_mark} {desc}: {pct}%")


if __name__ == "__main__":
    # 실행시 현재 상태 출력
    print_status()
