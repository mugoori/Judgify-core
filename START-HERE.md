# 🎯 시작하기 - Judgify-core Ver2.0 Final

**당신의 컴퓨터에서 바로 실행할 수 있습니다!**

---

## ⚡ 1분 요약

```powershell
# 1. 필수 도구 설치 (최초 1회)
winget install Rustlang.Rustup
winget install Microsoft.VisualStudio.2022.BuildTools
npm install -g pnpm

# 2. PowerShell 재시작 후

# 3. OpenAI API Key 설정
notepad .env
# OPENAI_API_KEY=sk-your-real-key-here

# 4. 실행!
pnpm tauri dev
```

**30분 후 앱이 실행됩니다!** 🎉

---

## 📚 상세 가이드 선택

### 🏃 빠르게 시작하고 싶으신가요?
→ **[RUN-LOCALLY.md](RUN-LOCALLY.md)** - 로컬 실행 완벽 가이드
  - 현재 환경에 맞춘 단계별 설명
  - 모든 문제 해결 방법 포함
  - 예상 소요 시간 명시

### 🔰 처음 접하시나요?
→ **[QUICKSTART.md](QUICKSTART.md)** - 5분 빠른 시작
  - 핵심만 간단히 정리
  - 기본 기능 테스트 방법

### 🛠️ 개발 환경을 완벽히 설정하고 싶으신가요?
→ **[README-SETUP.md](README-SETUP.md)** - 개발 환경 완벽 가이드
  - Windows 전용 상세 설명
  - 모든 도구 설치 방법
  - 트러블슈팅 완벽 정리

### 📊 프로젝트 전체를 이해하고 싶으신가요?
→ **[PROJECT-STATUS.md](PROJECT-STATUS.md)** - 현재 프로젝트 상태
  - 완료된 기능 확인
  - 다음 개발 단계
  - 전체 진행도

### 🤖 개발을 시작하고 싶으신가요?
→ **[CLAUDE.md](CLAUDE.md)** - Claude 개발 가이드
  - Ver2.0 Final 아키텍처
  - 9개 마이크로서비스 설명
  - 18개 AI 에이전트 협업 전략

---

## 🎯 현재 상황별 추천

### 상황 1: "지금 바로 앱을 실행하고 싶어요!"
```
1. RUN-LOCALLY.md 열기
2. "1단계: 필수 도구 설치" 따라하기 (10분)
3. "2단계: 환경 설정" 따라하기 (2분)
4. "3단계: 실행!" (5-10분)
```

### 상황 2: "Rust가 뭔지 모르겠어요"
```
Rust는 C 언어가 아닙니다!
- 현대적이고 안전한 시스템 프로그래밍 언어
- Tauri 데스크톱 앱을 만들기 위해 필수
- 설치만 하면 자동으로 동작합니다

→ RUN-LOCALLY.md의 "1단계 B. Rust 설치" 참조
```

### 상황 3: "OpenAI API Key가 없어요"
```
1. https://platform.openai.com/api-keys 방문
2. 계정 생성 (신용카드 등록 필요)
3. "Create new secret key" 클릭
4. 생성된 키를 .env 파일에 입력

→ RUN-LOCALLY.md의 "2단계 A. OpenAI API Key 설정" 참조
```

### 상황 4: "에러가 발생했어요"
```
→ RUN-LOCALLY.md의 "문제 해결" 섹션 참조

가장 흔한 에러:
1. cargo: command not found → Rust 설치 + PowerShell 재시작
2. linker 'link.exe' not found → Visual Studio Build Tools 설치
3. OpenAI API error: 401 → .env에 실제 API Key 입력
```

---

## ✅ 빠른 체크리스트

### 실행 전 확인
```
□ Node.js v20+ 설치됨 (node --version)
□ Rust 설치됨 (cargo --version)
□ Visual Studio Build Tools 설치됨
□ .env 파일에 실제 OpenAI API Key 입력
□ PowerShell 재시작 완료
```

### 실행 명령
```powershell
# pnpm 사용 (권장)
pnpm tauri dev

# 또는 npm 사용
npm run tauri:dev
```

---

## 🎉 성공 확인

앱 창이 열리고 다음 화면이 보이면 **성공**입니다!

```
┌─────────────────────────────────────────┐
│  Judgify AI Platform          🔍 ⚙️ 👤  │
├──────────┬──────────────────────────────┤
│          │                              │
│ 💬 Chat  │   환영합니다!                │
│ 📊 Dashboard                            │
│ 🔧 Workflow                             │
│ 📈 BI    │   Chat에서 "안녕하세요!"     │
│ ⚙️ Settings  메시지를 보내보세요        │
│          │                              │
└──────────┴──────────────────────────────┘
```

---

## 📞 도움말

### 문서 찾기
```
실행 관련:     RUN-LOCALLY.md
빠른 시작:     QUICKSTART.md
환경 설정:     README-SETUP.md
프로젝트 상태: PROJECT-STATUS.md
개발 가이드:   CLAUDE.md
```

### 추가 지원
- **GitHub Issues**: https://github.com/your-org/Judgify-core/issues
- **Discord**: [링크 추가 필요]

---

**지금 바로 시작하세요! 🚀**

최종 업데이트: 2025-01-16
