---
feature: lazy-validation
components:
  - data-bridge
lead: data-bridge
status: done
created: 2025-12-15
branch: feature/cross_data-bridge
---

# Specification: Lazy Validation

## 1. Problem Definition

### 1.1 Current State

data-bridge validates data twice for DB-loaded documents:
1. On load: `User(**data)` → Full Pydantic validation
2. On save: `user.save()` → Re-validation

**Problem**: Database data is already valid! We're wasting CPU on trusted data.

### 1.2 Proposed Solution

Skip validation for DB-loaded documents, validate only user input:

```python
# User creates document → VALIDATE
user = User(email="test@example.com", age=30)

# Load from DB → SKIP validation
user = await User.find_one(User.email == "test@example.com")

# Save → Optional validation
await user.save(validate=True)  # Explicit if needed
```

### 1.3 Success Criteria

- ✅ DB reads are 2-3x faster (skip Pydantic validation)
- ✅ User input still validated (security maintained)
- ✅ All 449 existing tests pass
- ✅ Backward compatible (validation defaults match current behavior)

---

## 2. Technical Design

### 2.1 from_db() Method

```python
# data_bridge/document.py

class Document:
    @classmethod
    def _from_db(cls: Type[T], data: Dict[str, Any], validate: bool = False) -> T:
        """Create instance from database document.

        Args:
            data: Raw document from MongoDB
            validate: If True, run Pydantic validation (default: False)

        Returns:
            Document instance
        """
        if validate:
            # Slow path: Full Pydantic validation
            return cls(**data)
        else:
            # Fast path: Direct attribute assignment (skip validation)
            instance = object.__new__(cls)
            instance._id = data.get("_id")
            instance._data = data
            instance._state = StateTracker(data)
            return instance
```

### 2.2 find() Integration

```python
# data_bridge/query_builder.py

class QueryBuilder:
    async def to_list(self, validate: bool = False) -> List[T]:
        """Execute query and return documents.

        Args:
            validate: If True, validate documents loaded from DB

        Returns:
            List of documents
        """
        cursor = await _engine.find(
            collection=self._collection,
            filter=self._filter,
            # ... other params ...
        )

        return [
            self._document_class._from_db(doc, validate=validate)
            for doc in cursor
        ]
```

### 2.3 Configuration

```python
# data_bridge/__init__.py

# Global setting for validation mode
LAZY_VALIDATION = True  # Default: skip DB validation

class DocumentSettings:
    """Per-document validation settings."""
    validate_on_load: bool = not LAZY_VALIDATION  # Override global
    validate_on_save: bool = True  # Always validate saves
```

---

## 3. Testing Strategy

### 3.1 Unit Tests

```python
# tests/test_lazy_validation.py

import pytest
from data_bridge import Document

class User(Document):
    email: str
    age: int

    class Settings:
        name = "users"

@pytest.mark.asyncio
async def test_from_db_skips_validation():
    """Verify from_db(validate=False) skips Pydantic."""
    # Invalid data (age is string)
    data = {"email": "test@example.com", "age": "not_a_number"}

    # from_db with validate=False should NOT raise
    user = User._from_db(data, validate=False)
    assert user.email == "test@example.com"
    assert user.age == "not_a_number"  # Invalid type accepted

@pytest.mark.asyncio
async def test_from_db_validates_when_requested():
    """Verify from_db(validate=True) runs Pydantic."""
    data = {"email": "test@example.com", "age": "not_a_number"}

    # from_db with validate=True should raise
    with pytest.raises(ValidationError):
        User._from_db(data, validate=True)

@pytest.mark.asyncio
async def test_user_input_always_validated():
    """Verify __init__ always validates."""
    # Invalid data (age is string)
    with pytest.raises(ValidationError):
        User(email="test@example.com", age="not_a_number")
```

### 3.2 Performance Benchmarks

```python
# tests/benchmarks/test_lazy_validation.py

@pytest.mark.benchmark(group="validation")
def test_validation_overhead(benchmark):
    """Baseline: Pydantic validation time."""
    data = {"email": "test@example.com", "age": 30}

    def create_with_validation():
        return User(**data)

    benchmark(create_with_validation)
    # Expected: ~10-50μs

@pytest.mark.benchmark(group="validation")
def test_lazy_load_overhead(benchmark):
    """Lazy load without validation."""
    data = {"email": "test@example.com", "age": 30}

    def create_without_validation():
        return User._from_db(data, validate=False)

    benchmark(create_without_validation)
    # Expected: <5μs (2-10x faster)
```

---

## 4. Implementation Plan

### Phase 1: Core Implementation (Day 1)
- Add `validate` parameter to `_from_db()`
- Implement fast path (direct attribute assignment)
- Add unit tests

### Phase 2: Integration (Day 2)
- Update `QueryBuilder.to_list()` with `validate` param
- Update `find_one()`, `find_many()`
- Ensure `__init__` always validates user input

### Phase 3: Testing (Day 3)
- Run all 449 tests
- Add performance benchmarks
- Verify 2-3x speedup on queries

---

## 5. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Security: accepting invalid data from DB | High | Only skip validation for trusted DB reads, validate user input |
| Breaking changes | Medium | Default behavior unchanged (opt-in lazy validation) |
| Data corruption if DB has invalid data | Medium | Document migration path, provide validate=True option |

---

## 6. Dependencies

**Depends on**: None
**Enables**: 104 (Rust-Side Query Execution) - combines well with batch processing

---

## 7. Out of Scope

- ❌ Rust validation (see specs 105-108)
- ❌ Custom validators
- ❌ Schema evolution

---

## Success Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| Query performance | ~10-50μs validation overhead | <5μs | pytest-benchmark |
| Tests passing | 449 | 449 | pytest |
| Security | User input validated | Unchanged | Security tests |

**Status**: Ready for implementation
**Next Step**: Implement `_from_db(validate=False)` fast path
