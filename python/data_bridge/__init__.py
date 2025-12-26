"""
data-bridge: High-performance MongoDB ORM with Rust backend.

A Beanie-compatible Python API with all BSON serialization handled in Rust.

Quick Start:
    >>> from data_bridge import Document, Field, init
    >>>
    >>> # Initialize connection
    >>> await init("mongodb://localhost:27017/mydb")
    >>>
    >>> # Define a model
    >>> class User(Document):
    ...     email: str
    ...     name: str
    ...     age: int = 0
    ...
    ...     class Settings:
    ...         name = "users"
    >>>
    >>> # Create and save
    >>> user = User(email="alice@example.com", name="Alice", age=30)
    >>> await user.save()
    >>>
    >>> # Query with type-safe expressions
    >>> user = await User.find_one(User.email == "alice@example.com")
    >>> users = await User.find(User.age > 25).to_list()

Features:
    - Beanie-compatible API for easy migration
    - Type-safe query expressions (User.email == "x")
    - All BSON serialization in Rust for maximum performance
    - Async-first design with asyncio/tokio bridge
    - Chainable query builder with sort, skip, limit

Architecture:
    - Python layer: Provides Beanie-like API, Document base class, QueryBuilder
    - Rust engine: Handles BSON encoding/decoding, MongoDB operations
    - Zero Python byte handling: Data never touches Python heap as raw bytes
"""

__version__ = "0.1.0"

# Import the Rust binary extension
from data_bridge import data_bridge as _rust_module

# Re-export mongodb submodule from Rust
try:
    from data_bridge.data_bridge import mongodb
except ImportError:
    mongodb = None  # MongoDB module not built

# Re-export http submodule from Rust
try:
    from data_bridge.data_bridge import http
except ImportError:
    http = None  # HTTP module not built

# Re-export test submodule from Rust
try:
    from data_bridge.data_bridge import test as _test_rust
    from . import test  # Python test package (uses Rust bindings)
except ImportError:
    test = None  # Test module not built

# Re-export postgres submodule from Rust
try:
    from data_bridge.data_bridge import postgres
except ImportError:
    postgres = None  # PostgreSQL module not built

# Import _engine to provide the bridge to Rust backend
from . import _engine

# Core classes - Python layer with Beanie-compatible API
from .document import Document, Settings
from .embedded import EmbeddedDocument
from .fields import Field, FieldProxy, QueryExpr, merge_filters, text_search, TextSearch, escape_regex
from .query import QueryBuilder, AggregationBuilder

# Lifecycle actions/hooks
from .actions import (
    before_event,
    after_event,
    Insert,
    Replace,
    Save,
    Delete,
    ValidateOnSave,
    EventType,
)

# Bulk operations with fluent API
from .bulk import (
    BulkOperation,
    UpdateOne,
    UpdateMany,
    InsertOne,
    DeleteOne,
    DeleteMany,
    ReplaceOne,
    BulkWriteResult,
)

# Type support
from .types import (
    PydanticObjectId,
    Indexed,
    IndexModelField,
    get_index_fields,
)

# Document relations/links
from .links import (
    Link,
    BackLink,
    WriteRules,
    DeleteRules,
    get_link_fields,
)

# Transactions (stub - requires Rust implementation)
from .transactions import (
    Session,
    Transaction,
    start_session,
    TransactionNotSupportedError,
)

# Time-series collections (MongoDB 5.0+)
from .timeseries import (
    TimeSeriesConfig,
    Granularity,
)

# Programmatic migrations
from .migrations import (
    Migration,
    MigrationHistory,
    IterativeMigration,
    FreeFallMigration,
    iterative_migration,
    free_fall_migration,
    run_migrations,
    get_pending_migrations,
    get_applied_migrations,
    get_migration_status,
)

# Constraint validators for Annotated types
from .constraints import (
    Constraint,
    MinLen,
    MaxLen,
    Min,
    Max,
    Email,
    Url,
)

# Connection management
from .connection import init, is_connected, close, reset

# Public API
__all__ = [
    # Version
    "__version__",
    # Rust submodules
    "mongodb",
    "http",
    "test",
    "postgres",
    # Core
    "Document",
    "Settings",
    "EmbeddedDocument",
    # Fields
    "Field",
    "FieldProxy",
    "QueryExpr",
    "merge_filters",
    "text_search",
    "TextSearch",
    "escape_regex",
    # Query
    "QueryBuilder",
    "AggregationBuilder",
    # Connection
    "init",
    "is_connected",
    "close",
    "reset",
    # Actions/Hooks
    "before_event",
    "after_event",
    "Insert",
    "Replace",
    "Save",
    "Delete",
    "ValidateOnSave",
    "EventType",
    # Bulk Operations
    "BulkOperation",
    "UpdateOne",
    "UpdateMany",
    "InsertOne",
    "DeleteOne",
    "DeleteMany",
    "ReplaceOne",
    "BulkWriteResult",
    # Type Support
    "PydanticObjectId",
    "Indexed",
    "IndexModelField",
    "get_index_fields",
    # Document Relations
    "Link",
    "BackLink",
    "WriteRules",
    "DeleteRules",
    "get_link_fields",
    # Transactions (stub)
    "Session",
    "Transaction",
    "start_session",
    "TransactionNotSupportedError",
    # Time-series Collections
    "TimeSeriesConfig",
    "Granularity",
    # Migrations
    "Migration",
    "MigrationHistory",
    "IterativeMigration",
    "FreeFallMigration",
    "iterative_migration",
    "free_fall_migration",
    "run_migrations",
    "get_pending_migrations",
    "get_applied_migrations",
    "get_migration_status",
]
