"""Find One benchmark."""

from data_bridge.test import BenchmarkGroup, register_group
from tests.mongo.benchmarks.models import DBUser, BeanieUser

group = BenchmarkGroup("Find One")


@group.add("data-bridge")
async def db_find_one():
    return await DBUser.find_one(DBUser.age == 35)


@group.add("Beanie")
async def beanie_find_one():
    return await BeanieUser.find_one({"age": 35})


register_group(group)
