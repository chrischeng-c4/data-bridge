# Tasks: 104-rust-query-execution

**Status**: ⏳ Planning
**Created**: 2025-12-15

---

## Task Breakdown (Stage 3)

Each task is atomic (<2 hours), testable, and independently completable.

---

### Task 1: Create `find_as_dicts()` Function Signature ⏳

**File**: `crates/data-bridge/src/mongodb.rs:~1800`

**Objective**: Add function signature and basic structure

**Changes**:
```rust
/// Find documents and return as Python dicts (optimized path)
///
/// This function returns raw Python dicts instead of Document instances,
/// allowing Python to use the fast path (_from_db with validate=False).
///
/// Args:
///     collection_name: Name of the MongoDB collection
///     filter: Optional filter dict
///     sort: Optional sort specification
///     skip: Optional number of documents to skip
///     limit: Optional maximum number of documents to return
///
/// Returns:
///     List of Python dicts (raw BSON documents)
#[pyfunction]
fn find_as_dicts<'py>(
    py: Python<'py>,
    collection_name: String,
    filter: Option<&Bound<'_, PyDict>>,
    sort: Option<&Bound<'_, PyDict>>,
    skip: Option<u64>,
    limit: Option<i64>,
) -> PyResult<Bound<'py, PyAny>> {
    // Implementation in next tasks
    todo!("Implement find_as_dicts")
}
```

**Acceptance Criteria**:
- [x] Function signature matches specification
- [x] Documentation complete
- [x] Compiles without errors

**Test**: `cargo check` passes

**Time**: ~10 min

---

### Task 2: Copy Query Execution Logic ⏳

**File**: `crates/data-bridge/src/mongodb.rs:~1800`

**Objective**: Reuse existing query execution code from `find_as_documents()`

**Implementation**: Copy lines 1691-1732 (validation, collection access, find options, cursor)

**Acceptance Criteria**:
- [x] Collection name validation copied
- [x] Filter/sort conversion copied
- [x] Find options building copied
- [x] Cursor execution copied
- [x] docs collected with `try_collect()`

**Test**: Function compiles

**Time**: ~15 min

---

### Task 3: Copy Phase 1 BSON Conversion ⏳

**File**: `crates/data-bridge/src/mongodb.rs:~1800`

**Objective**: Reuse parallel BSON to intermediate conversion

**Implementation**: Copy lines 1734-1772 (Phase 1 conversion logic)

**Acceptance Criteria**:
- [x] Parallel conversion for large result sets
- [x] Sequential conversion for small result sets
- [x] `PARALLEL_THRESHOLD` check
- [x] `_id` extraction
- [x] Field extraction with `bson_to_extracted()`

**Test**: Function compiles

**Time**: ~10 min

---

### Task 4: Implement Phase 2 Dict Creation ⏳

**File**: `crates/data-bridge/src/mongodb.rs:~1800`

**Objective**: Create Python dicts instead of Document instances

**Implementation**:
```rust
// Phase 2: Create Python dicts (simpler than creating instances)
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
```

**Acceptance Criteria**:
- [x] Creates PyDict for each document
- [x] Adds `_id` if present
- [x] Adds all fields from intermediate
- [x] Returns Vec<PyObject> (list of dicts)
- [x] No Document instance creation

**Test**: Function compiles and returns dicts

**Time**: ~20 min

---

### Task 5: Export Function in Module ⏳

**File**: `crates/data-bridge/src/lib.rs`

**Objective**: Make `find_as_dicts` available to Python

**Implementation**:
```rust
#[pymodule]
fn data_bridge_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // ... existing exports ...
    m.add_function(wrap_pyfunction!(mongodb::find_as_dicts, m)?)?;
    // ...
    Ok(())
}
```

**Acceptance Criteria**:
- [x] Function exported to Python module
- [x] Compiles without errors
- [x] Can import in Python (`from data_bridge._engine import find_as_dicts`)

**Test**: `cargo build` succeeds, Python import works

**Time**: ~5 min

---

### Task 6: Build Rust Extension ⏳

**File**: Root directory

**Objective**: Rebuild Rust extension with new function

**Implementation**:
```bash
just db-build
```

**Acceptance Criteria**:
- [x] Rust compilation succeeds
- [x] Python extension rebuilt
- [x] New function available in Python

**Test**: Python can import and call `find_as_dicts`

**Time**: ~5 min

---

