---
name: generate-api
description: Generate CRUD API endpoints for a FastAPI service following RESTful patterns
---

Generate CRUD (Create, Read, Update, Delete) API endpoints for a FastAPI microservice.

## 🎯 언제 사용하나요?

### ✅ 사용 조건
- 표준 CRUD API 엔드포인트가 필요할 때
- RESTful 패턴을 빠르게 구현하고 싶을 때
- Pydantic 모델 기반 데이터 검증이 필요할 때

### ❌ 사용하지 말아야 할 경우
- 복잡한 비즈니스 로직이 있는 API → **ai-engineer Agent가 설계 필요**
- GraphQL API 설계 → **Task tool로 graphql-architect Agent 사용**
- 보안이 중요한 API (결제, 인증 등) → **security-engineer Agent 검토 필요**

---

## 📋 사용 방법

```bash
/generate-api model-name service-name
```

**예시:**
```bash
/generate-api Workflow workflow-service
/generate-api JudgmentExecution judgment-service
```

---

## 🔧 생성되는 코드

### 1. Pydantic 스키마 (app/models/schemas.py)

```python
from pydantic import BaseModel, Field
from typing import Optional
from datetime import datetime
from uuid import UUID

class {ModelName}Base(BaseModel):
    """Base schema for {ModelName}"""
    name: str = Field(..., min_length=1, max_length=255)
    description: Optional[str] = None
    is_active: bool = True

class {ModelName}Create({ModelName}Base):
    """Schema for creating {ModelName}"""
    pass

class {ModelName}Update(BaseModel):
    """Schema for updating {ModelName}"""
    name: Optional[str] = Field(None, min_length=1, max_length=255)
    description: Optional[str] = None
    is_active: Optional[bool] = None

class {ModelName}Response({ModelName}Base):
    """Schema for {ModelName} response"""
    id: UUID
    created_at: datetime
    updated_at: Optional[datetime] = None

    class Config:
        from_attributes = True
```

### 2. API 라우터 (app/routers/api.py)

```python
from fastapi import APIRouter, Depends, HTTPException, status
from typing import List
from uuid import UUID
from sqlalchemy.ext.asyncio import AsyncSession

from app.models.schemas import (
    {ModelName}Create,
    {ModelName}Update,
    {ModelName}Response
)
from app.dependencies import get_database
from app.services.core import {ModelName}Service

router = APIRouter(tags=["{model-name}"])

@router.post(
    "/{model-name}",
    response_model={ModelName}Response,
    status_code=status.HTTP_201_CREATED,
    summary="Create new {model-name}"
)
async def create_{model_name}(
    data: {ModelName}Create,
    db: AsyncSession = Depends(get_database)
):
    """
    Create a new {model-name}:
    - **name**: Required, 1-255 characters
    - **description**: Optional description
    - **is_active**: Boolean, defaults to True
    """
    service = {ModelName}Service(db)
    return await service.create(data)

@router.get(
    "/{model-name}",
    response_model=List[{ModelName}Response],
    summary="Get all {model-name}s"
)
async def get_{model_name}s(
    skip: int = 0,
    limit: int = 100,
    db: AsyncSession = Depends(get_database)
):
    """
    Retrieve all {model-name}s with pagination:
    - **skip**: Number of records to skip (default: 0)
    - **limit**: Maximum records to return (default: 100)
    """
    service = {ModelName}Service(db)
    return await service.get_all(skip=skip, limit=limit)

@router.get(
    "/{model-name}/{id}",
    response_model={ModelName}Response,
    summary="Get {model-name} by ID"
)
async def get_{model_name}(
    id: UUID,
    db: AsyncSession = Depends(get_database)
):
    """
    Retrieve a specific {model-name} by ID
    """
    service = {ModelName}Service(db)
    result = await service.get_by_id(id)
    if not result:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"{ModelName} not found"
        )
    return result

@router.put(
    "/{model-name}/{id}",
    response_model={ModelName}Response,
    summary="Update {model-name}"
)
async def update_{model_name}(
    id: UUID,
    data: {ModelName}Update,
    db: AsyncSession = Depends(get_database)
):
    """
    Update an existing {model-name}:
    - All fields are optional
    - Only provided fields will be updated
    """
    service = {ModelName}Service(db)
    result = await service.update(id, data)
    if not result:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"{ModelName} not found"
        )
    return result

@router.delete(
    "/{model-name}/{id}",
    status_code=status.HTTP_204_NO_CONTENT,
    summary="Delete {model-name}"
)
async def delete_{model_name}(
    id: UUID,
    db: AsyncSession = Depends(get_database)
):
    """
    Delete a {model-name} by ID
    """
    service = {ModelName}Service(db)
    success = await service.delete(id)
    if not success:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail=f"{ModelName} not found"
        )
```

### 3. 서비스 레이어 (app/services/core.py)

```python
from typing import List, Optional
from uuid import UUID
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from app.models.schemas import {ModelName}Create, {ModelName}Update
# from app.models.db_models import {ModelName}  # SQLAlchemy model

class {ModelName}Service:
    def __init__(self, db: AsyncSession):
        self.db = db

    async def create(self, data: {ModelName}Create):
        """Create new {model-name}"""
        # TODO: Implement database insertion
        pass

    async def get_all(self, skip: int = 0, limit: int = 100):
        """Get all {model-name}s with pagination"""
        # TODO: Implement database query
        pass

    async def get_by_id(self, id: UUID):
        """Get {model-name} by ID"""
        # TODO: Implement database query
        pass

    async def update(self, id: UUID, data: {ModelName}Update):
        """Update {model-name}"""
        # TODO: Implement database update
        pass

    async def delete(self, id: UUID) -> bool:
        """Delete {model-name}"""
        # TODO: Implement database deletion
        pass
```

---

## 📊 생성되는 API 엔드포인트

| Method | Endpoint | Description | Status Code |
|--------|----------|-------------|-------------|
| POST | `/api/v2/{service}/{model}` | Create new record | 201 |
| GET | `/api/v2/{service}/{model}` | Get all records | 200 |
| GET | `/api/v2/{service}/{model}/{id}` | Get by ID | 200 |
| PUT | `/api/v2/{service}/{model}/{id}` | Update record | 200 |
| DELETE | `/api/v2/{service}/{model}/{id}` | Delete record | 204 |

---

## 🚀 다음 단계 추천

API 엔드포인트 생성 후:

1. **서비스 레이어 구현**: `app/services/core.py`에 실제 비즈니스 로직 작성
2. **데이터베이스 모델 생성**: database-optimization Agent로 SQLAlchemy 모델 설계
3. **테스트 생성**: `/generate-tests` Skill로 API 테스트 작성
4. **API 문서 동기화**: `/sync-docs` Skill로 OpenAPI 문서 업데이트
5. **성능 테스트**: `/run-load-test` Skill로 부하 테스트 실행

---

## 💡 주의사항

- **데이터 검증**: Pydantic 모델이 자동으로 입력 검증 수행
- **에러 처리**: HTTPException으로 표준 HTTP 에러 반환
- **페이지네이션**: GET all 엔드포인트는 기본 100개 제한
- **UUID 사용**: 모든 ID는 UUID 타입 (보안 강화)

---

## 🔗 관련 리소스

- **Agent 활용**: database-optimization (DB 모델), security-engineer (보안 검토)
- **다음 Skill**: `/generate-tests`, `/sync-docs`
- **문서**: [docs/architecture/api_specifications.md](../../docs/architecture/api_specifications.md)
