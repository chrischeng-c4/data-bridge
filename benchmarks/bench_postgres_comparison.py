#!/usr/bin/env python3
"""
PostgreSQL Performance Benchmarks

Compares data-bridge-postgres against asyncpg, psycopg2, and SQLAlchemy.

This script runs comprehensive benchmarks across different PostgreSQL libraries
to measure performance of common operations:
- Insert One: Single row insert latency
- Insert Bulk: Bulk inserts (1000, 10000 rows)
- Select One: Find by primary key
- Select Many: Find with filter, limit 1000
- Update One: Update single row
- Update Many: Update multiple rows
- Delete Many: Delete multiple rows
- Count: Count with filter

Usage:
    POSTGRES_URI="postgresql://user:pass@localhost:5432/bench" \\
    uv run python benchmarks/bench_postgres_comparison.py

    # With custom parameters
    POSTGRES_URI="postgresql://postgres:postgres@localhost:5432/data_bridge_benchmark" \\
    uv run python benchmarks/bench_postgres_comparison.py

Environment Variables:
    POSTGRES_URI: PostgreSQL connection URI
                 Default: postgresql://postgres:postgres@localhost:5432/data_bridge_benchmark

Performance Targets:
    Based on MongoDB performance gains (1.4-5.4x vs Beanie), we target:
    - Insert 1000 rows: ≥2x faster than SQLAlchemy
    - Select 1000 rows: ≥1.5x faster than SQLAlchemy
    - Single operations: ≥1.3x faster than asyncpg

Output:
    - Console: Real-time benchmark results
    - benchmark_results_postgres.txt: Detailed results
    - benchmark_comparison_postgres.png: Visual comparison chart
"""

import asyncio
import os
import sys
import time
from typing import List, Dict, Any
from pathlib import Path

# Add project root to path
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))


# =====================
# Configuration
# =====================

POSTGRES_URI = os.environ.get(
    "POSTGRES_URI",
    "postgresql://postgres:postgres@localhost:5432/data_bridge_benchmark"
)

# Parse connection parameters
def parse_postgres_uri(uri: str) -> Dict[str, str]:
    """Parse PostgreSQL URI into components."""
    import urllib.parse
    parsed = urllib.parse.urlparse(uri)
    return {
        "host": parsed.hostname or "localhost",
        "port": parsed.port or 5432,
        "user": parsed.username or "postgres",
        "password": parsed.password or "postgres",
        "database": parsed.path.lstrip("/") or "data_bridge_benchmark",
    }

CONN_PARAMS = parse_postgres_uri(POSTGRES_URI)


# =====================
# Benchmark Operations
# =====================

async def benchmark_insert_one(framework: str, conn_pool: Any) -> float:
    """Benchmark single row insert."""
    iterations = 100
    total_time = 0.0

    if framework == "data_bridge":
        from data_bridge.postgres import Table, Column

        class User(Table):
            id: int = Column(primary_key=True)
            name: str
            email: str
            age: int

            class Settings:
                table_name = "bench_insert_one_db"

        for i in range(iterations):
            start = time.perf_counter()
            user = User(name=f"User{i}", email=f"user{i}@test.com", age=30)
            await user.save()
            total_time += time.perf_counter() - start

    elif framework == "asyncpg":
        for i in range(iterations):
            start = time.perf_counter()
            async with conn_pool.acquire() as conn:
                await conn.execute(
                    "INSERT INTO bench_insert_one_asyncpg (name, email, age) VALUES ($1, $2, $3)",
                    f"User{i}",
                    f"user{i}@test.com",
                    30,
                )
            total_time += time.perf_counter() - start

    elif framework == "psycopg2":
        for i in range(iterations):
            start = time.perf_counter()
            conn = conn_pool.getconn()
            try:
                with conn.cursor() as cur:
                    cur.execute(
                        "INSERT INTO bench_insert_one_psycopg2 (name, email, age) VALUES (%s, %s, %s)",
                        (f"User{i}", f"user{i}@test.com", 30),
                    )
                    conn.commit()
            finally:
                conn_pool.putconn(conn)
            total_time += time.perf_counter() - start

    elif framework == "sqlalchemy":
        from sqlalchemy.ext.asyncio import AsyncSession
        from sqlalchemy.orm import sessionmaker

        async_session = sessionmaker(conn_pool, class_=AsyncSession, expire_on_commit=False)

        from tests.postgres.benchmarks.models import SAUser

        for i in range(iterations):
            start = time.perf_counter()
            async with async_session() as session:
                user = SAUser(name=f"User{i}", email=f"user{i}@test.com", age=30)
                session.add(user)
                await session.commit()
            total_time += time.perf_counter() - start

    return (total_time / iterations) * 1000  # Convert to ms


