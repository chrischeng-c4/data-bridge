---
feature: lazy-validation
spec_ref: .specify/specs/102-lazy-validation/spec.md
status: done
updated: 2025-12-15
---

# Implementation Plan: Lazy Validation

## Overview

Implement lazy validation for data-bridge to skip Pydantic validation when loading documents from the database. Database data is already valid (was validated on insert), so re-validating on every load wastes CPU. User input still goes through `__init__` which validates.

## Task Breakdown

### Phase 1: Core Implementation
**Objective**: Add `validate` parameter to `_from_db()` method
**Estimated Duration**: 2 hours

| Task | Description | Dependencies | Complexity |
|------|-------------|--------------|------------|
| 1.1 | Add `validate: bool = False` parameter to `_from_db()` | None | Simple |
| 1.2 | Implement fast path using `object.__new__()` + direct assignment | 1.1 | Medium |
| 1.3 | Keep slow path (`cls(**data)`) for `validate=True` | 1.2 | Simple |

### Phase 2: Integration
**Objective**: Update all DB load operations to use fast path
**Estimated Duration**: 1 hour

| Task | Description | Dependencies | Complexity |
|------|-------------|--------------|------------|
| 2.1 | Update `find_one()` to use `validate=False` | Phase 1 | Simple |
| 2.2 | Update `find_one_and_update()` to use `validate=False` | Phase 1 | Simple |
| 2.3 | Update `find_one_and_replace()` to use `validate=False` | Phase 1 | Simple |
| 2.4 | Update `find_one_and_delete()` to use `validate=False` | Phase 1 | Simple |
| 2.5 | Verify `QueryBuilder.to_list()` uses Rust direct creation | Phase 1 | Simple |

### Phase 3: Testing
**Objective**: Achieve 100% test coverage for lazy validation
**Estimated Duration**: 2 hours

| Task | Description | Dependencies | Complexity |
|------|-------------|--------------|------------|
| 3.1 | Test user input still validated via `__init__` | Phase 2 | Medium |
| 3.2 | Test DB loads skip validation (fast path) | Phase 2 | Medium |
| 3.3 | Test fast path correctness (same results as slow path) | Phase 2 | Medium |
| 3.4 | Test all find operations use fast path | Phase 2 | Medium |
| 3.5 | Test special fields preserved (_id, revision_id) | Phase 2 | Simple |
| 3.6 | Test custom __init__ validation bypassed on load | Phase 2 | Medium |

### Phase 4: Benchmarks & Documentation
**Objective**: Verify performance improvement and document
**Estimated Duration**: 1 hour

| Task | Description | Dependencies | Complexity |
|------|-------------|--------------|------------|
| 4.1 | Create benchmark: single document with/without validation | Phase 3 | Simple |
| 4.2 | Create benchmark: bulk (100 docs) with/without validation | Phase 3 | Simple |
| 4.3 | Create benchmark: large document (50 fields) | Phase 3 | Simple |
| 4.4 | Update spec status to done | Phase 3 | Simple |

## Dependencies

### Internal Dependencies
- `document.py`: Document base class with `_from_db()` method
- `query.py`: QueryBuilder that creates documents from DB results
- `_engine.py`: Rust engine that handles DB operations

### External Dependencies
- None (pure Python/Rust implementation)

## Test Strategy

### Framework & Coverage
- **Framework**: Pytest with async support
- **Coverage Target**: 100% lazy validation paths covered
- **Test Location**: `tests/test_lazy_validation.py`

**Required Tests**:
- [x] User input validation still works via `__init__`
- [x] Database loads skip validation (fast path)
- [x] Fast path produces correct results
- [x] All find operations use fast path
- [x] Special fields (_id, revision_id) preserved
- [x] Custom `__init__` validation bypassed on load
- [x] Missing fields handled correctly

### Benchmark Tests
- **Location**: `tests/benchmarks/test_lazy_validation_performance.py`
- **Groups**: lazy-validation, lazy-validation-bulk, lazy-validation-large

## Risk Assessment

### Risk 1: Security - Accepting invalid data from DB
- **Probability**: Low
- **Impact**: Medium
- **Mitigation**: Only skip validation for trusted DB reads. User input always goes through `__init__`.

### Risk 2: Breaking changes for existing code
- **Probability**: Low
- **Impact**: Medium
- **Mitigation**: Default behavior unchanged - `validate=False` is the new default which is faster. Code that explicitly needs validation can pass `validate=True`.

### Risk 3: Data corruption if DB has invalid data
- **Probability**: Low
- **Impact**: Medium
- **Mitigation**: Provide `validate=True` option for paranoid cases. Document migration path.

## Success Criteria

### Functional Criteria
- [x] `_from_db(validate=False)` skips `__init__` (fast path)
- [x] `_from_db(validate=True)` runs full validation (slow path)
- [x] User input via `__init__` always validated
- [x] All DB load operations use fast path by default

### Non-Functional Criteria
- [x] All 12 lazy validation tests passing
- [x] All 6 benchmark tests passing
- [x] No regressions in existing tests
- [x] Documentation complete

## Rollback Plan

No database changes required. Rollback by:
1. Revert to previous `_from_db()` implementation without `validate` parameter
2. All find operations would use `__init__` path (slower but safe)

## Implementation Notes

**Key Insight**: Benchmark results show minimal performance difference (~1-2%) because data-bridge doesn't use Pydantic. The base `__init__` is already lightweight:
- Sets `self._data = kwargs.copy()`
- Handles some defaults
- No heavy validation framework

This is actually ideal - the architecture avoids Pydantic overhead entirely while still providing a validation hook point in `__init__` for user input.
