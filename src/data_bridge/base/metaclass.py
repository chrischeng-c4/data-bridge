from __future__ import annotations

from typing import Any, get_type_hints

from .fields import Field


class ModelMetaclass(type):
    """Metaclass for ODM models that processes field definitions."""

    def __new__(
        mcs,
        name: str,
        bases: tuple[type[Any], ...],
        namespace: dict[str, Any],
        **kwargs: Any,
    ) -> ModelMetaclass:
        cls = super().__new__(mcs, name, bases, namespace)

        # Collect fields from class and its bases
        fields: dict[str, Field[Any]] = {}

        # First, collect fields from base classes
        for base in bases:
            if hasattr(base, "_fields"):
                fields.update(base._fields)  # type: ignore[attr-defined]

        # Then, collect fields from current class
        for attr_name, attr_value in namespace.items():
            if isinstance(attr_value, Field):
                fields[attr_name] = attr_value

        # Store fields metadata on the class
        cls._fields = fields  # type: ignore[attr-defined]
        cls._collection = kwargs.get("collection") or name.lower() + "s"  # type: ignore[attr-defined]
        cls._database = kwargs.get("database")  # type: ignore[attr-defined]

        # Try to get type hints if class is fully defined
        if not name.startswith("_") and name != "Model":  # Skip internal/base classes
            try:
                hints = get_type_hints(cls)
                for field_name, field in fields.items():
                    if field_name in hints:
                        field.type = hints[field_name]
            except (NameError, AttributeError):
                # Type hints might not be available yet during class creation
                pass

        # Find primary key field
        pk_fields = [f for f in fields.values() if f.primary_key]
        if len(pk_fields) > 1:
            raise ValueError(f"Model {name} has multiple primary key fields")

        cls._pk_field = pk_fields[0] if pk_fields else None  # type: ignore[attr-defined]

        return cls  # type: ignore[return-value]
