# Changelog

모든 중요한 변경사항은 이 파일에 기록됩니다.

이 프로젝트는 [Semantic Versioning](https://semver.org/lang/ko/) (초기 개발: 0.x.x)을 따릅니다.

---

## [Unreleased]

### 계획 중
- Judgment Service 구현 (하이브리드 판단 엔진)
- Learning Service 구현 (자동학습 시스템)
- 나머지 7개 마이크로서비스 구현

---

## [0.1.0] - 2025-10-22

### 추가
- **버전 관리 시스템 도입** (version.py + bump_version.py)
  - Single Source of Truth 확립
  - 3개 파일 자동 동기화 (version.py, package.json, Cargo.toml)
  - 프로젝트 진행도 추적 시스템

- **프로젝트 구조 정립**
  - Desktop App 프로토타입 (Tauri + React)
  - 상세 설계 문서 완성 (~5,000줄)
  - 18개 AI 에이전트 협업 프레임워크

- **문서화**
  - CLAUDE.md: AI 개발 가이드 (1,091줄)
  - docs/: 9개 서비스별 상세 설계
  - 버전 관리 전략 (3단계 Phase)

### 변경
- 버전 번호 현실화: 2.0.0 → 0.1.0 (실제 진행도 반영)

### 현재 상태
- **전체 완료율**: ~45%
- **Desktop App**: 60% (Frontend UI + Rust Backend 구조)
- **마이크로서비스**: 0% (설계 완료, 구현 예정)
- **문서화**: 100% (아키텍처, 서비스, 알고리즘)

---

## 버전 기록 형식

### [버전] - YYYY-MM-DD

#### 추가 (Added)
- 새로운 기능

#### 변경 (Changed)
- 기존 기능 변경

#### 제거 (Removed)
- 제거된 기능

#### 수정 (Fixed)
- 버그 수정

#### 보안 (Security)
- 보안 관련 수정

---

## 버전 관리 규칙

### Phase 1: 초기 개발 (0.x.x)
- **0.1.x**: Desktop App 프로토타입
- **0.2.x**: Judgment Service 구현
- **0.3.x**: Learning Service 구현
- **0.4~0.8.x**: 나머지 마이크로서비스
- **0.9.x**: 베타 릴리스

### Phase 2: 베타 테스트
- **0.9.0**: 베타 릴리스 (9개 서비스 완성)
- **1.0.0-rc.1**: Release Candidate 1
- **1.0.0**: 정식 릴리스 🎉

### Phase 3: 정식 운영
- **YYYY.MINOR.PATCH**: CalVer 전환

---

**참고**: 상세한 버전 관리 전략은 [docs/development/versioning-strategy.md](docs/development/versioning-strategy.md) 참조
