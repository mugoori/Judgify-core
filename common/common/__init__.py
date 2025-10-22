"""
Judgify Common Library - 공유 라이브러리

이 패키지는 모든 마이크로서비스에서 공통으로 사용하는 기능을 제공합니다.

Modules:
    - base: 추상 클래스 (BaseService, BaseRepository, BaseModel)
    - utils: 유틸리티 (database, cache, logger, validators)
    - middleware: FastAPI 미들웨어 (auth, cors, error_handler)
    - exceptions: 커스텀 예외 클래스

Usage:
    from common.base import BaseService
    from common.utils import get_database
    from common.exceptions import NotFoundError
"""

__version__ = "0.1.0"

from common.base import BaseService, BaseRepository, BaseModel
from common.utils import get_database, get_redis_cache, setup_logger
from common.exceptions import (
    JudgifyException,
    ValidationError,
    NotFoundError,
    UnauthorizedError
)

__all__ = [
    # Base classes
    "BaseService",
    "BaseRepository",
    "BaseModel",
    # Utils
    "get_database",
    "get_redis_cache",
    "setup_logger",
    # Exceptions
    "JudgifyException",
    "ValidationError",
    "NotFoundError",
    "UnauthorizedError",
]
