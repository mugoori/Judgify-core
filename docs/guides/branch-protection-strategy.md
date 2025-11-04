# GitHub 브랜치 보호 설정: 팀 확장 대응 전략

**생성일**: 2025-11-04
**프로젝트**: Judgify-core Ver2.0 Final
**목적**: 1인 → 2인 → 3-5인 팀 확장 시나리오별 브랜치 보호 전략

---

## 📊 3단계 로드맵 개요

### 전환 타이밍 매트릭스

| 단계 | 팀 규모 | 시기 | PR 승인 | Status Checks | GPG 서명 | 마이그레이션 시간 |
|------|---------|------|---------|---------------|----------|-----------------|
| **Phase 1** | 1명 | 현재 | 0명 | Required | 선택 | - |
| **Phase 2** | 2명 | 3개월 후 | 1명 | Required | 권장 | 2시간 |
| **Phase 3** | 3-5명 | 6개월 후 | 2명 | Required | 필수 | 4시간 |

### 설정 변화 요약

```
Phase 1 (1인) → Phase 2 (2인) → Phase 3 (3-5인)
    ↓                ↓                  ↓
Self-review OK   1명 승인 필수      2명 승인 필수
Status Checks    + CODEOWNERS       + GPG 서명 강제
                 + GPG 권장         + Linear History
                                    + Security Scan
```

---

## Phase 1: 1인 개발 (현재)

### 브랜치 보호 설정 (main)

**GitHub Settings → Branches → Add rule**

```yaml
Branch name pattern: main

✅ Require a pull request before merging
   Require approvals: 0  # Self-merge 허용
   ✅ Dismiss stale pull request approvals

✅ Require status checks to pass before merging
   ✅ Require branches to be up to date
   Status checks:
     - lighthouse-ci
     - rust-criterion-benchmark

❌ Require review from Code Owners (파일 없음)
❌ Require conversation resolution (1인이라 불필요)
❌ Require signed commits (Phase 2 도입)
❌ Require linear history (Phase 3 도입)

✅ Include administrators (자신에게도 규칙 적용)

❌ Allow force pushes (절대 금지!)
❌ Allow deletions (절대 금지!)
```

### 브랜치 전략

```
main          # 안정 버전만 (v0.1.0, v0.2.0)
  ↑
develop       # 개발 기본 브랜치
  ↑
feature/*     # 기능 개발
docs/*        # 문서 수정
fix/*         # 버그 수정
```

---

## Phase 2: 2인 팀 (3개월 후)

### 전환 트리거

다음 조건 중 **하나라도 충족**시 Phase 2 전환:

1. ✅ 팀원 1명 추가 확정 (입사일 D-7일)
2. ✅ 외부 컨트리뷰터 3회 이상 PR
3. ✅ 주요 마일스톤 달성 (v1.0.0 릴리스)

### 마이그레이션 체크리스트

**D-7일 (팀원 입사 1주일 전)**:
- [ ] `.github/CODEOWNERS` 파일 생성 (템플릿 사용)
- [ ] PR 템플릿 업데이트 (Reviewer 가이드 추가)
- [ ] `docs/guides/gpg-setup.md` 작성

**D-1일 (입사 전날)**:
- [ ] main 브랜치 보호: 승인 0 → 1명
- [ ] GitHub Team 생성 (`@judgify-core/developers`)
- [ ] 팀원 계정 초대 (Role: Write)

**D-Day (입사일)**:
- [ ] 팀원 온보딩 (90분)
- [ ] GPG 키 설정 지원 (30분)
- [ ] 첫 PR 함께 진행 (60분)

### 브랜치 보호 설정 변경 (main)

```yaml
# 변경사항만 표시

✅ Require approvals: 1 ← 변경! (0 → 1)
✅ Require review from Code Owners ← 신규!
✅ Require conversation resolution ← 신규!
✅ Require signed commits ← 신규! (권장, 강제 아님)
```

### CODEOWNERS 파일 예시 (2인 팀)

**`.github/CODEOWNERS`**

