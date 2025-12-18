---
feature: constraint-validation
spec_ref: .specify/specs/108-constraint-validation/spec.md
status: planned
created: 2025-12-15
updated: 2025-12-16
---

# Implementation Plan: Constraint Validation (108)

## Overview

Add constraint validation to data-bridge for string length (min/max), numeric ranges (min/max), and format validation (email, url) while maintaining 10-100x speedup over Pydantic v2.

## Prerequisites

- [x] Feature 105 (Type Schema Extraction) - COMPLETED
- [x] Feature 106 (Basic Type Validation) - COMPLETED
- [x] Feature 107 (Complex Type Validation) - COMPLETED
- [ ] Python `Annotated` type support

---

## Phase 1: Python Constraint Classes (~3 hours)

### Objective
Create Python constraint classes that integrate with `typing.Annotated`.

### Tasks
1. Create `constraints.py` module
2. Implement `MinLen(n)` and `MaxLen(n)` for strings
3. Implement `Min(n)` and `Max(n)` for numbers
4. Implement `Email()` and `Url()` format validators
5. Export from `__init__.py`

### Files
- `python/data_bridge/constraints.py` (NEW)
- `python/data_bridge/__init__.py`

### API Design
```python
from typing import Annotated
from data_bridge import Document, MinLen, MaxLen, Min, Max, Email

class User(Document):
    name: Annotated[str, MinLen(3), MaxLen(50)]
    age: Annotated[int, Min(0), Max(150)]
    email: Annotated[str, Email()]
```

### Acceptance Criteria
- Constraint classes are importable from data_bridge
- Classes store constraint parameters
- Classes have `__constraint_type__` attribute for identification

---

## Phase 2: Type Extraction Enhancement (~2 hours)

### Objective
Extend type extraction to capture constraint metadata from `Annotated` types.

### Tasks
1. Detect `Annotated` types using `get_origin()` and `get_args()`
2. Extract constraint instances from Annotated metadata
3. Add constraints to type descriptor dict
4. Handle multiple constraints per field

### Files
- `python/data_bridge/type_extraction.py`

### Type Descriptor Format
```python
# Before (type only):
{'type': 'string'}

# After (type + constraints):
{
    'type': 'string',
    'constraints': {
        'min_length': 3,
        'max_length': 50
    }
}
```

### Acceptance Criteria
- `Annotated[str, MinLen(3)]` produces `{'type': 'string', 'constraints': {'min_length': 3}}`
- Multiple constraints merged into single dict
- Non-annotated types work as before (no constraints key)

---

## Phase 3: Rust Constraint Validation (~4 hours)

### Objective
Implement constraint checking in Rust validation pipeline.

### Tasks
1. Add `Constraints` struct to hold constraint metadata
2. Extend `BsonTypeDescriptor` to include optional constraints
3. Implement `validate_string_constraints()`:
   - min_length, max_length checks
   - email regex validation
   - url pattern validation
4. Implement `validate_numeric_constraints()`:
   - min, max range checks
5. Integrate into `validate_field()` workflow
6. Add constraint-specific error messages

### Files
- `crates/data-bridge/src/validation.rs`

### Rust Design
```rust
#[derive(Debug, Clone, Default)]
pub struct Constraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub format: Option<String>,  // "email", "url"
}

pub enum BsonTypeDescriptor {
    String { constraints: Constraints },
    Int64 { constraints: Constraints },
    // ... other types
}
```

### Acceptance Criteria
- String length validated against min/max
- Numeric values validated against min/max
- Email format validated with regex
- URL format validated with pattern
- Error messages include constraint details

---

## Phase 4: Integration (~2 hours)

### Objective
Wire constraint validation into save workflow.

### Tasks
1. Parse constraints from Python schema dict
2. Pass constraints through `save_validated()`
3. Ensure constraints checked before MongoDB insert
4. Test end-to-end workflow

### Files
- `crates/data-bridge/src/mongodb.rs`
- `python/data_bridge/_engine.py`

### Acceptance Criteria
- `Document.save()` validates constraints
- Invalid data rejected before MongoDB call
- Clear error messages for constraint violations

---

## Phase 5: Testing (~3 hours)

### Objective
Comprehensive test coverage for constraint validation.

### Test Categories
1. **Unit Tests (Python)**: Constraint class instantiation
2. **Unit Tests (Rust)**: Individual constraint validators
3. **Integration Tests**: Full save workflow with constraints
4. **Error Message Tests**: Verify constraint error details
5. **Edge Cases**: Boundary values, empty strings, null handling

### Files
- `tests/test_constraints.py` (NEW)
- `tests/test_rust_validation.py` (enhance)

### Test Cases
```python
# String length
test_min_length_accepts_valid()
test_min_length_rejects_short()
test_max_length_accepts_valid()
test_max_length_rejects_long()

# Numeric range
test_min_accepts_valid()
test_min_rejects_below()
test_max_accepts_valid()
test_max_rejects_above()

# Format
test_email_accepts_valid()
test_email_rejects_invalid()
test_url_accepts_valid()
test_url_rejects_invalid()

# Edge cases
test_optional_with_constraints()
test_null_bypasses_constraints()
test_multiple_constraints()
```

### Acceptance Criteria
- 20+ tests for constraint validation
- All constraint types covered
- Error messages verified
- Edge cases handled

---

## Phase 6: Performance Benchmarks (~1 hour)

### Objective
Verify 10-100x speedup over Pydantic constraint validation.

### Benchmarks
1. String length validation throughput
2. Numeric range validation throughput
3. Email format validation throughput
4. Mixed constraints document validation

### Files
- `tests/benchmarks/test_constraint_performance.py` (NEW)

### Acceptance Criteria
- Constraint validation adds <0.5ms overhead
- 10-100x faster than Pydantic equivalent
- No regression in type-only validation

---

## Implementation Summary

**Total Estimated Time**: 15 hours (~2 days)

| Phase | Hours | Status |
|-------|-------|--------|
| Phase 1: Python Classes | 3 | Not Started |
| Phase 2: Type Extraction | 2 | Not Started |
| Phase 3: Rust Validation | 4 | Not Started |
| Phase 4: Integration | 2 | Not Started |
| Phase 5: Testing | 3 | Not Started |
| Phase 6: Benchmarks | 1 | Not Started |

---

## Technical Decisions

### 1. Annotated vs Field()
**Decision**: Use `typing.Annotated` for constraints
**Rationale**: Standard library approach, cleaner than Pydantic's `Field()`

### 2. Constraint Storage
**Decision**: Separate `Constraints` struct, not embedded in type enum
**Rationale**: Not all types need all constraints, keeps enum clean

### 3. Email Validation
**Decision**: Use simple regex, not full RFC 5322
**Rationale**: Pragmatic - catches obvious errors, fast execution

### 4. Null Handling
**Decision**: `None` bypasses constraint validation for Optional fields
**Rationale**: Constraints apply to values, not absence of values

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Regex performance | Medium | Medium | Benchmark, use lazy_static |
| Annotated edge cases | Low | Low | Comprehensive test coverage |
| Constraint interaction | Low | Medium | Clear precedence rules |

---

## Dependencies

### Upstream (Required)
- Feature 107 (Complex Type Validation) - COMPLETED

### Downstream (Blocked by this)
- Full Pydantic feature parity (~80%)
- Production readiness for strict validation

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
| `tests/benchmarks/test_constraint_performance.py` | CREATE | ~100 |
