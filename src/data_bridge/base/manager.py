from __future__ import annotations

from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, Any, Generic, TypeVar

from .fields import CompoundExpression, QueryExpression

if TYPE_CHECKING:
    from .model import BaseModel
    from .query import BaseQuery

T = TypeVar("T", bound="BaseModel")


class BaseManager(Generic[T], ABC):
    """Abstract base manager class for model queries."""

    def __init__(self, model_class: type[T]) -> None:
        self.model_class = model_class

    @abstractmethod
    def find(
        self,
        *expressions: QueryExpression | CompoundExpression,
    ) -> BaseQuery[T]:
        """Create a query with the given expressions."""
        pass

    @abstractmethod
    def all(self) -> BaseQuery[T]:
        """Return a query for all documents."""
        pass

    def _create_field_expressions(self, **kwargs: Any) -> list[QueryExpression]:
        """Helper to create field expressions from kwargs."""
        expressions = []
        for field_name, value in kwargs.items():
            if field_name not in self.model_class._fields:  # type: ignore[attr-defined]
                raise ValueError(f"Unknown field: {field_name}")
            field = self.model_class._fields[field_name]  # type: ignore[attr-defined]
            expressions.append(field == value)
        return expressions
