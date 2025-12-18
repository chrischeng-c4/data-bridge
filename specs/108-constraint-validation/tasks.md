---
feature: constraint-validation
spec_ref: .specify/specs/108-constraint-validation/spec.md
plan_ref: .specify/specs/108-constraint-validation/plan.md
status: implemented
updated: 2025-12-16
total_tasks: 24
completed_tasks: 24
---

# Atomic Tasks: Constraint Validation (108)

## Task Summary

| Category | Tasks | Total Time | Status |
|----------|-------|------------|--------|
| Phase 1: Python Constraint Classes | 1-6 | ~3 hours | ✅ Completed |
| Phase 2: Type Extraction | 7-10 | ~2 hours | ✅ Completed |
| Phase 3: Rust Validation | 11-17 | ~4 hours | ✅ Completed |
| Phase 4: Integration | 18-20 | ~2 hours | ✅ Completed |
| Phase 5: Testing | 21-24 | ~4 hours | ✅ Completed |
| **Total** | **24 tasks** | **~15 hours** | **✅ Completed** |

## Task Criteria

Each task must be:
- **Atomic**: Cannot be meaningfully subdivided
- **Testable**: Clear verification criteria
- **Independent**: Can be completed without waiting (except explicit deps)
- **Time-boxed**: Completable in < 2 hours
- **Specific**: No ambiguity about what to do

---

## Phase 1: Python Constraint Classes

**Objective**: Create constraint classes that integrate with `typing.Annotated`
**Duration**: ~3 hours

- [x] **Task 1.1**: Create constraints.py module
  - **File**: `python/data_bridge/constraints.py`
  - **Details**: Create module with base Constraint class
  - **Testing**: Module imports without error
  - **Duration**: 15 min

- [x] **Task 1.2**: Implement MinLen constraint
  - **Details**: `class MinLen: def __init__(self, length: int)`
  - **Testing**: `MinLen(3).min_length == 3`
  - **Duration**: 20 min

- [x] **Task 1.3**: Implement MaxLen constraint
  - **Details**: `class MaxLen: def __init__(self, length: int)`
  - **Testing**: `MaxLen(50).max_length == 50`
  - **Duration**: 20 min

- [x] **Task 1.4**: Implement Min and Max constraints
  - **Details**: Numeric range validators for int/float
  - **Testing**: `Min(0).min == 0`, `Max(100).max == 100`
  - **Duration**: 30 min

- [x] **Task 1.5**: Implement Email and Url format validators
  - **Details**: Format constraint classes with `format` attribute
  - **Testing**: `Email().format == 'email'`
  - **Duration**: 30 min

- [x] **Task 1.6**: Export constraints from __init__.py
  - **File**: `python/data_bridge/__init__.py`
  - **Details**: Add exports: MinLen, MaxLen, Min, Max, Email, Url
  - **Testing**: `from data_bridge import MinLen` works
  - **Duration**: 15 min

---

## Phase 2: Type Extraction Enhancement

**Objective**: Extract constraint metadata from Annotated types
**Duration**: ~2 hours

- [x] **Task 2.1**: Detect Annotated types
  - **File**: `python/data_bridge/type_extraction.py`
  - **Details**: Check `get_origin(field_type) is Annotated`
  - **Testing**: Recognizes `Annotated[str, MinLen(3)]`
  - **Duration**: 30 min

- [x] **Task 2.2**: Extract constraint instances
  - **Details**: Get constraint objects from `get_args()`[1:]
  - **Testing**: Extracts MinLen(3) from Annotated type
  - **Duration**: 30 min

- [x] **Task 2.3**: Build constraints dict
  - **Details**: Convert constraint objects to `{'min_length': 3, ...}`
  - **Testing**: Produces correct constraint dict
  - **Duration**: 30 min

- [x] **Task 2.4**: Handle multiple constraints
  - **Details**: Merge multiple constraints into single dict
  - **Testing**: `Annotated[str, MinLen(3), MaxLen(50)]` produces both
  - **Duration**: 30 min

---

## Phase 3: Rust Constraint Validation

**Objective**: Implement constraint checking in validation.rs
**Duration**: ~4 hours

- [x] **Task 3.1**: Define Constraints struct
  - **File**: `crates/data-bridge/src/validation.rs`
  - **Details**: Struct with min_length, max_length, min, max, format fields
  - **Testing**: Struct compiles with derive(Debug, Clone, Default)
  - **Duration**: 20 min

- [x] **Task 3.2**: Parse constraints from Python dict
  - **Details**: Extract constraint values from schema dict
  - **Testing**: `{'constraints': {'min_length': 3}}` parses correctly
  - **Duration**: 30 min

- [x] **Task 3.3**: Implement validate_string_constraints()
  - **Details**: Check min_length and max_length
  - **Testing**: Short string rejected, valid string accepted
  - **Duration**: 45 min

- [x] **Task 3.4**: Implement email format validation
  - **Details**: Regex pattern for basic email validation
  - **Testing**: "user@example.com" valid, "invalid" rejected
  - **Duration**: 30 min

- [x] **Task 3.5**: Implement url format validation
  - **Details**: Pattern for http/https URLs
  - **Testing**: "https://example.com" valid, "not-a-url" rejected
  - **Duration**: 30 min

