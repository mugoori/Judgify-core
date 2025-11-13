# GPG 서명 설정 가이드

**작성일**: 2025-01-21
**대상**: 새 팀원 온보딩 (Phase 2/3)
**소요 시간**: 30분
**목적**: Git 커밋에 GPG 서명 추가로 보안 강화

---

## 📋 목차

1. [GPG란 무엇인가](#1-gpg란-무엇인가)
2. [GPG 키 생성 (Windows)](#2-gpg-키-생성-windows)
3. [GPG 키 생성 (Mac/Linux)](#3-gpg-키-생성-maclinux)
4. [GitHub에 GPG 키 등록](#4-github에-gpg-키-등록)
5. [Git 설정](#5-git-설정)
6. [서명된 커밋 생성 테스트](#6-서명된-커밋-생성-테스트)
7. [문제 해결](#7-문제-해결)
8. [팀 정책](#8-팀-정책)

---

## 1. GPG란 무엇인가?

**GPG (GNU Privacy Guard)**:
- 전자 서명 및 암호화 도구
- Git 커밋에 서명하여 **작성자 신원 보장**
- GitHub "Verified" 배지 표시

**왜 필요한가?**:
- 누구나 `git config user.name`을 임의로 설정 가능
- GPG 서명으로 **실제 커밋 작성자 검증**
- 팀 정책: Phase 2 권장, Phase 3 필수

---

## 2. GPG 키 생성 (Windows)

### 2.1 GPG 설치

**GPG4Win 설치**:
```powershell
# Chocolatey로 설치 (권장)
choco install gpg4win

# 또는 수동 다운로드
# https://www.gpg4win.org/download.html
```

**설치 확인**:
```powershell
gpg --version
# gpg (GnuPG) 2.4.x 출력 확인
```

### 2.2 GPG 키 생성

**대화형 키 생성**:
```powershell
gpg --full-generate-key
```

**설정 옵션**:
```
선택 1: 키 종류
  → (1) RSA and RSA (기본값)

선택 2: 키 크기
  → 4096 (보안 강화)

선택 3: 유효 기간
  → 2y (2년, 권장)

입력 4: 이름
  → 실명 (예: 홍길동)

입력 5: 이메일
  → GitHub 이메일 (예: gildong@example.com)
  ⚠️ 중요: GitHub 계정의 Primary Email과 일치해야 함!

입력 6: Comment (선택)
  → Judgify-core Developer

확인: 정보 확인
  → O (Okay)

입력 7: Passphrase
  → 강력한 비밀번호 입력 (최소 12자)
  → 저장 필수! (1Password, Bitwarden 등 활용)
```

**키 생성 완료**:
```
gpg: key ABCD1234 marked as ultimately trusted
public and secret key created and signed.
```

---

## 3. GPG 키 생성 (Mac/Linux)

### 3.1 GPG 설치

**macOS (Homebrew)**:
```bash
brew install gnupg
```

**Ubuntu/Debian**:
```bash
sudo apt update
sudo apt install gnupg
```

**Fedora/RHEL**:
```bash
sudo dnf install gnupg
```

### 3.2 GPG 키 생성

**Windows와 동일한 대화형 프로세스**:
```bash
gpg --full-generate-key

# 설정 옵션은 Windows와 동일
# RSA 4096, 2년 유효, 실명, GitHub 이메일
```

---

## 4. GitHub에 GPG 키 등록

### 4.1 GPG 키 ID 확인

**키 목록 조회**:
```bash
gpg --list-secret-keys --keyid-format=long

# 출력 예시:
# sec   rsa4096/ABCD1234EFGH5678 2025-01-21 [SC] [expires: 2027-01-21]
#       1234567890ABCDEF1234567890ABCDEF12345678
# uid                 [ultimate] 홍길동 (Judgify-core Developer) <gildong@example.com>
# ssb   rsa4096/5678IJKL9012MNOP 2025-01-21 [E] [expires: 2027-01-21]
```

**키 ID 추출**:
```
rsa4096/ABCD1234EFGH5678
        ^^^^^^^^^^^^^^^^
        이 부분이 KEY_ID
```

### 4.2 공개 키 내보내기

**ASCII 형식으로 내보내기**:
```bash
gpg --armor --export ABCD1234EFGH5678

# 출력 예시:
# -----BEGIN PGP PUBLIC KEY BLOCK-----
#
# mQINBGa...
# ...
# -----END PGP PUBLIC KEY BLOCK-----
```

**클립보드 복사 (선택)**:
```bash
# Windows (Git Bash)
gpg --armor --export ABCD1234EFGH5678 | clip

# macOS
gpg --armor --export ABCD1234EFGH5678 | pbcopy

# Linux
gpg --armor --export ABCD1234EFGH5678 | xclip -selection clipboard
```

### 4.3 GitHub 등록

**단계별 가이드**:
```
1. GitHub 로그인
   → https://github.com

2. Settings 이동
   → 우측 상단 프로필 → Settings

3. SSH and GPG keys 메뉴
   → 좌측 메뉴에서 선택

4. New GPG key 클릭
   → "GPG keys" 섹션에서 클릭

5. Key 붙여넣기
   → -----BEGIN PGP PUBLIC KEY BLOCK----- 전체 복사
   → Title: "Judgify-core GPG Key (Desktop)"

6. Add GPG key 클릭
   → 비밀번호 재입력 (2FA 활성화시)
```

**등록 확인**:
- GitHub 프로필 → Settings → SSH and GPG keys
- "GPG keys" 섹션에 키 표시 확인

---

## 5. Git 설정

### 5.1 전역 설정 (모든 프로젝트)

**GPG 키 ID 설정**:
```bash
git config --global user.signingkey ABCD1234EFGH5678
```

**자동 서명 활성화**:
```bash
git config --global commit.gpgsign true
```

**태그 자동 서명** (선택):
```bash
git config --global tag.gpgsign true
```

**GPG 프로그램 경로 설정** (Windows 필수):
```bash
# GPG4Win 경로 확인
where gpg

# Git에 GPG 경로 설정
git config --global gpg.program "C:/Program Files (x86)/GnuPG/bin/gpg.exe"

# 또는 Git Bash 경로 사용
git config --global gpg.program gpg
```

### 5.2 로컬 설정 (프로젝트별)

**Judgify-core만 서명** (선택):
```bash
cd c:\dev\Judgify-core

# 프로젝트별 설정
git config user.signingkey ABCD1234EFGH5678
git config commit.gpgsign true

# 전역 설정 미적용
git config --global --unset commit.gpgsign
```

---

## 6. 서명된 커밋 생성 테스트

### 6.1 테스트 커밋

**파일 수정 및 커밋**:
```bash
cd c:\dev\Judgify-core

# 테스트 파일 생성
echo "GPG Test" > test-gpg.txt

# 스테이징
git add test-gpg.txt

# 서명된 커밋 (자동 서명 활성화시)
git commit -m "test: GPG 서명 테스트"

# 또는 수동 서명 (자동 서명 미활성화시)
git commit -S -m "test: GPG 서명 테스트"
```

**Passphrase 입력**:
- GPG 키 생성시 설정한 비밀번호 입력
- Windows: GPG4Win 팝업창
- Mac/Linux: 터미널 프롬프트

### 6.2 서명 확인

**로컬 확인**:
```bash
git log --show-signature -1

# 출력 예시:
# commit abc123... (HEAD -> main)
# gpg: Signature made 2025-01-21
# gpg: Good signature from "홍길동 (Judgify-core Developer) <gildong@example.com>"
# Author: 홍길동 <gildong@example.com>
# Date:   2025-01-21
#
#     test: GPG 서명 테스트
```

**GitHub 확인**:
```bash
# GitHub에 푸시
git push origin main

# GitHub 웹에서 커밋 확인
# → "Verified" 배지 표시 확인 ✅
```

### 6.3 테스트 파일 제거

```bash
git rm test-gpg.txt
git commit -m "chore: Remove GPG test file"
git push origin main
```

---

## 7. 문제 해결

### 7.1 "gpg failed to sign the data" 오류

**원인**: GPG 프로그램 경로 미설정

**해결**:
```bash
# GPG 경로 확인
where gpg  # Windows
which gpg  # Mac/Linux

# Git에 경로 설정
git config --global gpg.program "C:/Program Files (x86)/GnuPG/bin/gpg.exe"
```

### 7.2 Passphrase 반복 입력 문제

**원인**: GPG Agent 미실행

**해결 (Windows)**:
```powershell
# GPG Agent 시작
gpg-connect-agent /bye

# 캐시 시간 연장 (1시간)
echo "default-cache-ttl 3600" >> %APPDATA%\gnupg\gpg-agent.conf
echo "max-cache-ttl 86400" >> %APPDATA%\gnupg\gpg-agent.conf

# GPG Agent 재시작
gpg-connect-agent reloadagent /bye
```

**해결 (Mac/Linux)**:
```bash
# ~/.gnupg/gpg-agent.conf 편집
echo "default-cache-ttl 3600" >> ~/.gnupg/gpg-agent.conf
echo "max-cache-ttl 86400" >> ~/.gnupg/gpg-agent.conf

# GPG Agent 재시작
gpgconf --kill gpg-agent
gpg-agent --daemon
```

### 7.3 "No public key" 오류 (GitHub)

**원인**: GitHub에 GPG 키 미등록

**해결**:
1. GPG 공개 키 다시 내보내기: `gpg --armor --export KEYID`
2. GitHub Settings → SSH and GPG keys 재확인
3. 이메일 주소 일치 확인 (git config user.email == GitHub email)

### 7.4 "Email not verified" 오류

**원인**: GitHub 이메일 미인증

**해결**:
```
1. GitHub Settings → Emails
2. Primary email address 확인
3. "Verify email address" 링크 클릭
4. GPG 키 재등록
```

---

## 8. 팀 정책

### 8.1 Phase별 GPG 요구사항

**Phase 1 (1인 개발)**:
- GPG 서명: 선택 사항
- 권장: 개인 습관화 목적

**Phase 2 (2인 팀)**:
- GPG 서명: **권장**
- 온보딩 시 30분 할애
- Passphrase 관리 교육

**Phase 3 (3-5인 팀)**:
- GPG 서명: **필수**
- Branch protection 설정: "Require signed commits"
- 서명 없는 커밋 거부

### 8.2 GPG 키 관리 원칙

**보안 원칙**:
```
1. Passphrase는 팀원별 독립 관리 (공유 금지)
2. 비밀 키 백업 (안전한 위치 보관)
3. 유효 기간 도래 전 갱신 (D-30일 알림)
4. 퇴사 시 GitHub에서 GPG 키 삭제
```

**백업 방법**:
```bash
# 비밀 키 백업 (안전한 USB/암호화 드라이브)
gpg --export-secret-keys -a KEYID > gpg-private-key-backup.asc

# 복원
gpg --import gpg-private-key-backup.asc
```

### 8.3 온보딩 체크리스트

**새 팀원 GPG 설정 (30분)**:
- [ ] GPG 설치 확인 (`gpg --version`)
- [ ] GPG 키 생성 (RSA 4096, 2년 유효)
- [ ] GitHub 이메일과 일치 확인
- [ ] Passphrase 안전 저장 (1Password 등)
- [ ] 공개 키 GitHub 등록
- [ ] Git 전역 설정 (`commit.gpgsign true`)
- [ ] 테스트 커밋 생성
- [ ] GitHub "Verified" 배지 확인
- [ ] Passphrase 캐싱 설정
- [ ] 비밀 키 백업 완료

---

## 📚 참고 자료

**공식 문서**:
- GitHub: [Managing commit signature verification](https://docs.github.com/en/authentication/managing-commit-signature-verification)
- Git: [Signing Your Work](https://git-scm.com/book/en/v2/Git-Tools-Signing-Your-Work)
- GPG: [GnuPG Documentation](https://www.gnupg.org/documentation/)

**Judgify-core 문서**:
- [Branch Protection Strategy](./branch-protection-strategy.md)
- [Git Branch Strategy](../development/git-branch-strategy.md)

**다음 단계**:
- Phase 2 마이그레이션 시: [Branch Protection Strategy - Phase 2](./branch-protection-strategy.md#phase-2-2인-팀-3개월-후)
- 팀 확장 계획: [Hybrid AI Strategy](./hybrid-ai-strategy.md)

---

**작성자**: Claude Code
**최종 업데이트**: 2025-01-21
**버전**: 1.0.0
**대상 Phase**: Phase 2/3
