"""
Not found exception for missing resources

HTTP 404 Not Found
"""

from common.exceptions.base import JudgifyException


class NotFoundError(JudgifyException):
    """
    Exception for resource not found errors

    HTTP Status: 404 Not Found

    Usage:
        raise NotFoundError(resource="Workflow", id="abc-123")

        # Custom message
        raise NotFoundError("Resource not found")
    """

    def __init__(self, resource: str = None, id: str = None, message: str = None):
        if message is None:
            if resource and id:
                message = f"{resource} {id} not found"
            else:
                message = "Resource not found"

        details = {}
        if resource:
            details["resource"] = resource
        if id:
            details["id"] = id

        super().__init__(message, status_code=404, details=details)
