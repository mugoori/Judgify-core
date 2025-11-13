"""
Base Service Layer for business logic

Provides common service operations that all services inherit.
Implements the Service Layer Pattern for clean architecture.
"""

from typing import TypeVar, Generic, List, Optional
from uuid import UUID
from sqlalchemy.ext.asyncio import AsyncSession
from pydantic import BaseModel

from common.base.base_repository import BaseRepository
from common.exceptions import NotFoundError
from common.utils.logger import setup_logger

T = TypeVar('T')  # Repository model type
CreateSchema = TypeVar('CreateSchema', bound=BaseModel)
UpdateSchema = TypeVar('UpdateSchema', bound=BaseModel)
ResponseSchema = TypeVar('ResponseSchema', bound=BaseModel)


class BaseService(Generic[T, CreateSchema, UpdateSchema, ResponseSchema]):
    """
    Generic service layer for business logic

    Type parameters:
        T: SQLAlchemy ORM model
        CreateSchema: Pydantic model for creation
        UpdateSchema: Pydantic model for updates
        ResponseSchema: Pydantic model for responses

    Example:
        class WorkflowService(BaseService[
            WorkflowDBModel,
            WorkflowCreate,
            WorkflowUpdate,
            WorkflowResponse
        ]):
            def __init__(self, db: AsyncSession):
                super().__init__(db, WorkflowRepository(db, WorkflowDBModel))

            async def simulate(self, workflow_id: UUID, test_data: dict):
                # Custom business logic
                workflow = await self.get_by_id(workflow_id)
                # ... simulation logic
    """

    def __init__(self, db: AsyncSession, repository: BaseRepository[T]):
        """
        Initialize service with database session and repository

        Args:
            db: SQLAlchemy async session
            repository: Repository instance for data access
        """
        self.db = db
        self.repository = repository
        self.logger = setup_logger(self.__class__.__name__)

    async def create(self, data: CreateSchema) -> ResponseSchema:
        """
        Create new entity

        Args:
            data: Creation data (Pydantic model)

        Returns:
            Created entity as response model

        Raises:
            ValidationError: If data validation fails
        """
        self.logger.info(f"Creating {self.__class__.__name__} with data: {data}")

        # Convert Pydantic model to SQLAlchemy model
        entity = self.repository.model(**data.model_dump())

        # Save to database
        saved_entity = await self.repository.save(entity)

        # Convert to response schema
        return ResponseSchema.model_validate(saved_entity)

    async def get_by_id(self, id: UUID) -> ResponseSchema:
        """
        Get entity by ID

        Args:
            id: Entity UUID

        Returns:
            Entity as response model

        Raises:
            NotFoundError: If entity not found
        """
        self.logger.debug(f"Getting {self.__class__.__name__} by ID: {id}")

        entity = await self.repository.find_by_id(id)

        if not entity:
            raise NotFoundError(
                resource=self.__class__.__name__.replace("Service", ""),
                id=str(id)
            )

        return ResponseSchema.model_validate(entity)

    async def get_all(self, skip: int = 0, limit: int = 100) -> List[ResponseSchema]:
        """
        Get all entities with pagination

        Args:
            skip: Number of records to skip (default: 0)
            limit: Maximum number of records to return (default: 100)

        Returns:
            List of entities as response models
        """
        self.logger.debug(f"Getting all {self.__class__.__name__} (skip={skip}, limit={limit})")

        entities = await self.repository.find_all(skip=skip, limit=limit)

        return [ResponseSchema.model_validate(entity) for entity in entities]

    async def update(self, id: UUID, data: UpdateSchema) -> ResponseSchema:
        """
        Update entity by ID

        Args:
            id: Entity UUID
            data: Update data (Pydantic model, only provided fields)

        Returns:
            Updated entity as response model

        Raises:
            NotFoundError: If entity not found
        """
        self.logger.info(f"Updating {self.__class__.__name__} {id} with data: {data}")

        # Check if entity exists
        exists = await self.repository.exists(id)
        if not exists:
            raise NotFoundError(
                resource=self.__class__.__name__.replace("Service", ""),
                id=str(id)
            )

        # Convert Pydantic model to dict, excluding None values
        update_data = data.model_dump(exclude_unset=True)

        # Update in database
        updated_entity = await self.repository.update_by_id(id, update_data)

        return ResponseSchema.model_validate(updated_entity)

    async def delete(self, id: UUID) -> bool:
        """
        Delete entity by ID

        Args:
            id: Entity UUID

        Returns:
            True if deleted successfully

        Raises:
            NotFoundError: If entity not found
        """
        self.logger.info(f"Deleting {self.__class__.__name__} {id}")

        success = await self.repository.delete_by_id(id)

        if not success:
            raise NotFoundError(
                resource=self.__class__.__name__.replace("Service", ""),
                id=str(id)
            )

        return True
