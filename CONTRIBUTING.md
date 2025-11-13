# Contributing to Judgify-core Ver2.0 Final 🤝

**Judgify-core** 프로젝트에 기여해주셔서 감사합니다! 이 문서는 프로젝트에 효과적으로 기여하는 방법을 안내합니다.

## 📋 목차
- [행동 강령](#-행동-강령)
- [시작하기](#-시작하기)
- [개발 환경 설정](#-개발-환경-설정)
- [브랜치 전략](#-브랜치-전략)
- [커밋 메시지 규칙](#-커밋-메시지-규칙)
- [코드 스타일](#-코드-스타일)
- [Pull Request 프로세스](#-pull-request-프로세스)
- [테스트 가이드](#-테스트-가이드)
- [이슈 보고](#-이슈-보고)

---

## 📜 행동 강령
이 프로젝트는 [Code of Conduct](CODE_OF_CONDUCT.md)를 따릅니다. 프로젝트에 참여함으로써 이 규칙을 준수하는 데 동의하게 됩니다.

---

## 🚀 시작하기

### 1. 레포지토리 포크
```bash
# GitHub에서 Fork 버튼 클릭 후
git clone https://github.com/YOUR_USERNAME/Judgify-core.git
cd Judgify-core
```

### 2. 업스트림 원격 저장소 추가
```bash
git remote add upstream https://github.com/mugoori/Judgify-core.git
git fetch upstream
```

### 3. 최신 코드 동기화
```bash
git checkout main
git pull upstream main
```

---

## 💻 개발 환경 설정

### 필수 요구사항
- **Python**: 3.11 이상
- **Node.js**: 18 이상 (프론트엔드 작업시)
- **Docker**: 최신 버전
- **PostgreSQL**: 15+ with pgvector
- **Redis**: 7.0+

### 로컬 개발 환경 구축

#### 1. Python 가상환경 생성
```bash
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate
pip install -r requirements.txt
pip install -r requirements-dev.txt
```

#### 2. 환경 변수 설정
```bash
cp .env.example .env
# .env 파일을 편집하여 실제 값으로 변경
```

#### 3. Docker Compose로 인프라 실행
```bash
docker-compose up -d postgres redis
```

#### 4. 데이터베이스 마이그레이션
```bash
alembic upgrade head
```

#### 5. 개발 서버 실행 (예: Judgment Service)
```bash
cd services/judgment
uvicorn app:app --reload --port 8002
```

### 프론트엔드 개발 (해당시)
```bash
cd frontend
npm install
npm run dev
```

---

## 🌿 브랜치 전략

### 브랜치 네이밍 규칙
```
main                     # 프로덕션 브랜치 (보호됨)
develop                  # 개발 통합 브랜치
feature/기능명           # 새로운 기능 개발
service/서비스명         # 마이크로서비스 개발
bugfix/버그명            # 버그 수정
hotfix/긴급수정명        # 긴급 수정
docs/문서명              # 문서 업데이트
refactor/리팩토링명      # 리팩토링
```

### 브랜치 생성 예시
```bash
# 새로운 기능 개발
git checkout -b feature/auto-rule-extraction

# 마이크로서비스 개발
git checkout -b service/learning-service

# 버그 수정
git checkout -b bugfix/fix-judgment-timeout
```

---

## ✍️ 커밋 메시지 규칙

### Conventional Commits 형식 사용
```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type 종류
- `feat`: 새로운 기능
- `fix`: 버그 수정
- `docs`: 문서 변경
- `style`: 코드 포맷팅 (기능 변경 없음)
- `refactor`: 리팩토링
- `test`: 테스트 추가/수정
- `chore`: 빌드, 설정 변경
- `perf`: 성능 개선

### Scope (서비스명)
- `judgment`: Judgment Service (8002)
- `learning`: Learning Service (8009)
- `bi`: BI Service (8007)
- `chat`: Chat Interface Service (8008)
- `workflow`: Workflow Service (8001)
- `data-viz`: Data Visualization Service (8006)
- `action`: Action Service (8003)
- `notification`: Notification Service (8004)
- `logging`: Logging Service (8005)
- `gateway`: API Gateway (8000)

### 커밋 메시지 예시
```bash
# 좋은 예시
feat(learning): Add frequency analysis algorithm for rule extraction
fix(judgment): Fix timeout issue in hybrid judgment engine
docs(readme): Update installation instructions

# 나쁜 예시
update code
fix bug
add feature
```

### 상세 커밋 메시지 예시
```
feat(learning): Implement auto rule extraction with 3 algorithms

- Add frequency analysis algorithm
- Implement decision tree learning with sklearn
- Add LLM pattern discovery algorithm
- Integrate with Few-shot learning system

Closes #123
```

---

## 🎨 코드 스타일

### Python (Backend)
- **스타일 가이드**: PEP 8
- **포매터**: Black (line length: 100)
- **린터**: Flake8, pylint
- **타입 체킹**: mypy

#### 코드 포맷 적용
```bash
# Black 자동 포맷
black services/judgment/

# Import 정렬
isort services/judgment/

# 린트 체크
flake8 services/judgment/
mypy services/judgment/
```

### TypeScript/JavaScript (Frontend)
- **스타일 가이드**: Airbnb JavaScript Style Guide
- **포매터**: Prettier
- **린터**: ESLint

#### 코드 포맷 적용
```bash
npm run format
npm run lint
```

### 네이밍 규칙
```python
# Python
class JudgmentEngine:  # 클래스: PascalCase
    def execute_judgment(self):  # 함수/메서드: snake_case
        api_key = "..."  # 변수: snake_case
        MAX_RETRIES = 3  # 상수: UPPER_SNAKE_CASE

# TypeScript
class WorkflowEditor {  // 클래스: PascalCase
  executeWorkflow() {}  // 메서드: camelCase
  const apiKey = "..."  // 변수: camelCase
  const MAX_RETRIES = 3  // 상수: UPPER_SNAKE_CASE
}
```

---

## 🔀 Pull Request 프로세스

### 1. 작업 전 최신 코드 동기화
```bash
git checkout main
git pull upstream main
git checkout feature/your-feature
git rebase main
```

### 2. 변경사항 커밋
```bash
git add .
git commit -m "feat(service): Your commit message"
```

### 3. 포크한 레포지토리에 푸시
```bash
git push origin feature/your-feature
```

### 4. Pull Request 생성
1. GitHub에서 "New Pull Request" 클릭
2. PR 템플릿에 따라 내용 작성
3. 관련 이슈 번호 연결 (`Closes #123`)
4. 리뷰어 지정 (CODEOWNERS 자동 지정)
5. 라벨 추가 (서비스, 우선순위 등)

### 5. 코드 리뷰 대응
- 리뷰어 피드백에 신속히 응답
- 요청사항 반영 후 코멘트
- CI/CD 통과 확인

### 6. 머지
- 리뷰어 승인 후 Squash and Merge

---

## 🧪 테스트 가이드

### 유닛 테스트 (pytest)
```bash
# 전체 테스트 실행
pytest

# 특정 서비스 테스트
pytest services/judgment/tests/

# 커버리지 확인
pytest --cov=services/judgment --cov-report=html
```

### E2E 테스트 (Playwright)
```bash
# Playwright 테스트 실행
pytest tests/e2e/ --headed

# 특정 시나리오 테스트
pytest tests/e2e/test_judgment_workflow.py
```

### 테스트 작성 규칙
```python
# Good
def test_hybrid_judgment_returns_correct_result():
    """Test that hybrid judgment returns correct result with high confidence."""
    engine = HybridJudgmentEngine()
    result = engine.execute(input_data={"temp": 90})

    assert result.success is True
    assert result.confidence >= 0.8
    assert result.method_used in ["rule", "llm", "hybrid"]

# Bad
def test_judgment():
    engine = JudgmentEngine()
    assert engine.run() == True
```

---

## 🐛 이슈 보고

### 버그 리포트
1. GitHub Issues에서 "Bug Report" 템플릿 사용
2. 재현 가능한 단계 상세히 기술
3. 예상 동작 vs 실제 동작 명시
4. 환경 정보 (OS, Python 버전 등)
5. 관련 로그/스크린샷 첨부

### 기능 제안
1. "Feature Request" 템플릿 사용
2. 해결하려는 문제 설명
3. 제안하는 해결 방법
4. 예상 효과 및 KPI

### 서비스 구현 이슈
1. "Service Implementation" 템플릿 사용
2. 구현 체크리스트 작성
3. API 명세 및 DB 스키마 정의
4. 성능 목표 설정

---

## 📚 추가 리소스

### 문서
- [CLAUDE.md](CLAUDE.md): Claude Code 개발 가이드
- [initial.md](initial.md): Ver2.0 Final 요구사항
- [system-structure.md](system-structure.md): 시스템 아키텍처
- [docs/](docs/): 상세 설계 문서

### 커뮤니케이션
- **Issues**: 버그 리포트, 기능 제안
- **Pull Requests**: 코드 리뷰 및 토론
- **Discussions**: 일반 질문 및 아이디어 공유

---

## ✅ 체크리스트

PR을 제출하기 전에 다음 사항을 확인하세요:

- [ ] 코드 스타일 가이드 준수 (Black, ESLint)
- [ ] 모든 테스트 통과 (pytest, E2E)
- [ ] 새로운 기능에 대한 테스트 추가
- [ ] 문서 업데이트 (해당시)
- [ ] 커밋 메시지 Conventional Commits 형식
- [ ] PR 템플릿 완전히 작성
- [ ] CI/CD 파이프라인 통과
- [ ] 민감정보 노출 확인 (토큰, 비밀번호)

---

## 🙏 감사합니다!

Judgify-core 프로젝트에 기여해주셔서 감사합니다. 여러분의 기여가 제조업 SME를 위한 혁신적인 AI 판단 플랫폼을 만드는 데 큰 도움이 됩니다! 🚀

**Happy Coding!** 🤖⚡