async def benchmark_bulk_insert(framework: str, conn_pool: Any, count: int) -> float:
    """Benchmark bulk insert operations."""
    data = [
        {"name": f"User{i}", "email": f"user{i}@test.com", "age": 20 + (i % 50)}
        for i in range(count)
    ]

    if framework == "data_bridge":
        from data_bridge.postgres import Table, Column

        class User(Table):
            id: int = Column(primary_key=True)
            name: str
            email: str
            age: int

            class Settings:
                table_name = f"bench_bulk_insert_{count}_db"

        start = time.perf_counter()
        users = [User(**d) for d in data]
        await User.insert_many(users)
        return (time.perf_counter() - start) * 1000

    elif framework == "asyncpg":
        start = time.perf_counter()
        async with conn_pool.acquire() as conn:
            await conn.executemany(
                f"INSERT INTO bench_bulk_insert_{count}_asyncpg (name, email, age) VALUES ($1, $2, $3)",
                [(d["name"], d["email"], d["age"]) for d in data],
            )
        return (time.perf_counter() - start) * 1000

    elif framework == "psycopg2":
        start = time.perf_counter()
        conn = conn_pool.getconn()
        try:
            with conn.cursor() as cur:
                from psycopg2.extras import execute_values

                execute_values(
                    cur,
                    f"INSERT INTO bench_bulk_insert_{count}_psycopg2 (name, email, age) VALUES %s",
                    [(d["name"], d["email"], d["age"]) for d in data],
                )
                conn.commit()
        finally:
            conn_pool.putconn(conn)
        return (time.perf_counter() - start) * 1000

    elif framework == "sqlalchemy":
        from sqlalchemy.ext.asyncio import AsyncSession
        from sqlalchemy.orm import sessionmaker
        from tests.postgres.benchmarks.models import SAUser

        async_session = sessionmaker(conn_pool, class_=AsyncSession, expire_on_commit=False)

        start = time.perf_counter()
        async with async_session() as session:
            users = [SAUser(**d) for d in data]
            session.add_all(users)
            await session.commit()
        return (time.perf_counter() - start) * 1000


# =====================
# Main Benchmark Runner
# =====================

