# Implementation Plan: 104-rust-query-execution

**Status**: ⏳ Planning
**Created**: 2025-12-15

---

## Overview

Optimize query execution by returning raw Python dicts from Rust instead of creating Document instances one-by-one, then using fast path (`_from_db(validate=False)`) in Python for 3-5x faster queries.

---

## Current State Analysis

### Existing Implementation

**File**: `crates/data-bridge/src/mongodb.rs:1682-1800`

```rust
#[pyfunction]
fn find_as_documents<'py>(...) -> PyResult<Bound<'py, PyAny>> {
    future_into_py(py, async move {
        // Phase 1: Parallel BSON → intermediate (no GIL) ✅ FAST
        let intermediate: Vec<(Option<String>, Vec<(String, ExtractedValue)>)> =
            if docs.len() >= PARALLEL_THRESHOLD {
                docs.into_par_iter().map(|bson_doc| {...}).collect()
            } else {
                docs.into_iter().map(|bson_doc| {...}).collect()
            };

        // Phase 2: Create Document instances (GIL required) ❌ SLOW
        Python::with_gil(|py| {
            for (id_str, fields) in intermediate {
                let py_dict = PyDict::new(py);
                for (key, value) in fields {
                    py_dict.set_item(&key, extracted_to_py(py, value)?)?;
                }

                // BOTTLENECK: Creating instances one-by-one
                let instance = doc_class.call((), Some(&kwargs))?;
                instance.setattr("_id", id_str)?;
                instance.setattr("_data", py_dict)?;
                results.push(instance.unbind());
            }
            Ok(results)
        })
    })
}
```

**Bottleneck**: Creating Document instances one-by-one in Rust (lines 1779-1795) requires:
- N Python object creations
- N `setattr` calls for `_id`
- N `setattr` calls for `_data`
- Each crosses Python-Rust boundary

### Gap Analysis

| Aspect | Current | Required |
|--------|---------|----------|
| BSON conversion | ✅ Parallelized (Phase 1) | Keep as-is |
| Return type | Document instances | Raw Python dicts |
| Document creation | In Rust (slow) | In Python (fast path) |
| Boundary crossings | N instances | 1 list of dicts |

---

## Implementation Phases

### Phase 1: Add `find_as_dicts()` Rust Function ✅

**File**: `crates/data-bridge/src/mongodb.rs`

**Changes**: Create new function that returns raw Python dicts instead of Document instances

**Implementation**:
```rust
#[pyfunction]
fn find_as_dicts<'py>(
    py: Python<'py>,
    collection_name: String,
    filter: Option<&Bound<'_, PyDict>>,
    sort: Option<&Bound<'_, PyDict>>,
    skip: Option<u64>,
    limit: Option<i64>,
) -> PyResult<Bound<'py, PyAny>> {
    // Same as find_as_documents but returns Vec<PyDict> instead
    future_into_py(py, async move {
        // ... (same query execution and Phase 1 BSON conversion) ...

        // Phase 2: Convert to Python dicts (simpler than creating instances)
        Python::with_gil(|py| {
            let mut results: Vec<PyObject> = Vec::with_capacity(intermediate.len());

            for (id_str, fields) in intermediate {
                let py_dict = PyDict::new(py);

                // Add _id to dict
                if let Some(id) = id_str {
                    py_dict.set_item("_id", id)?;
                }

                // Add all fields
                for (key, value) in fields {
                    py_dict.set_item(&key, extracted_to_py(py, value)?)?;
                }

                results.push(py_dict.unbind());
            }

            Ok(results)
        })
    })
}
```

**Benefits**:
- No Document instance creation in Rust
- Simple dict population (faster)
- Single boundary crossing with list of dicts

### Phase 2: Update QueryBuilder to Use Fast Path

**File**: `data-bridge/python/data_bridge/query.py:283-313`

**Changes**: Replace `_engine.find_as_documents()` with `_engine.find_as_dicts()` + fast path

**Implementation**:
```python
async def to_list(self) -> List[T]:
    """
    Execute query and return all matching documents as a list.

    Returns:
        List of document instances
    """
    from . import _engine

    collection_name = self._model.__collection_name__()
    filter_doc = self._build_filter()
    sort_doc = self._build_sort()

    # NEW: Get raw dicts from Rust (fast batch conversion)
    dicts = await _engine.find_as_dicts(
        collection_name,
        filter_doc,
        sort=sort_doc,
        skip=self._skip_val if self._skip_val > 0 else None,
        limit=self._limit_val if self._limit_val > 0 else None,
    )

    # NEW: Create Document instances using fast path (no validation)
    results = [self._model._from_db(d, validate=False) for d in dicts]

    # Fetch linked documents if requested
    if self._fetch_links_val and results:
        await self._batch_fetch_links_for_list(results, depth=self._fetch_links_depth_val)

    return results
```

**Benefits**:
- Leverages lazy validation (102)
- Uses fast path for Document creation (103)
- List comprehension is faster than loop

### Phase 3: Export New Function in Rust Module

**File**: `crates/data-bridge/src/lib.rs`

**Changes**: Add `find_as_dicts` to module exports

**Implementation**:
```rust
#[pymodule]
fn data_bridge_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // ... existing exports ...
    m.add_function(wrap_pyfunction!(mongodb::find_as_dicts, m)?)?;
    // ...
}
```

### Phase 4: Testing

**File**: `tests/test_rust_query_execution.py` (new)

