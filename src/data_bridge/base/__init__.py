"""
Base utilities and abstractions for Data Bridge ORM.

This package provides shared components used by all database backends.
"""

from .fields import CompoundExpression, Field, QueryExpression
from .manager import BaseManager
from .metaclass import ModelMetaclass
from .model import BaseModel
from .query import BaseQuery

__all__ = [
    "BaseManager",
    "BaseModel",
    "BaseQuery",
    "CompoundExpression",
    "Field",
    "ModelMetaclass",
    "QueryExpression",
]
