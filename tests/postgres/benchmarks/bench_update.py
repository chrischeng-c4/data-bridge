"""Update benchmarks for PostgreSQL."""

import pytest
from data_bridge.test import BenchmarkGroup, register_group
from .models import DBUser, SAUser, SQLALCHEMY_AVAILABLE
from .conftest import generate_user_data


# =====================
# Update One
# =====================

update_one = BenchmarkGroup("Update One")


@update_one.add("data-bridge", setup="await _setup_db_update_one()")
async def db_update_one():
    """Update one record with data-bridge."""
    user = await DBUser.find_one(DBUser.id == 1)
    if user:
        user.age = 35
        await user.save()
    return user


async def _setup_db_update_one():
    """Setup: Insert test data for data-bridge update."""
    user = DBUser(name="Test", email="test@test.com", age=30, active=True)
    await user.save()


@update_one.add("asyncpg", setup="await _setup_asyncpg_update_one(asyncpg_pool)")
async def asyncpg_update_one(asyncpg_pool):
    """Update one record with asyncpg."""
    async with asyncpg_pool.acquire() as conn:
        await conn.execute(
            "UPDATE bench_asyncpg_users SET age = $1 WHERE id = $2", 35, 1
        )


async def _setup_asyncpg_update_one(asyncpg_pool):
    """Setup: Insert test data for asyncpg update."""
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


@update_one.add("psycopg2", setup="_setup_psycopg2_update_one(psycopg2_conn)")
def psycopg2_update_one(psycopg2_conn):
    """Update one record with psycopg2."""
    conn = psycopg2_conn.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute(
                "UPDATE bench_psycopg2_users SET age = %s WHERE id = %s", (35, 1)
            )
            conn.commit()
    finally:
        psycopg2_conn.putconn(conn)


def _setup_psycopg2_update_one(psycopg2_conn):
    """Setup: Insert test data for psycopg2 update."""
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

    @update_one.add("SQLAlchemy", setup="await _setup_sa_update_one(sqlalchemy_session)")
    async def sqlalchemy_update_one(sqlalchemy_session):
        """Update one record with SQLAlchemy."""
        from sqlalchemy import select

        result = await sqlalchemy_session.execute(select(SAUser).where(SAUser.id == 1))
        user = result.scalar_one_or_none()
        if user:
            user.age = 35
            await sqlalchemy_session.commit()
        return user

    async def _setup_sa_update_one(sqlalchemy_session):
        """Setup: Insert test data for SQLAlchemy update."""
        user = SAUser(name="Test", email="test@test.com", age=30, active=True)
        sqlalchemy_session.add(user)
        await sqlalchemy_session.commit()


register_group(update_one)


# =====================
# Update Many (1000 records)
# =====================

DATA_1000 = generate_user_data(1000)

update_many = BenchmarkGroup("Update Many (1000)")


@update_many.add("data-bridge", setup="await _setup_db_update_many()")
async def db_update_many():
    """Update 1000 records with data-bridge."""
    # Note: This is a placeholder - adjust based on actual data-bridge API
    result = await DBUser.update_many(
        DBUser.age > 25, {"age": 40}
    )
    return result


async def _setup_db_update_many():
    """Setup: Insert test data for data-bridge bulk update."""
    users = [DBUser(**d) for d in DATA_1000]
    await DBUser.insert_many(users)


@update_many.add("asyncpg", setup="await _setup_asyncpg_update_many(asyncpg_pool)")
async def asyncpg_update_many(asyncpg_pool):
    """Update 1000 records with asyncpg."""
    async with asyncpg_pool.acquire() as conn:
        result = await conn.execute(
            "UPDATE bench_asyncpg_users SET age = $1 WHERE age > $2", 40, 25
        )
        return result


async def _setup_asyncpg_update_many(asyncpg_pool):
    """Setup: Insert test data for asyncpg bulk update."""
    async with asyncpg_pool.acquire() as conn:
        await conn.executemany(
            """
            INSERT INTO bench_asyncpg_users (name, email, age, city, score, active)
            VALUES ($1, $2, $3, $4, $5, $6)
            """,
            [(d["name"], d["email"], d["age"], d["city"], d["score"], d["active"]) for d in DATA_1000],
        )


