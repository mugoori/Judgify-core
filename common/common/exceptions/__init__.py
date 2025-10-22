"""
Custom exception classes for all microservices

Provides consistent exception hierarchy with HTTP status codes.
"""

from common.exceptions.base import JudgifyException
from common.exceptions.validation import ValidationError
from common.exceptions.not_found import NotFoundError
from common.exceptions.unauthorized import UnauthorizedError

__all__ = [
    "JudgifyException",
    "ValidationError",
    "NotFoundError",
    "UnauthorizedError",
]
