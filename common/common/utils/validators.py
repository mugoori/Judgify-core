"""
Input validation utilities for all microservices

Provides common validation functions for UUIDs, emails, etc.
"""

import re
from uuid import UUID
from typing import Optional


def validate_uuid(value: str) -> bool:
    """
    Validate UUID string format

    Args:
        value: String to validate

    Returns:
        True if valid UUID, False otherwise

    Example:
        >>> validate_uuid("550e8400-e29b-41d4-a716-446655440000")
        True
        >>> validate_uuid("invalid-uuid")
        False
    """
    try:
        UUID(value)
        return True
    except (ValueError, AttributeError, TypeError):
        return False


def validate_email(email: str) -> bool:
    """
    Validate email address format

    Args:
        email: Email address to validate

    Returns:
        True if valid email format, False otherwise

    Example:
        >>> validate_email("user@example.com")
        True
        >>> validate_email("invalid-email")
        False
    """
    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
    return re.match(pattern, email) is not None


def validate_phone(phone: str, country_code: str = "KR") -> bool:
    """
    Validate phone number format

    Args:
        phone: Phone number to validate
        country_code: Country code (default: KR for South Korea)

    Returns:
        True if valid phone number, False otherwise

    Example:
        >>> validate_phone("010-1234-5678", "KR")
        True
        >>> validate_phone("123", "KR")
        False
    """
    patterns = {
        "KR": r'^01[0-9]-\d{3,4}-\d{4}$',  # 한국 휴대전화
        "US": r'^\+?1?\d{10}$',  # 미국 전화번호
    }

    pattern = patterns.get(country_code)
    if not pattern:
        return False

    return re.match(pattern, phone) is not None


def validate_url(url: str) -> bool:
    """
    Validate URL format

    Args:
        url: URL to validate

    Returns:
        True if valid URL, False otherwise

    Example:
        >>> validate_url("https://example.com")
        True
        >>> validate_url("not a url")
        False
    """
    pattern = r'^https?://[^\s/$.?#].[^\s]*$'
    return re.match(pattern, url) is not None


def sanitize_string(value: str, max_length: Optional[int] = None) -> str:
    """
    Sanitize string input (remove dangerous characters)

    Args:
        value: String to sanitize
        max_length: Maximum length (truncate if longer)

    Returns:
        Sanitized string

    Example:
        >>> sanitize_string("<script>alert('xss')</script>")
        "scriptalert('xss')/script"
        >>> sanitize_string("Hello World", max_length=5)
        "Hello"
    """
    # Remove HTML tags
    value = re.sub(r'<[^>]+>', '', value)

    # Remove SQL injection patterns
    value = re.sub(r'[\'\";]', '', value)

    # Trim whitespace
    value = value.strip()

    # Truncate if needed
    if max_length and len(value) > max_length:
        value = value[:max_length]

    return value
