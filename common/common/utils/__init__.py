"""
Utility functions for all microservices

Provides common utilities:
- database: PostgreSQL connection pool management
- cache: Redis client helpers
- logger: Structured logging setup
- validators: Input validation utilities
"""

from common.utils.database import get_database, AsyncSessionLocal
from common.utils.cache import get_redis_cache, RedisCache
from common.utils.logger import setup_logger
from common.utils.validators import validate_uuid, validate_email

__all__ = [
    # Database
    "get_database",
    "AsyncSessionLocal",
    # Cache
    "get_redis_cache",
    "RedisCache",
    # Logger
    "setup_logger",
    # Validators
    "validate_uuid",
    "validate_email",
]
