---
feature: complex-type-validation
spec_ref: .specify/specs/107-complex-type-validation/spec.md
status: completed
created: 2025-12-15
updated: 2025-12-16
---

# Implementation Plan: Complex Type Validation (107)

## Overview

Extend Rust validation to handle complex types: List[T], Dict[K,V], nested documents, ObjectId, and DateTime while maintaining 10-100x speedup over Pydantic v2.

## Prerequisites

- [x] Feature 105 (Type Schema Extraction) - COMPLETED
- [x] Feature 106 (Basic Type Validation) - COMPLETED
- [x] BsonTypeDescriptor enum with Array/Object variants

---

## Phase 1: List[T] Validation (~3 hours)

### Objective
Validate array elements against item type descriptor with indexed error reporting.

### Tasks
1. Implement Array variant in BsonTypeDescriptor
2. Recursive element validation in validate_field()
3. Error messages with array indices (e.g., "tags[2]")
4. Support for List[EmbeddedDocument]

### Files
- `crates/data-bridge/src/validation.rs`

### Acceptance Criteria
- List[str] rejects non-string elements
- List[int] rejects non-integer elements
- Error shows exact index: "tags[2]: expected string, got int64"

---

## Phase 2: Nested Document Validation (~3 hours)

### Objective
Recursively validate embedded documents and nested object structures.

### Tasks
1. Implement Object variant with schema HashMap
2. Recursive validation for nested fields
3. Dot-notation field paths (e.g., "address.city")
4. Support for deeply nested structures

### Files
- `crates/data-bridge/src/validation.rs`

### Acceptance Criteria
- Nested EmbeddedDocument fields validated
- Error paths show full hierarchy: "user.address.city"
- Arbitrary nesting depth supported

---

## Phase 3: ObjectId & DateTime (~2 hours)

### Objective
Add validation for MongoDB-specific types.

### Tasks
1. ObjectId type descriptor and validation
2. DateTime type descriptor and validation
3. Handle BSON date representations

### Files
- `crates/data-bridge/src/validation.rs`

### Acceptance Criteria
- ObjectId fields accept valid ObjectIds only
- DateTime fields accept BSON dates
- Clear error messages for type mismatches

---

## Phase 4: Dict[K,V] Support (~2 hours)

### Objective
Validate dictionary/map types with key and value type checking.

### Tasks
1. Dict type descriptor (keys always string in BSON)
2. Value type validation for all entries
3. Error messages showing dict keys

### Files
- `crates/data-bridge/src/validation.rs`
- `python/data_bridge/type_extraction.py`

### Acceptance Criteria
- Dict[str, int] validates all values are integers
- Dict[str, Any] accepts any values
- Error shows key path: "metadata.count"

---

## Phase 5: Testing & Benchmarks (~4 hours)

### Objective
Comprehensive test coverage and performance verification.

### Test Categories
1. List validation with various element types
2. Nested document validation (multiple levels)
3. Mixed complex types (List[EmbeddedDocument])
4. Error message quality verification
5. Performance benchmarks vs Pydantic

### Files
- `tests/test_rust_validation.py`

### Acceptance Criteria
- 20+ tests for complex types
- All error paths verified
- 10-100x speedup confirmed

---

## Implementation Summary

**Total Estimated Time**: 14 hours (~2 days)

| Phase | Hours | Status |
|-------|-------|--------|
| Phase 1: List[T] | 3 | Completed |
| Phase 2: Nested Docs | 3 | Completed |
| Phase 3: ObjectId/DateTime | 2 | Completed |
| Phase 4: Dict[K,V] | 2 | Completed |
| Phase 5: Testing | 4 | Completed |

---

## Technical Decisions

### 1. BsonTypeDescriptor Design
**Decision**: Use Box<> for recursive types (Array, Optional)
**Rationale**: Rust requires known size at compile time

### 2. Dict Key Type
**Decision**: Keys always validated as strings
**Rationale**: BSON/JSON objects have string keys only

### 3. Error Path Format
**Decision**: Dot notation with brackets for indices
**Rationale**: Standard JSON path format, familiar to developers

### 4. Nested Depth Limit
**Decision**: No artificial limit
**Rationale**: Stack handles reasonable depths, pathological cases rare

---

## Files Modified

| File | Changes |
|------|---------|
| `crates/data-bridge/src/validation.rs` | Array, Object, DateTime, ObjectId validation |
| `python/data_bridge/type_extraction.py` | Complex type descriptor generation |
| `tests/test_rust_validation.py` | Complex type test cases |
