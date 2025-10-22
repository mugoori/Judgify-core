"""
Global exception handling for FastAPI

Provides consistent error response format across all microservices.
"""

from fastapi import Request, status
from fastapi.responses import JSONResponse
from fastapi.exceptions import RequestValidationError
from pydantic import ValidationError

from common.exceptions import JudgifyException
from common.utils.logger import setup_logger

logger = setup_logger(__name__)


async def global_exception_handler(
    request: Request,
    exc: JudgifyException
) -> JSONResponse:
    """
    Handle custom Judgify exceptions

    Args:
        request: FastAPI request
        exc: Judgify custom exception

    Returns:
        JSON error response with consistent format

    Usage:
        from fastapi import FastAPI
        from common.exceptions import JudgifyException
        from common.middleware import global_exception_handler

        app = FastAPI()
        app.add_exception_handler(JudgifyException, global_exception_handler)

    Response format:
        {
            "error": "NotFoundError",
            "message": "Workflow abc-123 not found",
            "status_code": 404,
            "path": "/api/v2/workflows/abc-123"
        }
    """
    logger.error(
        f"Exception: {exc.__class__.__name__} - {exc.message}",
        extra={
            "path": request.url.path,
            "method": request.method,
            "status_code": exc.status_code,
        }
    )

    return JSONResponse(
        status_code=exc.status_code,
        content={
            "error": exc.__class__.__name__,
            "message": exc.message,
            "status_code": exc.status_code,
            "path": request.url.path,
        }
    )


async def validation_exception_handler(
    request: Request,
    exc: RequestValidationError
) -> JSONResponse:
    """
    Handle Pydantic validation errors

    Args:
        request: FastAPI request
        exc: Pydantic validation error

    Returns:
        JSON error response with validation details

    Usage:
        from fastapi import FastAPI
        from fastapi.exceptions import RequestValidationError
        from common.middleware import validation_exception_handler

        app = FastAPI()
        app.add_exception_handler(RequestValidationError, validation_exception_handler)

    Response format:
        {
            "error": "ValidationError",
            "message": "Validation failed",
            "status_code": 422,
            "path": "/api/v2/workflows",
            "details": [
                {
                    "field": "name",
                    "message": "field required",
                    "type": "value_error.missing"
                }
            ]
        }
    """
    logger.warning(
        f"Validation error: {exc.errors()}",
        extra={
            "path": request.url.path,
            "method": request.method,
        }
    )

    # Format validation errors
    details = []
    for error in exc.errors():
        details.append({
            "field": ".".join(str(loc) for loc in error["loc"]),
            "message": error["msg"],
            "type": error["type"],
        })

    return JSONResponse(
        status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
        content={
            "error": "ValidationError",
            "message": "Validation failed",
            "status_code": 422,
            "path": request.url.path,
            "details": details,
        }
    )


async def generic_exception_handler(
    request: Request,
    exc: Exception
) -> JSONResponse:
    """
    Handle unexpected exceptions

    Args:
        request: FastAPI request
        exc: Any unhandled exception

    Returns:
        JSON error response (500 Internal Server Error)

    Usage:
        from fastapi import FastAPI
        from common.middleware import generic_exception_handler

        app = FastAPI()
        app.add_exception_handler(Exception, generic_exception_handler)
    """
    logger.exception(
        f"Unhandled exception: {str(exc)}",
        extra={
            "path": request.url.path,
            "method": request.method,
        }
    )

    return JSONResponse(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        content={
            "error": "InternalServerError",
            "message": "An unexpected error occurred",
            "status_code": 500,
            "path": request.url.path,
        }
    )