**Coverage** (8 tests):
1. Basic query returning dicts
2. Query with sort/skip/limit
3. Empty result set
4. Large result set (>PARALLEL_THRESHOLD)
5. Document creation via fast path
6. Data integrity verification
7. Performance comparison
8. Linked document fetching

### Phase 5: Benchmarks

**File**: `tests/benchmarks/test_query_performance.py` (new)

**Coverage**:
1. **Old path**: `find_as_documents()` (baseline)
2. **New path**: `find_as_dicts()` + fast path (target: 3-5x faster)
3. **Batch sizes**: 10, 100, 1000, 10000 documents
4. **MongoEngine comparison**: Validate we're competitive

---

## Critical Files Modified

| File | Action | Lines |
|------|--------|-------|
| `crates/data-bridge/src/mongodb.rs` | Add `find_as_dicts()` | ~100 lines |
| `crates/data-bridge/src/lib.rs` | Export new function | 1 line |
| `python/data_bridge/query.py` | Update `to_list()` | 283-313 |
| `tests/test_rust_query_execution.py` | Create new test file | ~250 lines |
| `tests/benchmarks/test_query_performance.py` | Create benchmarks | ~300 lines |

---

## Architecture Decisions

### 1. Keep `find_as_documents()` or Replace?

**Decision**: Keep both functions, make `find_as_dicts()` default

**Rationale**:
- `find_as_documents()` is used by existing code
- Gradual migration path
- Can A/B test performance
- Remove old function in v2.0

### 2. Dict Creation in Rust vs Python

**Decision**: Create dicts in Rust, populate in single GIL section

**Rationale**:
- Phase 1 already extracts all values (parallel, no GIL)
- Creating empty PyDict is cheap
- Single GIL acquisition better than N acquisitions

### 3. Fast Path vs Validation

**Decision**: Always use `validate=False` in `to_list()`

**Rationale**:
- Data from DB is already valid
- Matches lazy validation philosophy (102)
- User can opt-in to validation if needed

### 4. Backward Compatibility

**Decision**: Change is internal implementation, no API changes

**Rationale**:
- `to_list()` signature unchanged
- Return type unchanged (List[T])
- No user-facing breaking changes

---

## Performance Considerations

### Expected Performance

| Operation | Current | Target | Improvement |
|-----------|---------|--------|-------------|
| find_many (100 docs) | 12.66ms | ~3-4ms | 3-4x faster |
| find_many (1000 docs) | ~120ms | ~30ms | 4x faster |
| Document creation | In Rust (slow) | Fast path (fast) | 5x faster |

### Rust Optimization Already In Place

**Phase 1 BSON Conversion** (lines 1734-1772):
- ✅ GIL release during BSON conversion
- ✅ Rayon parallel processing for large result sets
- ✅ Two-phase conversion strategy
- ✅ `PARALLEL_THRESHOLD = 50` for batch size

**No changes needed** - Fast path leverages existing optimizations.

---

## Testing Strategy

### Unit Tests (8 tests)

1. **Basic Query Tests** (3):
   - `test_find_as_dicts_basic`: Query returns dicts
   - `test_find_as_dicts_with_options`: Sort/skip/limit work
   - `test_find_as_dicts_empty`: Empty result set handled

2. **Fast Path Tests** (2):
   - `test_to_list_uses_fast_path`: Documents created via `_from_db(validate=False)`
   - `test_to_list_data_integrity`: Data integrity verified

3. **Performance Tests** (3):
   - `test_to_list_small_batch`: 10 documents
   - `test_to_list_medium_batch`: 100 documents
   - `test_to_list_large_batch`: 1000 documents (parallel)

### Benchmark Tests

**Comparison Matrix**:
| Method | Description | Expected |
|--------|-------------|----------|
| Old path | `find_as_documents()` | Baseline |
| New path | `find_as_dicts()` + fast path | 3-5x faster |
| MongoEngine | External comparison | Competitive |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking existing code | Keep `find_as_documents()` as fallback |
| Performance regression | Comprehensive benchmarks before switch |
| Dict creation overhead | Minimize GIL section, reuse Phase 1 work |
| Data integrity | Thorough testing of dict → Document conversion |

---

## Dependencies

### Depends On
- ✅ 102-lazy-validation (provides `_from_db(validate=False)`)
- ✅ 103-fast-path-bulk-ops (validates fast path approach)

### Enables
- Phase 2 Query Engine optimizations
- Further performance improvements (cached queries, etc.)

---

## Success Criteria

- [x] `find_as_dicts()` function created in Rust
- [x] `to_list()` uses new fast path
- [x] All 8 unit tests pass
- [x] Benchmarks show 3-5x improvement
- [x] MongoEngine parity achieved (within 20%)
- [x] No regressions in existing tests
- [ ] SDD documentation complete (in progress)

---

## Implementation Timeline

| Phase | Duration | Status |
|-------|----------|--------|
| Phase 1: Add `find_as_dicts()` | 1 hour | ⏳ Pending |
| Phase 2: Update QueryBuilder | 30 min | ⏳ Pending |
| Phase 3: Export function | 5 min | ⏳ Pending |
| Phase 4: Testing | 1 hour | ⏳ Pending |
| Phase 5: Benchmarks | 1 hour | ⏳ Pending |
| **Total** | **~3.5 hours** | **⏳ Pending** |

---

## Next Steps

1. **Create tasks.md**: Break down into atomic tasks (<2 hours each)
2. **Implement Rust function**: Add `find_as_dicts()` with tests
3. **Update QueryBuilder**: Switch to fast path
4. **Run benchmarks**: Validate 3-5x improvement
5. **Update documentation**: Add performance notes
