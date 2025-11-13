---
name: generate-service
description: Generate Service class inheriting from BaseService for business logic
---

Generate Service class that inherits from BaseService with automatic CRUD business logic.

## 🎯 언제 사용하나요?

### ✅ 사용 조건
- 비즈니스 로직 레이어 필요
- BaseService 상속으로 기본 CRUD 로직 자동 포함
- 고유 비즈니스 로직만 추가 구현하고 싶을 때

### ❌ 사용하지 말아야 할 경우
- 복잡한 도메인 설계 필요 → **Task tool로 ai-engineer Agent 사용**
- 이벤트 기반 아키텍처 → **Task tool로 graphql-architect Agent 사용**

---

## 📋 사용 방법

```bash
/generate-service EntityName service-name
```

**예시:**
```bash
/generate-service Workflow workflow-service
/generate-service JudgmentExecution judgment-service
```

---

## 🔧 생성되는 파일

### services/{service-name}/app/services/{entity}_service.py

```python
from uuid import UUID
from sqlalchemy.ext.asyncio import AsyncSession

from common.base import BaseService
from app.repositories.{entity}_repository import {EntityName}Repository
from app.models.schemas import (
    {EntityName}Create,
    {EntityName}Update,
    {EntityName}Response
)
from app.models.db_models import {EntityName}DBModel


class {EntityName}Service(BaseService[
    {EntityName}DBModel,
    {EntityName}Create,
    {EntityName}Update,
    {EntityName}Response
]):
    """
    Service for {EntityName} business logic

    Inherits from BaseService:
    - create(data: {EntityName}Create) -> {EntityName}Response
    - get_by_id(id: UUID) -> {EntityName}Response
    - get_all(skip: int = 0, limit: int = 100) -> List[{EntityName}Response]
    - update(id: UUID, data: {EntityName}Update) -> {EntityName}Response
    - delete(id: UUID) -> bool

    Add custom business logic below:
    """

    def __init__(self, db: AsyncSession):
        repository = {EntityName}Repository(db)
        super().__init__(db, repository)

    async def get_active(self) -> list[{EntityName}Response]:
        """
        Get all active {entity} entities

        Returns:
            List of active {entity} entities

        Example:
            service = {EntityName}Service(db)
            active_items = await service.get_active()
        """
        entities = await self.repository.find_active()
        return [{EntityName}Response.model_validate(e) for e in entities]

    async def find_by_name(self, name: str) -> {EntityName}Response | None:
        """
        Find {entity} by name

        Args:
            name: Entity name

        Returns:
            Entity if found, None otherwise

        Example:
            service = {EntityName}Service(db)
            item = await service.find_by_name("My Workflow")
        """
        entity = await self.repository.find_by_name(name)

        if not entity:
            return None

        return {EntityName}Response.model_validate(entity)

    # TODO: Add custom business logic here
    # Example:
    # async def simulate(self, id: UUID, test_data: dict):
    #     """Simulate workflow execution"""
    #     workflow = await self.get_by_id(id)  # Base 메서드 재사용!
    #
    #     # 비즈니스 로직 구현
    #     result = self._run_simulation(workflow, test_data)
    #
    #     self.logger.info(f"Simulation completed for {id}")
    #     return result
```

---

## 💡 BaseService가 제공하는 것

자동으로 포함되는 메서드 (85% 재사용!):
- ✅ `create(data)` - 엔티티 생성
- ✅ `get_by_id(id)` - ID로 조회
- ✅ `get_all(skip, limit)` - 전체 조회 (페이지네이션)
- ✅ `update(id, data)` - 수정
- ✅ `delete(id)` - 삭제
- ✅ 자동 로깅 (구조화 JSON 로그)
- ✅ 자동 예외 처리 (NotFoundError 자동 발생)

고유 비즈니스 로직만 추가:
- `get_active()` - 활성 엔티티만 조회
- `simulate()` - 워크플로우 시뮬레이션
- ... 비즈니스 요구사항에 따라 추가

---

## 🚀 다음 단계 추천

Service 생성 후:

1. **API 엔드포인트**: `/generate-api EntityName service-name`
2. **고유 비즈니스 로직 추가**: 도메인 요구사항 구현
3. **테스트 작성**: `/generate-tests`
4. **검증**: `/validate-architecture`

---

## 📊 재사용률

| 기능 | 재사용 여부 | 설명 |
|------|-----------|------|
| **CRUD 로직** | ✅ 100% | BaseService 상속 |
| **로깅** | ✅ 100% | 자동 구조화 로깅 |
| **예외 처리** | ✅ 100% | NotFoundError 자동 |
| **Pydantic 변환** | ✅ 100% | Schema 자동 변환 |
| **고유 로직** | ❌ 0% | 비즈니스별 구현 |
| **평균** | **85%** | 대부분 코드 재사용! |

---

## 💼 실전 사용 예시

### API 엔드포인트에서 사용

```python
from fastapi import APIRouter, Depends
from sqlalchemy.ext.asyncio import AsyncSession
from common.utils import get_database

router = APIRouter()

@router.post("/workflows")
async def create_workflow(
    data: WorkflowCreate,
    db: AsyncSession = Depends(get_database)
):
    service = WorkflowService(db)
    return await service.create(data)  # Base 메서드 재사용!

@router.get("/workflows/{id}")
async def get_workflow(
    id: UUID,
    db: AsyncSession = Depends(get_database)
):
    service = WorkflowService(db)
    return await service.get_by_id(id)  # Base 메서드 재사용!

@router.post("/workflows/{id}/simulate")
async def simulate_workflow(
    id: UUID,
    test_data: dict,
    db: AsyncSession = Depends(get_database)
):
    service = WorkflowService(db)
    return await service.simulate(id, test_data)  # 고유 메서드!
```

---

## 🔗 관련 리소스

- **Base Classes**: [common/base/base_service.py](../../common/common/base/base_service.py)
- **다음 Skill**: `/generate-api`
- **문서**: [docs/guides/code-reusability.md](../../docs/guides/code-reusability.md)
