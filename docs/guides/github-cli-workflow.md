# GitHub CLI 자동화 워크플로우 가이드

**작성일**: 2025-11-04
**대상**: Phase 1 개발자 (1인 개발)
**소요 시간**: 초기 설정 15분, 이후 매 PR 30초
**목적**: PR 생성 + 자동 머지 프로세스를 3단계로 단축

---

## 📋 목차

1. [개요](#1-개요)
2. [GitHub CLI 설치](#2-github-cli-설치)
3. [인증 설정](#3-인증-설정)
4. [일일 워크플로우](#4-일일-워크플로우-3단계)
5. [자동화 스크립트 사용법](#5-자동화-스크립트-사용법)
6. [고급 사용법](#6-고급-사용법)
7. [문제 해결](#7-문제-해결)

---

## 1. 개요

### 🎯 Before vs After

| 방식 | 단계 | 소요 시간 | 수동 작업 |
|------|------|----------|----------|
| **수동 (웹)** | 7단계 | 5분 | PR 생성, 머지 클릭, 브랜치 삭제 |
| **GitHub CLI** | 3단계 | 30초 | 커밋, 푸시, 스크립트 실행 |

### ✨ 핵심 효과

- ⏱️ **시간 절감**: 5분 → 30초 (90% 단축)
- 🤖 **자동화**: CI 통과 후 자동 머지
- 📊 **이력 관리**: PR 단위 추적 유지
- 🚀 **생산성**: 하루 10개 PR 시 45분 절감

---

## 2. GitHub CLI 설치

### 🪟 Windows

#### 방법 1: 공식 설치 파일 (권장)

```
1. https://cli.github.com/ 접속

2. "Download for Windows" 클릭

3. gh_X.XX.X_windows_amd64.msi 다운로드

4. 설치 파일 실행

5. 설치 완료 후 터미널 재시작
```

#### 방법 2: Chocolatey (설치되어 있는 경우)

```powershell
choco install gh -y
```

#### 방법 3: Scoop (설치되어 있는 경우)

```powershell
scoop install gh
```

### 🍎 macOS

```bash
brew install gh
```

### 🐧 Linux

**Ubuntu/Debian**:
```bash
sudo apt install gh
```

**Fedora/RHEL**:
```bash
sudo dnf install gh
```

### ✅ 설치 확인

```bash
gh --version
# 출력 예시: gh version 2.40.1 (2024-01-10)
```

---

## 3. 인증 설정

### 🔐 GitHub 로그인 (최초 1회)

```bash
gh auth login
```

### 📋 대화형 설정

```
? What account do you want to log into?
  → GitHub.com

? What is your preferred protocol for Git operations?
  → HTTPS

? Authenticate Git with your GitHub credentials?
  → Yes

? How would you like to authenticate GitHub CLI?
  → Login with a web browser  (권장)
  또는
  → Paste an authentication token
```

### 🌐 브라우저 인증 (권장)

```
1. 터미널에 표시된 One-time code 복사
   예: 1234-5678

2. Enter를 누르면 브라우저 자동 열림

3. GitHub 로그인 (이미 로그인된 경우 생략)

4. One-time code 입력

5. "Authorize github" 클릭

6. 터미널에 "✓ Authentication complete" 표시
```

### 🔑 토큰 인증 (고급)

Personal Access Token 생성:
```
1. GitHub → Settings → Developer settings

2. Personal access tokens → Tokens (classic)

3. "Generate new token"

4. 권한 선택:
   ✅ repo (모든 항목)
   ✅ workflow
   ✅ admin:org (read:org)

5. 토큰 복사 (한 번만 표시됨!)

6. gh auth login → Paste an authentication token
```

### ✅ 인증 확인

```bash
gh auth status

# 출력 예시:
# github.com
#   ✓ Logged in to github.com as mugoori
#   ✓ Git operations for github.com configured to use https protocol.
#   ✓ Token: ghp_************************************
```

---

## 4. 일일 워크플로우 (3단계)

### 🚀 간소화된 프로세스

#### 1️⃣ 브랜치 생성 및 작업

```bash
# 새 기능 브랜치 생성
git checkout -b feature/my-new-feature

# 작업 완료 후 커밋
git add .
git commit -m "feat: Add my new feature"
```

#### 2️⃣ GitHub에 푸시

```bash
git push origin feature/my-new-feature
```

#### 3️⃣ 자동 PR + 머지

**Git Bash/Linux/Mac**:
```bash
./scripts/pr-auto-merge.sh "feat: Add my new feature"
```

**Windows PowerShell**:
```powershell
.\scripts\pr-auto-merge.ps1 -Title "feat: Add my new feature"
```

**결과**:
```
🚀 PR 생성 중...
   브랜치: feature/my-new-feature → main
   제목: feat: Add my new feature

✅ PR #5 생성 완료!
🔗 URL: https://github.com/mugoori/Judgify-core/pull/5

🔄 다음 단계:
   1. CI 실행 중 (Lighthouse + Criterion)
   2. CI 통과 시 자동 머지
   3. 브랜치 자동 삭제

💡 진행 상황 확인: gh pr view 5
```

### ⏱️ 전체 소요 시간: 30초!

---

## 5. 자동화 스크립트 사용법

### 📜 pr-auto-merge.sh (Git Bash/Linux/Mac)

**위치**: `scripts/pr-auto-merge.sh`

**기본 사용**:
```bash
./scripts/pr-auto-merge.sh "PR 제목"
```

**예시**:
```bash
# 새 기능
./scripts/pr-auto-merge.sh "feat: Add chat interface"

# 버그 수정
./scripts/pr-auto-merge.sh "fix: Fix memory leak in WebSocket"

# 문서 업데이트
./scripts/pr-auto-merge.sh "docs: Update API documentation"

# 성능 개선
./scripts/pr-auto-merge.sh "perf: Optimize database queries"
```

### 📜 pr-auto-merge.ps1 (Windows PowerShell)

**위치**: `scripts\pr-auto-merge.ps1`

**기본 사용**:
```powershell
.\scripts\pr-auto-merge.ps1 -Title "PR 제목"
```

**예시**:
```powershell
# 새 기능
.\scripts\pr-auto-merge.ps1 -Title "feat: Add chat interface"

# 버그 수정
.\scripts\pr-auto-merge.ps1 -Title "fix: Fix memory leak in WebSocket"
```

### 🔍 스크립트가 하는 일

1. **브랜치 확인**: main/develop 브랜치에서 실행 방지
2. **GitHub CLI 확인**: gh 설치 여부 검증
3. **인증 확인**: GitHub 로그인 상태 검증
4. **PR 생성**: 제목 + 기본 Body 템플릿
5. **자동 머지 설정**: CI 통과 후 자동 머지 활성화
6. **브랜치 삭제 예약**: 머지 후 자동 삭제

---

## 6. 고급 사용법

### 🔄 PR 상태 확인

```bash
# 현재 브랜치의 PR 확인
gh pr view

# 특정 PR 확인
gh pr view 5

# PR 목록 보기
gh pr list

# PR 상태만 간단히
gh pr status
```

### ✏️ PR 수정

```bash
# PR 제목 변경
gh pr edit 5 --title "feat: Updated feature title"

# PR 본문 변경
gh pr edit 5 --body "New description"

# PR에 라벨 추가
gh pr edit 5 --add-label "enhancement"
```

### 🚫 자동 머지 취소

```bash
# 자동 머지 비활성화
gh pr merge 5 --disable-auto

# 수동 머지
gh pr merge 5 --squash --delete-branch
```

### 🔍 CI 로그 확인

```bash
# PR의 CI 상태 확인
gh pr checks 5

# 실시간 CI 로그 보기 (watch 모드)
gh pr checks 5 --watch
```

### 📊 PR 리뷰

```bash
# PR 승인
gh pr review 5 --approve

# PR 코멘트
gh pr review 5 --comment --body "LGTM!"

# 변경 요청
gh pr review 5 --request-changes --body "Please fix..."
```

---

## 7. 문제 해결

### ❌ "gh: command not found"

**원인**: GitHub CLI 미설치

**해결**:
```bash
# Windows: https://cli.github.com/ 에서 설치
# macOS: brew install gh
# Linux: sudo apt install gh

# 설치 후 터미널 재시작
```

### ❌ "not logged in to any hosts"

**원인**: GitHub 인증 안 됨

**해결**:
```bash
gh auth login
# → 브라우저 인증 선택
```

### ❌ "pull request create failed"

**원인**: PR이 이미 존재하거나 충돌

**해결**:
```bash
# 기존 PR 확인
gh pr list

# 기존 PR 있으면 재사용
gh pr view

# 또는 기존 PR 닫고 새로 생성
gh pr close <PR 번호>
```

### ❌ "auto-merge is not allowed"

**원인**: Private 레포지토리 + Personal 계정

**해결 1**: Public 레포로 전환 (Settings → Change visibility)

**해결 2**: GitHub Team 업그레이드 ($4/월)

**해결 3**: 수동 머지 사용
```bash
# 스크립트 대신 직접 PR 생성
gh pr create --title "..." --body "..."

# CI 통과 후 수동 머지
gh pr merge <PR 번호> --squash --delete-branch
```

### ❌ "permission denied: scripts/pr-auto-merge.sh"

**원인**: 스크립트 실행 권한 없음

**해결**:
```bash
chmod +x scripts/pr-auto-merge.sh
```

### ❌ CI 통과했는데 자동 머지 안 됨

**원인 1**: Private 레포 + Personal 계정 (auto-merge 미지원)

**해결**: 수동 머지 또는 Public 전환

**원인 2**: Branch Protection에서 Status Checks 미설정

**해결**:
```
1. GitHub → Settings → Branches
2. main 브랜치 규칙 편집
3. "Require status checks to pass before merging" 체크
4. lighthouse, benchmark 체크
```

---

## 🎯 Quick Reference

### 📝 Cheat Sheet

```bash
# === 일일 워크플로우 ===
git checkout -b feature/my-feature
# ... 작업 ...
git commit -m "feat: My feature"
git push origin feature/my-feature
./scripts/pr-auto-merge.sh "feat: My feature"

# === PR 관리 ===
gh pr view          # 현재 PR 보기
gh pr list          # PR 목록
gh pr checks        # CI 상태
gh pr merge --auto  # 자동 머지 설정

# === 인증 ===
gh auth login       # 로그인
gh auth status      # 상태 확인
gh auth logout      # 로그아웃

# === 고급 ===
gh pr edit 5 --title "New title"
gh pr review 5 --approve
gh pr close 5
```

### 🔗 관련 문서

- **GitHub CLI 공식 문서**: https://cli.github.com/manual/
- **Branch Protection 전략**: [./branch-protection-strategy.md](./branch-protection-strategy.md)
- **자기 규율 워크플로우**: [./self-discipline-workflow.md](./self-discipline-workflow.md) (생성 예정)
- **CLAUDE.md**: [../../CLAUDE.md](../../CLAUDE.md)

---

**작성자**: Claude Code
**최종 업데이트**: 2025-11-04
**버전**: 1.0.0
**대상 Phase**: Phase 1