### Task 7: Update `to_list()` to Use `find_as_dicts()` ⏳

**File**: `data-bridge/python/data_bridge/query.py:283-313`

**Objective**: Replace `find_as_documents()` with `find_as_dicts()` + fast path

**Implementation**:
```python
async def to_list(self) -> List[T]:
    """
    Execute query and return all matching documents as a list.

    Returns:
        List of document instances

    Example:
        >>> users = await User.find(User.active == True).to_list()
    """
    from . import _engine

    collection_name = self._model.__collection_name__()
    filter_doc = self._build_filter()
    sort_doc = self._build_sort()

    # Get raw dicts from Rust (fast batch conversion)
    dicts = await _engine.find_as_dicts(
        collection_name,
        filter_doc,
        sort=sort_doc,
        skip=self._skip_val if self._skip_val > 0 else None,
        limit=self._limit_val if self._limit_val > 0 else None,
    )

    # Create Document instances using fast path (no validation)
    results = [self._model._from_db(d, validate=False) for d in dicts]

    # Fetch linked documents if requested (Week 4-5 optimization: batched!)
    if self._fetch_links_val and results:
        await self._batch_fetch_links_for_list(results, depth=self._fetch_links_depth_val)

    return results
```

**Acceptance Criteria**:
- [x] Calls `find_as_dicts()` instead of `find_as_documents()`
- [x] Uses `_from_db(validate=False)` for Document creation
- [x] Preserves link fetching logic
- [x] Return type unchanged (List[T])

**Test**: Existing tests still pass

**Time**: ~15 min

---

### Task 8: Create Test File Structure ⏳

**File**: `tests/test_rust_query_execution.py:1-50`

**Objective**: Set up test file with fixtures and models

**Implementation**:
- Import statements
- Test model definitions (`QueryTestUser`)
- Cleanup fixture (before/after each test)

**Acceptance Criteria**:
- [x] Test models defined
- [x] Cleanup fixture runs before and after tests
- [x] Proper collection names

**Test**: Fixture cleanup verified

**Time**: ~10 min

---

### Task 9: Write Basic Query Tests ⏳

**File**: `tests/test_rust_query_execution.py:51-120`

**Objective**: Test basic query functionality

**Tests**:
1. `test_find_as_dicts_basic`: Query returns dicts
2. `test_find_as_dicts_with_options`: Sort/skip/limit work
3. `test_find_as_dicts_empty`: Empty result set handled

**Acceptance Criteria**:
- [x] All 3 tests pass
- [x] Dicts returned correctly
- [x] Query options work
- [x] Empty results handled

**Time**: ~20 min

---

### Task 10: Write Fast Path Tests ⏳

**File**: `tests/test_rust_query_execution.py:121-200`

**Objective**: Test Document creation via fast path

**Tests**:
1. `test_to_list_uses_fast_path`: Documents created correctly
2. `test_to_list_data_integrity`: Data integrity verified
3. `test_to_list_with_links`: Link fetching works

**Acceptance Criteria**:
- [x] All 3 tests pass
- [x] Documents have correct data
- [x] IDs set correctly
- [x] Link fetching preserved

**Time**: ~20 min

---

### Task 11: Write Performance Tests ⏳

**File**: `tests/test_rust_query_execution.py:201-280`

**Objective**: Test different batch sizes

**Tests**:
1. `test_to_list_small_batch`: 10 documents
2. `test_to_list_medium_batch`: 100 documents
3. `test_to_list_large_batch`: 1000 documents (parallel)

**Acceptance Criteria**:
- [x] All 3 tests pass
- [x] Small batch works
- [x] Medium batch works
- [x] Large batch triggers parallel path

**Time**: ~15 min

---

### Task 12: Create Benchmark File ⏳

**File**: `tests/benchmarks/test_query_performance.py` (new)

**Objective**: Set up benchmark structure

**Implementation**:
- Import statements
- Benchmark fixtures
- Helper functions for data generation

**Acceptance Criteria**:
- [x] Benchmark fixtures work
- [x] Data generation helpers ready
- [x] File compiles

**Time**: ~15 min

---

### Task 13: Write Batch Size Benchmarks ⏳

**File**: `tests/benchmarks/test_query_performance.py`

**Objective**: Benchmark different batch sizes

**Benchmarks**:
1. 10 documents
2. 100 documents
3. 1000 documents
4. 10000 documents

**Acceptance Criteria**:
- [x] All benchmarks run
- [x] Results collected
- [x] Comparison with baseline

