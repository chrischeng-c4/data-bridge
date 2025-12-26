"""
Quick test for insert_many function
"""
import asyncio
from data_bridge import postgres

async def test_insert_many():
    # Initialize connection
    await postgres.init("postgresql://postgres:postgres@localhost:5432/test_db")

    # Test insert_many
    rows_to_insert = [
        {"name": "Alice", "age": 30},
        {"name": "Bob", "age": 25},
        {"name": "Charlie", "age": 35}
    ]

    results = await postgres.insert_many("users", rows_to_insert)
    print(f"Inserted {len(results)} rows:")
    for result in results:
        print(f"  {result}")

    # Clean up
    await postgres.close()

if __name__ == "__main__":
    asyncio.run(test_insert_many())
