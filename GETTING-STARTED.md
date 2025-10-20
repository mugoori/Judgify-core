# 🚀 Judgify-core Ver2.0 Final - 시작 가이드

새 PC에서 개발 환경을 설정하거나 기존 환경을 업데이트하기 위한 완전한 가이드입니다.

---

## 📑 목차

1. [빠른 시작](#-빠른-시작)
2. [시스템 요구사항](#-시스템-요구사항)
3. [자동 설치 (권장)](#-자동-설치-권장)
4. [수동 설치](#-수동-설치)
5. [환경 설정](#-환경-설정)
6. [데이터베이스 설정](#-데이터베이스-설정)
7. [설치 검증](#-설치-검증)
8. [문제 해결](#-문제-해결)
9. [Docker 대안](#-docker-대안)
10. [다음 단계](#-다음-단계)

---

## ⚡ 빠른 시작

### 1단계: 레포지토리 클론
```bash
git clone https://github.com/mugoori/Judgify-core.git
cd Judgify-core
```

### 2단계: 자동 설치 실행

#### macOS / Linux
```bash
chmod +x scripts/install-all.sh
./scripts/install-all.sh
```

#### Windows (PowerShell)
```powershell
.\scripts\install-all.ps1
```

#### Windows (명령 프롬프트)
```cmd
scripts\install-all.bat
```

### 3단계: 환경 파일 설정
```bash
# .env 파일 편집
# - DATABASE_URL 설정
# - OPENAI_API_KEY 입력
# - GITHUB_TOKEN 입력

# .mcp.json 파일 편집
# - GITHUB_PERSONAL_ACCESS_TOKEN 입력
```

### 4단계: 개발 시작
```bash
# Python 가상 환경 활성화
source venv/bin/activate  # macOS/Linux
.\venv\Scripts\Activate.ps1  # Windows

# 개발 서버 시작
npm run dev
```

**자세한 내용은 아래 섹션을 참조하세요.**

---

## 💻 시스템 요구사항

### 최소 요구사항

- **운영체제**: Windows 10/11, macOS 11+, Ubuntu 20.04+ 또는 호환 Linux 배포판
- **RAM**: 8GB (개발용으로는 16GB 권장)
- **디스크 공간**: 10GB 여유 공간
- **인터넷**: 종속성 다운로드를 위해 필요

### 필수 소프트웨어 버전

| 도구 | 최소 버전 | 권장 버전 |
|------|----------|----------|
| **Git** | 2.30+ | 최신 버전 |
| **Node.js** | 18.x | 18.19.0 LTS |
| **Python** | 3.11+ | 3.11.x |
| **Rust** | 1.70+ | 최신 안정 버전 |
| **PostgreSQL** | 15+ | 15.x |
| **Redis** | 7.0+ | 7.2+ |

---

## 🤖 자동 설치 (권장)

자동 설치 스크립트는 모든 필수 도구를 자동으로 설치하고 설정합니다.

### 설치 옵션

```bash
# 대화형 모드 (기본값)
./scripts/install-all.sh

# 모든 프롬프트 자동 승인
./scripts/install-all.sh --yes

# 설치하지 않고 미리보기
./scripts/install-all.sh --dry-run

# 데이터베이스 설치 건너뛰기 (Docker 사용 시)
./scripts/install-all.sh --skip-db

# 시스템 도구 설치 건너뛰기 (이미 설치된 경우)
./scripts/install-all.sh --skip-system
```

### 자동 설치되는 항목

#### macOS
- Git (Homebrew 통해)
- Node.js 18.x LTS (Homebrew 통해)
- Python 3.11 (Homebrew 통해)
- Rust (rustup 통해)
- PostgreSQL 15 + pgvector (Homebrew 통해)
- Redis 7+ (Homebrew 통해)

#### Ubuntu/Debian
- Git (apt 통해)
- Node.js 18.x (NodeSource 저장소 통해)
- Python 3.11 (apt 통해)
- Rust (rustup 통해)
- PostgreSQL 15 + pgvector (apt 통해)
- Redis 7+ (apt 통해)

#### Windows
- Git (Chocolatey 통해)
- Node.js 18.x LTS (Chocolatey 통해)
- Python 3.11 (Chocolatey 통해)
- Rust (rustup-init.exe 통해)
- PostgreSQL 15 (Chocolatey 통해)
- Memurai (Windows용 Redis, Chocolatey 통해)

### Windows 사전 준비사항

**Chocolatey 패키지 관리자 설치:**
```powershell
# PowerShell 관리자 권한으로 실행
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
```

---

## 🔧 수동 설치

자동 설치를 사용할 수 없거나 완전히 제어하고 싶은 경우 수동으로 설치할 수 있습니다.

### 1. Git 설치

**macOS:**
```bash
brew install git
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y git
```

**Windows:**
- 다운로드: https://git-scm.com/download/win
- 또는 Chocolatey: `choco install git -y`

### 2. Node.js 18.x 설치

**macOS:**
```bash
brew install node@18
```

**Ubuntu/Debian:**
```bash
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs
```

**Windows:**
- 다운로드: https://nodejs.org/
- 또는 Chocolatey: `choco install nodejs-lts --version=18.19.0 -y`

### 3. Python 3.11 설치

**macOS:**
```bash
brew install python@3.11
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y python3.11 python3.11-venv python3.11-dev python3-pip
```

**Windows:**
- 다운로드: https://www.python.org/downloads/
- 또는 Chocolatey: `choco install python311 -y`

### 4. Rust 설치

**모든 플랫폼:**
```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# 다운로드 및 실행: https://win.rustup.rs/x86_64
```

### 5. PostgreSQL 15 설치

**macOS:**
```bash
brew install postgresql@15
brew services start postgresql@15

# pgvector 설치
brew install pgvector
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y postgresql-15 postgresql-contrib-15 postgresql-15-pgvector
sudo systemctl enable postgresql
sudo systemctl start postgresql
```

**Windows:**
- 다운로드: https://www.postgresql.org/download/windows/
- 또는 Chocolatey: `choco install postgresql15 -y --params "/Password:postgres"`
- pgvector: https://github.com/pgvector/pgvector

### 6. Redis 설치

**macOS:**
```bash
brew install redis
brew services start redis
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y redis-server
sudo systemctl enable redis-server
sudo systemctl start redis-server
```

**Windows:**
```powershell
# Memurai 설치 (Windows용 Redis)
choco install memurai-developer -y

# 또는 Docker 사용
docker run -d -p 6379:6379 redis:7-alpine
```

### 7. 프로젝트 종속성 설치

```bash
# 프로젝트 디렉토리로 이동
cd Judgify-core

# Python 가상 환경 생성
python3 -m venv venv

# 가상 환경 활성화
source venv/bin/activate      # macOS/Linux
.\venv\Scripts\Activate.ps1   # Windows PowerShell
venv\Scripts\activate.bat     # Windows CMD

# Python 종속성 설치
pip install --upgrade pip
pip install -r requirements.txt

# Node.js 종속성 설치
npm install

# Tauri 애플리케이션 빌드
cd src-tauri
cargo build
cd ..
```

---

## ⚙️ 환경 설정

### 1. 필수 설정 파일 생성

⚠️ **중요**: `.gitignore`에 포함된 파일들은 Git에 커밋되지 않으므로, 클론 후 반드시 생성해야 합니다.

#### 자동 생성 (권장)
```bash
# macOS/Linux
./scripts/setup-env.sh

# Windows (PowerShell)
.\scripts\setup-env.ps1

# Windows (Command Prompt)
scripts\setup-env.bat
```

#### 수동 생성
```bash
# macOS/Linux
cp .env.example .env
cp .mcp.template.json .mcp.json

# Windows
copy .env.example .env
copy .mcp.template.json .mcp.json
```

### 2. .env 파일 편집

`.env` 파일을 텍스트 에디터로 열고 다음 값을 입력하세요:

```bash
# PostgreSQL 데이터베이스
DATABASE_URL=postgresql://user:password@localhost:5432/judgify_core
POSTGRES_URL=${DATABASE_URL}

# Redis 캐시
REDIS_URL=redis://localhost:6379/0

# OpenAI API Key (AI 판단 엔진용)
OPENAI_API_KEY=sk-your-openai-api-key
OPENAI_MODEL=gpt-4-turbo-preview

# GitHub 연동
GITHUB_TOKEN=github_pat_xxxxxxxxxxxxxxxxxxxxx

# CI/CD (선택사항)
CIRCLECI_TOKEN=your-circleci-token-here

# JWT Secret (최소 32자 랜덤 문자열)
JWT_SECRET=your-secure-random-secret-min-32-chars
JWT_EXPIRES_IN=7d

# 프론트엔드
NEXT_PUBLIC_API_URL=http://localhost:8000
NEXT_PUBLIC_WS_URL=ws://localhost:8006
```

### 3. GitHub Personal Access Token 생성

#### 3.1 토큰 생성
1. GitHub 로그인 → [Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens)
2. "Generate new token (classic)" 클릭
3. **필요한 권한 선택:**
   - ✅ `repo` (전체) - 프라이빗 레포지토리 접근
   - ✅ `workflow` - GitHub Actions 관리
   - ✅ `read:org` - 조직 정보 읽기
4. **Expiration:** 90 days (권장)
5. "Generate token" 클릭
6. **토큰 복사** (한 번만 표시됨! 반드시 저장)

#### 3.2 토큰 만료 알림 설정
- GitHub Settings → Notifications
- ✅ "Email notifications for expiring tokens" 활성화

### 4. .mcp.json 파일 편집

`.mcp.json` 파일을 열고 GitHub Personal Access Token을 입력하세요:

```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_your_github_token_here"
      }
    },
    "postgresql": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres", "postgresql://user:password@localhost:5432/judgify_core"]
    }
  }
}
```

### 5. Claude Desktop MCP 설정 (선택사항)

#### 설정 파일 위치
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`
- **Mac/Linux:** `~/.config/claude/claude_desktop_config.json`

#### 방법 A: 시스템 환경 변수 사용 (권장)

**Windows PowerShell (관리자 권한):**
```powershell
[System.Environment]::SetEnvironmentVariable(
    "GITHUB_PERSONAL_ACCESS_TOKEN",
    "ghp_YOUR_ACTUAL_TOKEN_HERE",
    [System.EnvironmentVariableTarget]::User
)
```

**Mac/Linux:**
```bash
# ~/.bashrc 또는 ~/.zshrc에 추가
export GITHUB_PERSONAL_ACCESS_TOKEN="ghp_YOUR_ACTUAL_TOKEN_HERE"
source ~/.bashrc
```

**Claude Desktop 설정 파일:**
```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_PERSONAL_ACCESS_TOKEN}"
      }
    }
  }
}
```

#### 방법 B: 직접 토큰 입력 (간단하지만 덜 안전)

**Claude Desktop 설정 파일에 직접 입력:**
```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_YOUR_ACTUAL_TOKEN_HERE"
      }
    }
  }
}
```

⚠️ **주의:** 이 방법은 설정 파일에 토큰이 평문으로 저장되므로 주의 필요

#### Claude Desktop 재시작
1. Claude Desktop 완전 종료
2. Claude Desktop 재실행
3. MCP 서버 연결 확인

---

## 🗄️ 데이터베이스 설정

### 1. PostgreSQL 초기화

```bash
# PostgreSQL 서비스 시작 확인
# macOS:
brew services list | grep postgresql

# Ubuntu/Debian:
sudo systemctl status postgresql

# Windows:
net start | findstr postgres

# 데이터베이스 생성
createdb judgify_core

# 또는 psql 사용
psql -U postgres -c "CREATE DATABASE judgify_core;"

# 사용자 생성
createuser -s -P judgify_user

# 또는 psql 사용
psql -U postgres -c "CREATE USER judgify_user WITH PASSWORD 'your_password' SUPERUSER;"

# 권한 부여
psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE judgify_core TO judgify_user;"

# pgvector 확장 설치
psql -U postgres -d judgify_core -c "CREATE EXTENSION IF NOT EXISTS vector;"
```

### 2. Redis 연결 테스트

```bash
# Redis 서비스 시작 확인
redis-cli ping
# 응답: PONG

# Windows (Memurai):
memurai-cli ping
# 응답: PONG
```

---

## ✅ 설치 검증

### 1. 시스템 도구 확인
```bash
git --version
node --version
python3 --version  # 또는 Windows에서 python --version
rustc --version
psql --version
redis-cli --version  # 또는 Windows에서 memurai-cli --version
```

### 2. 환경 파일 확인
```bash
# macOS/Linux
ls -la .env .mcp.json

# Windows
dir .env .mcp.json
```

### 3. Python 가상 환경 테스트
```bash
# 가상 환경 활성화
source venv/bin/activate  # macOS/Linux
.\venv\Scripts\Activate.ps1  # Windows

# Python 버전 확인
python --version

# 설치된 패키지 확인
pip list
```

### 4. 개발 서버 시작 테스트

#### 프론트엔드만 시작
```bash
npm run dev
```

브라우저에서 `http://localhost:3000` 접속 확인

#### Tauri 데스크톱 앱 시작
```bash
npm run tauri:dev
```

데스크톱 애플리케이션 창이 뜨면 성공! 🎉

### 5. 서비스 Health Check
```bash
# API Gateway (개발 후)
curl http://localhost:8000/health

# Judgment Service (개발 후)
curl http://localhost:8002/health

# Frontend
curl http://localhost:3000
```

---

## 🔍 문제 해결

### 일반적인 문제들

#### 1. 설치 후 명령어를 찾을 수 없음

**문제:** `git: command not found`, `node: command not found` 등

**해결방법:**
```bash
# macOS/Linux
source ~/.bashrc
source ~/.zshrc

# Windows
# PowerShell/CMD를 재시작하여 PATH 새로고침
```

#### 2. Python 버전 불일치

**문제:** `python --version`이 Python 2.x 또는 잘못된 버전을 표시

**해결방법:**
```bash
# python3를 명시적으로 사용
python3 --version
python3 -m venv venv

# 또는 별칭 생성 (macOS/Linux)
alias python=python3
```

#### 3. PostgreSQL 연결 오류

**문제:** `FATAL: role "user" does not exist`

**해결방법:**
```bash
# PostgreSQL 사용자 생성
createuser -s -P judgify_user

# 또는 psql 사용
psql -U postgres -c "CREATE USER judgify_user WITH PASSWORD 'your_password' SUPERUSER;"

# .env 업데이트
POSTGRES_URL=postgresql://judgify_user:your_password@localhost:5432/judgify_core
```

#### 4. pgvector 확장 기능을 찾을 수 없음

**문제:** `ERROR: extension "vector" does not exist`

**해결방법:**

**macOS:**
```bash
brew install pgvector
```

**Ubuntu/Debian:**
```bash
sudo apt install postgresql-15-pgvector
```

**Windows:**
- 소스에서 컴파일: https://github.com/pgvector/pgvector#installation-notes
- 또는 Docker 사용:
  ```bash
  docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres pgvector/pgvector:pg15
  ```

#### 5. Windows에서 Redis가 시작되지 않음

**문제:** Redis가 Windows에서 공식적으로 지원되지 않음

**해결방법:**

**옵션 A: Memurai 사용**
```powershell
choco install memurai-developer -y
```

**옵션 B: Docker 사용**
```bash
docker run -d -p 6379:6379 --name redis redis:7-alpine
```

**옵션 C: WSL2 사용**
```bash
# WSL2 Ubuntu에서
sudo apt install redis-server
sudo service redis-server start
```

#### 6. Rust/Tauri 빌드 오류

**문제:** `error: linking with 'cc' failed`

**해결방법:**

**macOS:**
```bash
# Xcode 명령줄 도구 설치
xcode-select --install
```

**Ubuntu/Debian:**
```bash
# 빌드 필수 도구 설치
sudo apt install -y build-essential libssl-dev pkg-config
```

**Windows:**
- Visual Studio Build Tools 설치: https://visualstudio.microsoft.com/downloads/
- 또는 Chocolatey: `choco install visualstudio2022buildtools -y`

#### 7. npm install 실패

**문제:** `EACCES: permission denied`

**해결방법:**

**macOS/Linux:**
```bash
# npm 권한 수정
sudo chown -R $(whoami) ~/.npm
sudo chown -R $(whoami) /usr/local/lib/node_modules

# 또는 nvm 사용
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
```

**Windows:**
- PowerShell/CMD를 관리자 권한으로 실행

#### 8. 가상 환경 활성화 실패

**문제:** `Activate.ps1 cannot be loaded because running scripts is disabled`

**해결방법:**
```powershell
# Windows PowerShell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

---

## 🐳 Docker 대안

### Docker로 데이터베이스 설정

로컬 설치 대신 Docker를 사용하여 데이터베이스를 설치하는 경우:

#### docker-compose.yml 생성
```yaml
version: '3.8'

services:
  postgres:
    image: pgvector/pgvector:pg15
    environment:
      POSTGRES_USER: judgify_user
      POSTGRES_PASSWORD: your_password
      POSTGRES_DB: judgify_core
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

#### Docker 사용 방법
```bash
# 데이터베이스 시작
docker-compose up -d

# 데이터베이스 중지
docker-compose down

# 로그 보기
docker-compose logs -f

# .env 업데이트
POSTGRES_URL=postgresql://judgify_user:your_password@localhost:5432/judgify_core
REDIS_URL=redis://localhost:6379
```

#### 나머지 도구는 일반적으로 설치
```bash
# --skip-db 플래그와 함께 설치 스크립트 실행
./scripts/install-all.sh --skip-db  # macOS/Linux
.\scripts\install-all.ps1 -SkipDb   # Windows
```

---

## 🔐 보안 체크리스트

### 절대 하지 말아야 할 것
- ❌ `.env` 파일을 Git에 커밋
- ❌ 토큰을 코드에 하드코딩
- ❌ 토큰을 채팅/이메일로 공유
- ❌ 공개 GitHub Gist에 설정 파일 업로드
- ❌ 스크린샷에 토큰 노출

### 반드시 해야 할 것
- ✅ `.gitignore`에 `.env` 포함 확인
- ✅ 토큰 만료일 캘린더에 등록
- ✅ 주기적 토큰 갱신 (90일마다)
- ✅ 사용하지 않는 토큰 즉시 삭제
- ✅ 토큰 유출시 즉시 무효화

---

## 🔄 다른 컴퓨터로 이동시

### 기존 컴퓨터에서
```bash
# 최신 코드 푸시
git add .
git commit -m "Update: 작업 내용"
git push origin main
```

### 새 컴퓨터에서
```bash
# 최신 코드 가져오기
git clone https://github.com/mugoori/Judgify-core.git
cd Judgify-core

# 이 가이드의 자동 설치 또는 수동 설치 섹션 따라하기
./scripts/install-all.sh  # macOS/Linux
.\scripts\install-all.ps1  # Windows

# .env 및 .mcp.json 파일 설정
./scripts/setup-env.sh  # macOS/Linux
.\scripts\setup-env.ps1  # Windows
```

---

## 🎯 다음 단계

설치 성공 후:

### 1. 문서 확인
```bash
# 전체 아키텍처 이해
cat CLAUDE.md              # Claude 개발 가이드
cat initial.md             # Ver2.0 Final 요구사항
cat system-structure.md    # 시스템 구조도

# 서비스별 상세 설계
cat docs/services/learning_service.md        # Learning Service
cat docs/algorithms/auto_rule_extraction.md  # Rule 추출 알고리즘
cat docs/algorithms/data_aggregation.md      # 데이터 집계
```

### 2. 개발 우선순위
```
Priority 1: Learning Service (8009)
  - 3가지 Rule 추출 알고리즘 구현
  - Few-shot 학습 관리 (pgvector)

Priority 2: Judgment Service (8002)
  - 하이브리드 판단 로직 (Rule → LLM)
  - Few-shot 샘플 활용

Priority 3: BI Service (8007)
  - MCP 컴포넌트 검색 및 조립
  - 자동 대시보드 생성
```

### 3. 개발 시작
```bash
# Python 가상 환경 활성화
source venv/bin/activate  # macOS/Linux
.\venv\Scripts\Activate.ps1  # Windows

# 프론트엔드 개발 서버 시작
npm run dev

# 또는 Tauri 데스크톱 앱 시작
npm run tauri:dev
```

---

## 📚 추가 자료

### 프로젝트 문서
- [README.md](README.md) - 프로젝트 개요
- [CLAUDE.md](CLAUDE.md) - Claude 개발 가이드 (AI 에이전트 협업)
- [initial.md](initial.md) - Ver2.0 Final 전체 요구사항
- [prompt-guide.md](prompt-guide.md) - LLM Prompt 설계 전략
- [CONTRIBUTING.md](CONTRIBUTING.md) - 기여 가이드라인
- [SECURITY.md](SECURITY.md) - 보안 정책

### 외부 자료
- [Node.js 문서](https://nodejs.org/docs)
- [Python 문서](https://docs.python.org/3/)
- [Rust 북](https://doc.rust-lang.org/book/)
- [PostgreSQL 문서](https://www.postgresql.org/docs/)
- [Redis 문서](https://redis.io/documentation)
- [Tauri 문서](https://tauri.app/v1/guides/)

### 지원
- GitHub 이슈: https://github.com/mugoori/Judgify-core/issues
- GitHub 토론: https://github.com/mugoori/Judgify-core/discussions

---

## 🔄 토큰 갱신 프로세스 (90일마다)

### 만료 2주 전
1. GitHub에서 새 토큰 생성 (동일한 권한)
2. `.env` 파일의 `GITHUB_TOKEN` 업데이트
3. `.mcp.json` 파일의 `GITHUB_PERSONAL_ACCESS_TOKEN` 업데이트
4. 시스템 환경 변수 업데이트 (방법 A 사용시)
5. Claude Desktop 재시작
6. 이전 토큰 GitHub에서 삭제

---

**즐거운 코딩 되세요! 🚀**

이 가이드에서 다루지 않은 문제가 발생하면:
1. [문제 해결](#-문제-해결) 섹션 확인
2. [GitHub 이슈](https://github.com/mugoori/Judgify-core/issues) 검색
3. 자세한 오류 메시지와 시스템 정보를 포함한 새 이슈 생성
