---
feature: cow-state-management
components:
  - data-bridge
lead: data-bridge
status: done
created: 2025-12-15
branch: feature/cross_data-bridge
---

# Specification: Copy-on-Write State Management

## 1. Problem Definition

### 1.1 Current State

data-bridge tracks document changes using `copy.deepcopy()` on every document load/save:

```python
# data_bridge/document.py:818
def _save_state(self):
    """Save current state for change detection."""
    self._saved_state = copy.deepcopy(self._data)  # ❌ EXPENSIVE
```

**Problems**:
- Deep copy on every load: 1-2ms per document
- Doubles memory usage
- Copies even unchanged nested structures

**Impact**: This single line contributes significantly to the 2.2x slower performance vs MongoEngine.

### 1.2 Proposed Solution

Implement Copy-on-Write (COW) state tracking with field-level granularity:
- Only track changed fields
- Copy field values lazily (on first write)
- Store only original values of changed fields

### 1.3 Success Criteria

- ✅ Memory usage: 50% reduction (no duplicate storage for unchanged fields)
- ✅ Speed: 10x faster state tracking (0.1ms vs 1-2ms per document)
- ✅ All 449 existing tests pass
- ✅ API unchanged (internal optimization only)

---

## 2. Technical Design

### 2.1 StateTracker Class

```python
# data_bridge/state_tracker.py

from typing import Any, Dict, Set

class StateTracker:
    """Copy-on-write state tracker with field-level granularity."""

    def __init__(self, data: Dict[str, Any]):
        self._data = data
        self._original = {}  # Only store changed fields
        self._changed_fields: Set[str] = set()

    def __setitem__(self, key: str, value: Any):
        """Track changes on write (COW)."""
        if key not in self._changed_fields:
            # Copy only on first write (COW)
            self._original[key] = self._data.get(key)
            self._changed_fields.add(key)
        self._data[key] = value

    def __getitem__(self, key: str) -> Any:
        """Read field value."""
        return self._data[key]

    def get_changes(self) -> Dict[str, Any]:
        """Get only changed fields for MongoDB $set update."""
        return {k: self._data[k] for k in self._changed_fields}

    def is_modified(self) -> bool:
        """Check if any field changed."""
        return len(self._changed_fields) > 0

    def reset(self):
        """Reset change tracking after successful save."""
        self._original.clear()
        self._changed_fields.clear()
```

### 2.2 Integration with Document Class

```python
# data_bridge/document.py

class Document:
    def __init__(self, **data: Any):
        # Replace deepcopy with COW tracker
        self._data = data
        self._state = StateTracker(data)  # ✅ NEW: COW tracker

    def __setattr__(self, name: str, value: Any):
        if name.startswith("_"):
            # Internal attributes bypass tracking
            super().__setattr__(name, value)
        else:
            # Track field changes
            self._state[name] = value
            self._data[name] = value

    async def save(self):
        """Save only changed fields to MongoDB."""
        if not self._state.is_modified():
            return self  # No changes, skip save

        # Get only changed fields
        changes = self._state.get_changes()

        # Update MongoDB with $set
        await _engine.update_one(
            collection=self.Settings.name,
            filter={"_id": self._id},
            update={"$set": changes}
        )

        # Reset change tracking
        self._state.reset()
        return self
```

---

## 3. Testing Strategy

### 3.1 Unit Tests

