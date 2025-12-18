---
feature: basic-type-validation
spec_ref: .specify/specs/106-basic-type-validation/spec.md
status: completed
created: 2025-12-15
updated: 2025-12-16
---

# Implementation Plan: Basic Type Validation (106)

## Overview

Implement runtime type validation for basic Python types (str, int, float, bool, Optional[T]) in Rust, achieving 10-100x speedup over Pydantic v2.

## Prerequisites

- [x] Feature 105 (Type Schema Extraction) - COMPLETED
- [x] Rust validation infrastructure in validation.rs
- [x] PyO3 integration layer

---

## Phase 1: Rust Validation Logic (~4 hours)

### Objective
Implement type validation functions in Rust that check BSON values against schema descriptors.

### Tasks
1. Define `BsonTypeDescriptor` enum for all supported types
2. Implement `validate_field()` for recursive field validation
3. Implement `validate_document()` for full document validation
4. Handle Optional[T] with null checking
5. Add detailed error messages with field paths

### Files
- `crates/data-bridge/src/validation.rs`

### Acceptance Criteria
- All basic types validated: String, Int64, Double, Bool
- Optional fields accept None values
- Required fields reject missing values
- Error messages show exact field path

---

## Phase 2: Python/Rust Integration (~3 hours)

### Objective
Wire validation into Document save operations via PyO3.

### Tasks
1. Add `save_validated()` method to RustDocument
2. Convert Python schema dict to Rust type descriptors
3. Call validation before MongoDB insert
4. Raise PyValueError with descriptive messages on failure
5. Integrate with `_engine.save()` function

### Files
- `crates/data-bridge/src/mongodb.rs`
- `python/data_bridge/_engine.py`

### Acceptance Criteria
- Validation called automatically on save
- Schema extracted once per class (cached)
- Errors propagate to Python cleanly
- No performance regression for valid data

---

## Phase 3: Testing & Validation (~5 hours)

### Objective
Comprehensive test coverage for all basic type validation scenarios.

### Test Categories
1. **Unit Tests** - Individual type validation functions
2. **Integration Tests** - Full save workflow with validation
3. **Error Tests** - Validation error detection and reporting
4. **Edge Cases** - Optional fields, nested objects, arrays

### Files
- `tests/test_rust_validation.py`

### Acceptance Criteria
- 40+ tests covering all basic types
- Error message validation (field paths)
- Nested document validation
- List element validation with array indices

---

## Phase 4: Performance Benchmarks (~2 hours)

### Objective
Verify 10-100x speedup over Pydantic v2 validation.

### Benchmarks
1. Single document validation overhead
2. Bulk validation throughput
3. Schema extraction caching effectiveness

### Acceptance Criteria
- Validation adds <1ms overhead per document
- 10-100x faster than equivalent Pydantic validation
- Zero overhead after initial schema caching

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Type mismatch edge cases | Medium | Low | Comprehensive test coverage |
| Performance regression | Low | Medium | Benchmark before/after |
| Error message clarity | Medium | Low | User testing feedback |

---

## Dependencies & Blockers

### Upstream
- Feature 105 (Type Schema Extraction) - **COMPLETED**

### Downstream (Blocked by this feature)
- Feature 107 (Complex Type Validation) - List[T], Dict, nested
- Feature 108 (Constraint Validation) - min/max, regex, custom

---

## Implementation Summary

**Total Estimated Time**: 14 hours (~2 days)

| Phase | Hours | Status |
|-------|-------|--------|
| Phase 1: Rust Logic | 4 | Completed |
| Phase 2: Integration | 3 | Completed |
| Phase 3: Testing | 5 | Completed |
| Phase 4: Benchmarks | 2 | Completed |

---

## Technical Decisions

### 1. Validation Timing
**Decision**: Validate on save (not on field assignment)
**Rationale**: Allows incremental document building, validates complete state

### 2. Error Aggregation
**Decision**: Collect all errors, not fail-fast
**Rationale**: Better UX - user sees all issues at once

### 3. Type Coercion
**Decision**: No implicit coercion (strict validation)
**Rationale**: Explicit type safety, matches Python typing semantics

### 4. Null vs Missing
**Decision**: Both treated as "absent" for Optional fields
**Rationale**: MongoDB semantics - missing field === null

---

## Files Modified

| File | Changes |
|------|---------|
| `crates/data-bridge/src/validation.rs` | Core validation logic |
| `crates/data-bridge/src/mongodb.rs` | save_validated() integration |
| `python/data_bridge/_engine.py` | Python-side integration |
| `tests/test_rust_validation.py` | 40+ test cases |
