---
name: create-service
description: Create a new FastAPI microservice with standard directory structure for Judgify-core Ver2.0
---

Create a new FastAPI microservice following Judgify-core Ver2.0 standards.

## 🎯 언제 사용하나요?

### ✅ 사용 조건
- 9개 마이크로서비스 외에 새로운 서비스 추가가 필요할 때
- 표준 FastAPI 프로젝트 구조를 빠르게 생성하고 싶을 때
- 일관된 디렉토리 구조와 기본 설정이 필요할 때

### ❌ 사용하지 말아야 할 경우
- 복잡한 아키텍처 설계가 필요할 때 → **Task tool로 ai-engineer Agent 사용**
- 데이터베이스 스키마 설계가 필요할 때 → **Task tool로 database-optimization Agent 사용**
- 보안 설계가 필요할 때 → **Task tool로 security-engineer Agent 사용**

---

## 📋 사용 방법

```bash
/create-service service-name port-number
```

**예시:**
```bash
/create-service payment-service 8010
```

---

## 🔧 생성되는 구조

```
services/{service-name}/
├── app/
│   ├── __init__.py
│   ├── main.py              # FastAPI 앱 진입점
│   ├── config.py            # 환경 설정
│   ├── dependencies.py      # 의존성 주입
│   ├── models/              # Pydantic 모델
│   │   ├── __init__.py
│   │   └── schemas.py
│   ├── routers/             # API 라우터
│   │   ├── __init__.py
│   │   └── api.py
│   ├── services/            # 비즈니스 로직
│   │   ├── __init__.py
│   │   └── core.py
│   └── utils/               # 유틸리티
│       ├── __init__.py
│       └── logger.py
├── tests/
│   ├── __init__.py
│   ├── conftest.py
│   └── test_api.py
├── Dockerfile
├── requirements.txt
├── .env.example
└── README.md
```

---

## 📝 생성되는 파일 내용

### 1. main.py (FastAPI 앱)
```python
from fastapi import FastAPI
from app.routers import api
from app.config import settings
from app.utils.logger import setup_logger

logger = setup_logger(__name__)

app = FastAPI(
    title="{service-name}",
    version="2.0.0",
    description="Judgify-core Ver2.0 {service-name} microservice"
)

# Health check
@app.get("/health")
async def health_check():
    return {"status": "healthy", "service": "{service-name}"}

# Include routers
app.include_router(api.router, prefix="/api/v2/{service-name}")

@app.on_event("startup")
async def startup_event():
    logger.info(f"{service-name} started on port {port-number}")

@app.on_event("shutdown")
async def shutdown_event():
    logger.info(f"{service-name} shutting down")
```

### 2. config.py (환경 설정)
```python
from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    DATABASE_URL: str
    REDIS_URL: str
    LOG_LEVEL: str = "INFO"

    class Config:
        env_file = ".env"

settings = Settings()
```

### 3. Dockerfile
```dockerfile
FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

EXPOSE {port-number}

CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "{port-number}"]
```

### 4. requirements.txt
```
fastapi==0.104.1
uvicorn[standard]==0.24.0
pydantic==2.5.0
pydantic-settings==2.1.0
sqlalchemy==2.0.23
asyncpg==0.29.0
redis==5.0.1
python-jose[cryptography]==3.3.0
pytest==7.4.3
pytest-asyncio==0.21.1
httpx==0.25.1
```

---

## 🚀 다음 단계 추천

서비스 템플릿 생성 후:

1. **아키텍처 검증**: `/validate-architecture` Skill 실행
2. **비즈니스 로직 개발**: 수동으로 `app/services/core.py` 구현
3. **데이터베이스 연동**: database-optimization Agent로 스키마 설계
4. **API 엔드포인트 추가**: `/generate-api` Skill로 CRUD 생성
5. **테스트 작성**: `/generate-tests` Skill로 테스트 템플릿 생성
6. **문서화**: `/sync-docs` Skill로 API 문서 동기화

---

## 💡 주의사항

- **포트 번호 충돌 주의**: 기존 9개 서비스는 8000-8009 사용 중
- **환경 변수 설정**: `.env.example`을 `.env`로 복사하고 실제 값 입력
- **데이터베이스 마이그레이션**: Alembic 설정은 별도로 필요
- **API Gateway 라우팅**: 새 서비스를 API Gateway에 등록 필요

---

## 🔗 관련 리소스

- **Agent 활용**: Task tool로 ai-engineer, database-optimization, security-engineer 호출
- **다음 Skill**: `/generate-api`, `/generate-tests`, `/sync-docs`
- **문서**: [docs/development/implementation_plan.md](../../docs/development/implementation_plan.md)
