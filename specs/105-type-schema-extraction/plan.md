# Implementation Plan: Type Schema Extraction

**Feature**: 105-type-schema-extraction
**Status**: Planning
**Created**: 2025-12-15

---

## Overview

Extract Python type hints from Document classes and pass them to Rust for validation. This enables runtime type checking while maintaining zero performance overhead through schema caching.

**Key Goals**:
- Extract type annotations from Python Document classes
- Build Rust schema representation from Python types
- Cache schemas for one-time overhead (zero runtime cost)
- Support basic types: str, int, float, bool, Optional[T]

**Value Proposition**:
- Enables runtime type validation (foundation for 106-basic-type-validation)
- Zero performance impact after first class instantiation (cached)
- Type-safe validation without manual schema definitions
- Leverages existing Python type hints (no new syntax)

---

## Current State

**What exists**:
- Document classes with Python type hints (`__annotations__`)
- Rust validation infrastructure (from 102-lazy-validation)
- No schema extraction or registration mechanism

**What's missing**:
- Python module to extract type hints
- Rust schema structs and registration function
- Schema caching infrastructure
- Integration with Document class lifecycle

**Technical Debt**:
- None (new feature)

---

## Implementation Phases

### Phase 1: Python Type Extraction Module (~6 hours)

**Objective**: Create Python module to extract type hints and convert to serializable format

**Files**:
- `python/data_bridge/type_extraction.py` (new)

**Key Functions**:
```python
def extract_validation_schema(document_class: type) -> Dict[str, Any]:
    """
    Extract type schema from Document class annotations.

    Returns dict mapping field_name -> {base_type, is_optional}

    Example:
        class User(Document):
            name: str
            age: int
            email: Optional[str]

        → {
            "name": {"base_type": "string", "is_optional": False},
            "age": {"base_type": "integer", "is_optional": False},
            "email": {"base_type": "string", "is_optional": True}
        }
    """
```

**Implementation Steps**:
1. Use `typing.get_type_hints()` to extract annotations
2. Use `typing.get_origin()` and `typing.get_args()` to detect Optional[T]
3. Map Python types → Rust type strings:
   - `str` → `"string"`
   - `int` → `"integer"`
   - `float` → `"float"`
   - `bool` → `"boolean"`
   - `Optional[T]` → `{base_type: T, is_optional: True}`
4. Skip private fields (starting with `_`)
5. Return dict for serialization to Rust

**Tests**:
- Test basic type extraction (str, int, float, bool)
- Test Optional[T] detection
- Test private field skipping
- Test nested types (future: List[T], Dict[K,V])

**Risks**:
- Complex Python types (Union, Literal, etc.) - Mitigation: Phase 1 only supports basic types
- Runtime type hint resolution - Mitigation: Use `include_extras=True` in get_type_hints()

**Success Criteria**:
- All basic types extracted correctly
- Optional detection works
- 100% test coverage for supported types

---

### Phase 2: Rust Schema Registration (~8 hours)

**Objective**: Create Rust infrastructure to store and cache Document schemas

**Files**:
- `crates/data-bridge/src/validation.rs` (new)
- `crates/data-bridge/src/lib.rs` (update - export function)

**Key Rust Structs**:
```rust
#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub base_type: FieldType,
    pub is_optional: bool,
}

#[derive(Debug, Clone)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
}

#[derive(Debug, Clone)]
pub struct DocumentSchema {
    pub fields: HashMap<String, FieldSchema>,
}

// Global cache (thread-safe, one-time initialization)
static SCHEMA_CACHE: OnceCell<Mutex<HashMap<String, DocumentSchema>>> = OnceCell::new();
```

**Key Rust Functions**:
```rust
#[pyfunction]
pub fn register_validation_schema(
    class_name: &str,
    schema_dict: &Bound<'_, PyDict>,
) -> PyResult<()> {
    let schema = parse_schema_dict(schema_dict)?;
    let mut cache = get_schema_cache().lock().unwrap();
    cache.insert(class_name.to_string(), schema);
    Ok(())
}

fn parse_schema_dict(schema_dict: &Bound<'_, PyDict>) -> PyResult<DocumentSchema> {
    // Parse Python dict → Rust DocumentSchema
    // Example: {"name": {"base_type": "string", "is_optional": false}}
}
```

