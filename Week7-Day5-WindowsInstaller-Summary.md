# Week 7 Day 5: Windows Installer & GitHub Release Automation 완료 ✅

**작업 일자**: 2025-11-11
**소요 시간**: 약 30분
**상태**: 기본 구현 완료

---

## 🎯 완료된 작업

### 1. NSIS Installer 설정 (Tauri Config)
- ✅ 파일: [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json#L46-L52)
- ✅ 설정 추가:
  ```json
  "nsis": {
    "license": "../../LICENSE",
    "installerIcon": "icons/icon.ico",
    "install_mode": "perUser",
    "languages": ["Korean", "English"],
    "displayLanguageSelector": true
  }
  ```
- ✅ WiX Toolset 설정 (ko-KR 언어)
- ✅ 설치 모드: 사용자별 설치 (perUser)
- ✅ 다국어 지원: 한국어, 영어

### 2. GitHub Actions Release Workflow
- ✅ 파일: [.github/workflows/release.yml](.github/workflows/release.yml) (165줄)
- ✅ 3개 Job 구성:
  1. **create-release**: GitHub Release 자동 생성
  2. **build-tauri**: Windows .msi/.exe 빌드 및 업로드
  3. **generate-update-manifest**: latest.json 자동 생성

### 3. LICENSE 파일 생성
- ✅ 파일: [LICENSE](LICENSE) (MIT License)
- ✅ NSIS Installer 라이선스 표시용

---

## 🚀 GitHub Actions Workflow 상세

### Job 1: create-release
```yaml
트리거:
  - Git 태그 푸시 (v*.*.*)
  - 수동 실행 (workflow_dispatch)

역할:
  - GitHub Release 생성
  - 릴리스 노트 자동 작성
  - upload_url 반환 (다음 Job에서 사용)
```

### Job 2: build-tauri
```yaml
플랫폼: windows-latest
타겟: x86_64-pc-windows-msvc

단계:
  1. Node.js 18 설치
  2. Rust Stable 설치
  3. 의존성 설치 (npm ci)
  4. Tauri App 빌드 (tauri-apps/tauri-action@v0)
  5. .msi, .exe, .sig 파일 자동 업로드

환경 변수:
  - TAURI_PRIVATE_KEY (GitHub Secrets)
  - TAURI_KEY_PASSWORD (GitHub Secrets)
  - GITHUB_TOKEN (자동 제공)
```

### Job 3: generate-update-manifest
```yaml
역할:
  - 릴리스된 .msi 및 .sig 파일 확인
  - latest.json 생성 (Auto Update용)
  - latest.json을 Release에 업로드

latest.json 구조:
{
  "version": "2.0.0",
  "notes": "TriFlow AI Desktop Application 업데이트",
  "pub_date": "2025-11-11T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "https://github.com/.../TriFlow_2.0.0_x64.msi.sig",
      "url": "https://github.com/.../TriFlow_2.0.0_x64.msi"
    }
  }
}
```

---

## 📦 빌드 산출물

### Windows Installer
- **파일명**: `TriFlow_2.0.0_x64.msi` (예상)
- **형식**: Windows Installer (MSI)
- **설치 모드**: 사용자별 설치 (관리자 권한 불필요)
- **언어**: 한국어, 영어 선택 가능
- **라이선스**: MIT License 표시

### Windows Portable
- **파일명**: `TriFlow_2.0.0_x64.exe` (예상)
- **형식**: NSIS 실행 파일
- **설치 모드**: 설치 마법사 제공

### 서명 파일
- **파일명**: `TriFlow_2.0.0_x64.msi.sig`
- **역할**: Auto Update 무결성 검증

### Update Manifest
- **파일명**: `latest.json`
- **역할**: Auto Update 엔드포인트
- **URL**: `https://github.com/{owner}/{repo}/releases/latest/download/latest.json`

---

## 🔧 사용 방법

### 1. 릴리스 생성 (자동)
```bash
# 버전 태그 푸시
git tag v2.0.0
git push origin v2.0.0

# GitHub Actions 자동 실행:
# 1. Release 생성
# 2. Windows Installer 빌드
# 3. latest.json 생성
# 4. 모든 파일 업로드
```

### 2. 릴리스 생성 (수동)
```yaml
# GitHub Actions 탭에서 "Release" Workflow 선택
# "Run workflow" 클릭
# Version 입력: v2.0.0
# "Run workflow" 실행
```

### 3. Signing Keys 설정 (최초 1회)
```bash
# 1. Signing Key 생성 (로컬)
npm run tauri signer generate

# 출력 예시:
# Private Key: dW50cnVzdGVkIGNvbW1lbnQ6...
# Public Key: dW50cnVzdGVkIGNvbW1lbnQ6...

# 2. GitHub Secrets 등록
# Settings → Secrets and variables → Actions → New repository secret
# - Name: TAURI_PRIVATE_KEY
#   Value: [Private Key 전체 문자열]
# - Name: TAURI_KEY_PASSWORD
#   Value: [생성시 입력한 패스워드]

# 3. tauri.conf.json 업데이트
# "updater": {
#   "pubkey": "[Public Key 전체 문자열]"
# }
```

---

## ⚠️ 설정 필요 사항

### 1. GitHub Repository 설정 변경 🔥 가장 중요!
```
현재 updater.endpoints:
  "https://github.com/your-org/judgify-desktop/releases/latest/download/latest.json"

변경 필요:
  1. GitHub에서 실제 레포지토리 생성
  2. tauri.conf.json의 updater.endpoints 수정
     예: "https://github.com/mugoori/Judgify-core/releases/latest/download/latest.json"
```

### 2. Signing Keys 생성 (Production 필수!)
```bash
# 현재 상태:
"pubkey": ""  # 빈 문자열 (개발 단계)

# Production 배포 전:
1. npm run tauri signer generate 실행
2. GitHub Secrets에 Private Key + Password 등록
3. tauri.conf.json에 Public Key 추가
```

### 3. GitHub Actions Secrets 등록
```
필수 Secrets:
  - TAURI_PRIVATE_KEY (서명용 Private Key)
  - TAURI_KEY_PASSWORD (Private Key 패스워드)
  - GITHUB_TOKEN (자동 제공, 수동 설정 불필요)
```

---

## 🧪 테스트 방법

### 로컬 빌드 테스트
```bash
# 1. Windows Installer 빌드 (로컬)
npm run tauri build

# 예상 산출물 (src-tauri/target/release/bundle/):
# - msi/TriFlow_2.0.0_x64.msi
# - nsis/TriFlow_2.0.0_x64.exe
# - msi/TriFlow_2.0.0_x64.msi.sig (Signing Key 설정시)

# 2. 설치 테스트
# - .msi 파일 실행
# - 설치 마법사 따라가기
# - 언어 선택 확인 (Korean/English)
# - 라이선스 표시 확인
# - 설치 완료 후 앱 실행
```

### GitHub Actions 테스트
```bash
# 1. 테스트 태그 푸시
git tag v2.0.0-beta.1
git push origin v2.0.0-beta.1

# 2. GitHub Actions 탭에서 진행 확인
# - create-release Job: Release 생성 확인
# - build-tauri Job: 빌드 로그 확인 (약 10-15분 소요)
# - generate-update-manifest Job: latest.json 생성 확인

# 3. Release 페이지 확인
# https://github.com/{owner}/{repo}/releases
# - .msi 파일 다운로드 가능 확인
# - .exe 파일 다운로드 가능 확인
# - .sig 파일 존재 확인
# - latest.json 파일 존재 확인

# 4. Auto Update 테스트
# - 앱 실행 → Settings → "업데이트 확인" 클릭
# - latest.json 파싱 확인
# - 업데이트 가능 메시지 표시 확인
```

---

## 📊 성능 지표

| 항목 | 수치 | 비고 |
|------|------|------|
| **Workflow 파일** | 165줄 | release.yml |
| **새 파일** | 2개 | release.yml, LICENSE |
| **수정 파일** | 1개 | tauri.conf.json (NSIS 설정 추가) |
| **예상 빌드 시간** | 10-15분 | GitHub Actions (windows-latest) |
| **Installer 크기** | ~100MB | .msi 파일 (예상, 압축 포함) |

---

## 🔗 관련 문서

- **Tauri Bundler 공식 문서**: https://tauri.app/v1/guides/building/
- **tauri-action GitHub**: https://github.com/tauri-apps/tauri-action
- **NSIS 공식 문서**: https://nsis.sourceforge.io/
- **Week 7 전체 계획**: [TASKS.md](TASKS.md) - Week 7 섹션
- **개발 계획**: [docs/development/plan.md](docs/development/plan.md) - Week 7

---

## 📝 다음 커밋 메시지 (예시)

```
feat(week7): Implement Windows Installer and GitHub Release Automation (Day 5)

Windows Installer (NSIS) 및 GitHub Actions Release Workflow 구현:

추가된 파일:
- .github/workflows/release.yml (165줄) - GitHub Release 자동화
- LICENSE (MIT License) - NSIS 라이선스 표시

변경된 파일:
- src-tauri/tauri.conf.json - NSIS + WiX 설정 추가

GitHub Actions Workflow (3 Jobs):
- ✅ create-release: GitHub Release 자동 생성
- ✅ build-tauri: Windows .msi/.exe 빌드 및 업로드
- ✅ generate-update-manifest: latest.json 자동 생성

NSIS Installer 설정:
- ✅ 사용자별 설치 (perUser)
- ✅ 다국어 지원 (Korean, English)
- ✅ 라이선스 표시 (MIT)
- ✅ 설치 아이콘 (icon.ico)

산출물:
- TriFlow_2.0.0_x64.msi (Windows Installer)
- TriFlow_2.0.0_x64.exe (NSIS Portable)
- TriFlow_2.0.0_x64.msi.sig (서명 파일)
- latest.json (Auto Update Manifest)

설정 필요:
- ⏸️ GitHub Repository URL 변경 (updater.endpoints)
- ⏸️ Signing Keys 생성 및 등록 (Production 배포시)
- ⏸️ GitHub Secrets 등록 (TAURI_PRIVATE_KEY, TAURI_KEY_PASSWORD)

트리거:
- Git 태그 푸시 (v*.*.*)
- 수동 실행 (workflow_dispatch)

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

---

**Week 7 Day 5 진행률**: 80% (Installer + Workflow 완료! Signing Keys + Repository URL 설정 연기)

**Week 7 전체 진행률**: 85% (System Tray + Auto Update + Windows Installer 완료!)
