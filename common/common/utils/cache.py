"""
Redis cache management for all microservices

Provides Redis client helpers and caching utilities.
"""

import os
import json
from typing import Optional, Any
from redis import asyncio as aioredis
from redis.asyncio import Redis

# Redis URL from environment variable
REDIS_URL = os.getenv(
    "REDIS_URL",
    "redis://localhost:6379/0"
)

# Global Redis client (singleton)
_redis_client: Optional[Redis] = None


async def get_redis_client() -> Redis:
    """
    Get singleton Redis client

    Returns:
        Redis async client

    Example:
        redis = await get_redis_client()
        await redis.set("key", "value", ex=300)
    """
    global _redis_client

    if _redis_client is None:
        _redis_client = aioredis.from_url(
            REDIS_URL,
            encoding="utf-8",
            decode_responses=True,
            max_connections=int(os.getenv("REDIS_MAX_CONNECTIONS", "50")),
        )

    return _redis_client


async def get_redis_cache():
    """
    FastAPI dependency for Redis cache injection

    Usage:
        @router.get("/workflows/{id}")
        async def get_workflow(
            id: UUID,
            cache: Redis = Depends(get_redis_cache)
        ):
            # Try cache first
            cached = await cache.get(f"workflow:{id}")
            if cached:
                return json.loads(cached)

            # ... fetch from database

    Yields:
        Redis: Redis async client
    """
    redis = await get_redis_client()
    try:
        yield redis
    finally:
        # Redis client는 singleton이므로 닫지 않음
        pass


class RedisCache:
    """
    High-level Redis cache wrapper with JSON serialization

    Example:
        cache = RedisCache()

        # Set with TTL
        await cache.set("user:123", {"name": "John"}, ttl=300)

        # Get
        user = await cache.get("user:123")

        # Delete
        await cache.delete("user:123")
    """

    def __init__(self):
        self.redis: Optional[Redis] = None

    async def _get_client(self) -> Redis:
        """Get Redis client (lazy initialization)"""
        if self.redis is None:
            self.redis = await get_redis_client()
        return self.redis

    async def get(self, key: str) -> Optional[Any]:
        """
        Get value from cache with JSON deserialization

        Args:
            key: Cache key

        Returns:
            Deserialized value or None if not found
        """
        redis = await self._get_client()
        value = await redis.get(key)

        if value is None:
            return None

        try:
            return json.loads(value)
        except json.JSONDecodeError:
            return value  # Return as string if not JSON

    async def set(self, key: str, value: Any, ttl: int = 300) -> bool:
        """
        Set value in cache with JSON serialization

        Args:
            key: Cache key
            value: Value to cache (will be JSON serialized)
            ttl: Time-to-live in seconds (default: 300 = 5 minutes)

        Returns:
            True if successful
        """
        redis = await self._get_client()

        # Serialize to JSON
        if not isinstance(value, str):
            value = json.dumps(value)

        await redis.set(key, value, ex=ttl)
        return True

    async def delete(self, key: str) -> bool:
        """
        Delete key from cache

        Args:
            key: Cache key

        Returns:
            True if deleted, False if not found
        """
        redis = await self._get_client()
        result = await redis.delete(key)
        return result > 0

    async def exists(self, key: str) -> bool:
        """
        Check if key exists in cache

        Args:
            key: Cache key

        Returns:
            True if exists, False otherwise
        """
        redis = await self._get_client()
        return await redis.exists(key) > 0


async def close_redis():
    """
    Close Redis connection pool

    Call this on application shutdown
    """
    global _redis_client

    if _redis_client:
        await _redis_client.close()
        _redis_client = None
