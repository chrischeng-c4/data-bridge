"""
Data Bridge - A type-based ODM supporting multiple backends with sync/async operations.

This package provides database-specific implementations rather than generic abstractions:
- MongoDB: Document-based operations with sync/async support
- Redis: HashModel and JSONModel with sync/async support
- Future: Firestore, PostgreSQL, etc.

Usage:
    # MongoDB (sync)
    from data_bridge.mongo.sync import Document, Field, MongoSyncBackend

    # MongoDB (async)
    from data_bridge.mongo.async_ import AsyncDocument, Field, MongoAsyncBackend

    # Redis Hash (sync)
    from data_bridge.redis.sync import HashModel, Field, RedisSyncBackend

    # Redis JSON (sync)
    from data_bridge.redis.sync import JSONModel, Field, RedisSyncBackend
"""

# Re-export commonly used base components for convenience
from .base.fields import CompoundExpression, Field, QueryExpression

__version__ = "0.1.0"

__all__ = [
    "CompoundExpression",
    # Base components (for convenience, but users should import from specific backends)
    "Field",
    "QueryExpression",
]
