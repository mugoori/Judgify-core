# 코드 재사용 실전 가이드 (Code Reusability Guide)

**문서 버전**: v1.0
**작성일**: 2025-01-22
**대상**: 백엔드 개발자, AI 에이전트, Claude Code
**목적**: Common Library 활용한 효율적인 마이크로서비스 개발

---

## 📋 목차

1. [개요](#1-개요)
2. [Base Service 활용](#2-base-service-활용)
3. [Base Repository 활용](#3-base-repository-활용)
4. [Pydantic 모델 활용](#4-pydantic-모델-활용)
5. [Utils (Database, Cache, Logger)](#5-utils-database-cache-logger)
6. [Middleware 활용](#6-middleware-활용)
7. [Exception Handling](#7-exception-handling)
8. [실전 예시: Workflow Service](#8-실전-예시-workflow-service)
9. [테스트 전략](#9-테스트-전략)
10. [주의사항 및 베스트 프랙티스](#10-주의사항-및-베스트-프랙티스)

---

## 1. 개요

### 1.1 왜 Common Library인가?

**문제**: 9개 마이크로서비스 개발시 80% 코드 중복
- DB 연결 로직 9번 작성
- CRUD API 9번 반복
- 에러 처리 9번 구현
- 로깅 설정 9번 설정

**해결**: Common Library로 **84% 코드 재사용**
- DB 연결: 1번 작성, 9번 재사용
- CRUD: BaseService 상속으로 자동 획득
- 에러 처리: 전역 핸들러 자동 적용
- 로깅: 구조화 JSON 로그 자동 생성

### 1.2 아키텍처 개요

```
계층 구조 (Layered Architecture):

[API Layer]           # FastAPI 라우터
    ↓ Depends
[Service Layer]       # 비즈니스 로직 (BaseService 상속)
    ↓
[Repository Layer]    # 데이터 액세스 (BaseRepository 상속)
    ↓
[Database]            # PostgreSQL + pgvector
```

---

## 2. Base Service 활용

### 2.1 기본 사용법

```python
# services/workflow/app/services/workflow_service.py
from uuid import UUID
from sqlalchemy.ext.asyncio import AsyncSession

from common.base import BaseService
from app.repositories.workflow_repository import WorkflowRepository
from app.models.schemas import (
    WorkflowCreate,
    WorkflowUpdate,
    WorkflowResponse
)
from app.models.db_models import WorkflowDBModel


class WorkflowService(BaseService[
    WorkflowDBModel,      # SQLAlchemy ORM 모델
    WorkflowCreate,       # 생성 스키마
    WorkflowUpdate,       # 수정 스키마
    WorkflowResponse      # 응답 스키마
]):
    """Workflow 비즈니스 로직 서비스"""

    def __init__(self, db: AsyncSession):
        repository = WorkflowRepository(db)
        super().__init__(db, repository)

    # ✅ 무료 획득! BaseService가 제공하는 메서드:
    # - async create(data: WorkflowCreate) -> WorkflowResponse
    # - async get_by_id(id: UUID) -> WorkflowResponse
    # - async get_all(skip: int, limit: int) -> List[WorkflowResponse]
    # - async update(id: UUID, data: WorkflowUpdate) -> WorkflowResponse
    # - async delete(id: UUID) -> bool

    # 고유 비즈니스 로직만 추가 구현
    async def simulate(self, workflow_id: UUID, test_data: dict):
        """워크플로우 시뮬레이션 실행"""
        workflow = await self.get_by_id(workflow_id)  # Base 메서드 재사용!

        self.logger.info(f"Simulating workflow {workflow_id}")

        # 시뮬레이션 로직 구현
        result = {
            "workflow_id": workflow_id,
            "test_data": test_data,
            "result": "success"
        }

        return result
```

### 2.2 재사용률: 85%

**자동 제공** (무료):
- ✅ `create()` - 엔티티 생성
- ✅ `get_by_id()` - ID로 조회
- ✅ `get_all()` - 전체 조회 (페이지네이션)
- ✅ `update()` - 수정
- ✅ `delete()` - 삭제
- ✅ 자동 로깅 (구조화 JSON)
- ✅ 자동 예외 처리 (NotFoundError)

**추가 구현** (고유 로직):
- ❌ `simulate()` - 워크플로우 시뮬레이션
- ❌ `validate_definition()` - 워크플로우 정의 검증

---

## 3. Base Repository 활용

### 3.1 기본 사용법

```python
# services/workflow/app/repositories/workflow_repository.py
from typing import List, Optional
from uuid import UUID
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from common.base import BaseRepository
from app.models.db_models import WorkflowDBModel


class WorkflowRepository(BaseRepository[WorkflowDBModel]):
    """Workflow 데이터 액세스 레이어"""

    def __init__(self, db: AsyncSession):
        super().__init__(db, WorkflowDBModel)

    # ✅ 무료 획득! BaseRepository가 제공하는 메서드:
    # - async find_by_id(id: UUID) -> Optional[WorkflowDBModel]
    # - async find_all(skip: int, limit: int) -> List[WorkflowDBModel]
    # - async save(entity: WorkflowDBModel) -> WorkflowDBModel
    # - async update_by_id(id: UUID, data: dict) -> Optional[WorkflowDBModel]
    # - async delete_by_id(id: UUID) -> bool
    # - async exists(id: UUID) -> bool

    # 고유 쿼리만 추가 구현
    async def find_active(self) -> List[WorkflowDBModel]:
        """활성 워크플로우만 조회"""
        result = await self.db.execute(
            select(self.model).where(self.model.is_active == True)
        )
        return list(result.scalars().all())

    async def find_by_name(self, name: str) -> Optional[WorkflowDBModel]:
        """이름으로 워크플로우 조회"""
        result = await self.db.execute(
            select(self.model).where(self.model.name == name)
        )
        return result.scalar_one_or_none()
```

### 3.2 재사용률: 80%

**자동 제공**:
- ✅ 기본 CRUD 6개 메서드
- ✅ 페이지네이션
- ✅ 에러 처리 (IntegrityError → ValidationError)
- ✅ 트랜잭션 관리 (commit, rollback)

**추가 구현**:
- ❌ `find_active()` - 비즈니스 필터링
- ❌ `find_by_name()` - 고유 검색 조건

---

## 4. Pydantic 모델 활용

### 4.1 BaseEntity 상속

```python
# services/workflow/app/models/schemas.py
from datetime import datetime
from uuid import UUID
from typing import Optional
from pydantic import Field

from common.base import BaseEntity, BaseCreateModel, BaseUpdateModel


# ========== Create Schema ==========
class WorkflowCreate(BaseCreateModel):
    """워크플로우 생성 요청 (id, timestamp 자동 생성)"""
    name: str = Field(..., min_length=1, max_length=255)
    description: Optional[str] = None
    definition: dict = Field(..., description="Workflow JSON definition")
    is_active: bool = True


# ========== Update Schema ==========
class WorkflowUpdate(BaseUpdateModel):
    """워크플로우 수정 요청 (모든 필드 선택)"""
    name: Optional[str] = Field(None, min_length=1, max_length=255)
    description: Optional[str] = None
    definition: Optional[dict] = None
    is_active: Optional[bool] = None


# ========== Response Schema ==========
class WorkflowResponse(BaseEntity):
    """워크플로우 응답 (자동: id, created_at, updated_at)"""
    name: str
    description: Optional[str]
    definition: dict
    is_active: bool

    class Config:
        from_attributes = True  # SQLAlchemy ORM 호환
```

### 4.2 자동 포함 필드

**BaseEntity가 제공**:
- `id: UUID` - 고유 식별자 (자동 생성)
- `created_at: datetime` - 생성 시간 (자동)
- `updated_at: Optional[datetime]` - 수정 시간 (자동)
- `mark_updated()` - 타임스탬프 갱신 메서드

---

## 5. Utils (Database, Cache, Logger)

### 5.1 Database (PostgreSQL)

```python
# services/*/app/main.py
from fastapi import FastAPI, Depends
from sqlalchemy.ext.asyncio import AsyncSession
from common.utils import get_database, init_database, close_database

app = FastAPI()

# Startup: DB 초기화
@app.on_event("startup")
async def startup():
    await init_database()

# Shutdown: DB 연결 풀 종료
@app.on_event("shutdown")
async def shutdown():
    await close_database()

# API 엔드포인트에서 사용
@app.post("/workflows")
async def create_workflow(
    data: WorkflowCreate,
    db: AsyncSession = Depends(get_database)  # 의존성 주입!
):
    service = WorkflowService(db)
    return await service.create(data)
```

### 5.2 Cache (Redis)

```python
from fastapi import Depends
from redis.asyncio import Redis
from common.utils import get_redis_cache
import json

@app.get("/workflows/{id}")
async def get_workflow(
    id: UUID,
    db: AsyncSession = Depends(get_database),
    cache: Redis = Depends(get_redis_cache)
):
    # 1. 캐시 확인
    cache_key = f"workflow:{id}"
    cached = await cache.get(cache_key)

    if cached:
        return json.loads(cached)

    # 2. DB 조회
    service = WorkflowService(db)
    workflow = await service.get_by_id(id)

    # 3. 캐시 저장 (TTL 5분)
    await cache.set(cache_key, workflow.model_dump_json(), ex=300)

    return workflow
```

### 5.3 Logger (구조화 로깅)

```python
from common.utils import setup_logger

logger = setup_logger(__name__)

# 기본 로깅
logger.info("Workflow created successfully")

# 컨텍스트 로깅 (workflow_id 추가)
logger.info(
    "Processing workflow",
    extra={"workflow_id": "abc-123", "user_id": "user-456"}
)

# 에러 로깅
try:
    result = await service.create(data)
except Exception as e:
    logger.exception("Failed to create workflow", extra={"data": data})
    raise
```

**출력 (JSON 형식)**:
```json
{
  "timestamp": "2025-01-22T10:30:45.123456",
  "level": "INFO",
  "logger": "workflow_service",
  "message": "Processing workflow",
  "service": "workflow-service",
  "environment": "production",
  "workflow_id": "abc-123",
  "user_id": "user-456"
}
```

---

## 6. Middleware 활용

### 6.1 전역 설정 (main.py)

```python
from fastapi import FastAPI
from fastapi.exceptions import RequestValidationError

from common.exceptions import JudgifyException
from common.middleware import (
    setup_cors,
    global_exception_handler,
    validation_exception_handler,
)

app = FastAPI(title="Workflow Service", version="0.1.0")

# CORS 설정
setup_cors(app)

# 전역 예외 처리기 등록
app.add_exception_handler(JudgifyException, global_exception_handler)
app.add_exception_handler(RequestValidationError, validation_exception_handler)
```

### 6.2 JWT 인증

```python
from fastapi import Depends
from common.middleware import get_current_user, require_roles

# 인증 필수 엔드포인트
@app.get("/me")
async def get_current_user_info(user_id: str = Depends(get_current_user)):
    return {"user_id": user_id}

# 특정 역할 필수 (RBAC)
@app.delete("/workflows/{id}")
async def delete_workflow(
    id: UUID,
    user_id: str = Depends(require_roles("admin"))  # admin만 삭제 가능
):
    service = WorkflowService(db)
    await service.delete(id)
    return {"message": "Deleted successfully"}
```

---

## 7. Exception Handling

### 7.1 사용 가능한 예외

```python
from common.exceptions import (
    JudgifyException,     # 기본 예외
    ValidationError,      # 400 Bad Request
    NotFoundError,        # 404 Not Found
    UnauthorizedError,    # 401 Unauthorized
)

# ValidationError 사용
if not workflow_id:
    raise ValidationError("Workflow ID is required")

# NotFoundError 사용
workflow = await repository.find_by_id(id)
if not workflow:
    raise NotFoundError(resource="Workflow", id=str(id))

# UnauthorizedError 사용
if not user_has_permission:
    raise UnauthorizedError("Insufficient permissions")
```

### 7.2 자동 JSON 응답

예외 발생시 **자동으로 JSON 응답** 생성:

```json
{
  "error": "NotFoundError",
  "message": "Workflow abc-123 not found",
  "status_code": 404,
  "path": "/api/v2/workflows/abc-123"
}
```

---

## 8. 실전 예시: Workflow Service

### 전체 코드 (services/workflow/)

```
services/workflow/
├── app/
│   ├── main.py                # FastAPI 앱 진입점
│   ├── models/
│   │   ├── schemas.py         # Pydantic 모델 (BaseEntity 상속)
│   │   └── db_models.py       # SQLAlchemy ORM 모델
│   ├── repositories/
│   │   └── workflow_repository.py  # BaseRepository 상속
│   ├── services/
│   │   └── workflow_service.py     # BaseService 상속
│   └── routers/
│       └── api.py             # API 엔드포인트
├── pyproject.toml             # judgify-common 의존성
└── tests/
    └── test_workflow_service.py
```

### main.py

```python
from fastapi import FastAPI
from common.middleware import setup_cors, global_exception_handler
from common.exceptions import JudgifyException
from common.utils import init_database, close_database
from app.routers import api

app = FastAPI(title="Workflow Service", version="0.1.0")

# CORS 설정
setup_cors(app)

# 예외 처리
app.add_exception_handler(JudgifyException, global_exception_handler)

# 라우터 등록
app.include_router(api.router, prefix="/api/v2/workflows", tags=["workflows"])

# Startup
@app.on_event("startup")
async def startup():
    await init_database()

# Shutdown
@app.on_event("shutdown")
async def shutdown():
    await close_database()

# Health check
@app.get("/health")
async def health_check():
    return {"status": "healthy", "service": "workflow-service"}
```

### routers/api.py

```python
from fastapi import APIRouter, Depends
from uuid import UUID
from sqlalchemy.ext.asyncio import AsyncSession

from common.utils import get_database
from common.middleware import get_current_user
from app.services.workflow_service import WorkflowService
from app.models.schemas import WorkflowCreate, WorkflowUpdate, WorkflowResponse

router = APIRouter()

@router.post("/", response_model=WorkflowResponse, status_code=201)
async def create_workflow(
    data: WorkflowCreate,
    db: AsyncSession = Depends(get_database),
    user_id: str = Depends(get_current_user)
):
    """워크플로우 생성"""
    service = WorkflowService(db)
    return await service.create(data)  # Base 메서드 재사용!

@router.get("/{id}", response_model=WorkflowResponse)
async def get_workflow(
    id: UUID,
    db: AsyncSession = Depends(get_database)
):
    """워크플로우 조회"""
    service = WorkflowService(db)
    return await service.get_by_id(id)  # Base 메서드 재사용!

@router.get("/", response_model=list[WorkflowResponse])
async def get_workflows(
    skip: int = 0,
    limit: int = 100,
    db: AsyncSession = Depends(get_database)
):
    """워크플로우 목록 조회"""
    service = WorkflowService(db)
    return await service.get_all(skip=skip, limit=limit)  # Base 메서드 재사용!

@router.put("/{id}", response_model=WorkflowResponse)
async def update_workflow(
    id: UUID,
    data: WorkflowUpdate,
    db: AsyncSession = Depends(get_database),
    user_id: str = Depends(get_current_user)
):
    """워크플로우 수정"""
    service = WorkflowService(db)
    return await service.update(id, data)  # Base 메서드 재사용!

@router.delete("/{id}", status_code=204)
async def delete_workflow(
    id: UUID,
    db: AsyncSession = Depends(get_database),
    user_id: str = Depends(get_current_user)
):
    """워크플로우 삭제"""
    service = WorkflowService(db)
    await service.delete(id)  # Base 메서드 재사용!
```

### 재사용률: 91%!

**코드 분석**:
- Base Service: 85% (CRUD 5개 메서드 무료)
- Base Repository: 80% (데이터 액세스 자동)
- Utils: 100% (DB, Logger 자동)
- Middleware: 100% (CORS, Auth, Error 자동)
- **총 평균: 91%**

---

## 9. 테스트 전략

### 9.1 서비스 테스트

```python
# tests/test_workflow_service.py
import pytest
from uuid import uuid4
from app.services.workflow_service import WorkflowService
from app.models.schemas import WorkflowCreate

@pytest.mark.asyncio
async def test_create_workflow(db_session):
    """워크플로우 생성 테스트"""
    service = WorkflowService(db_session)

    data = WorkflowCreate(
        name="Test Workflow",
        description="Test",
        definition={"nodes": []},
        is_active=True
    )

    result = await service.create(data)

    assert result.name == "Test Workflow"
    assert result.id is not None
    assert result.created_at is not None

@pytest.mark.asyncio
async def test_get_workflow_not_found(db_session):
    """존재하지 않는 워크플로우 조회 테스트"""
    service = WorkflowService(db_session)

    with pytest.raises(NotFoundError):
        await service.get_by_id(uuid4())
```

### 9.2 Common Library 테스트

**중요**: `common/` 라이브러리는 **90% 이상 테스트 커버리지 필수**!

```bash
# common/ 디렉토리에서
pytest --cov=common --cov-report=term --cov-fail-under=90
```

---

## 10. 주의사항 및 베스트 프랙티스

### 10.1 80/20 법칙

**✅ common/으로 이동**:
- 80% 이상 서비스에서 사용하는 코드
- 인프라 로직 (DB, Cache, Logger, Auth)
- 공통 패턴 (CRUD, Pagination, Error Handling)

**❌ common/으로 이동하지 말 것**:
- 비즈니스 로직 (각 서비스 고유)
- 도메인 특화 알고리즘
- 서비스별 데이터 모델

### 10.2 의존성 방향

**절대 규칙**: `common/ ← services/` (단방향!)

```python
# ✅ 올바른 방향
# services/workflow/app/services/workflow_service.py
from common.base import BaseService  # OK!

# ❌ 절대 금지!
# common/base/base_service.py
from services.workflow import WorkflowService  # 순환 의존성!
```

### 10.3 테스트 커버리지

**공유 코드는 품질이 생명**:
- `common/`: 90% 이상 필수
- `services/`: 80% 이상 권장
- 버그 1개 = 9개 서비스 모두 영향!

### 10.4 버전 관리 (Living at HEAD)

```toml
# services/*/pyproject.toml
[tool.poetry.dependencies]
judgify-common = { path = "../../common", develop = true }

# develop = true → 항상 최신 코드 참조!
```

**장점**:
- 버전 충돌 없음
- 변경 즉시 반영
- 단일 저장소 (Monorepo) 최적화

**단점**:
- Breaking change 주의 (모든 서비스 영향)
- 테스트 커버리지 필수

---

## 📚 참고 자료

- [CLAUDE.md 섹션 16: 코드 재사용 전략](../../CLAUDE.md#-16-코드-재사용-전략-common-library)
- [Common Library README](../../common/README.md)
- [API Specifications](../architecture/api_specifications.md)
- [Database Design](../architecture/database_design.md)

---

**작성**: Claude Code + AI Agents
**마지막 업데이트**: 2025-01-22