- [x] **Task 3.6**: Implement validate_numeric_constraints()
  - **Details**: Check min and max for Int64/Double
  - **Testing**: Out of range values rejected
  - **Duration**: 30 min

- [x] **Task 3.7**: Integrate constraints into validate_field()
  - **Details**: Call constraint validators after type check
  - **Testing**: Full validation pipeline works
  - **Duration**: 30 min

---

## Phase 4: Integration

**Objective**: Wire constraint validation into save workflow
**Duration**: ~2 hours

- [x] **Task 4.1**: Update schema dict parsing in mongodb.rs
  - **File**: `crates/data-bridge/src/mongodb.rs`
  - **Details**: Extract constraints from schema when creating descriptors
  - **Testing**: Constraints reach validation functions
  - **Duration**: 45 min

- [x] **Task 4.2**: Update save_validated() method
  - **Details**: Ensure constraints passed through validation
  - **Testing**: save() rejects constraint violations
  - **Duration**: 30 min

- [x] **Task 4.3**: Test end-to-end workflow
  - **Details**: Document with constraints saves/fails correctly
  - **Testing**: Integration test passes
  - **Duration**: 45 min

---

## Phase 5: Testing

**Objective**: Comprehensive test coverage
**Duration**: ~4 hours

- [x] **Task 5.1**: Write string constraint tests
  - **File**: `tests/test_constraints.py`
  - **Details**: Test min_length, max_length acceptance/rejection
  - **Testing**: 6 tests pass
  - **Duration**: 45 min

- [x] **Task 5.2**: Write numeric constraint tests
  - **Details**: Test min, max for int and float
  - **Testing**: 6 tests pass
  - **Duration**: 45 min

- [x] **Task 5.3**: Write format validation tests
  - **Details**: Test email and url validation
  - **Testing**: 6 tests pass
  - **Duration**: 45 min

- [x] **Task 5.4**: Write edge case tests
  - **Details**: Optional with constraints, null handling, boundary values
  - **Testing**: 6 tests pass
  - **Duration**: 45 min

---

## Testing Checklist

### String Constraints
- [x] min_length accepts strings >= limit
- [x] min_length rejects strings < limit
- [x] max_length accepts strings <= limit
- [x] max_length rejects strings > limit
- [x] Combined min/max works correctly

### Numeric Constraints
- [x] min accepts values >= limit
- [x] min rejects values < limit
- [x] max accepts values <= limit
- [x] max rejects values > limit
- [x] Works for both int and float

### Format Validation
- [x] Email accepts valid emails
- [x] Email rejects invalid formats
- [x] URL accepts valid URLs
- [x] URL rejects invalid formats

### Edge Cases
- [x] Optional[Annotated[str, MinLen(3)]] accepts None
- [x] Null field bypasses constraint check
- [x] Empty string vs min_length=0
- [x] Boundary values (exactly at min/max)

---

## Completion Criteria

**Phase 1 Complete When**:
- [x] All constraint classes created
- [x] Exported from data_bridge
- [x] Unit tests for class instantiation

**Phase 2 Complete When**:
- [x] Annotated types detected
- [x] Constraints extracted to dict
- [x] Multiple constraints merged

**Phase 3 Complete When**:
- [x] Constraints struct defined
- [x] String constraints validated
- [x] Numeric constraints validated
- [x] Format validation working

**Phase 4 Complete When**:
- [x] save() validates constraints
- [x] End-to-end workflow tested

**Phase 5 Complete When**:
- [x] 20+ constraint tests passing
- [x] All edge cases covered

---

## Dependencies Between Tasks

```
Phase 1 (Python classes)
  ├─► Phase 2 (Type extraction)
  │     └─► Phase 4.1 (mongodb.rs parsing)
  │           └─► Phase 4.2 (save_validated)
  └─► Phase 3 (Rust validation)
        └─► Phase 4.3 (E2E test)
              └─► Phase 5 (Full testing)
```

---

## Files to Create/Modify

| File | Action | Lines Est. |
|------|--------|------------|
| `python/data_bridge/constraints.py` | CREATE | ~100 |
| `python/data_bridge/type_extraction.py` | MODIFY | ~50 |
| `python/data_bridge/__init__.py` | MODIFY | ~10 |
| `crates/data-bridge/src/validation.rs` | MODIFY | ~200 |
| `crates/data-bridge/src/mongodb.rs` | MODIFY | ~30 |
| `tests/test_constraints.py` | CREATE | ~300 |

---

## Implementation Notes

### Annotated Type Detection
```python
from typing import get_origin, get_args, Annotated

def extract_constraints(field_type):
    if get_origin(field_type) is Annotated:
        base_type = get_args(field_type)[0]
        metadata = get_args(field_type)[1:]
        constraints = {}
        for item in metadata:
            if hasattr(item, 'min_length'):
                constraints['min_length'] = item.min_length
            # ... etc
        return base_type, constraints
    return field_type, {}
```

### Email Regex (Simple)
```rust
lazy_static! {
    static ref EMAIL_RE: Regex = Regex::new(
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
    ).unwrap();
}
```

### Error Message Format
```
ValidationError: name: string too short (min: 3, got: 2)
ValidationError: age: value below minimum (min: 0, got: -5)
ValidationError: email: invalid email format
```
