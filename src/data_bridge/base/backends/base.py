from __future__ import annotations

from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, Any, TypeVar

if TYPE_CHECKING:
    from ..model import Model
    from ..query import Query

T = TypeVar("T", bound="Model")


class BaseBackend(ABC):
    """Abstract base class for database backends."""

    @abstractmethod
    def save(self, instance: Model) -> None:
        """Save a model instance."""
        pass

    @abstractmethod
    async def asave(self, instance: Model) -> None:
        """Async save a model instance."""
        pass

    @abstractmethod
    def delete(self, instance: Model) -> None:
        """Delete a model instance."""
        pass

    @abstractmethod
    async def adelete(self, instance: Model) -> None:
        """Async delete a model instance."""
        pass

    @abstractmethod
    def execute_query(self, query: Query[T]) -> list[T]:
        """Execute a query and return results."""
        pass

    @abstractmethod
    async def aexecute_query(self, query: Query[T]) -> list[T]:
        """Async execute a query and return results."""
        pass

    @abstractmethod
    def count_query(self, query: Query[T]) -> int:
        """Count documents matching a query."""
        pass

    @abstractmethod
    async def acount_query(self, query: Query[T]) -> int:
        """Async count documents matching a query."""
        pass

    @abstractmethod
    def delete_query(self, query: Query[T]) -> int:
        """Delete documents matching a query."""
        pass

    @abstractmethod
    async def adelete_query(self, query: Query[T]) -> int:
        """Async delete documents matching a query."""
        pass

    @abstractmethod
    def update_query(self, query: Query[T], updates: dict[str, Any]) -> int:
        """Update documents matching a query."""
        pass

    @abstractmethod
    async def aupdate_query(
        self, query: Query[T], updates: dict[str, Any]
    ) -> int:
        """Async update documents matching a query."""
        pass
