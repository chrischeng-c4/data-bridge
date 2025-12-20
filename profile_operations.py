"""Profile data-bridge operations to find bottlenecks."""

import asyncio
import cProfile
import pstats
from io import StringIO
from data_bridge import Document, init


class User(Document):
    name: str
    email: str
    age: int

    class Settings:
        name = "profile_users"


async def profile_insert_one():
    """Profile single insert operation."""
    await init("mongodb://localhost:27017/profile")

    # Warm up
    for _ in range(10):
        user = User(name="Test", email="test@test.com", age=30)
        await user.save()

    # Profile 100 inserts
    profiler = cProfile.Profile()
    profiler.enable()

    for i in range(100):
        user = User(name=f"User{i}", email=f"user{i}@test.com", age=20 + i % 50)
        await user.save()

    profiler.disable()

    # Print results
    s = StringIO()
    stats = pstats.Stats(profiler, stream=s).sort_stats('cumulative')
    stats.print_stats(30)  # Top 30 functions
    print("\n=== INSERT ONE PROFILE ===")
    print(s.getvalue())

    # Clean up
    await User.delete_many({})


async def profile_find_one():
    """Profile single find operation."""
    # Insert test data
    for i in range(1000):
        user = User(name=f"User{i}", email=f"user{i}@test.com", age=20 + i % 50)
        await user.save()

    # Warm up
    for _ in range(10):
        await User.find_one(User.age == 35)

    # Profile 100 finds
    profiler = cProfile.Profile()
    profiler.enable()

    for _ in range(100):
        await User.find_one(User.age == 35)

    profiler.disable()

    # Print results
    s = StringIO()
    stats = pstats.Stats(profiler, stream=s).sort_stats('cumulative')
    stats.print_stats(30)  # Top 30 functions
    print("\n=== FIND ONE PROFILE ===")
    print(s.getvalue())

    # Clean up
    await User.delete_many({})


async def profile_bulk_insert():
    """Profile bulk insert operation."""
    # Clean up any existing data first
    await User.delete_many({})

    # Warm up with small batch
    warmup_users = [User(name=f"Warmup{i}", email=f"warmup{i}@test.com", age=20) for i in range(10)]
    await User.insert_many(warmup_users)
    await User.delete_many({})  # Clean up warm-up data

    # Prepare fresh data for profiling
    users = [User(name=f"User{i}", email=f"user{i}@test.com", age=20 + i % 50) for i in range(1000)]

    # Profile bulk insert
    profiler = cProfile.Profile()
    profiler.enable()

    await User.insert_many(users)

    profiler.disable()

    # Print results
    s = StringIO()
    stats = pstats.Stats(profiler, stream=s).sort_stats('cumulative')
    stats.print_stats(30)  # Top 30 functions
    print("\n=== BULK INSERT PROFILE ===")
    print(s.getvalue())

    # Clean up
    await User.delete_many({})


async def main():
    """Run all profiling tests."""
    print("Starting profiling...")

    print("\n" + "="*60)
    print("PROFILING: Insert One (100 operations)")
    print("="*60)
    await profile_insert_one()

    print("\n" + "="*60)
    print("PROFILING: Find One (100 operations)")
    print("="*60)
    await profile_find_one()

    print("\n" + "="*60)
    print("PROFILING: Bulk Insert (1000 documents)")
    print("="*60)
    await profile_bulk_insert()

    print("\n✅ Profiling complete!")


if __name__ == "__main__":
    asyncio.run(main())
