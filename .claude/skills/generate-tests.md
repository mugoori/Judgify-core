---
name: generate-tests
description: Generate pytest test templates for FastAPI services with async support
---

Generate pytest test templates for FastAPI microservices with async support and fixtures.

## 🎯 언제 사용하나요?

### ✅ 사용 조건
- FastAPI API 엔드포인트 테스트가 필요할 때
- 표준 pytest 템플릿을 빠르게 생성하고 싶을 때
- 90% 이상 코드 커버리지를 목표로 할 때

### ❌ 사용하지 말아야 할 경우
- 복잡한 테스트 시나리오 설계 → **Task tool로 performance-engineer Agent 사용**
- E2E 테스트 자동화 → **playwright MCP 서버 사용**
- 성능/부하 테스트 → **/run-load-test Skill 사용**

---

## 📋 사용 방법

```bash
/generate-tests service-name api-endpoint
```

**예시:**
```bash
/generate-tests judgment-service /api/v2/judgment/execute
/generate-tests workflow-service /api/v2/workflow
```

---

## 🔧 생성되는 파일 구조

```
tests/
├── __init__.py
├── conftest.py              # Pytest fixtures
├── test_api.py              # API 엔드포인트 테스트
├── test_services.py         # 서비스 레이어 테스트
└── test_integration.py      # 통합 테스트
```

---

## 📝 생성되는 테스트 코드

### 1. conftest.py (Pytest Fixtures)

```python
import pytest
import asyncio
from typing import AsyncGenerator
from httpx import AsyncClient
from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine
from sqlalchemy.orm import sessionmaker

from app.main import app
from app.dependencies import get_database
from app.config import settings

# Async test engine
engine = create_async_engine(
    settings.TEST_DATABASE_URL,
    echo=True
)

AsyncSessionLocal = sessionmaker(
    engine,
    class_=AsyncSession,
    expire_on_commit=False
)

@pytest.fixture(scope="session")
def event_loop():
    """Create event loop for async tests"""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()

@pytest.fixture
async def db_session() -> AsyncGenerator[AsyncSession, None]:
    """Database session fixture"""
    async with AsyncSessionLocal() as session:
        yield session
        await session.rollback()

@pytest.fixture
async def client(db_session: AsyncSession) -> AsyncGenerator[AsyncClient, None]:
    """HTTP client fixture"""
    async def override_get_database():
        yield db_session

    app.dependency_overrides[get_database] = override_get_database

    async with AsyncClient(app=app, base_url="http://test") as client:
        yield client

    app.dependency_overrides.clear()

@pytest.fixture
def sample_data():
    """Sample test data"""
    return {
        "name": "Test Item",
        "description": "Test description",
        "is_active": True
    }
```

### 2. test_api.py (API 엔드포인트 테스트)

```python
import pytest
from httpx import AsyncClient
from uuid import uuid4

class TestAPIEndpoints:
    """Test API endpoints"""

    @pytest.mark.asyncio
    async def test_create_{endpoint}(self, client: AsyncClient, sample_data):
        """Test creating new {endpoint}"""
        response = await client.post("/api/v2/{service}/{endpoint}", json=sample_data)

        assert response.status_code == 201
        data = response.json()
        assert "id" in data
        assert data["name"] == sample_data["name"]
        assert data["is_active"] == sample_data["is_active"]

    @pytest.mark.asyncio
    async def test_get_all_{endpoint}s(self, client: AsyncClient):
        """Test getting all {endpoint}s"""
        response = await client.get("/api/v2/{service}/{endpoint}")

        assert response.status_code == 200
        data = response.json()
        assert isinstance(data, list)

    @pytest.mark.asyncio
    async def test_get_{endpoint}_by_id(self, client: AsyncClient, sample_data):
        """Test getting {endpoint} by ID"""
        # Create first
        create_response = await client.post(
            "/api/v2/{service}/{endpoint}",
            json=sample_data
        )
        created_id = create_response.json()["id"]

        # Get by ID
        response = await client.get(f"/api/v2/{service}/{endpoint}/{created_id}")

        assert response.status_code == 200
        data = response.json()
        assert data["id"] == created_id

    @pytest.mark.asyncio
    async def test_get_{endpoint}_not_found(self, client: AsyncClient):
        """Test 404 when {endpoint} not found"""
        fake_id = str(uuid4())
        response = await client.get(f"/api/v2/{service}/{endpoint}/{fake_id}")

        assert response.status_code == 404

    @pytest.mark.asyncio
    async def test_update_{endpoint}(self, client: AsyncClient, sample_data):
        """Test updating {endpoint}"""
        # Create first
        create_response = await client.post(
            "/api/v2/{service}/{endpoint}",
            json=sample_data
        )
        created_id = create_response.json()["id"]

        # Update
        update_data = {"name": "Updated Name"}
        response = await client.put(
            f"/api/v2/{service}/{endpoint}/{created_id}",
            json=update_data
        )

        assert response.status_code == 200
        data = response.json()
        assert data["name"] == "Updated Name"

    @pytest.mark.asyncio
    async def test_delete_{endpoint}(self, client: AsyncClient, sample_data):
        """Test deleting {endpoint}"""
        # Create first
        create_response = await client.post(
            "/api/v2/{service}/{endpoint}",
            json=sample_data
        )
        created_id = create_response.json()["id"]

        # Delete
        response = await client.delete(f"/api/v2/{service}/{endpoint}/{created_id}")

        assert response.status_code == 204

    @pytest.mark.asyncio
    async def test_validation_error(self, client: AsyncClient):
        """Test validation error with invalid data"""
        invalid_data = {"name": ""}  # Empty name should fail
        response = await client.post(
            "/api/v2/{service}/{endpoint}",
            json=invalid_data
        )

        assert response.status_code == 422  # Unprocessable Entity
```