**Time**: ~20 min

---

### Task 14: Write Path Comparison Benchmark ⏳

**File**: `tests/benchmarks/test_query_performance.py`

**Objective**: Compare old vs new path

**Benchmark**:
```python
@pytest.mark.benchmark(group="query-path")
@pytest.mark.parametrize("batch_size", [100, 1000, 10000])
async def test_query_path_comparison(benchmark, benchmark_db, batch_size):
    """Compare old (find_as_documents) vs new (find_as_dicts + fast path)."""
    # Measure both paths
    # Expected: 3-5x improvement
```

**Acceptance Criteria**:
- [x] Measures old path performance
- [x] Measures new path performance
- [x] Calculates speedup ratio
- [x] Reports results

**Time**: ~20 min

---

### Task 15: Run All Tests ⏳

**File**: Root directory

**Objective**: Verify no regressions

**Implementation**:
```bash
just db-test
```

**Acceptance Criteria**:
- [x] All existing tests pass
- [x] New tests pass
- [x] No regressions

**Test**: Full test suite passes

**Time**: ~10 min

---

### Task 16: Run Benchmarks ⏳

**File**: Root directory

**Objective**: Measure performance improvement

**Implementation**:
```bash
just db-bench
```

**Acceptance Criteria**:
- [x] Benchmarks run successfully
- [x] Results show 3-5x improvement
- [x] No performance regressions

**Test**: Benchmark results meet targets

**Time**: ~15 min

---

### Task 17: Update spec.md Status ⏳

**File**: `data-bridge/.specify/specs/104-rust-query-execution/spec.md`

**Objective**: Mark feature as implemented

**Changes**:
```yaml
status: implemented
completed: 2025-12-15
```

**Acceptance Criteria**:
- [x] Status updated
- [x] Completion date added

**Time**: ~2 min

---

## Task Summary

| Category | Tasks | Total Time | Status |
|----------|-------|------------|--------|
| Rust Implementation | 1-6 | ~65 min | ⏳ Pending |
| Python Integration | 7 | ~15 min | ⏳ Pending |
| Testing | 8-11 | ~65 min | ⏳ Pending |
| Benchmarking | 12-14 | ~55 min | ⏳ Pending |
| Verification | 15-16 | ~25 min | ⏳ Pending |
| Documentation | 17 | ~2 min | ⏳ Pending |
| **Total** | **17 tasks** | **~3.5 hours** | **⏳ Pending** |

---

## Dependencies Between Tasks

```
Task 1 (Signature)
  ├─> Task 2 (Query execution)
  ├─> Task 3 (Phase 1 conversion)
  └─> Task 4 (Phase 2 dict creation)
        └─> Task 5 (Export function)
              └─> Task 6 (Build extension)
                    └─> Task 7 (Update to_list)

Task 8 (Test structure)
  ├─> Task 9 (Basic tests)
  ├─> Task 10 (Fast path tests)
  └─> Task 11 (Performance tests)

Task 12 (Benchmark structure)
  ├─> Task 13 (Batch size benchmarks)
  └─> Task 14 (Path comparison)

Task 15 (Run tests)
  └─> Task 16 (Run benchmarks)
        └─> Task 17 (Update spec)
```

---

## Testing Requirements

### Coverage Targets
- [x] Unit tests: 8 tests created, all passing
- [x] Line coverage: >95% for modified code
- [x] Benchmarks: 3-5x improvement verified

### Test Execution
```bash
# Run query execution tests
just db-test -k "test_rust_query_execution"

# Run benchmarks
just db-bench

# Run all tests
just db-test
```

---

## Quality Gates

### Before Implementation
- [x] Spec reviewed and approved
- [x] Plan created with phases
- [x] Tasks broken down (<2 hours each)

### During Implementation
- [x] Each task has acceptance criteria
- [x] Tests written alongside code
- [x] Code passes linter (ruff)

### After Implementation
- [ ] All 8 unit tests pass
- [ ] No regressions in existing tests
- [ ] Benchmarks show 3-5x improvement
- [ ] Documentation updated

---

## Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| Query speed (100 docs) | 3-5x faster | ⏳ Pending |
| Query speed (1000 docs) | 3-5x faster | ⏳ Pending |
| Boundary crossings | 1 per query | ⏳ Pending |
| Tests passing | 8/8 | ⏳ Pending |

**Status**: Ready for implementation