**Implementation Steps**:
1. Define `FieldSchema`, `FieldType`, `DocumentSchema` structs
2. Implement `parse_schema_dict()` to convert Python dict → Rust structs
3. Use `OnceCell<Mutex<HashMap>>` for global schema cache
4. Implement `register_validation_schema()` PyO3 function
5. Export function in `lib.rs` module
6. Add `get_schema()` function to retrieve cached schema (for 106)

**Dependencies**:
- `once_cell` crate (already in Cargo.toml)
- `std::sync::Mutex` for thread-safe cache

**Tests**:
- Test schema parsing (Python dict → Rust structs)
- Test schema registration and retrieval
- Test cache thread-safety (concurrent registrations)
- Test duplicate registration (should overwrite or error?)

**Risks**:
- Schema cache memory growth - Mitigation: Schemas are small, one per Document class
- Thread contention on Mutex - Mitigation: Registration is one-time, not per-query

**Success Criteria**:
- Schema registration works for all supported types
- Cache retrieval returns correct schema
- Thread-safe cache operations
- No memory leaks

---

### Phase 3: Schema Caching Integration (~4 hours)

**Objective**: Integrate schema extraction/registration into Document class lifecycle

**Files**:
- `python/data_bridge/document.py` (update `__init_subclass__`)

**Implementation**:
```python
class Document(BaseModel):
    @classmethod
    def __init_subclass__(cls, **kwargs):
        """Called when Document subclass is created."""
        super().__init_subclass__(**kwargs)

        # Extract schema from class annotations
        from .type_extraction import extract_validation_schema
        from . import _engine

        schema = extract_validation_schema(cls)
        class_name = f"{cls.__module__}.{cls.__name__}"

        # Register schema with Rust (one-time)
        _engine.register_validation_schema(class_name, schema)
```

**Implementation Steps**:
1. Import `type_extraction` module in `document.py`
2. Add schema extraction to `__init_subclass__` hook
3. Call `register_validation_schema()` after extraction
4. Use fully-qualified class name (`module.ClassName`) as cache key
5. Handle exceptions gracefully (log warning if schema registration fails)

**Edge Cases**:
- Document class with no fields → Empty schema (valid)
- Document class inheritance → Extract schema from ALL parent classes
- Circular imports → Lazy import `type_extraction` inside `__init_subclass__`

**Tests**:
- Test schema registration on class creation
- Test inherited Document classes
- Test Document with no fields
- Test schema retrieval after registration

**Risks**:
- Import cycles (type_extraction ← document) - Mitigation: Lazy import
- Schema registration failure - Mitigation: Catch exceptions, log warnings

**Success Criteria**:
- Schema registered automatically on Document class creation
- No import errors
- Works with class inheritance

---

### Phase 4: Testing & Validation (~6 hours)

**Objective**: Comprehensive test coverage and performance validation

**Test Files**:
- `tests/test_type_extraction.py` (new - 10 tests)
- `tests/test_schema_registration.py` (new - 8 tests)
- `tests/test_schema_integration.py` (new - 6 tests)
- `tests/benchmarks/test_schema_overhead.py` (new - 3 benchmarks)

**Test Categories**:

1. **Type Extraction Tests** (10 tests):
   - Basic types (str, int, float, bool)
   - Optional types (Optional[str], Optional[int])
   - Mixed required/optional fields
   - Private field skipping
   - Empty class (no fields)
   - Inherited classes
   - Complex types (unsupported, should skip or error)

2. **Schema Registration Tests** (8 tests):
   - Register schema successfully
   - Retrieve registered schema
   - Overwrite existing schema
   - Thread-safe concurrent registrations
   - Invalid schema dict (should error)
   - Empty schema (valid)
   - Large schema (100+ fields)

