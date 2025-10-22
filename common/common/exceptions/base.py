"""
Base exception class for all Judgify exceptions

All custom exceptions inherit from JudgifyException.
"""


class JudgifyException(Exception):
    """
    Base exception for all Judgify custom exceptions

    Attributes:
        message: Error message
        status_code: HTTP status code
        details: Additional error details (optional)

    Usage:
        raise JudgifyException("Something went wrong", status_code=500)
    """

    def __init__(
        self,
        message: str,
        status_code: int = 500,
        details: dict = None
    ):
        self.message = message
        self.status_code = status_code
        self.details = details or {}
        super().__init__(self.message)

    def __str__(self) -> str:
        return f"[{self.status_code}] {self.message}"

    def __repr__(self) -> str:
        return (
            f"{self.__class__.__name__}("
            f"message={self.message!r}, "
            f"status_code={self.status_code}, "
            f"details={self.details!r})"
        )
