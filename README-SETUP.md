# Judgify-core Ver2.0 Final - 개발 환경 설정 가이드

이 문서는 Judgify Desktop Application을 개발 및 실행하기 위한 **완전한 단계별 가이드**입니다.

---

## 📋 목차

1. [필수 요구사항](#필수-요구사항)
2. [Windows 개발 환경 설정](#windows-개발-환경-설정)
3. [프로젝트 설정](#프로젝트-설정)
4. [개발 서버 실행](#개발-서버-실행)
5. [빌드 및 배포](#빌드-및-배포)
6. [문제 해결](#문제-해결)

---

## 📦 필수 요구사항

### 1. Node.js 20+ (LTS)
```powershell
# winget 사용 (Windows 10/11)
winget install OpenJS.NodeJS.LTS

# 설치 확인
node --version  # v20.x.x 이상이어야 함
npm --version   # v10.x.x 이상이어야 함
```

### 2. pnpm (Node 패키지 관리자)
```powershell
npm install -g pnpm

# 설치 확인
pnpm --version  # v8.x.x 이상이어야 함
```

### 3. Rust (1.75+)
```powershell
# Rustup 설치 (공식 방법)
# https://rustup.rs/ 에서 rustup-init.exe 다운로드 및 실행

# 또는 winget 사용
winget install Rustlang.Rustup

# 설치 확인
rustc --version  # rustc 1.75.0 이상
cargo --version  # cargo 1.75.0 이상
```

### 4. Visual Studio Build Tools (Rust 컴파일용)

**방법 1: Visual Studio Installer 사용 (권장)**
1. [Visual Studio 다운로드](https://visualstudio.microsoft.com/downloads/)에서 **Community 버전** 설치
2. **"Desktop development with C++"** 워크로드 선택
3. 설치 (약 6GB, 30분 소요)

**방법 2: Build Tools만 설치 (가벼움)**
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools

# 설치 후 "C++ build tools" 선택
```

### 5. OpenAI API Key (필수!)
```powershell
# OpenAI 계정 생성 및 API Key 발급
# https://platform.openai.com/api-keys

# API Key 형식: sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

---

## 🔧 Windows 개발 환경 설정

### 1. 전체 자동 설치 스크립트 (PowerShell 관리자 권한 실행)

```powershell
# PowerShell 관리자로 실행
Set-ExecutionPolicy Bypass -Scope Process -Force

# Node.js 설치
winget install OpenJS.NodeJS.LTS

# Rust 설치
winget install Rustlang.Rustup

# Visual Studio Build Tools 설치
winget install Microsoft.VisualStudio.2022.BuildTools

# pnpm 설치
npm install -g pnpm

Write-Host "설치 완료! 터미널을 재시작하세요." -ForegroundColor Green
```

### 2. 환경 변수 설정 확인

```powershell
# PATH 확인
$env:PATH

# Rust 경로 확인 (다음이 포함되어야 함)
# - C:\Users\YourName\.cargo\bin
# - C:\Users\YourName\.rustup\toolchains\...

# Node.js 경로 확인
# - C:\Program Files\nodejs\
```

---

## 🚀 프로젝트 설정

### 1. 프로젝트 클론 및 이동

```powershell
git clone https://github.com/your-org/Judgify-core.git
cd Judgify-core
```

### 2. 환경 변수 파일 설정

```powershell
# .env 파일 생성
Copy-Item .env.example .env

# .env 파일 편집 (notepad 또는 VSCode)
notepad .env
```

**`.env` 필수 설정 항목**:

```env
# OpenAI API Key (필수!)
OPENAI_API_KEY=sk-your-actual-api-key-here

# 데이터베이스 (자동 생성됨, 수정 불필요)
DATABASE_URL=sqlite:///AppData/Roaming/Judgify/judgify.db

# Redis (선택, 고급 기능)
REDIS_URL=redis://localhost:6379
```

### 3. Node 의존성 설치

```powershell
# 프로젝트 루트에서 실행
pnpm install

# 설치 시간: 약 2-5분
# 설치되는 패키지: React, Vite, shadcn/ui, Tauri API 등
```

### 4. Rust 의존성 다운로드

```powershell
cd src-tauri
cargo fetch

# Rust 의존성 다운로드 (약 5-10분 소요)
cd ..
```

---

## 🎯 개발 서버 실행

### 1. 개발 모드 실행 (핫 리로드 지원)

```powershell
# 프로젝트 루트에서 실행
pnpm tauri dev
```

**실행 과정**:
1. Frontend 번들링 (Vite) - 약 10초
2. Rust 백엔드 컴파일 - **최초 실행시 약 5-10분** (이후는 빠름)
3. 데스크톱 앱 창 띄우기

**예상 출력**:
```
vite v5.0.11 building for development...
✓ built in 1.5s

Running BeforeDevCommand (`pnpm dev`)...
    Compiling judgify-desktop v2.0.0
    Finished dev [unoptimized + debuginfo] target(s) in 3m 12s

[Tauri] Running on http://localhost:1420/
```

### 2. 앱 화면 구성

앱이 실행되면 다음과 같은 화면이 나타납니다:

```
┌─────────────────────────────────────────┐
│  Judgify Desktop                        │
├──────────┬──────────────────────────────┤
│ Sidebar  │  Main Content                │
│          │                              │
│ • Chat   │  ← Chat Interface            │
│ • Dashboard                             │
│ • Workflow                              │
│ • BI     │                              │
│ • Settings                              │
│          │                              │
└──────────┴──────────────────────────────┘
```

### 3. 첫 기능 테스트

#### Test 1: Chat Interface
1. 앱 좌측 **"Chat"** 클릭
2. 메시지 입력: **"안녕하세요!"**
3. 전송 버튼 클릭
4. AI 응답 확인

#### Test 2: Judgment 실행 (간단한 예제)
1. **"Workflow"** 페이지로 이동
2. "New Workflow" 버튼 클릭
3. 워크플로우 이름: **"온도 체크"**
4. Rule 표현식 입력: `temperature > 85`
5. 저장 후 실행 테스트

#### Test 3: Dashboard 확인
1. **"Dashboard"** 페이지로 이동
2. 실시간 데이터 차트 확인
3. KPI 카드 표시 확인

---

## 📦 빌드 및 배포

### 1. 프로덕션 빌드

```powershell
# Windows 실행 파일 (.exe) 생성
pnpm tauri build --target x86_64-pc-windows-msvc
```

**빌드 결과**:
```
src-tauri/target/release/bundle/msi/
  ├── judgify-desktop_2.0.0_x64.msi  (Windows Installer)
  └── judgify-desktop_2.0.0_x64_en-US.msi.zip

src-tauri/target/release/
  └── judgify-desktop.exe  (Portable 실행 파일)
```

### 2. 실행 파일 크기

- **MSI Installer**: 약 25-30MB
- **Portable EXE**: 약 20-25MB
- **설치 후 크기**: 약 50-70MB

### 3. 배포 방법

#### 방법 1: MSI Installer 배포 (권장)
```powershell
# 1. MSI 파일을 사용자에게 전달
# 2. 사용자가 더블클릭하여 설치
# 3. 시작 메뉴에 "Judgify Desktop" 추가됨
```

#### 방법 2: Portable EXE 배포
```powershell
# 1. judgify-desktop.exe 파일만 전달
# 2. 사용자가 원하는 폴더에 저장
# 3. 더블클릭하여 바로 실행 (설치 불필요)
```

### 4. Auto Update 설정 (선택)

**GitHub Releases 사용**:
```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build
        run: pnpm tauri build
      - name: Upload Release
        uses: softprops/action-gh-release@v1
        with:
          files: src-tauri/target/release/bundle/msi/*.msi
```

---

## 🐛 문제 해결

### Issue 1: `cargo: command not found`

**원인**: Rust가 설치되지 않았거나 PATH에 없음

**해결**:
```powershell
# Rust 재설치
winget install Rustlang.Rustup

# 터미널 재시작
# PowerShell을 닫고 다시 열기

# 확인
cargo --version
```

### Issue 2: `error: linker 'link.exe' not found`

**원인**: Visual Studio Build Tools가 설치되지 않음

**해결**:
```powershell
# Visual Studio Build Tools 설치
winget install Microsoft.VisualStudio.2022.BuildTools

# 설치 후 "Desktop development with C++" 선택
```

### Issue 3: `OpenAI API error: 401 Unauthorized`

**원인**: `.env` 파일에 유효한 API Key가 없음

**해결**:
```powershell
# .env 파일 확인
notepad .env

# OPENAI_API_KEY=sk-... 가 올바른지 확인
# OpenAI 계정에서 새 API Key 발급: https://platform.openai.com/api-keys
```

### Issue 4: `pnpm install` 실패 (EACCES 오류)

**원인**: 권한 문제

**해결**:
```powershell
# PowerShell 관리자 권한으로 실행
# 또는 npm 캐시 삭제
pnpm store prune
pnpm install --force
```

### Issue 5: `tauri dev` 실행시 컴파일 오류

**원인**: Rust 의존성 버전 충돌

**해결**:
```powershell
cd src-tauri
cargo clean
cargo build
cd ..
pnpm tauri dev
```

### Issue 6: 앱 실행시 빈 화면 (White Screen)

**원인**: Frontend 빌드 오류

**해결**:
```powershell
# Vite dev server가 제대로 실행되는지 확인
pnpm dev

# 브라우저에서 http://localhost:5173 접속 테스트
# 문제 없으면 Ctrl+C로 중지 후
pnpm tauri dev
```

---

## 🔍 개발 도구 및 디버깅

### 1. Chrome DevTools 열기

앱 실행 중 **F12** 또는 **Ctrl+Shift+I** 키를 눌러 DevTools를 엽니다.

```javascript
// Console에서 Tauri API 테스트
import { invoke } from '@tauri-apps/api/tauri';

// 간단한 테스트
await invoke('get_system_status');
```

### 2. Rust 로그 확인

```powershell
# 개발 모드 실행시 Rust 로그 출력
pnpm tauri dev

# 로그 레벨 설정
$env:RUST_LOG="debug"
pnpm tauri dev
```

### 3. SQLite 데이터베이스 확인

```powershell
# 데이터베이스 위치
$db_path = "$env:APPDATA\Judgify\judgify.db"

# SQLite Browser 사용 (DB Browser for SQLite)
winget install DB.Browser.SQLite

# DB 파일 열기
& "DB Browser for SQLite" $db_path
```

---

## 📚 추가 리소스

### 공식 문서
- **Tauri 공식 문서**: https://tauri.app/
- **React 문서**: https://react.dev/
- **shadcn/ui 컴포넌트**: https://ui.shadcn.com/
- **Rust 문서**: https://doc.rust-lang.org/

### 프로젝트 문서
- `CLAUDE.md`: Claude 개발 가이드
- `initial.md`: Ver2.0 Final 요구사항
- `system-structure.md`: 시스템 아키텍처
- `prompt-guide.md`: LLM Prompt 설계 전략
- `docs/`: 상세 설계 문서

### 커뮤니티 지원
- **GitHub Issues**: https://github.com/your-org/Judgify-core/issues
- **Discord**: [링크 추가 필요]

---

## ✅ 설치 검증 체크리스트

완료하면 ☑로 변경하세요:

- [ ] Node.js 20+ 설치 완료 (`node --version`)
- [ ] pnpm 설치 완료 (`pnpm --version`)
- [ ] Rust 1.75+ 설치 완료 (`rustc --version`)
- [ ] Visual Studio Build Tools 설치 완료
- [ ] OpenAI API Key 발급 및 `.env` 설정
- [ ] `pnpm install` 성공
- [ ] `pnpm tauri dev` 실행 성공
- [ ] 앱 화면이 정상적으로 표시됨
- [ ] Chat Interface 테스트 통과
- [ ] Workflow 생성 및 실행 테스트 통과

모든 항목이 완료되면 **개발 준비 완료**입니다! 🎉

---

## 🚀 다음 단계

개발 환경 설정이 완료되었다면 다음을 진행하세요:

1. **`docs/development-plan.md`** 읽기 - 8주 개발 일정 확인
2. **Phase 1 Week 2** 시작 - Judgment Engine 핵심 로직 구현
3. **Learning Service** 개발 - 자동학습 시스템 구현
4. **BI Service** 개발 - MCP 기반 컴포넌트 조립
5. **Visual Workflow Builder** 개발 - n8n 스타일 에디터

Happy Coding! 🤖⚡