3. **Integration Tests** (6 tests):
   - Document class creation triggers registration
   - Schema available after class creation
   - Inherited Document classes
   - Multiple Document classes (separate schemas)
   - Schema persistence across instances
   - Schema immutability (cache not modified)

4. **Performance Benchmarks** (3 benchmarks):
   - Schema extraction overhead (once per class) - Target: <1ms
   - Schema registration overhead (once per class) - Target: <1ms
   - Schema retrieval overhead (per validation) - Target: <0.1ms
   - Zero overhead after caching (no repeated extraction)

**Coverage Target**: >95% for new code

**Success Criteria**:
- All 24 tests passing
- >95% code coverage
- Benchmarks confirm zero runtime overhead (after caching)
- No memory leaks in schema cache

---

## Dependencies

**Upstream Dependencies** (must complete first):
- None (feature is independent)

**Downstream Dependencies** (blocked by this feature):
- 106-basic-type-validation (requires schema extraction)
- 107-nested-type-validation (requires schema infrastructure)
- 108-custom-validators (requires schema metadata)

**External Dependencies**:
- Python `typing` module (standard library)
- Rust `once_cell` crate (already in Cargo.toml)

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Complex Python types not supported | High | Low | Phase 1 only supports basic types, document limitations |
| Import cycles (type_extraction ← document) | Medium | Medium | Lazy import in `__init_subclass__` |
| Schema cache memory growth | Low | Low | Schemas are small (few KB per class) |
| Thread contention on schema cache | Low | Low | Registration is one-time, not per-query |
| Schema registration failure | Low | Medium | Catch exceptions, log warnings, continue without validation |
| Type hint resolution at runtime | Medium | Medium | Use `get_type_hints(include_extras=True)` |

**Overall Risk**: Low - Feature is well-scoped with clear boundaries

---

## Test Strategy

**TDD Approach**:
1. Write tests FIRST for each function
2. Implement function to pass tests
3. Refactor while keeping tests green

**Test Pyramid**:
- Unit tests: 18 tests (type extraction, schema parsing, registration)
- Integration tests: 6 tests (Document lifecycle integration)
- Benchmarks: 3 tests (performance validation)

**Coverage Requirements**:
- Line coverage: >95% for new code
- Branch coverage: >90% for conditional logic
- Edge cases: All identified edge cases tested

**Continuous Testing**:
```bash
# Run type extraction tests
just db-test -k "test_type_extraction"

# Run schema registration tests
just db-test -k "test_schema_registration"

# Run integration tests
just db-test -k "test_schema_integration"

# Run all new tests
just db-test -k "test_type or test_schema"

# Run benchmarks
just db-bench
```

---

## Rollout Plan

### Development (Week 1)
- Day 1: Phase 1 (Python type extraction)
- Day 2: Phase 2 (Rust schema registration)
- Day 3: Phase 3 (Schema caching integration)

### Testing & Validation (Week 2)
- Day 4: Phase 4 (Tests and benchmarks)
- Day 5: Integration testing, documentation
- Day 6: Code review, final testing

### Deployment
- Not applicable (library code, no deployment)

---

## Success Criteria

| Criterion | Target | Measurement |
|-----------|--------|-------------|
| Types supported | str, int, float, bool, Optional[T] | All 5 types extracted correctly |
| Schema caching | 100% (one-time per class) | Benchmark confirms zero repeated extraction |
| Performance impact | 0 after caching | Benchmark: <0.1ms schema retrieval |
| Test coverage | >95% line coverage | pytest-cov report |
| Tests passing | 24/24 (100%) | All unit + integration + benchmarks pass |
| Documentation | Complete | Type hints, docstrings, examples |

**Definition of Done**:
- [ ] All 24 tests passing
- [ ] >95% code coverage
- [ ] Benchmarks confirm zero runtime overhead
- [ ] Documentation complete (docstrings, examples)
- [ ] Code review approved
- [ ] No regressions in existing tests (449 tests)

---

## Alternative Approaches Considered

