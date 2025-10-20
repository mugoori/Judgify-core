# Judgify-core Ver2.0 환경 설정 가이드 🚀

이 문서는 새로운 컴퓨터나 팀원이 프로젝트를 설정할 때 참조하는 가이드입니다.

---

## 📋 사전 준비사항

### 필수 도구 설치
- [ ] Git
- [ ] Node.js (v18 이상)
- [ ] Python (v3.10 이상)
- [ ] PostgreSQL (v14 이상)
- [ ] Redis
- [ ] Docker Desktop (선택적)

---

## 🔐 1단계: GitHub Token 준비

### 1.1 GitHub Personal Access Token 생성
1. GitHub 로그인 → [Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens)
2. "Generate new token (classic)" 클릭
3. **필요한 권한 선택:**
   - ✅ `repo` (전체) - 프라이빗 레포지토리 접근
   - ✅ `workflow` - GitHub Actions 관리
   - ✅ `read:org` - 조직 정보 읽기
4. **Expiration:** 90 days (권장)
5. "Generate token" 클릭
6. **토큰 복사** (한 번만 표시됨! 반드시 저장)

### 1.2 토큰 만료 알림 설정
- GitHub Settings → Notifications
- ✅ "Email notifications for expiring tokens" 활성화
- 만료 7일 전 이메일 수신

---

## 📦 2단계: 프로젝트 클론 및 초기 설정

### 2.1 프로젝트 클론
```bash
# 프라이빗 레포지토리 클론
git clone https://github.com/YOUR_USERNAME/judgify-core-v2.git
cd judgify-core-v2
```

### 2.2 환경 변수 파일 생성
```bash
# .env.example을 .env로 복사
cp .env.example .env

# Windows에서는:
copy .env.example .env
```

### 2.3 .env 파일 편집
```bash
# 텍스트 에디터로 .env 파일 열기
notepad .env

# 또는 VS Code
code .env
```

**반드시 수정해야 할 항목:**
```bash
# GitHub Token (1단계에서 생성한 토큰)
GITHUB_TOKEN=ghp_YOUR_ACTUAL_TOKEN_HERE

# PostgreSQL (로컬 DB 설정에 맞게)
POSTGRES_URL=postgresql://user:password@localhost:5432/judgify_core

# OpenAI API Key (AI 판단 엔진용)
OPENAI_API_KEY=sk-YOUR_OPENAI_API_KEY

# JWT Secret (최소 32자 랜덤 문자열)
JWT_SECRET=your-secure-random-secret-min-32-chars
```

---

## 🔧 3단계: Claude Desktop MCP 설정

### 3.1 설정 파일 위치
**Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

**Mac/Linux:** `~/.config/claude/claude_desktop_config.json`

### 3.2 GitHub MCP 설정 방법

#### 방법 A: 시스템 환경 변수 사용 (권장)

**Windows PowerShell (관리자 권한):**
```powershell
# 영구 환경 변수 설정
[System.Environment]::SetEnvironmentVariable(
    "GITHUB_PERSONAL_ACCESS_TOKEN",
    "ghp_YOUR_ACTUAL_TOKEN_HERE",
    [System.EnvironmentVariableTarget]::User
)

# 설정 확인
$env:GITHUB_PERSONAL_ACCESS_TOKEN
```

**Mac/Linux:**
```bash
# ~/.bashrc 또는 ~/.zshrc에 추가
export GITHUB_PERSONAL_ACCESS_TOKEN="ghp_YOUR_ACTUAL_TOKEN_HERE"

# 적용
source ~/.bashrc  # 또는 source ~/.zshrc
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

### 3.3 Claude Desktop 재시작
```
1. Claude Desktop 완전 종료
2. Claude Desktop 재실행
3. MCP 서버 연결 확인
```

---

## 🗄️ 4단계: 데이터베이스 설정

### 4.1 PostgreSQL 설치 및 초기화
```bash
# PostgreSQL 서비스 시작 (Windows)
net start postgresql-x64-14

# 데이터베이스 생성
psql -U postgres
CREATE DATABASE judgify_core;
CREATE USER judgify_user WITH PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE judgify_core TO judgify_user;

# pgvector 확장 설치 (RAG용)
CREATE EXTENSION vector;
```

### 4.2 Redis 설치 및 시작
```bash
# Windows (WSL2 또는 Docker 권장)
docker run -d -p 6379:6379 redis:alpine

