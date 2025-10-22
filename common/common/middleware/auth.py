"""
JWT authentication middleware for FastAPI

Provides JWT token verification and user authentication.
"""

import os
from typing import Optional
from datetime import datetime, timedelta
from fastapi import Depends, HTTPException, status
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from jose import JWTError, jwt

from common.exceptions import UnauthorizedError

# JWT configuration from environment
SECRET_KEY = os.getenv("JWT_SECRET_KEY", "your-secret-key-change-in-production")
ALGORITHM = os.getenv("JWT_ALGORITHM", "HS256")
ACCESS_TOKEN_EXPIRE_MINUTES = int(os.getenv("JWT_EXPIRE_MINUTES", "30"))

security = HTTPBearer()


def create_access_token(data: dict, expires_delta: Optional[timedelta] = None) -> str:
    """
    Create JWT access token

    Args:
        data: Payload data to encode (typically {"sub": user_id})
        expires_delta: Token expiration time (default: 30 minutes)

    Returns:
        Encoded JWT token string

    Example:
        token = create_access_token({"sub": "user-123"})
    """
    to_encode = data.copy()

    if expires_delta:
        expire = datetime.utcnow() + expires_delta
    else:
        expire = datetime.utcnow() + timedelta(minutes=ACCESS_TOKEN_EXPIRE_MINUTES)

    to_encode.update({"exp": expire, "iat": datetime.utcnow()})

    encoded_jwt = jwt.encode(to_encode, SECRET_KEY, algorithm=ALGORITHM)
    return encoded_jwt


def verify_token(credentials: HTTPAuthorizationCredentials = Depends(security)) -> dict:
    """
    Verify JWT token and extract payload

    Args:
        credentials: Bearer token from Authorization header

    Returns:
        Decoded token payload

    Raises:
        UnauthorizedError: If token is invalid or expired

    Usage:
        @router.get("/protected")
        async def protected_route(payload: dict = Depends(verify_token)):
            user_id = payload.get("sub")
            return {"user_id": user_id}
    """
    token = credentials.credentials

    try:
        payload = jwt.decode(token, SECRET_KEY, algorithms=[ALGORITHM])
        return payload

    except JWTError as e:
        raise UnauthorizedError(f"Invalid token: {str(e)}")


async def get_current_user(payload: dict = Depends(verify_token)) -> str:
    """
    Get current user ID from JWT token

    Args:
        payload: Decoded JWT payload from verify_token

    Returns:
        User ID string

    Raises:
        UnauthorizedError: If user ID not found in token

    Usage:
        @router.get("/me")
        async def get_me(user_id: str = Depends(get_current_user)):
            return {"user_id": user_id}
    """
    user_id = payload.get("sub")

    if not user_id:
        raise UnauthorizedError("User ID not found in token")

    return user_id


def require_roles(*required_roles: str):
    """
    Dependency to require specific roles (RBAC)

    Args:
        required_roles: Role names required (e.g., "admin", "editor")

    Returns:
        FastAPI dependency function

    Usage:
        @router.delete("/workflows/{id}")
        async def delete_workflow(
            id: UUID,
            user_id: str = Depends(require_roles("admin"))
        ):
            # Only admins can delete
            pass
    """

    async def check_roles(payload: dict = Depends(verify_token)) -> str:
        user_roles = payload.get("roles", [])

        if not any(role in user_roles for role in required_roles):
            raise UnauthorizedError(
                f"Insufficient permissions. Required roles: {', '.join(required_roles)}"
            )

        return payload.get("sub")

    return check_roles
