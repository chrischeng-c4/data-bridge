from __future__ import annotations

from abc import ABC, abstractmethod
from collections.abc import Sequence
from typing import (
    TYPE_CHECKING,
    Generic,
    TypeVar,
)

from .fields import CompoundExpression, QueryExpression

if TYPE_CHECKING:
    from .model import BaseModel

T = TypeVar("T", bound="BaseModel")


class BaseQuery(Generic[T], ABC):
    """Abstract base query builder for database operations."""

    def __init__(
        self,
        model_class: type[T],
        expressions: Sequence[QueryExpression | CompoundExpression],
    ) -> None:
        self.model_class = model_class
        self.expressions = list(expressions)
        self._limit_value: int | None = None
        self._skip_value: int = 0
        self._sort_fields: list[tuple[str, int]] = []
        self._projection: list[str] | None = None

    @abstractmethod
    def filter(
        self,
        *expressions: QueryExpression | CompoundExpression,
    ) -> BaseQuery[T]:
        """Add additional filter expressions."""
        pass

    @abstractmethod
    def limit(self, n: int) -> BaseQuery[T]:
        """Limit the number of results."""
        pass

    @abstractmethod
    def skip(self, n: int) -> BaseQuery[T]:
        """Skip the first n results."""
        pass

    @abstractmethod
    def sort(self, *fields: str | tuple[str, int]) -> BaseQuery[T]:
        """Sort results by field(s)."""
        pass

    @abstractmethod
    def select(self, *fields: str) -> BaseQuery[T]:
        """Select specific fields to return (projection)."""
        pass

    def _parse_sort_fields(self, *fields: str | tuple[str, int]) -> list[tuple[str, int]]:
        """Parse sort field specifications into (field_name, direction) tuples."""
        parsed_fields = []
        for field in fields:
            if isinstance(field, tuple):
                parsed_fields.append(field)
            elif isinstance(field, str):
                if field.startswith("-"):
                    parsed_fields.append((field[1:], -1))
                else:
                    parsed_fields.append((field, 1))
        return parsed_fields
