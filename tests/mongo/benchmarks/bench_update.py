"""Update benchmark."""

from data_bridge.test import BenchmarkGroup, register_group
from tests.mongo.benchmarks.models import DBUser, BeanieUser

group = BenchmarkGroup("Update Many")


@group.add("data-bridge")
async def db_update_many():
    await DBUser.update_many(DBUser.age >= 30, {"$inc": {"age": 1}})


@group.add("Beanie")
async def beanie_update_many():
    await BeanieUser.find({"age": {"$gte": 30}}).update_many({"$inc": {"age": 1}})


register_group(group)
