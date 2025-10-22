"""
Base Repository Pattern for data access layer

Provides common database operations (CRUD) that all repositories inherit.
Uses SQLAlchemy async sessions for PostgreSQL operations.
"""

from typing import TypeVar, Generic, Type, List, Optional
from uuid import UUID
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy import select, update, delete
from sqlalchemy.exc import IntegrityError

from common.exceptions import NotFoundError, ValidationError

T = TypeVar('T')  # SQLAlchemy model type


class BaseRepository(Generic[T]):
    """
    Generic repository for data access operations

    Type parameter T should be a SQLAlchemy ORM model

    Example:
        class WorkflowRepository(BaseRepository[WorkflowDBModel]):
            async def find_active(self):
                # Custom query for active workflows
                result = await self.db.execute(
                    select(self.model).where(self.model.is_active == True)
                )
                return result.scalars().all()
    """

    def __init__(self, db: AsyncSession, model: Type[T]):
        """
        Initialize repository with database session and model class

        Args:
            db: SQLAlchemy async session
            model: SQLAlchemy ORM model class
        """
        self.db = db
        self.model = model

    async def find_by_id(self, id: UUID) -> Optional[T]:
        """
        Find entity by ID

        Args:
            id: Entity UUID

        Returns:
            Entity if found, None otherwise
        """
        result = await self.db.execute(
            select(self.model).where(self.model.id == id)
        )
        return result.scalar_one_or_none()

    async def find_all(self, skip: int = 0, limit: int = 100) -> List[T]:
        """
        Find all entities with pagination

        Args:
            skip: Number of records to skip (default: 0)
            limit: Maximum number of records to return (default: 100)

        Returns:
            List of entities
        """
        result = await self.db.execute(
            select(self.model).offset(skip).limit(limit)
        )
        return list(result.scalars().all())

    async def save(self, entity: T) -> T:
        """
        Save new entity to database

        Args:
            entity: Entity instance to save

        Returns:
            Saved entity with generated ID

        Raises:
            ValidationError: If entity validation fails or constraint violation
        """
        try:
            self.db.add(entity)
            await self.db.commit()
            await self.db.refresh(entity)
            return entity
        except IntegrityError as e:
            await self.db.rollback()
            raise ValidationError(f"Database constraint violation: {str(e)}")

    async def update_by_id(self, id: UUID, data: dict) -> Optional[T]:
        """
        Update entity by ID

        Args:
            id: Entity UUID
            data: Dictionary of fields to update

        Returns:
            Updated entity if found, None otherwise
        """
        result = await self.db.execute(
            update(self.model)
            .where(self.model.id == id)
            .values(**data)
            .returning(self.model)
        )
        await self.db.commit()
        return result.scalar_one_or_none()

    async def delete_by_id(self, id: UUID) -> bool:
        """
        Delete entity by ID

        Args:
            id: Entity UUID

        Returns:
            True if deleted, False if not found
        """
        result = await self.db.execute(
            delete(self.model).where(self.model.id == id)
        )
        await self.db.commit()
        return result.rowcount > 0

    async def exists(self, id: UUID) -> bool:
        """
        Check if entity exists

        Args:
            id: Entity UUID

        Returns:
            True if exists, False otherwise
        """
        result = await self.db.execute(
            select(self.model.id).where(self.model.id == id)
        )
        return result.scalar_one_or_none() is not None
