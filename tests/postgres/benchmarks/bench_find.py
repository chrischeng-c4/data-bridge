"""Find/Select benchmarks for PostgreSQL."""

import pytest
from data_bridge.test import BenchmarkGroup, register_group
from .models import DBUser, SAUser, SQLALCHEMY_AVAILABLE
from .conftest import generate_user_data


# =====================
# Find One (by ID)
# =====================

find_one = BenchmarkGroup("Find One (by ID)")


@find_one.add("data-bridge", setup="await _setup_db_find_one()")
async def db_find_one():
    """Find one record by ID with data-bridge."""
    user = await DBUser.find_one(DBUser.id == 1)
    return user


async def _setup_db_find_one():
    """Setup: Insert test data for data-bridge."""
    user = DBUser(name="Test", email="test@test.com", age=30, active=True)
    await user.save()


@find_one.add("asyncpg", setup="await _setup_asyncpg_find_one(asyncpg_pool)")
async def asyncpg_find_one(asyncpg_pool):
    """Find one record by ID with asyncpg."""
    async with asyncpg_pool.acquire() as conn:
        row = await conn.fetchrow(
            "SELECT * FROM bench_asyncpg_users WHERE id = $1", 1
        )
        return dict(row) if row else None


async def _setup_asyncpg_find_one(asyncpg_pool):
    """Setup: Insert test data for asyncpg."""
    async with asyncpg_pool.acquire() as conn:
        await conn.execute(
            """
            INSERT INTO bench_asyncpg_users (name, email, age, active)
            VALUES ($1, $2, $3, $4)
            """,
            "Test",
            "test@test.com",
            30,
            True,
        )


@find_one.add("psycopg2", setup="_setup_psycopg2_find_one(psycopg2_conn)")
def psycopg2_find_one(psycopg2_conn):
    """Find one record by ID with psycopg2."""
    conn = psycopg2_conn.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute("SELECT * FROM bench_psycopg2_users WHERE id = %s", (1,))
            row = cur.fetchone()
            return row
    finally:
        psycopg2_conn.putconn(conn)


def _setup_psycopg2_find_one(psycopg2_conn):
    """Setup: Insert test data for psycopg2."""
    conn = psycopg2_conn.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute(
                """
                INSERT INTO bench_psycopg2_users (name, email, age, active)
                VALUES (%s, %s, %s, %s)
                """,
                ("Test", "test@test.com", 30, True),
            )
            conn.commit()
    finally:
        psycopg2_conn.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @find_one.add("SQLAlchemy", setup="await _setup_sa_find_one(sqlalchemy_session)")
    async def sqlalchemy_find_one(sqlalchemy_session):
        """Find one record by ID with SQLAlchemy."""
        from sqlalchemy import select

        result = await sqlalchemy_session.execute(
            select(SAUser).where(SAUser.id == 1)
        )
        return result.scalar_one_or_none()

    async def _setup_sa_find_one(sqlalchemy_session):
        """Setup: Insert test data for SQLAlchemy."""
        user = SAUser(name="Test", email="test@test.com", age=30, active=True)
        sqlalchemy_session.add(user)
        await sqlalchemy_session.commit()


register_group(find_one)


# =====================
# Find Many (1000 records)
# =====================

DATA_1000 = generate_user_data(1000)

find_many = BenchmarkGroup("Find Many (1000)")


@find_many.add("data-bridge", setup="await _setup_db_find_many()")
async def db_find_many():
    """Find 1000 records with data-bridge."""
    users = await DBUser.find(DBUser.age > 25).limit(1000).to_list()
    return users


async def _setup_db_find_many():
    """Setup: Insert test data for data-bridge."""
    users = [DBUser(**d) for d in DATA_1000]
    await DBUser.insert_many(users)


@find_many.add("asyncpg", setup="await _setup_asyncpg_find_many(asyncpg_pool)")
async def asyncpg_find_many(asyncpg_pool):
    """Find 1000 records with asyncpg."""
    async with asyncpg_pool.acquire() as conn:
        rows = await conn.fetch(
            "SELECT * FROM bench_asyncpg_users WHERE age > $1 LIMIT 1000", 25
        )
        return [dict(row) for row in rows]


async def _setup_asyncpg_find_many(asyncpg_pool):
    """Setup: Insert test data for asyncpg."""
    async with asyncpg_pool.acquire() as conn:
        await conn.executemany(
            """
            INSERT INTO bench_asyncpg_users (name, email, age, city, score, active)
            VALUES ($1, $2, $3, $4, $5, $6)
            """,
            [(d["name"], d["email"], d["age"], d["city"], d["score"], d["active"]) for d in DATA_1000],
        )