async def run_benchmarks():
    """Run all benchmarks and generate report."""
    print("=" * 80)
    print("PostgreSQL Performance Benchmarks")
    print("=" * 80)
    print(f"Connection: {POSTGRES_URI}")
    print(f"Database: {CONN_PARAMS['database']}")
    print("=" * 80)
    print()

    results = {}

    # Initialize connections
    print("Initializing connections...")

    # data-bridge
    databridge_available = False
    try:
        from data_bridge import postgres
        if postgres.is_connected():
            await postgres.close()
        await postgres.init(POSTGRES_URI)
        databridge_available = True
    except (ImportError, RuntimeError) as e:
        print(f"Warning: data-bridge postgres not available ({e}), skipping data-bridge benchmarks")

    # asyncpg
    try:
        import asyncpg

        asyncpg_pool = await asyncpg.create_pool(
            host=CONN_PARAMS["host"],
            port=CONN_PARAMS["port"],
            user=CONN_PARAMS["user"],
            password=CONN_PARAMS["password"],
            database=CONN_PARAMS["database"],
            min_size=2,
            max_size=10,
        )
        asyncpg_available = True
    except ImportError:
        asyncpg_available = False
        print("Warning: asyncpg not installed, skipping asyncpg benchmarks")

    # psycopg2
    try:
        import psycopg2
        import psycopg2.pool

        psycopg2_pool = psycopg2.pool.SimpleConnectionPool(
            minconn=2,
            maxconn=10,
            host=CONN_PARAMS["host"],
            port=CONN_PARAMS["port"],
            user=CONN_PARAMS["user"],
            password=CONN_PARAMS["password"],
            database=CONN_PARAMS["database"],
        )
        psycopg2_available = True
    except ImportError:
        psycopg2_available = False
        print("Warning: psycopg2 not installed, skipping psycopg2 benchmarks")

    # SQLAlchemy
    try:
        from sqlalchemy.ext.asyncio import create_async_engine

        async_uri = POSTGRES_URI.replace("postgresql://", "postgresql+asyncpg://")
        sqlalchemy_engine = create_async_engine(
            async_uri, echo=False, pool_size=10, max_overflow=20
        )
        sqlalchemy_available = True
    except ImportError:
        sqlalchemy_available = False
        print("Warning: SQLAlchemy not installed, skipping SQLAlchemy benchmarks")

    # Create benchmark tables
    print("Creating benchmark tables...")
    if asyncpg_available:
        async with asyncpg_pool.acquire() as conn:
            # Drop and create tables for each framework/operation
            for table in [
                "bench_insert_one_asyncpg", "bench_insert_one_psycopg2",
                "bench_insert_one_sqlalchemy", "bench_insert_one_db",
                "bench_bulk_asyncpg", "bench_bulk_psycopg2",
                "bench_bulk_sqlalchemy", "bench_bulk_db",
                "bench_bulk_insert_1000_asyncpg", "bench_bulk_insert_1000_psycopg2",
                "bench_bulk_insert_1000_sqlalchemy", "bench_bulk_insert_1000_db",
                "bench_bulk_insert_10000_asyncpg", "bench_bulk_insert_10000_psycopg2",
                "bench_bulk_insert_10000_sqlalchemy", "bench_bulk_insert_10000_db",
                "bench_sa_users",  # SQLAlchemy model table
            ]:
                await conn.execute(f"DROP TABLE IF EXISTS {table}")
                await conn.execute(f"""
                    CREATE TABLE {table} (
                        id SERIAL PRIMARY KEY,
                        name VARCHAR(255) NOT NULL,
                        email VARCHAR(255) NOT NULL,
                        age INTEGER NOT NULL,
                        city VARCHAR(100),
                        score FLOAT,
                        active BOOLEAN DEFAULT TRUE
                    )
                """)

    print()

    # Run benchmarks
    frameworks = []
    if databridge_available:
        frameworks.append("data_bridge")
    if asyncpg_available:
        frameworks.append("asyncpg")
    if psycopg2_available:
        frameworks.append("psycopg2")
    if sqlalchemy_available:
        frameworks.append("sqlalchemy")

    # Insert One
    print("Benchmarking: Insert One (100 iterations)")
    results["insert_one"] = {}
    for fw in frameworks:
        pool = {
            "data_bridge": None,
            "asyncpg": asyncpg_pool if asyncpg_available else None,
            "psycopg2": psycopg2_pool if psycopg2_available else None,
            "sqlalchemy": sqlalchemy_engine if sqlalchemy_available else None,
        }[fw]
        time_ms = await benchmark_insert_one(fw, pool)
        results["insert_one"][fw] = time_ms
        print(f"  {fw:15s}: {time_ms:8.2f} ms")

    print()

    # Bulk Insert 1000
    print("Benchmarking: Bulk Insert 1000 rows")
    results["bulk_insert_1000"] = {}
    for fw in frameworks:
        pool = {
            "data_bridge": None,
            "asyncpg": asyncpg_pool if asyncpg_available else None,
            "psycopg2": psycopg2_pool if psycopg2_available else None,
            "sqlalchemy": sqlalchemy_engine if sqlalchemy_available else None,
        }[fw]
        time_ms = await benchmark_bulk_insert(fw, pool, 1000)
        results["bulk_insert_1000"][fw] = time_ms
        print(f"  {fw:15s}: {time_ms:8.2f} ms")

    print()

    # Cleanup
    print("Cleaning up...")
    if databridge_available:
        await postgres.close()
    if asyncpg_available:
        await asyncpg_pool.close()
    if psycopg2_available:
        psycopg2_pool.closeall()
    if sqlalchemy_available:
        await sqlalchemy_engine.dispose()

    print()
    print("=" * 80)
    print("Benchmark Complete")
    print("=" * 80)

    # Print summary
    print()
    print("SUMMARY:")
    print("-" * 80)

    for operation, op_results in results.items():
        print(f"\n{operation}:")
        baseline = op_results.get("sqlalchemy", op_results.get("asyncpg"))
        for fw, time_ms in op_results.items():
            speedup = baseline / time_ms if baseline and time_ms else 1.0
            print(f"  {fw:15s}: {time_ms:8.2f} ms  ({speedup:.2f}x)")


if __name__ == "__main__":
    asyncio.run(run_benchmarks())
