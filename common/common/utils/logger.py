"""
Structured logging setup for all microservices

Provides consistent logging configuration with JSON formatting.
"""

import logging
import sys
import os
from typing import Optional
from datetime import datetime


class JSONFormatter(logging.Formatter):
    """
    JSON formatter for structured logging

    Output format:
    {
        "timestamp": "2025-01-22T10:30:45.123456",
        "level": "INFO",
        "logger": "WorkflowService",
        "message": "Creating workflow",
        "service": "workflow-service",
        "environment": "production"
    }
    """

    def format(self, record: logging.LogRecord) -> str:
        import json

        log_data = {
            "timestamp": datetime.utcnow().isoformat(),
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
            "service": os.getenv("SERVICE_NAME", "unknown"),
            "environment": os.getenv("ENVIRONMENT", "development"),
        }

        # Add exception info if present
        if record.exc_info:
            log_data["exception"] = self.formatException(record.exc_info)

        # Add extra fields from record
        if hasattr(record, "workflow_id"):
            log_data["workflow_id"] = record.workflow_id
        if hasattr(record, "execution_id"):
            log_data["execution_id"] = record.execution_id
        if hasattr(record, "user_id"):
            log_data["user_id"] = record.user_id

        return json.dumps(log_data)


def setup_logger(name: str, level: Optional[str] = None) -> logging.Logger:
    """
    Setup structured logger with JSON formatting

    Args:
        name: Logger name (typically __name__ or class name)
        level: Log level (DEBUG, INFO, WARNING, ERROR, CRITICAL)
               Defaults to LOG_LEVEL environment variable or INFO

    Returns:
        Configured logger instance

    Usage:
        # In service class
        logger = setup_logger(__name__)
        logger.info("Service started")

        # With context
        logger.info("Processing workflow", extra={"workflow_id": "123"})
    """
    logger = logging.getLogger(name)

    # Set log level
    log_level = level or os.getenv("LOG_LEVEL", "INFO")
    logger.setLevel(getattr(logging, log_level.upper()))

    # Remove existing handlers to avoid duplicates
    if logger.hasHandlers():
        logger.handlers.clear()

    # Create console handler
    handler = logging.StreamHandler(sys.stdout)
    handler.setLevel(getattr(logging, log_level.upper()))

    # Set formatter (JSON in production, simple in development)
    if os.getenv("ENVIRONMENT", "development") == "production":
        formatter = JSONFormatter()
    else:
        formatter = logging.Formatter(
            "%(asctime)s - %(name)s - %(levelname)s - %(message)s",
            datefmt="%Y-%m-%d %H:%M:%S"
        )

    handler.setFormatter(formatter)
    logger.addHandler(handler)

    # Prevent propagation to root logger
    logger.propagate = False

    return logger


# Example usage
if __name__ == "__main__":
    logger = setup_logger("test-service")
    logger.debug("Debug message")
    logger.info("Info message")
    logger.warning("Warning message")
    logger.error("Error message")

    # With extra context
    logger.info("Processing workflow", extra={"workflow_id": "abc-123"})