```bash
# Judgify-core Ver2.0 Final - CODEOWNERS
# Phase 2: 2인 팀

# 기본 소유자
* @mugoori

# Frontend
/src/               @frontend-dev
/UI/                @frontend-dev
package.json        @frontend-dev
vite.config.ts      @frontend-dev

# Backend
/src-tauri/         @mugoori
/services/          @mugoori
Cargo.toml          @mugoori

# 문서 (공동 소유)
/docs/              @mugoori @frontend-dev
CLAUDE.md           @mugoori
README.md           @mugoori @frontend-dev

# CI/CD
/.github/           @mugoori
.lighthouserc.json  @frontend-dev

# 중요 파일 (2명 모두 승인 필요)
version.py          @mugoori @frontend-dev
.env.example        @mugoori @frontend-dev
```

### GPG 서명 설정 (권장)

```bash
# 1. GPG 키 생성
gpg --full-generate-key
# RSA and RSA, 4096 bits, 유효기간 2y

# 2. 키 ID 확인
gpg --list-secret-keys --keyid-format=long

# 3. 공개 키 추출
gpg --armor --export {KEY_ID}

# 4. GitHub에 등록
# Settings → SSH and GPG keys → New GPG key

# 5. Git 설정
git config --global user.signingkey {KEY_ID}
git config --global commit.gpgsign true

# 6. 테스트
git commit -S -m "test: GPG 서명 테스트"
```

---

## Phase 3: 3-5인 팀 (6개월 후)

### 전환 트리거

1. ✅ 팀원 3명 이상 확정
2. ✅ v1.0.0 정식 릴리스 완료
3. ✅ 외부 컨트리뷰터 10회 이상 PR

### 브랜치 보호 설정 변경 (main)

```yaml
# 변경사항만 표시

✅ Require approvals: 2 ← 변경! (1 → 2)
✅ Require signed commits ← 강제! (권장 → 필수)
✅ Require linear history ← 신규! (Rebase only)

Status checks 추가:
  - security-scan ← 신규! (Dependabot)

✅ Restrict who can push ← 신규!
   팀: @judgify-core/maintainers
```

### CODEOWNERS 파일 예시 (3-5인 팀)

**`.github/CODEOWNERS`**

```bash
# Judgify-core Ver2.0 Final - CODEOWNERS
# Phase 3: 3-5인 팀 (마이크로서비스별 전문가)

# 기본 소유자
* @mugoori

# === 마이크로서비스별 소유권 (9개) ===

# Judgment Service (8002) - 하이브리드 판단 엔진
/services/judgment/         @mugoori @ai-engineer-dev

# Learning Service (8009) - 자동학습 시스템
/services/learning/         @mugoori @mlops-engineer-dev

# Workflow Service (8001) - Visual Workflow Builder
/services/workflow/         @mugoori @frontend-dev

# BI Service (8007) - MCP 기반 BI
/services/bi/               @mugoori @data-engineer-dev

# Chat Interface Service (8008) - 통합 AI 어시스턴트
/services/chat/             @mugoori @frontend-dev @ai-engineer-dev

# Data Visualization Service (8006) - 단순 대시보드
/services/data-viz/         @frontend-dev @data-engineer-dev

# Action Service (8003) - 외부 연동
/services/action/           @backend-dev @devops-dev

# Notification Service (8004) - 알림
/services/notification/     @backend-dev

# Logging Service (8005) - 로그 수집
/services/logging/          @devops-dev @backend-dev

# === Frontend ===
/src/                       @frontend-dev
/UI/                        @frontend-dev

# === Backend ===
/src-tauri/                 @mugoori @backend-dev
/rust-backend/              @mugoori @backend-dev

# === 공통 라이브러리 ===
/common/                    @mugoori @backend-dev
/common/base/               @mugoori  # 핵심 아키텍처 (1명만)

# === 문서 ===
/docs/                      @mugoori @technical-writer-dev
CLAUDE.md                   @mugoori  # 핵심 가이드 (1명만)

# === CI/CD ===
/.github/                   @mugoori @devops-dev

# === 보안 민감 파일 (3명 승인) ===
version.py                  @mugoori @backend-dev @devops-dev
.env.example                @mugoori @backend-dev @devops-dev
```

### GitHub Teams 구성

**Settings → Teams**

```yaml
@judgify-core/maintainers:
  members:
    - mugoori (Admin)
    - backend-dev (Maintainer)
  권한: Admin

@judgify-core/developers:
  members:
    - frontend-dev
    - ai-engineer-dev
    - data-engineer-dev
    - mlops-engineer-dev
  권한: Write

@judgify-core/contributors:
  members:
    - devops-dev
    - technical-writer-dev
  권한: Triage
```

