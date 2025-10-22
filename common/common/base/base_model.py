"""
Base Pydantic models for all microservices

Provides common fields and configurations that all entities share.
"""

from datetime import datetime
from typing import Optional
from uuid import UUID, uuid4
from pydantic import BaseModel as PydanticBaseModel, Field, ConfigDict


class BaseModel(PydanticBaseModel):
    """Base Pydantic model with common configuration"""

    model_config = ConfigDict(
        from_attributes=True,  # SQLAlchemy ORM 호환
        populate_by_name=True,  # 필드명 자동 변환
        use_enum_values=True,   # Enum 값 자동 변환
        validate_assignment=True,  # 할당시에도 검증
    )


class BaseEntity(BaseModel):
    """Base entity model with common fields (id, created_at, updated_at)"""

    id: UUID = Field(default_factory=uuid4, description="Unique identifier")
    created_at: datetime = Field(default_factory=datetime.utcnow, description="Creation timestamp")
    updated_at: Optional[datetime] = Field(default=None, description="Last update timestamp")

    def mark_updated(self) -> None:
        """Mark entity as updated with current timestamp"""
        self.updated_at = datetime.utcnow()


class BaseCreateModel(BaseModel):
    """Base model for creation requests (without id, timestamps)"""
    pass


class BaseUpdateModel(BaseModel):
    """Base model for update requests (all fields optional)"""
    pass


class BaseResponseModel(BaseEntity):
    """Base model for API responses (includes all entity fields)"""
    pass
