# 🚀 Judgify-core 실행하기 - 최종 안내

**자동 실행 스크립트가 준비되었습니다!**

---

## ⚡ 방법 1: 자동 스크립트 사용 (권장)

### 1단계: PowerShell 관리자 권한으로 열기

1. **시작 메뉴**에서 "PowerShell" 검색
2. **"Windows PowerShell"** 우클릭
3. **"관리자 권한으로 실행"** 선택

### 2단계: 프로젝트 폴더로 이동

```powershell
cd "c:\Users\dilel\Downloads\Judgify-core (2)\Judgify-core"
```

### 3단계: 실행 정책 설정 (최초 1회)

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 4단계: 자동 스크립트 실행

```powershell
.\EXECUTE-NOW.ps1
```

**스크립트가 자동으로 진행합니다**:
- ✅ 환경 확인
- ✅ Rust 설치 안내 (미설치 시)
- ✅ .env 파일 설정 안내
- ✅ 개발 서버 자동 실행

---

## 🛠️ 방법 2: 수동 설치 (단계별)

스크립트가 작동하지 않으면 다음을 수동으로 진행하세요.

### 1단계: Rust 설치

**PowerShell 관리자 권한**으로 실행:

```powershell
# winget으로 설치
winget install Rustlang.Rustup
```

**설치 완료 후**:
1. PowerShell을 **완전히 종료**
2. 새 PowerShell 창 열기
3. 확인: `cargo --version`

### 2단계: Visual Studio Build Tools 설치

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

**설치 시 주의**:
- **"Desktop development with C++"** 워크로드 선택 필수!
- 설치 완료 후 **컴퓨터 재부팅 권장**

### 3단계: pnpm 설치 (선택)

```powershell
npm install -g pnpm
```

### 4단계: .env 파일 설정

```powershell
# .env 파일 열기
notepad .env
```

**수정할 부분**:
```env
# 이 줄을 찾아서
OPENAI_API_KEY=sk-test-key-replace-with-actual-key

# 실제 API Key로 변경
OPENAI_API_KEY=sk-proj-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**OpenAI API Key 발급**:
1. https://platform.openai.com/api-keys 방문
2. 로그인 (계정 없으면 가입)
3. "Create new secret key" 클릭
4. 생성된 키 복사

### 5단계: 실행

```powershell
# pnpm 사용 (권장)
pnpm tauri dev

# 또는 npm 사용
npm run tauri:dev
```

---

## ⏱️ 예상 실행 시간

```
최초 실행 (전체):
├─ Frontend 빌드:      10초
├─ Rust 의존성 다운:   2-3분
├─ Rust 컴파일:        5-7분
└─ 앱 실행:            자동

총 소요 시간:          8-10분

이후 실행:
└─ 증분 컴파일:        30초-1분
```

**화면 출력 예시** (정상):
```
vite v5.0.11 building for development...
✓ built in 1.5s

    Downloading crates ...
    Compiling serde v1.0.195
    Compiling tokio v1.35.0
    ... (많은 패키지)
    Compiling judgify-desktop v2.0.0
    Finished dev [unoptimized + debuginfo] target(s) in 8m 45s

[Tauri] Running on http://localhost:1420/
```

---

## ✅ 성공 확인

### 앱 창이 열림

```
┌─────────────────────────────────────────┐
│  Judgify AI Platform          🔍 ⚙️ 👤  │
├──────────┬──────────────────────────────┤
│          │                              │
│ 💬 Chat  │   환영합니다!                │
│ 📊 Dashboard                            │
│ 🔧 Workflow                             │
│ 📈 BI    │   Judgify-core Ver2.0        │
│ ⚙️ Settings                             │
│          │                              │
└──────────┴──────────────────────────────┘
```

### 첫 기능 테스트

**Test 1: Chat Interface**
1. 좌측 "💬 Chat" 클릭
2. 메시지 입력: `안녕하세요!`
3. 전송 버튼 클릭
4. AI 응답 확인

**Test 2: Dashboard**
1. 좌측 "📊 Dashboard" 클릭
2. 차트 표시 확인

---

## 🐛 문제 해결

### Issue 1: "이 시스템에서 스크립트를 실행할 수 없습니다"

**증상**:
```
.\EXECUTE-NOW.ps1 : 이 시스템에서 스크립트를 실행할 수 없으므로...
```

**해결**:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

---

### Issue 2: "cargo: command not found"

**원인**: Rust가 설치되지 않았거나 PATH에 없음

**해결**:
```powershell
# 1. Rust 설치
winget install Rustlang.Rustup

# 2. PowerShell 완전히 종료
# 작업 관리자에서 모든 powershell.exe 프로세스 종료

# 3. 새 PowerShell에서 확인
cargo --version
```

---

### Issue 3: "linker 'link.exe' not found"

**증상**: Rust 컴파일 중 오류
```
error: linking with `link.exe` failed: exit code: 1181
```

**해결**:
```powershell
# Visual Studio Build Tools 설치
winget install Microsoft.VisualStudio.2022.BuildTools

# 설치 시 "Desktop development with C++" 선택!
# 설치 후 컴퓨터 재부팅
```

---

### Issue 4: "OpenAI API error: 401"

**원인**: .env 파일에 유효한 API Key가 없음

**해결**:
```powershell
# .env 파일 확인
notepad .env

# OPENAI_API_KEY가 실제 키인지 확인
# sk-proj-로 시작해야 함
```

---

### Issue 5: "Port 1420 already in use"

**증상**: 포트 충돌
```
Error: listen EADDRINUSE: address already in use :::1420
```

**해결**:
```powershell
# 포트 사용 프로세스 확인
netstat -ano | findstr :1420

# PID 확인 후 종료 (예: PID 12345)
taskkill /PID 12345 /F

# 다시 실행
pnpm tauri dev
```

---

## 📊 체크리스트

### 실행 전 확인
```
□ PowerShell 관리자 권한으로 실행
□ 프로젝트 폴더로 이동 완료
□ Rust 설치 완료 (cargo --version 확인)
□ Visual Studio Build Tools 설치 완료
□ .env 파일에 실제 OpenAI API Key 입력
□ PowerShell 재시작 완료 (Rust 설치 후)
```

### 실행 명령
```powershell
# 방법 1: 자동 스크립트
.\EXECUTE-NOW.ps1

# 방법 2: 직접 실행
pnpm tauri dev
# 또는
npm run tauri:dev
```

---

## 🎯 요약

### 가장 빠른 방법
```powershell
# 1. PowerShell 관리자 권한으로 실행
# 2. 프로젝트 폴더 이동
cd "c:\Users\dilel\Downloads\Judgify-core (2)\Judgify-core"

# 3. 자동 스크립트 실행
.\EXECUTE-NOW.ps1
```

**스크립트가 모든 것을 안내합니다!**

---

## 📞 추가 도움말

- **상세 가이드**: [RUN-LOCALLY.md](RUN-LOCALLY.md)
- **빠른 시작**: [QUICKSTART.md](QUICKSTART.md)
- **프로젝트 상태**: [PROJECT-STATUS.md](PROJECT-STATUS.md)

---

**지금 바로 실행해보세요! 🚀**

최종 업데이트: 2025-01-16