@update_many.add("psycopg2", setup="_setup_psycopg2_update_many(psycopg2_conn)")
def psycopg2_update_many(psycopg2_conn):
    """Update 1000 records with psycopg2."""
    conn = psycopg2_conn.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute(
                "UPDATE bench_psycopg2_users SET age = %s WHERE age > %s", (40, 25)
            )
            conn.commit()
            return cur.rowcount
    finally:
        psycopg2_conn.putconn(conn)


def _setup_psycopg2_update_many(psycopg2_conn):
    """Setup: Insert test data for psycopg2 bulk update."""
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

    @update_many.add("SQLAlchemy", setup="await _setup_sa_update_many(sqlalchemy_session)")
    async def sqlalchemy_update_many(sqlalchemy_session):
        """Update 1000 records with SQLAlchemy."""
        from sqlalchemy import update

        result = await sqlalchemy_session.execute(
            update(SAUser).where(SAUser.age > 25).values(age=40)
        )
        await sqlalchemy_session.commit()
        return result.rowcount

    async def _setup_sa_update_many(sqlalchemy_session):
        """Setup: Insert test data for SQLAlchemy bulk update."""
        users = [SAUser(**d) for d in DATA_1000]
        sqlalchemy_session.add_all(users)
        await sqlalchemy_session.commit()


register_group(update_many)


# =====================
# Delete Many (500 records)
# =====================

delete_many = BenchmarkGroup("Delete Many (500)")


@delete_many.add("data-bridge", setup="await _setup_db_delete_many()")
async def db_delete_many():
    """Delete 500 records with data-bridge."""
    # Note: This is a placeholder - adjust based on actual data-bridge API
    result = await DBUser.delete_many(DBUser.age > 45)
    return result


async def _setup_db_delete_many():
    """Setup: Insert test data for data-bridge delete."""
    users = [DBUser(**d) for d in DATA_1000]
    await DBUser.insert_many(users)


@delete_many.add("asyncpg", setup="await _setup_asyncpg_delete_many(asyncpg_pool)")
async def asyncpg_delete_many(asyncpg_pool):
    """Delete 500 records with asyncpg."""
    async with asyncpg_pool.acquire() as conn:
        result = await conn.execute(
            "DELETE FROM bench_asyncpg_users WHERE age > $1", 45
        )
        return result


async def _setup_asyncpg_delete_many(asyncpg_pool):
    """Setup: Insert test data for asyncpg delete."""
    async with asyncpg_pool.acquire() as conn:
        await conn.executemany(
            """
            INSERT INTO bench_asyncpg_users (name, email, age, city, score, active)
            VALUES ($1, $2, $3, $4, $5, $6)
            """,
            [(d["name"], d["email"], d["age"], d["city"], d["score"], d["active"]) for d in DATA_1000],
        )


@delete_many.add("psycopg2", setup="_setup_psycopg2_delete_many(psycopg2_conn)")
def psycopg2_delete_many(psycopg2_conn):
    """Delete 500 records with psycopg2."""
    conn = psycopg2_conn.getconn()
    try:
        with conn.cursor() as cur:
            cur.execute("DELETE FROM bench_psycopg2_users WHERE age > %s", (45,))
            conn.commit()
            return cur.rowcount
    finally:
        psycopg2_conn.putconn(conn)


def _setup_psycopg2_delete_many(psycopg2_conn):
    """Setup: Insert test data for psycopg2 delete."""
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

    @delete_many.add("SQLAlchemy", setup="await _setup_sa_delete_many(sqlalchemy_session)")
    async def sqlalchemy_delete_many(sqlalchemy_session):
        """Delete 500 records with SQLAlchemy."""
        from sqlalchemy import delete

        result = await sqlalchemy_session.execute(
            delete(SAUser).where(SAUser.age > 45)
        )
        await sqlalchemy_session.commit()
        return result.rowcount

    async def _setup_sa_delete_many(sqlalchemy_session):
        """Setup: Insert test data for SQLAlchemy delete."""
        users = [SAUser(**d) for d in DATA_1000]
        sqlalchemy_session.add_all(users)
        await sqlalchemy_session.commit()


register_group(delete_many)
