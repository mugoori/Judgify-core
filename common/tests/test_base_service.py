"""
Tests for BaseService

Tests common CRUD operations and service layer patterns.
"""

import pytest
from uuid import uuid4
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from common.base import BaseService, BaseRepository
from common.exceptions import NotFoundError


# Mock models for testing
class MockEntity:
    def __init__(self, id, name):
        self.id = id
        self.name = name


class MockCreate(BaseModel):
    name: str


class MockUpdate(BaseModel):
    name: str | None = None


class MockResponse(BaseModel):
    id: str
    name: str

    class Config:
        from_attributes = True


class MockRepository(BaseRepository[MockEntity]):
    def __init__(self):
        self.storage = {}

    async def save(self, entity: MockEntity):
        self.storage[entity.id] = entity
        return entity

    async def find_by_id(self, id):
        return self.storage.get(id)

    async def find_all(self, skip=0, limit=100):
        items = list(self.storage.values())
        return items[skip:skip+limit]

    async def update_by_id(self, id, data):
        entity = self.storage.get(id)
        if entity:
            for key, value in data.items():
                setattr(entity, key, value)
        return entity

    async def delete_by_id(self, id):
        if id in self.storage:
            del self.storage[id]
            return True
        return False

    async def exists(self, id):
        return id in self.storage


class MockService(BaseService[MockEntity, MockCreate, MockUpdate, MockResponse]):
    def __init__(self):
        self.repository = MockRepository()
        self.db = None
        self.logger = None


@pytest.mark.asyncio
async def test_create_entity():
    """Test creating new entity"""
    service = MockService()
    data = MockCreate(name="Test Workflow")

    # Mock the model creation
    entity_id = str(uuid4())
    service.repository.model = lambda **kwargs: MockEntity(id=entity_id, **kwargs)

    result = await service.create(data)

    assert result.name == "Test Workflow"
    assert result.id == entity_id


@pytest.mark.asyncio
async def test_get_by_id_found():
    """Test getting entity by ID when exists"""
    service = MockService()
    entity_id = str(uuid4())

    # Add entity to storage
    entity = MockEntity(id=entity_id, name="Test")
    service.repository.storage[entity_id] = entity

    # Mock ResponseSchema
    service.repository.model = MockEntity
    MockService.__orig_bases__ = (BaseService[MockEntity, MockCreate, MockUpdate, MockResponse],)

    result = await service.get_by_id(entity_id)

    assert result.id == entity_id
    assert result.name == "Test"


@pytest.mark.asyncio
async def test_get_by_id_not_found():
    """Test getting entity by ID when not exists"""
    service = MockService()
    entity_id = str(uuid4())

    with pytest.raises(NotFoundError) as exc_info:
        await service.get_by_id(entity_id)

    assert exc_info.value.status_code == 404


@pytest.mark.asyncio
async def test_get_all_with_pagination():
    """Test getting all entities with pagination"""
    service = MockService()

    # Add entities
    for i in range(5):
        entity = MockEntity(id=str(uuid4()), name=f"Entity {i}")
        service.repository.storage[entity.id] = entity

    results = await service.get_all(skip=1, limit=2)

    assert len(results) == 2
