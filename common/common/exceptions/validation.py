"""
Validation exception for input validation errors

HTTP 400 Bad Request
"""

from common.exceptions.base import JudgifyException


class ValidationError(JudgifyException):
    """
    Exception for data validation failures

    HTTP Status: 400 Bad Request

    Usage:
        if not validate_email(email):
            raise ValidationError(f"Invalid email format: {email}")

        # With details
        raise ValidationError(
            "Validation failed",
            details={"field": "email", "error": "Invalid format"}
        )
    """

    def __init__(self, message: str, details: dict = None):
        super().__init__(message, status_code=400, details=details)
