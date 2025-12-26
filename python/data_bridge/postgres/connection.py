"""PostgreSQL connection management."""

from typing import Optional

# Import from Rust engine when available
try:
    from data_bridge import postgres as _engine
except ImportError:
    _engine = None


async def init(
    connection_string: Optional[str] = None,
    *,
    host: str = "localhost",
    port: int = 5432,
    database: str = "postgres",
    username: Optional[str] = None,
    password: Optional[str] = None,
    min_connections: int = 1,
    max_connections: int = 10,
) -> None:
    """
    Initialize PostgreSQL connection pool.

    Args:
        connection_string: Full PostgreSQL connection string (postgres://user:pass@host:port/db)
        host: PostgreSQL server hostname (default: localhost)
        port: PostgreSQL server port (default: 5432)
        database: Database name (default: postgres)
        username: Database username
        password: Database password
        min_connections: Minimum number of connections in pool (default: 1)
        max_connections: Maximum number of connections in pool (default: 10)

    Example:
        >>> # Using connection string
        >>> await init("postgres://user:pass@localhost:5432/mydb")
        >>>
        >>> # Using individual parameters
        >>> await init(
        ...     host="localhost",
        ...     port=5432,
        ...     database="mydb",
        ...     username="user",
        ...     password="pass",
        ...     max_connections=20
        ... )

    Raises:
        RuntimeError: If connection fails or Rust engine is not available
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    if connection_string is None:
        # Build connection string from individual parameters
        auth = f"{username}:{password}@" if username else ""
        connection_string = f"postgres://{auth}{host}:{port}/{database}"

    await _engine.init(connection_string, min_connections, max_connections)


async def close() -> None:
    """
    Close the PostgreSQL connection pool.

    This should be called when shutting down your application to ensure
    all connections are properly closed.

    Example:
        >>> await close()

    Raises:
        RuntimeError: If Rust engine is not available
    """
    if _engine is None:
        raise RuntimeError(
            "PostgreSQL engine not available. Ensure data-bridge was built with PostgreSQL support."
        )

    await _engine.close()


def is_connected() -> bool:
    """
    Check if the PostgreSQL connection pool is active.

    Returns:
        True if connected, False otherwise

    Example:
        >>> if is_connected():
        ...     print("Connected to PostgreSQL")
        ... else:
        ...     print("Not connected")
    """
    if _engine is None:
        return False

    return _engine.is_connected()
