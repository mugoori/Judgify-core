"""
CORS middleware configuration for FastAPI

Provides Cross-Origin Resource Sharing setup for all microservices.
"""

import os
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware


def setup_cors(app: FastAPI) -> None:
    """
    Configure CORS middleware for FastAPI application

    Args:
        app: FastAPI application instance

    Usage:
        from fastapi import FastAPI
        from common.middleware import setup_cors

        app = FastAPI()
        setup_cors(app)

    Environment variables:
        CORS_ORIGINS: Comma-separated list of allowed origins
        CORS_ALLOW_CREDENTIALS: Allow credentials (default: true)
        CORS_ALLOW_METHODS: Allowed HTTP methods (default: *)
        CORS_ALLOW_HEADERS: Allowed headers (default: *)
    """
    # Get allowed origins from environment
    origins_str = os.getenv(
        "CORS_ORIGINS",
        "http://localhost:3000,http://localhost:5173,http://localhost:8080"
    )
    origins = [origin.strip() for origin in origins_str.split(",")]

    # Get other CORS settings
    allow_credentials = os.getenv("CORS_ALLOW_CREDENTIALS", "true").lower() == "true"
    allow_methods_str = os.getenv("CORS_ALLOW_METHODS", "*")
    allow_methods = [m.strip() for m in allow_methods_str.split(",")]
    allow_headers_str = os.getenv("CORS_ALLOW_HEADERS", "*")
    allow_headers = [h.strip() for h in allow_headers_str.split(",")]

    app.add_middleware(
        CORSMiddleware,
        allow_origins=origins,
        allow_credentials=allow_credentials,
        allow_methods=allow_methods,
        allow_headers=allow_headers,
    )
