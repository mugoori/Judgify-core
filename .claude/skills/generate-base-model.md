---
name: generate-base-model
description: Generate Pydantic models inheriting from BaseEntity for Judgify microservices
---

Generate Pydantic models that inherit from BaseEntity with automatic ID and timestamp fields.

## 🎯 언제 사용하나요?

### ✅ 사용 조건
- 새 엔티티 모델이 필요할 때
- BaseEntity 상속으로 공통 필드 (id, created_at, updated_at) 자동 포함
- CRUD API용 모델 세트 (Create, Update, Response) 생성

### ❌ 사용하지 말아야 할 경우
- 복잡한 도메인 모델 설계 → **Task tool로 ai-engineer Agent 사용**
- 데이터베이스 스키마 설계 → **Task tool로 database-optimization Agent 사용**

---

## 📋 사용 방법

```bash
/generate-base-model EntityName service-name
```

**예시:**
```bash
/generate-base-model Workflow workflow-service
/generate-base-model JudgmentExecution judgment-service
```

---

## 🔧 생성되는 파일

### services/{service-name}/app/models/schemas.py

```python
from datetime import datetime
from uuid import UUID
from typing import Optional
from pydantic import Field

from common.base import BaseEntity, BaseCreateModel, BaseUpdateModel, BaseResponseModel


# ========== Base Entity ==========
# 자동 포함: id, created_at, updated_at

class {EntityName}Base(BaseEntity):
    """Base schema for {EntityName} entity"""
    name: str = Field(..., min_length=1, max_length=255)
    description: Optional[str] = Field(None, max_length=1000)
    is_active: bool = Field(default=True)


# ========== Create Schema ==========
class {EntityName}Create(BaseCreateModel):
    """Schema for creating {EntityName} (without id, timestamps)"""
    name: str = Field(..., min_length=1, max_length=255)
    description: Optional[str] = None
    is_active: bool = True


# ========== Update Schema ==========
class {EntityName}Update(BaseUpdateModel):
    """Schema for updating {EntityName} (all fields optional)"""
    name: Optional[str] = Field(None, min_length=1, max_length=255)
    description: Optional[str] = None
    is_active: Optional[bool] = None


# ========== Response Schema ==========
class {EntityName}Response(BaseResponseModel):
    """Schema for {EntityName} API response (includes all fields)"""
    id: UUID
    name: str
    description: Optional[str]
    is_active: bool
    created_at: datetime
    updated_at: Optional[datetime]

    class Config:
        from_attributes = True  # SQLAlchemy ORM 호환
```

---

## 💡 BaseEntity가 제공하는 것

자동으로 포함되는 필드:
- `id: UUID` - 고유 식별자 (자동 생성)
- `created_at: datetime` - 생성 시간 (자동)
- `updated_at: Optional[datetime]` - 수정 시간 (자동)

자동으로 포함되는 메서드:
- `mark_updated()` - updated_at 필드 갱신

---

## 🚀 다음 단계 추천

모델 생성 후:

1. **SQLAlchemy ORM 모델 생성**: database-optimization Agent로 DB 모델 설계
2. **Repository 생성**: `/generate-repository EntityName service-name`
3. **Service 생성**: `/generate-service EntityName service-name`
4. **API 엔드포인트**: `/generate-api EntityName service-name`

---

## 📊 생성되는 모델 구조

```
{EntityName}Create
  └─ 입력: name, description, is_active
  └─ 사용: POST /api/v2/{service}/{entity}

{EntityName}Update
  └─ 입력: name?, description?, is_active? (모두 선택)
  └─ 사용: PUT /api/v2/{service}/{entity}/{id}

{EntityName}Response
  └─ 출력: id, name, description, is_active, created_at, updated_at
  └─ 사용: 모든 API 응답
```

---

## 🔗 관련 리소스

- **Base Classes**: [common/base/base_model.py](../../common/common/base/base_model.py)
- **다음 Skill**: `/generate-repository`, `/generate-service`
- **문서**: [docs/guides/code-reusability.md](../../docs/guides/code-reusability.md)
