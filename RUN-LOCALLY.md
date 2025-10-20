# 🚀 Judgify-core 로컬 실행 가이드 (Windows)

**현재 환경**: Windows, Node.js v22.14.0 설치됨

이 가이드는 **당신의 컴퓨터에서 바로 실행할 수 있도록** 작성되었습니다.

---

## ⚡ 빠른 시작 (이미 설치 완료된 경우)

```powershell
# PowerShell에서 실행
cd "c:\Users\dilel\Downloads\Judgify-core (2)\Judgify-core"

# 개발 서버 실행
pnpm tauri dev
```

**앱이 열리지 않으면**: 아래 "1단계: 필수 도구 설치"부터 진행하세요.

---

## 📋 현재 환경 상태

### ✅ 이미 완료된 것
```
✅ Node.js v22.14.0 (요구사항: v20+)
✅ npm 설치됨
✅ node_modules 설치됨 (의존성 준비 완료)
✅ .env 파일 존재
✅ 프로젝트 코드 완성 (Backend + Frontend)
```

### ❌ 추가 설치 필요
```
❌ pnpm (선택사항 - npm으로도 가능)
❌ Rust (필수! Tauri 백엔드 컴파일용)
❌ Visual Studio Build Tools (Rust 컴파일용)
```

---

## 🛠️ 1단계: 필수 도구 설치 (10-15분)

### A. pnpm 설치 (선택 - npm 사용 가능)

**PowerShell 관리자 권한**으로 실행:

```powershell
# pnpm 설치
npm install -g pnpm

# 확인
pnpm --version
```

### B. Rust 설치 (필수!)

**방법 1: winget 사용 (권장)**

```powershell
# PowerShell 관리자 권한으로 실행
winget install Rustlang.Rustup

# 설치 완료 후 PowerShell 재시작!
```

**방법 2: 수동 설치**

1. https://rustup.rs/ 방문
2. "DOWNLOAD RUSTUP-INIT.EXE (64-BIT)" 클릭
3. 다운로드한 `rustup-init.exe` 실행
4. "1) Proceed with installation (default)" 선택 (Enter)
5. 설치 완료 후 **PowerShell 재시작** (중요!)

**설치 확인**:

```powershell
# 새 PowerShell 창에서 실행
cargo --version
rustc --version

# 예상 출력:
# cargo 1.75.0 (1d8b05cdd 2023-11-20)
# rustc 1.75.0 (82e1608df 2023-12-21)
```

### C. Visual Studio Build Tools 설치 (Rust 컴파일용)

**방법 1: winget 사용**

```powershell
# PowerShell 관리자 권한으로 실행
winget install Microsoft.VisualStudio.2022.BuildTools
```

설치 시 **"Desktop development with C++"** 워크로드를 반드시 선택하세요!

**방법 2: 수동 설치**

1. https://visualstudio.microsoft.com/downloads/ 방문
2. "Tools for Visual Studio" 섹션에서 **"Build Tools for Visual Studio 2022"** 다운로드
3. 설치 프로그램 실행
4. **"Desktop development with C++"** 워크로드 선택
5. 설치 (약 6GB, 15-20분 소요)

**설치 후 재부팅 권장**

---

## 🔧 2단계: 환경 설정 (2-3분)

### A. OpenAI API Key 설정 (필수!)

현재 `.env` 파일에는 테스트 키가 설정되어 있습니다. 실제 API Key로 변경해야 합니다.

**1. OpenAI API Key 발급**

1. https://platform.openai.com/api-keys 방문
2. 로그인 (계정 없으면 가입)
3. **"Create new secret key"** 클릭
4. 키 이름 입력 (예: "Judgify Desktop")
5. **생성된 키 복사** (한 번만 보여집니다!)

**2. .env 파일 수정**

```powershell
# 메모장으로 .env 파일 열기
notepad .env
```

다음 줄을 찾아서:
```env
OPENAI_API_KEY=sk-test-key-replace-with-actual-key
```

실제 API Key로 변경:
```env
OPENAI_API_KEY=sk-proj-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**저장 후 메모장 닫기** (Ctrl+S)

### B. Rust 의존성 다운로드 (선택 - 첫 실행시 자동)

```powershell
cd src-tauri
cargo fetch
cd ..
```

---

## 🚀 3단계: 개발 서버 실행!

### PowerShell에서 실행

```powershell
# 프로젝트 루트로 이동
cd "c:\Users\dilel\Downloads\Judgify-core (2)\Judgify-core"

# pnpm 사용 (권장)
pnpm tauri dev

# 또는 npm 사용
npm run tauri:dev
```

### 실행 과정 (최초)

```
1. Frontend 빌드 시작...
   vite v5.0.11 building for development...
   ✓ built in 1.5s

2. Rust 컴파일 시작... (최초 5-10분 소요)
   Downloading crates...
   Compiling serde v1.0.195
   Compiling tokio v1.35.0
   ... (많은 의존성 컴파일)
   Compiling judgify-desktop v2.0.0
   Finished dev [unoptimized + debuginfo] target(s) in 8m 45s

3. 앱 실행!
   [Tauri] Running on http://localhost:1420/