@find_many.add("psycopg2", setup="_setup_psycopg2_find_many(psycopg2_conn)")
def psycopg2_find_many(psycopg2_conn):
    """Find 1000 records with psycopg2."""
    conn = psycopg2_conn.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute(
                "SELECT * FROM bench_psycopg2_users WHERE age > %s LIMIT 1000", (25,)
            )
            rows = cur.fetchall()
            return rows
    finally:
        psycopg2_conn.putconn(conn)


def _setup_psycopg2_find_many(psycopg2_conn):
    """Setup: Insert test data for psycopg2."""
    conn = psycopg2_conn.getconn()
    try:
        with conn.cursor() as cur:
            from psycopg2.extras import execute_values

            values = [
                (d["name"], d["email"], d["age"], d["city"], d["score"], d["active"])
                for d in DATA_1000
            ]
            execute_values(
                cur,
                """
                INSERT INTO bench_psycopg2_users (name, email, age, city, score, active)
                VALUES %s
                """,
                values,
            )
            conn.commit()
    finally:
        psycopg2_conn.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @find_many.add("SQLAlchemy", setup="await _setup_sa_find_many(sqlalchemy_session)")
    async def sqlalchemy_find_many(sqlalchemy_session):
        """Find 1000 records with SQLAlchemy."""
        from sqlalchemy import select

        result = await sqlalchemy_session.execute(
            select(SAUser).where(SAUser.age > 25).limit(1000)
        )
        return result.scalars().all()

    async def _setup_sa_find_many(sqlalchemy_session):
        """Setup: Insert test data for SQLAlchemy."""
        users = [SAUser(**d) for d in DATA_1000]
        sqlalchemy_session.add_all(users)
        await sqlalchemy_session.commit()


register_group(find_many)


# =====================
# Count with Filter
# =====================

count_filtered = BenchmarkGroup("Count (with filter)")


@count_filtered.add("data-bridge", setup="await _setup_db_count()")
async def db_count():
    """Count records with filter using data-bridge."""
    count = await DBUser.count(DBUser.age > 30)
    return count


async def _setup_db_count():
    """Setup: Insert test data for data-bridge count."""
    users = [DBUser(**d) for d in DATA_1000]
    await DBUser.insert_many(users)


@count_filtered.add("asyncpg", setup="await _setup_asyncpg_count(asyncpg_pool)")
async def asyncpg_count(asyncpg_pool):
    """Count records with filter using asyncpg."""
    async with asyncpg_pool.acquire() as conn:
        count = await conn.fetchval(
            "SELECT COUNT(*) FROM bench_asyncpg_users WHERE age > $1", 30
        )
        return count


async def _setup_asyncpg_count(asyncpg_pool):
    """Setup: Insert test data for asyncpg count."""
    async with asyncpg_pool.acquire() as conn:
        await conn.executemany(
            """
            INSERT INTO bench_asyncpg_users (name, email, age, city, score, active)
            VALUES ($1, $2, $3, $4, $5, $6)
            """,
            [(d["name"], d["email"], d["age"], d["city"], d["score"], d["active"]) for d in DATA_1000],
        )


@count_filtered.add("psycopg2", setup="_setup_psycopg2_count(psycopg2_conn)")
def psycopg2_count(psycopg2_conn):
    """Count records with filter using psycopg2."""
    conn = psycopg2_conn.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute(
                "SELECT COUNT(*) FROM bench_psycopg2_users WHERE age > %s", (30,)
            )
            count = cur.fetchone()[0]
            return count
    finally:
        psycopg2_conn.putconn(conn)


def _setup_psycopg2_count(psycopg2_conn):
    """Setup: Insert test data for psycopg2 count."""
    conn = psycopg2_conn.getconn()
    try:
        with conn.cursor() as cur:
            from psycopg2.extras import execute_values

            values = [
                (d["name"], d["email"], d["age"], d["city"], d["score"], d["active"])
                for d in DATA_1000
            ]
            execute_values(
                cur,
                """
                INSERT INTO bench_psycopg2_users (name, email, age, city, score, active)
                VALUES %s
                """,
                values,
            )
            conn.commit()
    finally:
        psycopg2_conn.putconn(conn)


if SQLALCHEMY_AVAILABLE:

    @count_filtered.add("SQLAlchemy", setup="await _setup_sa_count(sqlalchemy_session)")
    async def sqlalchemy_count(sqlalchemy_session):
        """Count records with filter using SQLAlchemy."""
        from sqlalchemy import select, func

        result = await sqlalchemy_session.execute(
            select(func.count()).select_from(SAUser).where(SAUser.age > 30)
        )
        return result.scalar()

    async def _setup_sa_count(sqlalchemy_session):
        """Setup: Insert test data for SQLAlchemy count."""
        users = [SAUser(**d) for d in DATA_1000]
        sqlalchemy_session.add_all(users)
        await sqlalchemy_session.commit()


register_group(count_filtered)
