"""
Unauthorized exception for authentication/authorization errors

HTTP 401 Unauthorized
"""

from common.exceptions.base import JudgifyException


class UnauthorizedError(JudgifyException):
    """
    Exception for authentication and authorization failures

    HTTP Status: 401 Unauthorized

    Usage:
        # Authentication failure
        raise UnauthorizedError("Invalid credentials")

        # Authorization failure (RBAC)
        raise UnauthorizedError("Insufficient permissions. Admin role required")

        # Token errors
        raise UnauthorizedError("Invalid token", details={"error": "expired"})
    """

    def __init__(self, message: str = "Unauthorized", details: dict = None):
        super().__init__(message, status_code=401, details=details)
