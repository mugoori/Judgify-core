# 📦 Judgify-core Ver2.0 Final - 설치 가이드

새 PC에서 개발 환경을 설정하기 위한 완전한 설치 가이드입니다.

## 📑 목차

1. [빠른 시작](#-빠른-시작)
2. [시스템 요구사항](#-시스템-요구사항)
3. [설치 방법](#-설치-방법)
4. [플랫폼별 가이드](#-플랫폼별-가이드)
5. [수동 설치](#-수동-설치)
6. [설치 후 설정](#-설치-후-설정)
7. [설치 검증](#-설치-검증)
8. [문제 해결](#-문제-해결)
9. [Docker 대안](#-docker-대안)

---

## 🚀 빠른 시작

### 자동 설치 (권장)

개발 환경을 가장 빠르게 구축하는 방법은 자동 설치 스크립트를 사용하는 것입니다:

#### macOS / Linux
```bash
# 레포지토리 클론
git clone https://github.com/mugoori/Judgify-core.git
cd Judgify-core

# 자동 설치 스크립트 실행
chmod +x scripts/install-all.sh
./scripts/install-all.sh
```

#### Windows (PowerShell)
```powershell
# 레포지토리 클론
git clone https://github.com/mugoori/Judgify-core.git
cd Judgify-core

# 자동 설치 스크립트 실행
.\scripts\install-all.ps1
```

#### Windows (명령 프롬프트)
```cmd
REM 레포지토리 클론
git clone https://github.com/mugoori/Judgify-core.git
cd Judgify-core

REM 자동 설치 스크립트 실행
scripts\install-all.bat
```

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

## 🛠 설치 방법

### 방법 1: 자동 설치 (권장)

**장점:**
- 가장 빠른 설치 시간 (5-15분)
- 자동 종속성 검사
- 오류 처리 및 검증 기능
- 플랫폼별 최적화

**다음과 같은 경우 사용:**
- 가장 빠른 설치를 원할 때
- 자동화 스크립트 사용에 익숙할 때
- 인터넷 연결이 가능할 때

**스크립트:**
- `scripts/install-all.sh` - macOS/Linux
- `scripts/install-all.ps1` - Windows PowerShell
- `scripts/install-all.bat` - Windows CMD

**옵션:**
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

### 방법 2: 수동 설치

**다음과 같은 경우 사용:**
- 설치를 완전히 제어하고 싶을 때
- 특정 버전을 사용하고 싶을 때
- 설치 문제를 해결할 때

아래 [수동 설치](#-수동-설치) 섹션을 참조하세요.

### 방법 3: Docker (부분적)

**다음과 같은 경우 사용:**
- 격리된 환경을 원할 때
- 데이터베이스만 필요할 때 (PostgreSQL + Redis)
- Docker 사용 경험이 있을 때

아래 [Docker 대안](#-docker-대안) 섹션을 참조하세요.

---

## 🖥 플랫폼별 가이드

### macOS

#### 사전 준비사항
```bash
# Homebrew 설치 (아직 설치하지 않은 경우)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### 자동 설치
```bash
./scripts/install-all.sh
```

#### 설치되는 항목:
- Git (Homebrew 통해)
- Node.js 18.x LTS (Homebrew 통해)
- Python 3.11 (Homebrew 통해)
- Rust (rustup 통해)
- PostgreSQL 15 + pgvector (Homebrew 통해)
- Redis 7+ (Homebrew 통해)

#### 설치 후 확인
```bash
# 설치 확인
git --version
node --version
python3 --version
rustc --version
psql --version
redis-cli --version

# Python 가상 환경 활성화
source venv/bin/activate

# 개발 시작
npm run dev
```

---

### Ubuntu / Debian Linux

#### 사전 준비사항
```bash
# 패키지 관리자 업데이트
sudo apt update
sudo apt upgrade -y

# curl 설치 (필요한 경우)
sudo apt install -y curl
```

#### 자동 설치
```bash
chmod +x scripts/install-all.sh
./scripts/install-all.sh
```

#### 설치되는 항목:
- Git (apt 통해)
- Node.js 18.x (NodeSource 저장소 통해)
- Python 3.11 (apt 통해)
- Rust (rustup 통해)
- PostgreSQL 15 + pgvector (apt 통해)
- Redis 7+ (apt 통해)

#### 설치 후 확인
```bash
# 설치 확인
git --version
node --version
python3 --version
rustc --version
psql --version
redis-cli --version

# Python 가상 환경 활성화
source venv/bin/activate

# 개발 시작
npm run dev
```

---

### Windows

#### 사전 준비사항

**옵션 A: PowerShell (권장)**
```powershell
# Chocolatey 패키지 관리자 설치
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# PowerShell 재시작
```

**옵션 B: 명령 프롬프트**
- Chocolatey를 수동으로 설치: https://chocolatey.org/install

#### 자동 설치

**PowerShell:**
```powershell
.\scripts\install-all.ps1
```

**명령 프롬프트:**
```cmd
scripts\install-all.bat
```

#### 설치되는 항목:
- Git (Chocolatey 통해)
- Node.js 18.x LTS (Chocolatey 통해)
- Python 3.11 (Chocolatey 통해)
- Rust (rustup-init.exe 통해)
- PostgreSQL 15 (Chocolatey 통해, 비밀번호: `postgres`)
- Memurai (Windows용 Redis, Chocolatey 통해)

#### 설치 후 확인
```powershell
# 설치 확인
git --version
node --version
python --version
rustc --version
psql --version
memurai-cli --version  # 또는 redis-cli --version

# Python 가상 환경 활성화
.\venv\Scripts\Activate.ps1

# 개발 시작
npm run dev
```

#### Windows 관련 주의사항

1. **관리자 권한**: 일부 설치는 PowerShell/CMD를 관리자 권한으로 실행해야 합니다
2. **PATH 업데이트**: 설치 후 터미널을 재시작하여 PATH를 새로고침하세요
3. **Redis 대안**: Redis 대신 Memurai를 사용합니다 (완전 호환)
4. **pgvector**: 수동 컴파일이 필요할 수 있습니다 - https://github.com/pgvector/pgvector

---

## 🔧 수동 설치

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
- 또는 Chocolatey 사용: `choco install git -y`

### 2. Node.js 18.x LTS 설치

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
- 또는 Chocolatey 사용: `choco install nodejs-lts --version=18.19.0 -y`

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
- 또는 Chocolatey 사용: `choco install python311 -y`

### 4. Rust 설치

**모든 플랫폼:**
```bash
# macOS/Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows (PowerShell)
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
- 또는 Chocolatey 사용: `choco install postgresql15 -y --params "/Password:postgres"`
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

## ⚙️ 설치 후 설정

### 1. 환경 파일 생성

**자동 (권장):**
```bash
# macOS/Linux
bash scripts/setup-env.sh

# Windows PowerShell
.\scripts\setup-env.ps1

# Windows CMD
scripts\setup-env.bat
```

**수동:**
```bash
# 템플릿 파일 복사
cp .env.example .env
cp .mcp.template.json .mcp.json
```

### 2. 환경 변수 설정

`.env` 파일 편집:

```bash
# 데이터베이스 설정
POSTGRES_URL=postgresql://user:password@localhost:5432/judgify_core
DATABASE_URL=${POSTGRES_URL}

# GitHub 연동
GITHUB_TOKEN=github_pat_xxxxxxxxxxxxxxxxxxxxx

# CI/CD
CIRCLECI_TOKEN=your-circleci-token-here

# AI/LLM 서비스
OPENAI_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
OPENAI_MODEL=gpt-4-turbo-preview

# 인증
JWT_SECRET=your-jwt-secret-key-min-32-characters
JWT_EXPIRES_IN=7d

# Redis
REDIS_URL=redis://localhost:6379

# 프론트엔드
NEXT_PUBLIC_API_URL=http://localhost:8000
NEXT_PUBLIC_WS_URL=ws://localhost:8006
```

### 3. MCP 서버 설정

`.mcp.json` 파일 편집:

```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_xxxxxxxxxxxxxxxxxxxxx"
      }
    },
    "postgresql": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres", "postgresql://user:password@localhost:5432/judgify_core"]
    }
  }
}
```

### 4. 데이터베이스 초기화

```bash
# 데이터베이스 생성
createdb judgify_core

# 또는 psql 사용
psql -U postgres -c "CREATE DATABASE judgify_core;"

# pgvector 확장 기능 활성화
psql -U postgres -d judgify_core -c "CREATE EXTENSION IF NOT EXISTS vector;"

# 마이그레이션 실행 (사용 가능한 경우)
# python manage.py migrate
```

---

## ✅ 설치 검증

### 검증 스크립트 실행

설치 스크립트가 자동으로 검증을 수행하거나, 수동으로 실행할 수 있습니다:

```bash
# 시스템 도구 확인
git --version
node --version
python3 --version
rustc --version

# 데이터베이스 확인
psql --version
redis-cli --version  # 또는 Windows에서 memurai-cli --version

# 환경 파일 확인
ls -la .env .mcp.json  # macOS/Linux
dir .env .mcp.json     # Windows

# Python 가상 환경 테스트
source venv/bin/activate  # macOS/Linux
python --version

# Node.js 테스트
npm --version

# Rust/Tauri 테스트
cargo --version
```

### 개발 서버 시작

```bash
# Python 가상 환경 활성화
source venv/bin/activate  # macOS/Linux
.\venv\Scripts\Activate.ps1  # Windows

# 프론트엔드만 시작
npm run dev

# Tauri 데스크톱 앱 시작
npm run tauri:dev
```

서버가 성공적으로 시작되면 설치 완료입니다! 🎉

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
- 또는 Docker 사용: `docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres pgvector/pgvector:pg15`

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
- 또는 Chocolatey 사용: `choco install visualstudio2022buildtools -y`

#### 7. npm install 실패

**문제:** `EACCES: permission denied`

**해결방법:**

**macOS/Linux:**
```bash
# npm 권한 수정
sudo chown -R $(whoami) ~/.npm
sudo chown -R $(whoami) /usr/local/lib/node_modules

# 또는 nvm (Node Version Manager) 사용
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

```yaml
# docker-compose.yml
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

**사용 방법:**
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

**나머지 도구는 일반적으로 설치:**
```bash
# --skip-db 플래그와 함께 설치 스크립트 실행
./scripts/install-all.sh --skip-db  # macOS/Linux
.\scripts\install-all.ps1 -SkipDb   # Windows
```

---

## 📚 추가 자료

### 문서
- [README.md](README.md) - 프로젝트 개요
- [SETUP.md](SETUP.md) - 빠른 설정 가이드
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

## 🎯 다음 단계

설치 성공 후:

1. **환경 설정**
   - 자격 증명으로 `.env` 편집
   - GitHub 토큰으로 `.mcp.json` 편집

2. **데이터베이스 초기화**
   - 데이터베이스 생성: `createdb judgify_core`
   - 확장 기능 활성화: `CREATE EXTENSION vector;`

3. **개발 시작**
   - 가상 환경 활성화: `source venv/bin/activate`
   - 개발 서버 시작: `npm run dev`
   - 또는 Tauri 앱 시작: `npm run tauri:dev`

4. **문서 읽기**
   - 아키텍처 세부사항은 [CLAUDE.md](CLAUDE.md) 참조
   - 서비스별 가이드는 [docs/](docs/) 확인

---

**즐거운 코딩 되세요! 🚀**

이 가이드에서 다루지 않은 문제가 발생하면:
1. [문제 해결](#-문제-해결) 섹션 확인
2. [GitHub 이슈](https://github.com/mugoori/Judgify-core/issues) 검색
3. 자세한 오류 메시지와 시스템 정보를 포함한 새 이슈 생성
