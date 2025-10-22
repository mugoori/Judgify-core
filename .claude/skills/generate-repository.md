---
name: generate-repository
description: Generate Repository class inheriting from BaseRepository for data access
---

Generate Repository class that inherits from BaseRepository with automatic CRUD operations.

## 🎯 언제 사용하나요?

### ✅ 사용 조건
- 데이터베이스 액세스 레이어 필요
- BaseRepository 상속으로 기본 CRUD 메서드 자동 포함
- 고유 쿼리만 추가 구현하고 싶을 때

### ❌ 사용하지 말아야 할 경우
- 복잡한 쿼리 최적화 필요 → **Task tool로 database-optimization Agent 사용**
- NoSQL 데이터베이스 사용 → Repository 패턴 직접 구현

---

## 📋 사용 방법

```bash
/generate-repository EntityName service-name
```

**예시:**
```bash
/generate-repository Workflow workflow-service
/generate-repository JudgmentExecution judgment-service
```

---

## 🔧 생성되는 파일

### services/{service-name}/app/repositories/{entity}_repository.py

```python
from typing import List, Optional
from uuid import UUID
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select

from common.base import BaseRepository
from app.models.db_models import {EntityName}DBModel


class {EntityName}Repository(BaseRepository[{EntityName}DBModel]):
    """
    Repository for {EntityName} data access

    Inherits from BaseRepository:
    - find_by_id(id: UUID) -> Optional[{EntityName}DBModel]
    - find_all(skip: int = 0, limit: int = 100) -> List[{EntityName}DBModel]
    - save(entity: {EntityName}DBModel) -> {EntityName}DBModel
    - update_by_id(id: UUID, data: dict) -> Optional[{EntityName}DBModel]
    - delete_by_id(id: UUID) -> bool
    - exists(id: UUID) -> bool

    Add custom queries below:
    """

    def __init__(self, db: AsyncSession):
        super().__init__(db, {EntityName}DBModel)

    async def find_active(self) -> List[{EntityName}DBModel]:
        """
        Find all active {entity} entities

        Returns:
            List of active {entity} entities

        Example:
            repo = {EntityName}Repository(db)
            active_items = await repo.find_active()
        """
        result = await self.db.execute(
            select(self.model).where(self.model.is_active == True)
        )
        return list(result.scalars().all())

    async def find_by_name(self, name: str) -> Optional[{EntityName}DBModel]:
        """
        Find {entity} by name

        Args:
            name: Entity name

        Returns:
            Entity if found, None otherwise

        Example:
            repo = {EntityName}Repository(db)
            item = await repo.find_by_name("My Workflow")
        """
        result = await self.db.execute(
            select(self.model).where(self.model.name == name)
        )
        return result.scalar_one_or_none()

    # TODO: Add more custom queries here
    # Example:
    # async def find_by_user(self, user_id: UUID):
    #     result = await self.db.execute(
    #         select(self.model).where(self.model.user_id == user_id)
    #     )
    #     return list(result.scalars().all())
```

---

## 💡 BaseRepository가 제공하는 것

자동으로 포함되는 메서드 (80% 재사용!):
- ✅ `find_by_id(id)` - ID로 조회
- ✅ `find_all(skip, limit)` - 전체 조회 (페이지네이션)
- ✅ `save(entity)` - 저장
- ✅ `update_by_id(id, data)` - 수정
- ✅ `delete_by_id(id)` - 삭제
- ✅ `exists(id)` - 존재 여부 확인

고유 쿼리만 추가 구현:
- `find_active()` - 활성 엔티티만 조회
- `find_by_name(name)` - 이름으로 조회
- ... 비즈니스 요구사항에 따라 추가

---

## 🚀 다음 단계 추천

Repository 생성 후:

1. **Service 생성**: `/generate-service EntityName service-name`
2. **API 엔드포인트**: `/generate-api EntityName service-name`
3. **테스트 작성**: `/generate-tests`
4. **고유 쿼리 추가**: 비즈니스 로직에 필요한 메서드 구현

---

## 📊 재사용률

| 기능 | 재사용 여부 | 설명 |
|------|-----------|------|
| **기본 CRUD** | ✅ 100% | BaseRepository 상속 |
| **페이지네이션** | ✅ 100% | find_all() 자동 지원 |
| **에러 처리** | ✅ 100% | IntegrityError 자동 처리 |
| **트랜잭션** | ✅ 100% | AsyncSession 자동 관리 |
| **고유 쿼리** | ❌ 0% | 비즈니스 로직별 구현 |
| **평균** | **80%** | 대부분 코드 재사용! |

---

## 🔗 관련 리소스

- **Base Classes**: [common/base/base_repository.py](../../common/common/base/base_repository.py)
- **다음 Skill**: `/generate-service`
- **문서**: [docs/guides/code-reusability.md](../../docs/guides/code-reusability.md)
