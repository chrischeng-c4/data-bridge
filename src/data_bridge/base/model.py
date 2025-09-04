from __future__ import annotations

from abc import ABCMeta
from typing import TYPE_CHECKING, Any, ClassVar, TypeVar, Union

from .fields import Field
from .metaclass import ModelMetaclass

if TYPE_CHECKING:
    from .backends.async_ import AsyncBackend
    from .backends.sync import SyncBackend

    BaseBackend = SyncBackend | AsyncBackend

T = TypeVar("T", bound="BaseModel")


class CombinedMeta(ModelMetaclass, ABCMeta):
    """Combined metaclass to handle both ModelMetaclass and ABCMeta."""
    pass


class BaseModel(metaclass=CombinedMeta):
    """Abstract base model class providing common functionality."""

    _fields: ClassVar[dict[str, Field[Any]]]
    _collection: ClassVar[str]
    _database: ClassVar[str | None]
    _pk_field: ClassVar[Field[Any] | None]
    _backend: ClassVar[BaseBackend | None] = None

    def __init__(self, **kwargs: Any) -> None:
        """Initialize model instance with field values."""
        for field_name, field in self._fields.items():
            if field_name in kwargs:
                setattr(self, field_name, kwargs[field_name])
            elif field.default is not None:
                setattr(self, field_name, field.default)
            elif field.default_factory is not None:
                setattr(self, field_name, field.default_factory())
            elif field.required:
                raise ValueError(f"Required field '{field_name}' not provided")

    @classmethod
    def set_backend(cls, backend: BaseBackend) -> None:
        """Set the database backend for this model."""
        cls._backend = backend

    def to_dict(self) -> dict[str, Any]:
        """Convert model instance to dictionary."""
        result = {}
        for field_name, field in self._fields.items():
            value = getattr(self, field_name, None)
            if value is not None:
                result[field.db_field or field_name] = value
        return result

    @classmethod
    def from_dict(cls: type[T], data: dict[str, Any]) -> T:
        """Create model instance from dictionary."""
        kwargs = {}
        for field_name, field in cls._fields.items():
            db_field = field.db_field or field_name
            if db_field in data:
                kwargs[field_name] = data[db_field]
        return cls(**kwargs)

    def __repr__(self) -> str:
        field_values = []
        for field_name in self._fields:
            value = getattr(self, field_name, None)
            if value is not None:
                field_values.append(f"{field_name}={value!r}")
        return f"{self.__class__.__name__}({', '.join(field_values)})"

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, self.__class__):
            return False
        for field_name in self._fields:
            if getattr(self, field_name, None) != getattr(other, field_name, None):
                return False
        return True
