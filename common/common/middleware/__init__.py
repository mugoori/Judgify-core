"""
FastAPI middleware for all microservices

Provides common middleware:
- auth: JWT authentication
- cors: CORS configuration
- error_handler: Global exception handling
"""

from common.middleware.auth import get_current_user, verify_token
from common.middleware.cors import setup_cors
from common.middleware.error_handler import (
    global_exception_handler,
    validation_exception_handler,
)

__all__ = [
    # Auth
    "get_current_user",
    "verify_token",
    # CORS
    "setup_cors",
    # Error handlers
    "global_exception_handler",
    "validation_exception_handler",
]
