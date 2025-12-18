---
feature: rust-query-execution
components:
  - data-bridge
lead: data-bridge
status: done
created: 2025-12-15
branch: feature/cross_data-bridge
worktree: ../cross_data-bridge
---

# Specification: Rust-Side Query Execution

## 1. Problem Definition

### 1.1 Current Problem

Query results cross Python-Rust boundary N times for N documents:
- Query → Rust → Python → Parse → Document (repeated N times)
- **Impact**: 6x slower find_many vs MongoEngine

### 1.2 Proposed Solution

Batch conversion in Rust:
- Execute query in Rust
- Convert ALL results to Python dicts in ONE batch
- Return to Python in single boundary crossing

### 1.3 Success Criteria

- ✅ 3-5x faster queries (batch processing)
- ✅ 1 boundary crossing per query (not per document)
- ✅ Parallel Python object creation with Rayon

---

## 2. Technical Design

### 2.1 Rust Batch Execution

```rust
// crates/data-bridge/src/mongodb.rs

#[pyfunction]
fn find_as_documents_batch<'py>(
    py: Python<'py>,
    collection_name: String,
    filter: Bound<'py, PyDict>,
    document_class: Py<PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    future_into_py(py, async move {
        let collection = get_collection(&collection_name)?;
        let cursor = collection.find(filter).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;

        // Convert to Python dicts in parallel (Rayon)
        let py_docs: Vec<PyObject> = docs
            .par_iter()
            .map(|doc| bson_to_python_fast(py, doc))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(py_docs.to_object(py))
    })
}
```

---

## 3. Testing Strategy

### 3.1 Benchmark

```python
@pytest.mark.benchmark(group="query")
def test_find_many_batch(benchmark):
    """Batch query execution."""
    result = benchmark(lambda: User.find(User.age > 25).to_list())
    # Expected: 3-5x faster than current
```

---

## 4. Implementation Plan

- **Day 1**: Rust `find_as_documents_batch()` function
- **Day 2**: Python integration with QueryBuilder
- **Day 3**: Benchmarks

---

## 5. Dependencies

**Depends on**: 102 (Lazy Validation), 103 (Fast Path)
**Enables**: Phase 2 Query Engine

---

## Success Metrics

| Metric | Target | Actual Result |
|--------|--------|---------------|
| Query speed | 3-5x faster | ❌ **10x SLOWER** |
| Boundary crossings | 1 per query | ✅ Achieved |

**Status**: ⚠️ **ABANDONED - Performance Regression Discovered**

---

## 6. Implementation Outcome (2025-12-15)

### 6.1 What Was Implemented

✅ **Completed**:
- `find_as_dicts()` Rust function (returns raw Python dicts)
- Test suite (9 tests, all passing)
- Benchmark suite
- Performance comparison

### 6.2 Critical Finding: Performance Regression

**Benchmarks revealed a 10x performance regression**:

| Approach | Time (10,000 docs) | Performance |
|----------|-------------------|-------------|
| **Old path** (find_as_documents) | 252ms | ✅ Baseline |
| **New path** (find_as_dicts + Python _from_db) | 2,330ms | ❌ **10x slower** |

### 6.3 Root Cause Analysis

**Original Hypothesis** (WRONG ❌):
- Creating Document instances one-by-one in Rust is slow
- Python's fast path (`_from_db(validate=False)`) would be faster

**Actual Reality** (Discovered):
- Creating Document instances in **Rust** is **FAST** (252ms for 10K docs)
- Creating Document instances in **Python** via `_from_db()` is **SLOW** (2,330ms)
- Python object creation overhead >> Rust overhead

**Why Python is slower**:
1. **PyO3 overhead**: Each `_from_db()` call crosses Python-Rust boundary
2. **Pydantic overhead**: Even with `validate=False`, model instantiation has overhead
3. **Dict unpacking**: `**dict` unpacking in Python is expensive at scale
4. **No parallelization**: Python list comprehension is sequential

### 6.4 Recommendation

**DO NOT USE `find_as_dicts()` for query execution.**

Keep the existing `find_as_documents()` Rust path, which:
- Creates Document instances directly in Rust
- Leverages existing parallel BSON conversion (Rayon)
- Is 10x faster than Python Document creation

### 6.5 Future Work

If query performance is still an issue, investigate:
1. **Rust-side optimizations**: Further optimize `find_as_documents()`
2. **Caching**: Cache parsed Documents
3. **Streaming**: Return results as they arrive (cursor-based)
4. **Projection**: Only fetch needed fields

**Status**: Abandoned - Keeping existing implementation