### 3. test_services.py (서비스 레이어 테스트)

```python
import pytest
from sqlalchemy.ext.asyncio import AsyncSession

from app.services.core import {ServiceName}Service
from app.models.schemas import {ModelName}Create, {ModelName}Update

class Test{ServiceName}Service:
    """Test {ServiceName} service layer"""

    @pytest.mark.asyncio
    async def test_create_{model}(self, db_session: AsyncSession, sample_data):
        """Test creating {model} via service"""
        service = {ServiceName}Service(db_session)
        create_data = {ModelName}Create(**sample_data)

        result = await service.create(create_data)

        assert result is not None
        assert result.name == sample_data["name"]

    @pytest.mark.asyncio
    async def test_get_all_{model}s(self, db_session: AsyncSession):
        """Test getting all {model}s via service"""
        service = {ServiceName}Service(db_session)

        results = await service.get_all(skip=0, limit=10)

        assert isinstance(results, list)

    @pytest.mark.asyncio
    async def test_pagination(self, db_session: AsyncSession):
        """Test pagination works correctly"""
        service = {ServiceName}Service(db_session)

        # Test skip and limit
        page1 = await service.get_all(skip=0, limit=5)
        page2 = await service.get_all(skip=5, limit=5)

        assert len(page1) <= 5
        assert len(page2) <= 5
```

---

## 📊 테스트 커버리지 목표

| 구분 | 목표 커버리지 | 주요 테스트 |
|------|--------------|------------|
| **API 엔드포인트** | 95% | CRUD 전체, 에러 케이스 |
| **서비스 레이어** | 90% | 비즈니스 로직, 예외 처리 |
| **데이터 검증** | 100% | Pydantic 스키마 검증 |
| **통합 테스트** | 80% | E2E 시나리오 |

---

## 🚀 테스트 실행 방법

```bash
# 전체 테스트 실행
pytest tests/ -v

# 커버리지 측정
pytest tests/ --cov=app --cov-report=html

# 특정 파일만 테스트
pytest tests/test_api.py -v

# 마크별 실행
pytest -m asyncio  # async 테스트만
```

---

## 🚀 다음 단계 추천

테스트 생성 후:

1. **테스트 실행**: `pytest tests/ -v` 실행하여 통과 확인
2. **커버리지 확인**: `pytest --cov` 실행하여 90% 이상 확인
3. **성능 테스트**: `/run-load-test` Skill로 부하 테스트
4. **CI/CD 통합**: GitHub Actions에 테스트 자동화 추가
5. **문서화**: `/sync-docs` Skill로 테스트 문서 업데이트

---

## 💡 주의사항

- **테스트 데이터베이스**: `TEST_DATABASE_URL` 환경변수 필수
- **Async 테스트**: `@pytest.mark.asyncio` 데코레이터 필수
- **Fixtures**: `conftest.py`의 fixture 재사용 권장
- **트랜잭션**: 각 테스트 후 자동 롤백 (격리 보장)

---

## 🔗 관련 리소스

- **Agent 활용**: performance-engineer (성능 테스트), mlops-engineer (모델 테스트)
- **다음 Skill**: `/run-load-test`, `/validate-architecture`
- **문서**: [docs/development/implementation_plan.md](../../docs/development/implementation_plan.md)
