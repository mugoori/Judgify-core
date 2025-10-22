"""
Base classes for all microservices

This module provides abstract base classes that implement common patterns:
- BaseService: Service layer with CRUD operations
- BaseRepository: Data access layer with database operations
- BaseModel: Pydantic models with common fields
"""

from common.base.base_service import BaseService
from common.base.base_repository import BaseRepository
from common.base.base_model import BaseModel, BaseEntity

__all__ = [
    "BaseService",
    "BaseRepository",
    "BaseModel",
    "BaseEntity",
]