### 브랜치 전략 (GitFlow 완전 도입)

```
main            # 프로덕션 (v1.0.0, v1.1.0)
  ↑
release/*       # 릴리스 준비 (release/v1.1.0)
  ↑
develop         # 개발 통합
  ↑
feature/*       # 기능 개발
hotfix/*        # 긴급 수정
```

**release/* 브랜치 보호** (신규):

```yaml
Branch name pattern: release/*

✅ Require approvals: 2
✅ Require review from Code Owners
✅ Require status checks
✅ Require conversation resolution
✅ Require signed commits
✅ Require linear history

❌ Allow force pushes
❌ Allow deletions
```

---

## 마이그레이션 가이드

### Phase 1 → Phase 2 전환 (2시간)

#### Step 1: 사전 준비 (D-7일, 30분)

```bash
cd /c/dev/Judgify-core

# 1. CODEOWNERS 파일 생성
cp .github/CODEOWNERS.phase2.template .github/CODEOWNERS

# 2. PR 템플릿 업데이트 (Reviewer 가이드 추가)
# .github/PULL_REQUEST_TEMPLATE.md 편집

# 3. 커밋
git checkout -b docs/phase2-preparation
git add .github/CODEOWNERS .github/PULL_REQUEST_TEMPLATE.md
git commit -m "docs: Phase 2 전환 준비"
git push origin docs/phase2-preparation
# Self-approve 후 머지
```

#### Step 2: 브랜치 보호 업데이트 (D-1일, 30분)

**GitHub 웹 UI**:

1. Settings → Branches → `main` 편집
2. 변경:
   - Require approvals: `0` → `1`
   - Require review from Code Owners: `OFF` → `ON`
   - Require conversation resolution: `OFF` → `ON`
   - Require signed commits: `OFF` → `ON` (권장)
3. Save changes

#### Step 3: GitHub Team 생성 (D-1일, 15분)

1. Settings → Teams → New team
   - Team name: `developers`
   - Members: @mugoori, @frontend-dev

#### Step 4: 팀원 초대 (D-1일, 15분)

1. Settings → Collaborators → Add people
2. Role: **Write**

#### Step 5: 팀원 온보딩 (D-Day, 90분)

[온보딩 가이드 섹션 참조](#팀원-온보딩-가이드)

### Phase 2 → Phase 3 전환 (4시간)

#### Step 1: CODEOWNERS 대폭 업데이트 (60분)

```bash
# Phase 3 템플릿으로 교체
cp .github/CODEOWNERS.phase3.template .github/CODEOWNERS

git checkout -b docs/phase3-codeowners
git add .github/CODEOWNERS
git commit -S -m "docs: Phase 3 CODEOWNERS (9 microservices)"
git push origin docs/phase3-codeowners
# 1명 승인 받고 머지
```

#### Step 2: 브랜치 보호 강화 (30분)

**main 브랜치**:
- Require approvals: `1` → `2`
- Require signed commits: 권장 → `필수` (강제)
- Require linear history: `OFF` → `ON` (신규!)
- Restrict who can push: `@judgify-core/maintainers` (신규!)
- Status checks 추가: `security-scan`

**release/* 브랜치 보호 추가** (main과 동일)

#### Step 3: GitHub Teams 재구성 (45분)

1. `maintainers` 팀 생성 (@mugoori, @backend-dev)
2. `developers` 팀 확장 (신규 멤버 추가)
3. `contributors` 팀 생성 (@devops-dev, @technical-writer-dev)

#### Step 4: 팀 교육 (90분)

**주제**:
- Linear History 정책 (Rebase 사용법)
- GPG 서명 필수화
- 2명 승인 정책
- CODEOWNERS 자동 할당

**교육 자료**:
- `docs/guides/git-rebase.md` (신규 작성)
- `docs/guides/code-review-guide.md` (신규 작성)

---

## 팀원 온보딩 가이드

### 온보딩 체크리스트 (D-Day, 90분)

#### 08:00 - 08:30 (30분): Git 설정

```bash
# 1. 리포지토리 클론
git clone git@github.com:mugoori/Judgify-core.git
cd Judgify-core

# 2. Git 사용자 정보 설정
git config user.name "Frontend Dev"
git config user.email "frontend@example.com"

# 3. GPG 서명 설정
gpg --full-generate-key
# (GPG 가이드 참조: docs/guides/gpg-setup.md)

# 4. SSH 키 등록 (이미 있으면 생략)
ssh-keygen -t ed25519 -C "frontend@example.com"
# GitHub Settings → SSH keys에 등록
```

#### 08:30 - 09:00 (30분): 프로젝트 이해

- [ ] README.md 읽기
- [ ] CLAUDE.md 핵심 섹션 읽기 (섹션 0, 1, 12)
- [ ] 담당 서비스 확인 (CODEOWNERS)

#### 09:00 - 09:30 (30분): 첫 PR 연습

```bash
# 1. 브랜치 생성
git checkout develop
git pull origin develop
git checkout -b docs/onboarding-test-{이름}

# 2. 간단한 수정
vim README.md
# "## 팀" 섹션에 자신 추가

# 3. 커밋 (GPG 서명 - Phase 2는 권장)
git add README.md
git commit -S -m "docs: Add team member (onboarding test)"

# 4. 푸시 및 PR 생성
git push origin docs/onboarding-test-{이름}
# GitHub에서 PR 생성 (Reviewer 자동 할당)
```

#### 09:30 - 10:00 (30분): 리뷰 프로세스 이해

1. mugoori가 리뷰 및 승인
2. Frontend Dev가 머지
3. 로컬 동기화

### 온보딩 1일차 종료 체크리스트

- [ ] Git 설정 완료 (user.name, user.email, GPG)
- [ ] GitHub 권한 확인 (Write)
- [ ] CODEOWNERS 이해
- [ ] PR 생성 및 머지 경험
- [ ] 담당 마이크로서비스 파악
- [ ] 팀 커뮤니케이션 채널 가입

---

## 부록: 무중단 마이그레이션 전략

### 원칙

1. **브랜치 보호는 마지막에 변경** (코드 변경 먼저)
2. **점진적 적용** (한 번에 모든 규칙 활성화 금지)
3. **롤백 계획 필수**

### 롤백 계획

**브랜치 보호 롤백** (5분):

1. Settings → Branches → `main` 편집
2. Require approvals: `1` → `0` (Phase 1 복귀)
3. Require review from Code Owners: `ON` → `OFF`
4. Save changes

**CODEOWNERS 롤백** (1분):

```bash
git checkout main
git revert {CODEOWNERS 추가 커밋 SHA}
git push origin main
```

---

## 요약: 단계별 핵심 변경사항

### Phase 1 (1인, 현재)

```yaml
승인: 0명
Status Checks: Required (Lighthouse + Criterion)
CODEOWNERS: 없음
GPG 서명: 선택
Linear History: 선택
```

### Phase 2 (2인, 3개월 후)

```yaml
승인: 1명 필수
Status Checks: Required
CODEOWNERS: ✅ 추가 (Frontend/Backend 분리)
GPG 서명: 권장
Linear History: 선택
팀: @judgify-core/developers (2명)
```

### Phase 3 (3-5인, 6개월 후)

```yaml
승인: 2명 필수
Status Checks: Required + Security Scan
CODEOWNERS: ✅ 9개 마이크로서비스별 세분화
GPG 서명: 필수 (강제)
Linear History: ✅ 필수 (Rebase only)
팀:
  - maintainers (2명)
  - developers (4명)
  - contributors (2명)
```

---

## 다음 단계 (Action Items)

### 즉시 (Phase 1 유지)

- [ ] 이 문서 참조하여 Phase 1 설정 (GitHub UI)
- [ ] PR 템플릿 및 CODEOWNERS 템플릿 확인

### 팀원 추가 D-7일

- [ ] `.github/CODEOWNERS.phase2.template` → `.github/CODEOWNERS` 복사
- [ ] PR 템플릿 업데이트
- [ ] `docs/guides/gpg-setup.md` 공유

### 팀원 추가 D-1일

- [ ] 브랜치 보호 규칙 업데이트
- [ ] GitHub Team 생성
- [ ] 팀원 계정 초대

### 팀원 추가 D-Day

- [ ] 온보딩 진행 (90분)
- [ ] 첫 PR 연습
- [ ] 리뷰 프로세스 교육

---

**문서 작성일**: 2025-11-04
**최종 업데이트**: 2025-11-04
**담당**: Performance Engineer 서브에이전트