### Alternative 1: Manual Schema Definition
**Approach**: Require users to define schema manually (like Beanie's `Settings`)
**Pros**: Explicit, no runtime type inspection
**Cons**: Duplicate work (type hints + manual schema), error-prone
**Decision**: Rejected - Auto-extraction is more user-friendly

### Alternative 2: Pydantic Schema Export
**Approach**: Export Pydantic's generated JSON schema to Rust
**Pros**: Leverage Pydantic's schema generation
**Cons**: JSON schema is verbose, harder to parse, includes validation rules we don't need
**Decision**: Rejected - Custom extraction is simpler and faster

### Alternative 3: No Caching (Extract on Every Validation)
**Approach**: Extract schema on every validation call
**Pros**: Simpler implementation (no cache)
**Cons**: Huge performance overhead, defeats purpose
**Decision**: Rejected - Caching is essential for zero overhead

---

## Open Questions

1. **How to handle Document class inheritance?**
   - **Answer**: Extract schema from ALL parent classes, merge into single schema
   - **Action**: Implement in Phase 1, test in Phase 4

2. **Should schema registration overwrite or error on duplicate?**
   - **Answer**: Overwrite (allows class redefinition in interactive shells)
   - **Action**: Document behavior, add test

3. **What happens if `__annotations__` is missing?**
   - **Answer**: Empty schema (valid, no validation)
   - **Action**: Add test for this edge case

4. **Should we support forward references (stringified types)?**
   - **Answer**: Not in Phase 1 (basic types only)
   - **Action**: Add to future work (Phase 2)

---

## Timeline

| Phase | Duration | Dependencies | Deliverable |
|-------|----------|--------------|-------------|
| Phase 1 | 6 hours | None | `type_extraction.py` module + tests |
| Phase 2 | 8 hours | Phase 1 | `validation.rs` module + tests |
| Phase 3 | 4 hours | Phase 1, 2 | `__init_subclass__` integration + tests |
| Phase 4 | 6 hours | Phase 1, 2, 3 | Full test suite + benchmarks |
| **Total** | **24 hours** | | **Complete feature** |

**Estimated Completion**: 3 days (8 hours/day)

---

## Monitoring & Metrics

**During Implementation**:
- Test coverage tracked with pytest-cov
- Benchmark results tracked in CI/CD
- Code quality tracked with ruff linter

**After Deployment**:
- Not applicable (library code)

**Success Metrics**:
- Zero runtime overhead confirmed by benchmarks
- 100% adoption (automatic for all Document classes)
- No performance regressions in existing code

---

## Rollback Plan

**If implementation fails**:
1. Remove `__init_subclass__` schema registration
2. Remove Rust `register_validation_schema()` export
3. Keep `type_extraction.py` and `validation.rs` (no harm)
4. Revert to no schema extraction (status quo)

**Rollback Triggers**:
- Test failures that can't be fixed in 1 day
- Performance regressions >5% in existing code
- Import cycles or circular dependency issues

**Rollback Cost**: Low (feature is additive, not breaking)

---

## Future Work (Post-Phase 1)

**Phase 2 Types** (106-basic-type-validation):
- List[T], Dict[K,V], Set[T]
- Nested Document types
- Union types (beyond Optional)
- Literal types

**Advanced Features**:
- Custom validators (Pydantic-style)
- Type coercion (str → int)
- Schema migrations (version handling)
- Schema introspection API

**Performance Optimizations**:
- Lazy schema registration (only when validation needed)
- Schema serialization to disk (faster startup)
- Schema sharing across processes (multiprocessing)

---

## Notes

- This feature is **foundational** for all future validation features
- Schema extraction is **one-time** per Document class (zero runtime cost)
- Supports **basic types only** in Phase 1 (expand in Phase 2)
- Uses **existing Python type hints** (no new syntax required)
- **Thread-safe** schema cache (safe for async/concurrent use)

**Related Features**:
- 102-lazy-validation (provides validation infrastructure)
- 103-fast-path-bulk-ops (benefits from validation)
- 106-basic-type-validation (uses schema for validation)
- 107-nested-type-validation (builds on schema infrastructure)