# 연결 테스트
redis-cli ping
# 응답: PONG
```

---

## 📦 5단계: 의존성 설치

### 5.1 Backend (Python)
```bash
# 가상 환경 생성
python -m venv venv

# 가상 환경 활성화
# Windows:
venv\Scripts\activate
# Mac/Linux:
source venv/bin/activate

# 의존성 설치
pip install -r requirements.txt
```

### 5.2 Frontend (Node.js)
```bash
cd frontend
npm install
# 또는
pnpm install
```

---

## 🚀 6단계: 서비스 실행

### 6.1 개발 환경 전체 실행 (Docker Compose)
```bash
# 모든 서비스 시작
docker-compose up -d

# 로그 확인
docker-compose logs -f
```

### 6.2 개별 서비스 실행

**API Gateway (8000):**
```bash
cd services/api-gateway
uvicorn main:app --reload --port 8000
```

**Judgment Service (8002):**
```bash
cd services/judgment-service
uvicorn main:app --reload --port 8002
```

**Frontend (3000):**
```bash
cd frontend
npm run dev
```

---

## ✅ 7단계: 설정 검증

### 7.1 서비스 Health Check
```bash
# API Gateway
curl http://localhost:8000/health

# Judgment Service
curl http://localhost:8002/health

# Frontend
curl http://localhost:3000
```

### 7.2 GitHub MCP 연결 테스트
Claude Desktop에서 다음 명령어 실행:
```
/mcp github status
```

### 7.3 데이터베이스 연결 테스트
```bash
# PostgreSQL
psql -U judgify_user -d judgify_core -c "SELECT version();"

# Redis
redis-cli ping
```

---

## 🔄 8단계: 다른 컴퓨터로 이동시

### 8.1 기존 컴퓨터에서
```bash
# 최신 코드 푸시
git add .
git commit -m "Update: 작업 내용"
git push origin develop
```

### 8.2 새 컴퓨터에서
```bash
# 최신 코드 가져오기
cd judgify-core-v2
git pull origin develop

# .env 파일만 확인 (이미 설정되어 있으면 생략)
# 없으면 2단계부터 다시 진행
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

## 🆘 문제 해결

### MCP 서버 연결 실패
```bash
# 1. 환경 변수 확인
echo $GITHUB_PERSONAL_ACCESS_TOKEN

# 2. Claude Desktop 로그 확인
# Windows: %APPDATA%\Claude\logs\
# Mac/Linux: ~/.config/claude/logs/

# 3. MCP 서버 수동 테스트
npx @modelcontextprotocol/server-github
```

### 데이터베이스 연결 실패
```bash
# PostgreSQL 서비스 상태 확인
# Windows:
net start | findstr postgres

# .env 파일의 DATABASE_URL 확인
cat .env | grep DATABASE_URL
```

### Docker 컨테이너 오류
```bash
# 컨테이너 재시작
docker-compose restart

# 로그 확인
docker-compose logs -f [service-name]

# 완전 재빌드
docker-compose down -v
docker-compose up --build -d
```

---

## 📞 도움 요청

문제가 해결되지 않으면:
1. **GitHub Issues:** 프로젝트 레포지토리에 이슈 등록
2. **팀 채널:** Slack/Discord 팀 채널에서 문의
3. **문서 확인:** [CLAUDE.md](./CLAUDE.md), [README.md](./README.md) 참조

---

## 🔄 토큰 갱신 프로세스 (90일마다)

### 만료 2주 전
1. GitHub에서 새 토큰 생성 (동일한 권한)
2. `.env` 파일의 `GITHUB_TOKEN` 업데이트
3. 시스템 환경 변수 업데이트 (방법 A 사용시)
4. Claude Desktop 재시작
5. 이전 토큰 GitHub에서 삭제

### 자동화 스크립트
```bash
# scripts/rotate-token.sh 실행
./scripts/rotate-token.sh
```

---

## 📚 추가 문서

- [시스템 아키텍처](./docs/architecture/system_overview.md)
- [개발 가이드](./CLAUDE.md)
- [API 문서](./docs/api/)
- [배포 가이드](./docs/deployment/)

---

**설정 완료되셨나요? 이제 개발을 시작하세요! 🎉**