```

### ✅ 성공 확인

**1. 앱 창이 자동으로 열림**

```
┌─────────────────────────────────────────┐
│  Judgify AI Platform          🔍 ⚙️ 👤  │
├──────────┬──────────────────────────────┤
│          │                              │
│ 💬 Chat  │   Chat Interface             │
│ 📊 Dashboard                            │
│ 🔧 Workflow                             │
│ 📈 BI    │   Judgify에 오신 것을         │
│ ⚙️ Settings  환영합니다!                │
│          │                              │
└──────────┴──────────────────────────────┘
```

**2. 첫 기능 테스트**

① **Chat Interface 테스트**:
- 좌측 사이드바에서 "💬 Chat" 클릭
- 메시지 입력: `안녕하세요!`
- 전송 버튼 클릭
- AI 응답 확인

② **Dashboard 확인**:
- 좌측 사이드바에서 "📊 Dashboard" 클릭
- 차트 표시 확인

---

## 🐛 문제 해결

### Issue 1: `cargo: command not found`

**증상**:
```
'cargo'은(는) 내부 또는 외부 명령, 실행할 수 있는 프로그램, 또는 배치 파일이 아닙니다.
```

**해결**:
```powershell
# 1. Rust 설치 확인
winget install Rustlang.Rustup

# 2. PowerShell 완전히 종료 후 재시작 (중요!)
# 작업 관리자에서 모든 powershell.exe 프로세스 종료

# 3. 새 PowerShell에서 확인
cargo --version
```

---

### Issue 2: `error: linker 'link.exe' not found`

**증상**:
```
error: linking with `link.exe` failed: exit code: 1181
  = note: link.exe not found
```

**해결**:
```powershell
# Visual Studio Build Tools 설치
winget install Microsoft.VisualStudio.2022.BuildTools

# 설치 시 "Desktop development with C++" 선택 필수!
# 설치 후 컴퓨터 재부팅
```

---

### Issue 3: `OpenAI API error: 401 Unauthorized`

**증상**: 앱은 실행되지만 Chat에서 "API 오류" 메시지

**해결**:
```powershell
# .env 파일 확인
notepad .env

# OPENAI_API_KEY가 실제 키인지 확인
# sk-proj-로 시작하는 실제 OpenAI API Key여야 함

# 수정 후 앱 재시작
# Ctrl+C로 앱 종료 후
pnpm tauri dev
```

---

### Issue 4: 앱이 빈 화면으로 나타남 (White Screen)

**증상**: 앱 창은 열리지만 내용이 비어있음

**해결**:
```powershell
# 1. Frontend만 별도 테스트
pnpm dev
# 또는
npm run dev

# 2. 브라우저에서 http://localhost:1420 접속
# 정상 작동하는지 확인

# 3. 정상이면 Ctrl+C로 중지 후 다시 실행
pnpm tauri dev
```

---

### Issue 5: `pnpm: command not found`

**증상**: pnpm 명령어가 인식되지 않음

**해결**:
```powershell
# 옵션 1: pnpm 설치
npm install -g pnpm

# 옵션 2: npm 사용
npm run tauri:dev
```

---

### Issue 6: 포트 충돌 (Port already in use)

**증상**:
```
Error: listen EADDRINUSE: address already in use :::1420
```

**해결**:
```powershell
# 1420 포트 사용 중인 프로세스 찾기
netstat -ano | findstr :1420

# PID 확인 후 프로세스 종료
taskkill /PID [PID번호] /F

# 다시 실행
pnpm tauri dev
```

---

## ⏱️ 예상 소요 시간

### 최초 실행 (전체 과정)
```
1단계 (도구 설치):        10-15분
  - pnpm:                  1분
  - Rust:                  5분
  - VS Build Tools:        5-10분
  - PowerShell 재시작:     1분

2단계 (환경 설정):        2-3분
  - API Key 설정:          1분
  - cargo fetch:           1-2분

3단계 (첫 실행):          5-10분
  - Frontend 빌드:         10초
  - Rust 컴파일:           5-10분 ⏰
  - 앱 실행:               자동

총 소요 시간:             20-30분
```

### 이후 실행
```
pnpm tauri dev 실행:      30초-1분
  - Rust 증분 컴파일:      10-20초
  - Frontend 빌드:         5-10초
  - 앱 실행:               자동
```

---

## ✅ 실행 성공 체크리스트

준비 단계:
- [ ] Node.js v20+ 설치 확인 (`node --version`)
- [ ] Rust 설치 확인 (`cargo --version`)
- [ ] Visual Studio Build Tools 설치 확인
- [ ] .env 파일에 실제 OpenAI API Key 입력
- [ ] PowerShell 재시작 완료

실행 단계:
- [ ] `pnpm tauri dev` 실행 성공
- [ ] 앱 창이 열림
- [ ] Chat Interface에서 메시지 전송 성공
- [ ] Dashboard 차트 표시 확인

모두 체크되면 **실행 성공!** 🎉

---

## 🎯 다음 단계

앱 실행에 성공했다면:

1. **Chat Interface**: AI와 대화하며 기능 테스트
2. **Workflow Builder**: 간단한 워크플로우 만들기
3. **Dashboard**: 실시간 데이터 시각화 확인
4. **Settings**: OpenAI API 설정 확인

개발을 시작하려면:
- [CLAUDE.md](CLAUDE.md) - 개발 가이드 읽기
- [docs/development-plan.md](docs/development-plan.md) - 8주 개발 계획

---

## 📞 도움이 필요하신가요?

- **문제 발생 시**: 위 "문제 해결" 섹션 참조
- **상세 가이드**: [README-SETUP.md](README-SETUP.md)
- **빠른 시작**: [QUICKSTART.md](QUICKSTART.md)
- **GitHub Issues**: https://github.com/your-org/Judgify-core/issues

---

**성공적인 실행을 기원합니다! 🚀**

마지막 업데이트: 2025-01-16