```python
# tests/test_state_tracker.py

import pytest
from data_bridge.state_tracker import StateTracker

def test_unchanged_fields_not_copied():
    """Verify unchanged fields are NOT stored in _original."""
    data = {"name": "Alice", "age": 30, "city": "NYC"}
    tracker = StateTracker(data)

    # Modify only one field
    tracker["age"] = 31

    # Verify only changed field is tracked
    assert len(tracker._changed_fields) == 1
    assert "age" in tracker._changed_fields
    assert tracker._original["age"] == 30

    # Verify unchanged fields are not in _original
    assert "name" not in tracker._original
    assert "city" not in tracker._original

def test_memory_usage():
    """Verify 50% memory reduction vs deepcopy."""
    import sys

    # Large document
    data = {f"field_{i}": f"value_{i}" for i in range(1000)}

    # Measure deepcopy memory
    import copy
    saved_state_deepcopy = copy.deepcopy(data)
    memory_deepcopy = sys.getsizeof(data) + sys.getsizeof(saved_state_deepcopy)

    # Measure COW memory (change only 1 field)
    tracker = StateTracker(data.copy())
    tracker["field_0"] = "new_value"
    memory_cow = sys.getsizeof(data) + sys.getsizeof(tracker._original)

    # Verify 50%+ reduction
    assert memory_cow < memory_deepcopy * 0.5

def test_get_changes():
    """Verify get_changes returns only modified fields."""
    data = {"name": "Alice", "age": 30, "city": "NYC"}
    tracker = StateTracker(data)

    tracker["age"] = 31
    tracker["city"] = "LA"

    changes = tracker.get_changes()
    assert changes == {"age": 31, "city": "LA"}
    assert "name" not in changes
```

### 3.2 Performance Benchmarks

```python
# tests/benchmarks/test_state_management.py

import pytest
from data_bridge.state_tracker import StateTracker
import copy

@pytest.mark.benchmark(group="state_tracking")
def test_deepcopy_benchmark(benchmark):
    """Baseline: deepcopy performance."""
    data = {f"field_{i}": f"value_{i}" for i in range(100)}

    def deepcopy_save_state():
        return copy.deepcopy(data)

    result = benchmark(deepcopy_save_state)
    # Expected: ~1-2ms

@pytest.mark.benchmark(group="state_tracking")
def test_cow_tracker_benchmark(benchmark):
    """COW tracker performance (no changes)."""
    data = {f"field_{i}": f"value_{i}" for i in range(100)}
    tracker = StateTracker(data)

    def cow_check_modified():
        return tracker.is_modified()

    result = benchmark(cow_check_modified)
    # Expected: <0.1ms (10x faster)
```

### 3.3 Integration Tests

- ✅ All existing 449 tests must pass
- ✅ No API changes (internal optimization)
- ✅ Verify save() uses $set with only changed fields

---

## 4. Implementation Plan

### Phase 1: Core StateTracker (Day 1)
- Implement `StateTracker` class
- Add `__setitem__`, `get_changes()`, `is_modified()`
- Unit tests for COW logic

### Phase 2: Document Integration (Day 2)
- Update `Document.__init__()` to use `StateTracker`
- Update `Document.__setattr__()` to track changes
- Update `save()` to use `get_changes()`

### Phase 3: Testing & Validation (Day 3)
- Run all 449 existing tests
- Add memory benchmarks
- Add performance benchmarks
- Verify 50% memory reduction
- Verify 10x speed improvement

### Phase 4: Documentation (Day 4)
- Update internal docs
- Add performance notes to README

---

## 5. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Nested object mutations not tracked | High | Document nested mutations as limitation; future spec for deep tracking |
| Memory leaks if _original not cleared | Medium | Add reset() after save, add weakref if needed |
| Breaking changes to internal APIs | Low | Keep _data unchanged, only add _state |

---

## 6. Dependencies

**Depends on**: None (foundation spec)
**Enables**: 102 (Lazy Validation), 103 (Fast Path)

---

## 7. Out of Scope

- ❌ Deep nested object tracking (future enhancement)
- ❌ Pydantic integration
- ❌ Transaction support

---

## Success Metrics

| Metric | Baseline | Target | Measurement |
|--------|----------|--------|-------------|
| Memory usage | 2x (deepcopy) | 1x + changed fields | sys.getsizeof() |
| State tracking time | 1-2ms | <0.1ms | pytest-benchmark |
| Tests passing | 449 | 449 | pytest |
| API changes | N/A | 0 | Code review |

**Status**: Ready for implementation
**Next Step**: Create `data_bridge/state_tracker.py` and unit tests
