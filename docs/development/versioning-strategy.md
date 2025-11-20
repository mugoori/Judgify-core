# 버전 관리 전략 (Versioning Strategy)

**작성일**: 2025-10-22
**버전**: 0.1.0
**프로젝트**: Judgify-core Ver2.0 Final

---

## 📋 목차

1. [현재 버전 정책](#1-현재-버전-정책)
2. [3단계 로드맵](#2-3단계-로드맵)
3. [버전 관리 파일](#3-버전-관리-파일)
4. [사용 가이드](#4-사용-가이드)
5. [마이크로서비스 독립 버전](#5-마이크로서비스-독립-버전)
6. [Git 태그 전략](#6-git-태그-전략)
7. [자동화 계획](#7-자동화-계획)

---

## 1. 현재 버전 정책

### 1.1 버전 형식

**Semantic Versioning 0.x.x 시리즈**

```
형식: 0.MINOR.PATCH

예시:
- 0.1.0: Desktop App 프로토타입 (현재)
- 0.2.0: Judgment Service 첫 구현
- 0.3.0: Learning Service 추가
- 0.9.0: 베타 릴리스 (9개 서비스 완성)
- 1.0.0: 정식 릴리스 🎉
```

### 1.2 버전 증가 규칙

| 변경 유형 | 버전 증가 | 예시 |
|----------|----------|------|
| **주요 기능 추가** | MINOR | 0.1.0 → 0.2.0 |
| **마이크로서비스 구현** | MINOR | 0.2.0 → 0.3.0 |
| **버그 수정** | PATCH | 0.2.0 → 0.2.1 |
| **문서 업데이트** | PATCH | 0.2.1 → 0.2.2 |
| **리팩토링** | PATCH | 0.2.2 → 0.2.3 |

### 1.3 개발 단계

| 단계 | 버전 범위 | 설명 |
|------|----------|------|
| **alpha** | 0.1.0 ~ 0.8.x | 초기 개발, API 변경 자유 |
| **beta** | 0.9.0 ~ 0.9.x | 기능 완성, 버그 수정 집중 |
| **rc** | 1.0.0-rc.1 ~ rc.x | Release Candidate, 최종 검증 |
| **stable** | 1.0.0 이상 | 정식 릴리스 |

---

## 2. 3단계 로드맵

### Phase 1: 초기 개발 (현재 ~ 3개월)

**목표**: 9개 마이크로서비스 구현

```yaml
0.1.0: Desktop App 프로토타입 ✅
  - Tauri + React 기본 구조
  - 상세 설계 문서 완성
  - 진행도: 45%

0.2.0: Judgment Service 첫 구현 (예정)
  - 하이브리드 판단 엔진 (Rule + LLM)
  - PostgreSQL + pgvector 통합
  - FastAPI 서버 구축

0.3.0: Learning Service 추가 (예정)
  - 자동학습 시스템 (ML 대체)
  - 3가지 Rule 추출 알고리즘
  - Few-shot 학습 관리

0.4.0: Workflow Service (예정)
  - n8n 스타일 Visual Builder
  - 워크플로우 CRUD

0.5.0: BI Service (예정)
  - MCP 컴포넌트 조립
  - AI 인사이트 생성

0.6.0: Chat Interface Service (예정)
  - 통합 AI 어시스턴트
  - 멀티턴 대화

0.7.0: Data Visualization Service (예정)
  - 단순 데이터 대시보드
  - 실시간 WebSocket

0.8.0: 나머지 서비스 (예정)
  - API Gateway
  - Action Service
  - Notification Service
  - Logging Service

0.9.0: 베타 릴리스 (예정)
  - 9개 서비스 모두 완성
  - 통합 테스트 완료
```

### Phase 2: 베타 테스트 (3~6개월)

**목표**: 안정화 및 버그 수정

```yaml
0.9.0: 베타 릴리스
  - 9개 마이크로서비스 완성
  - E2E 테스트 통과
  - 베타 테스터 모집

0.9.1, 0.9.2, ...: 베타 버그 수정
  - 사용자 피드백 반영
  - 성능 최적화
  - 보안 강화

1.0.0-rc.1: Release Candidate 1
  - 기능 동결 (Feature Freeze)
  - 최종 검증 단계

1.0.0-rc.2, ...: Release Candidate 패치
  - 치명적 버그만 수정
  - 문서 최종 검토

1.0.0: 정식 릴리스 🎉
  - 프로덕션 배포 준비 완료
  - 전체 문서화 완성
```

### Phase 3: 정식 운영 (1.0.0 이후)

**목표**: CalVer 전환 및 마이크로서비스 독립 버전

```yaml
버전 체계 전환: SemVer → CalVer

프로젝트 전체 (Monorepo):
  - 1.0.0 (마지막 SemVer)
  - 2025.2.0 (첫 CalVer, 2025년 2월)
  - 2025.3.1 (2025년 3월, 첫 패치)

마이크로서비스 독립 버전 (SemVer):
  - Judgment Service: 1.0.0 → 1.1.0 → 2.0.0
  - Learning Service: 1.0.0 → 1.0.1 → 1.1.0
  - BI Service: 1.0.0 → 1.0.2 → 1.1.0
  - (각 서비스 독립적으로 버전 증가)

장점:
  - 프로젝트 전체: 릴리스 시기 명확 (CalVer)
  - 각 서비스: API 호환성 명확 (SemVer)
  - 독립 배포 가능
```

---

## 3. 버전 관리 파일

### 3.1 Single Source of Truth

**파일**: `version.py` (프로젝트 루트)

```python
"""Judgify-core 버전 관리"""

__version__ = "0.1.0"
__stage__ = "alpha"  # alpha → beta → rc → stable
__release_date__ = "2025-10-22"
__description__ = "Desktop App 프로토타입 개발 중"

# 9개 마이크로서비스 구현 상태 추적
MICROSERVICES_STATUS = {
    8000: ("API Gateway", "planned", 0),
    8001: ("Workflow Service", "planned", 0),
    8002: ("Judgment Service", "planned", 0),
    # ... 나머지 서비스
}
```

### 3.2 자동 동기화 파일

| 파일 | 용도 | 동기화 방법 |
|------|------|------------|
| `package.json` | Node.js/Frontend | `scripts/bump_version.py` |
| `src-tauri/Cargo.toml` | Rust/Backend | `scripts/bump_version.py` |
| FastAPI 서비스들 | 마이크로서비스 | `from version import __version__` |

### 3.3 변경 이력

**파일**: `CHANGELOG.md`

```markdown
# Changelog

## [0.1.0] - 2025-10-22

### 추가
- 버전 관리 시스템 도입
- Desktop App 프로토타입

### 변경
- 버전 번호 현실화: 2.0.0 → 0.1.0
```

---

## 4. 사용 가이드

### ⚠️ 필수 규칙: 버전 변경시 bump_version.py 사용!

**문제 사례**: v0.3.1 배포시 `tauri.conf.json`이 0.3.0으로 남아있어서 Tauri 업데이트 체커가 계속 "업데이트 필요" 메시지 표시

#### 잘못된 방법 (금지!)

```bash
❌ package.json 직접 수정
❌ Cargo.toml 직접 수정
❌ tauri.conf.json 직접 수정
❌ version.py 직접 수정
```

**결과**: 파일 간 버전 불일치 → 업데이트 체커 오작동!

#### 올바른 방법 (필수!)

```bash
✅ python scripts/bump_version.py patch|minor|major
✅ 자동으로 4개 파일 동기화:
   - version.py
   - package.json
   - src-tauri/Cargo.toml
   - src-tauri/tauri.conf.json  ← 핵심! Tauri 업데이트 체커가 참조
```

### 4.1 버전 증가 (수동)

```bash
# 1. 기능 추가시 (MINOR 증가)
python scripts/bump_version.py minor

# 출력:
# 🔄 Current version: 0.1.0
# 🎯 New version: 0.2.0
# Bump version 0.1.0 → 0.2.0? (y/N): y
#
# ✅ version.py → 0.2.0
# ✅ package.json → 0.2.0
# ✅ Cargo.toml → 0.2.0
# ✅ tauri.conf.json → 0.2.0  ← 새로 추가됨!

# 2. 버그 수정시 (PATCH 증가)
python scripts/bump_version.py patch

# 3. Git 커밋 (4개 파일 모두 확인!)
git add version.py package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json CHANGELOG.md
git commit -m "chore: Bump version to 0.2.0"

# 4. Git 태그 (주요 마일스톤만)
git tag -a v0.2.0 -m "Release v0.2.0: Judgment Service 첫 구현"
git push origin develop --tags
```

### 4.2 배포 체크리스트 (필수!)

**배포 전 반드시 확인**:
```bash
1. [ ] bump_version.py 실행 (수동 수정 금지!)
2. [ ] 4개 파일 모두 변경되었는지 확인:
       git diff version.py
       git diff package.json
       git diff src-tauri/Cargo.toml
       git diff src-tauri/tauri.conf.json  ← 누락 금지!
3. [ ] 커밋 메시지: "chore: Bump version to X.Y.Z"
4. [ ] 태그 생성: git tag -a vX.Y.Z
5. [ ] 푸시: git push origin develop --tags
```

### 4.3 버전 불일치 방지 규칙

| 파일 | 역할 | Tauri 업데이트 체커 영향 |
|------|------|-------------------------|
| **version.py** | Single Source of Truth | ❌ 간접 영향 없음 |
| **package.json** | npm 패키지 메타데이터 | ❌ 간접 영향 없음 |
| **Cargo.toml** | Rust 크레이트 메타데이터 | ❌ 간접 영향 없음 |
| **tauri.conf.json** | **Tauri 앱 설정** | ✅ **직접 사용됨!** |

**핵심**: `tauri.conf.json`의 `package.version`이 **업데이트 체커의 현재 버전 소스**!

**업데이트 체크 로직**:
```rust
// src-tauri/src/commands/update.rs
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    let current_version = app.package_info().version.to_string();
    // ↑ tauri.conf.json의 package.version 읽기!

    // GitHub Pages latest.json과 비교
    // current_version != latest.json → "업데이트 필요" 메시지
}
```

### 4.2 현재 상태 확인

```bash
# version.py 실행
python version.py

# 출력:
# ╔═══════════════════════════════════════════════════════════╗
# ║  Judgify-core 0.1.0 (alpha)                               ║
# ║  Desktop App 프로토타입 개발 중                            ║
# ╠═══════════════════════════════════════════════════════════╣
# ║  전체 완료율: 45.0%                                       ║
# ╚═══════════════════════════════════════════════════════════╝
#
# 📱 Desktop App:
#   🟡 React + TypeScript: 60%
#   🟡 Tauri + Rust: 60%
#   🟡 SQLite: 70%
#
# 🔧 마이크로서비스 (9개):
#   ⚪ API Gateway (8000): 0%
#   ⚪ Workflow Service (8001): 0%
#   ...
```

### 4.3 CHANGELOG 업데이트

```markdown
# 새 버전 릴리스시 CHANGELOG.md 수동 업데이트

## [0.2.0] - 2025-11-05

### 추가
- Judgment Service 구현
  - 하이브리드 판단 엔진 (Rule + LLM)
  - PostgreSQL + pgvector 통합
  - FastAPI 서버 구축

### 변경
- Rule Engine 신뢰도 임계값: 0.8 → 0.7

### 수정
- LLM 보완 로직 에러 처리 개선
```

---

## 5. 마이크로서비스 독립 버전

### 5.1 Phase 3 (1.0.0 이후) 전용

**각 서비스 디렉토리**:
```
services/
├── judgment_service/
│   ├── __version__.py
│   └── ...
├── learning_service/
│   ├── __version__.py
│   └── ...
```

**예시**: `services/judgment_service/__version__.py`
```python
"""Judgment Service 독립 버전 관리"""

__version__ = "1.0.0"
__api_version__ = "v2"

# 다른 서비스와의 호환성
__compatibility__ = {
    "learning_service": ">=1.0.0",
    "workflow_service": ">=2.0.0"
}
```

### 5.2 버전 호환성 체크

```python
def check_compatibility(service_name: str, version: str) -> bool:
    """다른 서비스와 버전 호환성 체크"""
    required = __compatibility__.get(service_name)
    if not required:
        return True
    return compare_semver(version, required)
```

---

## 6. Git 태그 전략

### 6.1 태그 생성 규칙

```bash
# ✅ 주요 마일스톤만 태그 생성
- 서비스 구현 완료시
- 베타/RC/정식 릴리스시
- 중요한 버그 수정시

# ❌ 매번 커밋마다 태그 금지
- 문서 수정
- 소소한 버그 수정
```

### 6.2 태그 명명 규칙

```bash
# 형식: v<버전>
v0.1.0
v0.2.0
v0.9.0
v1.0.0-rc.1
v1.0.0

# 잘못된 예시:
0.1.0 (v 접두사 없음)
version-0.1.0 (불필요한 단어)
release_0.1.0 (언더스코어 사용)
```

### 6.3 태그 메시지

```bash
# Annotated Tag 사용 (권장)
git tag -a v0.2.0 -m "Release v0.2.0: Judgment Service 첫 구현

주요 변경사항:
- 하이브리드 판단 엔진 구현
- PostgreSQL + pgvector 통합
- FastAPI 서버 구축
"

# Lightweight Tag (비권장)
git tag v0.2.0  # 메시지 없음
```

---

## 7. 자동화 계획

### 7.1 Phase 1 (현재): 수동 관리

```yaml
현재 상태:
  - 버전 증가: 수동 스크립트 (bump_version.py)
  - Git 태그: 수동 생성
  - CHANGELOG: 수동 작성

장점:
  - 단순하고 명확
  - 초기 설정 불필요
  - 실수 방지 (확인 단계 필요)
```

### 7.2 Phase 2 (베타 이후): 반자동화

```yaml
도입 도구:
  - bump-my-version: 버전 증가 자동화
  - conventional-commits: 커밋 메시지 규칙
  - standard-version: CHANGELOG 자동 생성

워크플로우:
  1. 커밋: feat(judgment): Add confidence threshold
  2. 스크립트: bump-my-version bump minor
  3. 자동 생성: CHANGELOG.md 업데이트
  4. Git 태그: 자동 생성 및 푸시
```

### 7.3 Phase 3 (정식 이후): 완전 자동화

```yaml
CI/CD 통합:
  - GitHub Actions 워크플로우
  - PR 라벨 기반 버전 증가
  - 자동 릴리스 노트 생성
  - Docker 이미지 자동 태깅

.github/workflows/version-management.yml:
  - PR 머지시 자동 버전 증가
  - Git 태그 자동 생성
  - GitHub Release 자동 생성
  - 마이크로서비스 독립 배포
```

---

## 📚 참고 자료

### 내부 문서
- [CLAUDE.md 섹션 15](../../CLAUDE.md#-15-ver20-버전-관리-전략): 버전 관리 개요
- [CHANGELOG.md](../../CHANGELOG.md): 변경 이력
- [version.py](../../version.py): 버전 정보
- [scripts/bump_version.py](../../scripts/bump_version.py): 버전 증가 스크립트

### 외부 자료
- [Semantic Versioning 2.0.0](https://semver.org/lang/ko/)
- [Calendar Versioning](https://calver.org/)
- [Conventional Commits](https://www.conventionalcommits.org/ko/)
- [Keep a Changelog](https://keepachangelog.com/ko/)

---

**마지막 업데이트**: 2025-10-22
**작성자**: Claude (AI Assistant)
**승인**: Judgify Team
