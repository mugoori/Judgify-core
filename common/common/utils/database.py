"""
PostgreSQL database connection pool management

Provides async SQLAlchemy session factory and FastAPI dependency injection.
"""

import os
from typing import AsyncGenerator
from sqlalchemy.ext.asyncio import (
    create_async_engine,
    AsyncSession,
    async_sessionmaker,
)
from sqlalchemy.pool import NullPool, QueuePool

# Database URL from environment variable
DATABASE_URL = os.getenv(
    "DATABASE_URL",
    "postgresql+asyncpg://judgify:password@localhost:5432/judgify_core"
)

# Create async engine with connection pooling
engine = create_async_engine(
    DATABASE_URL,
    echo=os.getenv("SQL_ECHO", "false").lower() == "true",  # SQL 로깅
    pool_size=int(os.getenv("DB_POOL_SIZE", "10")),  # 연결 풀 크기
    max_overflow=int(os.getenv("DB_MAX_OVERFLOW", "20")),  # 최대 오버플로우
    pool_pre_ping=True,  # 연결 유효성 검사
    poolclass=QueuePool if os.getenv("DB_POOL_ENABLED", "true").lower() == "true" else NullPool,
)

# Session factory
AsyncSessionLocal = async_sessionmaker(
    engine,
    class_=AsyncSession,
    expire_on_commit=False,  # 커밋 후 객체 유지
    autoflush=False,  # 자동 flush 비활성화
    autocommit=False,  # 자동 commit 비활성화
)


async def get_database() -> AsyncGenerator[AsyncSession, None]:
    """
    FastAPI dependency for database session injection

    Usage:
        @router.post("/workflows")
        async def create_workflow(
            data: WorkflowCreate,
            db: AsyncSession = Depends(get_database)
        ):
            service = WorkflowService(db)
            return await service.create(data)

    Yields:
        AsyncSession: Database session

    Example:
        async with get_database() as session:
            result = await session.execute(select(User))
            users = result.scalars().all()
    """
    async with AsyncSessionLocal() as session:
        try:
            yield session
        except Exception:
            await session.rollback()
            raise
        finally:
            await session.close()


async def init_database():
    """
    Initialize database (create tables, run migrations)

    Note: In production, use Alembic for migrations
    """
    from sqlalchemy.ext.declarative import declarative_base

    Base = declarative_base()

    async with engine.begin() as conn:
        # await conn.run_sync(Base.metadata.create_all)  # 개발 환경용
        pass  # 프로덕션에서는 Alembic 사용


async def close_database():
    """
    Close database connection pool

    Call this on application shutdown
    """
    await engine.dispose()
