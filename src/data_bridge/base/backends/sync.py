from __future__ import annotations

from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, Any, TypeVar

if TYPE_CHECKING:
    from ..model import BaseModel
    from ..query import BaseQuery

T = TypeVar("T", bound="BaseModel")


class SyncBackend(ABC):
    """Abstract base class for synchronous database backends."""

    @abstractmethod
    def save(self, instance: BaseModel) -> None:
        """Save a model instance."""
        pass

    @abstractmethod
    def delete(self, instance: BaseModel) -> None:
        """Delete a model instance."""
        pass

    @abstractmethod
    def execute_query(self, query: BaseQuery[T]) -> list[T]:
        """Execute a query and return results."""
        pass

    @abstractmethod
    def count_query(self, query: BaseQuery[T]) -> int:
        """Count documents matching a query."""
        pass

    @abstractmethod
    def delete_query(self, query: BaseQuery[T]) -> int:
        """Delete documents matching a query."""
        pass

    @abstractmethod
    def update_query(self, query: BaseQuery[T], updates: dict[str, Any]) -> int:
        """Update documents matching a query."""
        pass
