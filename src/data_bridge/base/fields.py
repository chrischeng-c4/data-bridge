from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from typing import Any, Self, TypeVar, overload

T = TypeVar("T")


@dataclass(slots=True, frozen=True)
class QueryExpression:
    """Base class for query expressions."""

    field: str
    operator: str
    value: Any

    def __and__(self, other: QueryExpression) -> CompoundExpression:
        return CompoundExpression("and", [self, other])

    def __or__(self, other: QueryExpression) -> CompoundExpression:
        return CompoundExpression("or", [self, other])

    def __invert__(self) -> CompoundExpression:
        return CompoundExpression("not", [self])


@dataclass(slots=True, frozen=True)
class CompoundExpression:
    """Compound expression for combining multiple query expressions."""

    operator: str  # 'and', 'or', 'not'
    expressions: list[QueryExpression | CompoundExpression]

    def __and__(self, other: QueryExpression | CompoundExpression) -> CompoundExpression:
        return CompoundExpression("and", [self, other])

    def __or__(self, other: QueryExpression | CompoundExpression) -> CompoundExpression:
        return CompoundExpression("or", [self, other])

    def __invert__(self) -> CompoundExpression:
        return CompoundExpression("not", [self])


class Field[T]:
    """Base field descriptor with operator support for query building."""

    def __init__(
        self,
        default: T | None = None,
        *,
        default_factory: Callable[[], T] | None = None,
        required: bool = True,
        db_field: str | None = None,
        primary_key: bool = False,
        index: bool = False,
        unique: bool = False,
    ) -> None:
        if default is not None and default_factory is not None:
            raise ValueError("Cannot specify both default and default_factory")

        self.default = default
        self.default_factory = default_factory
        self.required = required
        self.db_field = db_field
        self.primary_key = primary_key
        self.index = index
        self.unique = unique
        self.name: str | None = None  # Set by metaclass
        self.type: type[T] | None = None  # Set by metaclass

    def __set_name__(self, owner: type[Any], name: str) -> None:
        self.name = name
        if self.db_field is None:
            self.db_field = name

    @overload
    def __get__(self, obj: None, objtype: type[Any]) -> Self: ...

    @overload
    def __get__(self, obj: Any, objtype: type[Any]) -> T: ...

    def __get__(self, obj: Any | None, objtype: type[Any]) -> Self | T:
        if obj is None:
            return self

        value = obj.__dict__.get(self.name)
        if value is None:
            if self.default_factory is not None:
                value = self.default_factory()
                obj.__dict__[self.name] = value
            else:
                value = self.default

        return value

    def __set__(self, obj: Any, value: T) -> None:
        if self.required and value is None:
            raise ValueError(f"Field {self.name} is required")
        obj.__dict__[self.name] = value

    def __eq__(self, other: Any) -> QueryExpression:  # type: ignore[override]
        return QueryExpression(self.db_field or self.name or "", "eq", other)

    def __ne__(self, other: Any) -> QueryExpression:  # type: ignore[override]
        return QueryExpression(self.db_field or self.name or "", "ne", other)

    def __lt__(self, other: Any) -> QueryExpression:
        return QueryExpression(self.db_field or self.name or "", "lt", other)

    def __le__(self, other: Any) -> QueryExpression:
        return QueryExpression(self.db_field or self.name or "", "lte", other)

    def __gt__(self, other: Any) -> QueryExpression:
        return QueryExpression(self.db_field or self.name or "", "gt", other)

    def __ge__(self, other: Any) -> QueryExpression:
        return QueryExpression(self.db_field or self.name or "", "gte", other)

    def in_(self, values: list[Any]) -> QueryExpression:
        """Check if field value is in a list of values."""
        return QueryExpression(self.db_field or self.name or "", "in", values)

    def not_in(self, values: list[Any]) -> QueryExpression:
        """Check if field value is not in a list of values."""
        return QueryExpression(self.db_field or self.name or "", "not_in", values)

    def contains(self, value: str) -> QueryExpression:
        """String contains operation."""
        return QueryExpression(self.db_field or self.name or "", "contains", value)

    def startswith(self, value: str) -> QueryExpression:
        """String starts with operation."""
        return QueryExpression(self.db_field or self.name or "", "startswith", value)

    def endswith(self, value: str) -> QueryExpression:
        """String ends with operation."""
        return QueryExpression(self.db_field or self.name or "", "endswith", value)

    def exists(self, value: bool = True) -> QueryExpression:
        """Check if field exists (for document databases)."""
        return QueryExpression(self.db_field or self.name or "", "exists", value)


class IntField(Field[int]):
    """Integer field."""

    pass


class FloatField(Field[float]):
    """Float field."""

    pass


class StringField(Field[str]):
    """String field with additional string-specific operations."""

    def __init__(
        self,
        default: str | None = None,
        *,
        max_length: int | None = None,
        min_length: int | None = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(default, **kwargs)
        self.max_length = max_length
        self.min_length = min_length

    def regex(self, pattern: str) -> QueryExpression:
        """Regex match operation."""
        return QueryExpression(self.db_field or self.name or "", "regex", pattern)


class BoolField(Field[bool]):
    """Boolean field."""

    pass


class ListField[T](Field[list[T]]):
    """List field for array/list values."""

    def __init__(
        self,
        item_type: type[T],
        default: list[T] | None = None,
        **kwargs: Any,
    ) -> None:
        super().__init__(default or [], **kwargs)
        self.item_type = item_type

    def contains_all(self, values: list[T]) -> QueryExpression:
        """Check if list contains all specified values."""
        return QueryExpression(self.db_field or self.name or "", "contains_all", values)

    def contains_any(self, values: list[T]) -> QueryExpression:
        """Check if list contains any of the specified values."""
        return QueryExpression(self.db_field or self.name or "", "contains_any", values)


class DictField(Field[dict[str, Any]]):
    """Dictionary/object field for nested data."""

    def get_nested(self, path: str) -> NestedFieldProxy:
        """Access nested field for queries."""
        full_path = f"{self.db_field or self.name or ''}.{path}"
        return NestedFieldProxy(full_path)


class NestedFieldProxy:
    """Proxy for accessing nested fields in queries."""

    def __init__(self, path: str) -> None:
        self.path = path

    def __eq__(self, other: Any) -> QueryExpression:  # type: ignore[override]
        return QueryExpression(self.path, "eq", other)

    def __ne__(self, other: Any) -> QueryExpression:  # type: ignore[override]
        return QueryExpression(self.path, "ne", other)

    def __lt__(self, other: Any) -> QueryExpression:
        return QueryExpression(self.path, "lt", other)

    def __le__(self, other: Any) -> QueryExpression:
        return QueryExpression(self.path, "lte", other)

    def __gt__(self, other: Any) -> QueryExpression:
        return QueryExpression(self.path, "gt", other)

    def __ge__(self, other: Any) -> QueryExpression:
        return QueryExpression(self.path, "gte", other)
