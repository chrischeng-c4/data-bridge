---
feature: fast-path-bulk-ops
components:
  - data-bridge
lead: data-bridge
status: implemented
created: 2025-12-15
completed: 2025-12-15
branch: feature/cross_data-bridge
---

# Specification: Fast Path for Bulk Operations

## 1. Problem Definition

### 1.1 Current Problem

`insert_many()` creates Pydantic models for each document before insertion:

```python
# Current implementation
await User.insert_many([user1, user2, ...])
# → Each user goes through Pydantic validation
# → Slow for bulk operations
```

**Impact**: 17% slower than MongoEngine on bulk inserts.

### 1.2 Proposed Solution

Add fast path for raw dicts (skip Pydantic):

```python
# Fast path: raw dicts
await User.insert_many([
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25},
], validate=False)  # Skip Pydantic validation
```

### 1.3 Success Criteria

- ✅ 5-10x faster bulk inserts for raw dicts
- ✅ API supports both Document instances and raw dicts
- ✅ Validation optional (validate=False for performance)
- ✅ Parallel BSON conversion with Rayon in Rust

---

## 2. Technical Design

### 2.1 Python API

```python
# data_bridge/document.py

@classmethod
async def insert_many(
    cls,
    documents: List[Union[Document, dict]],
    validate: bool = False
) -> List[T]:
    """Insert many documents with optional validation.

    Args:
        documents: List of Document instances or dicts
        validate: If True, validate all documents

    Returns:
        List of inserted documents
    """
    if all(isinstance(d, dict) for d in documents):
        # Fast path: raw dicts
        if validate:
            docs = [cls(**d).model_dump() for d in documents]
        else:
            docs = documents
    else:
        # Document instances
        docs = [d.model_dump() if validate else d._data for d in documents]

    return await _engine.insert_many_fast(
        collection=cls.Settings.name,
        documents=docs
    )
```

### 2.2 Rust Implementation

```rust
// crates/data-bridge/src/mongodb.rs

#[pyfunction]
fn insert_many_fast<'py>(
    py: Python<'py>,
    collection_name: String,
    documents: Vec<Py<PyDict>>,
) -> PyResult<Bound<'py, PyAny>> {
    future_into_py(py, async move {
        // Convert Python dicts → BSON in parallel (Rayon)
        let bson_docs: Vec<Document> = documents
            .par_iter()  // Parallel iterator
            .map(|py_dict| python_to_bson(py_dict))
            .collect::<Result<Vec<_>, _>>()?;

        // Bulk insert
        let collection = get_collection(&collection_name)?;
        collection.insert_many(bson_docs).await?;

        Ok(())
    })
}
```

---

## 3. Testing Strategy

### 3.1 Benchmarks

```python
@pytest.mark.benchmark(group="bulk_insert")
def test_insert_many_fast_path(benchmark):
    """Raw dicts without validation."""
    docs = [{"name": f"User{i}", "age": 20+i} for i in range(100)]

    result = benchmark(lambda: User.insert_many(docs, validate=False))
    # Expected: 5-10x faster than validated path

@pytest.mark.benchmark(group="bulk_insert")
def test_insert_many_validated(benchmark):
    """Validated insert (for comparison)."""
    docs = [{"name": f"User{i}", "age": 20+i} for i in range(100)]

    result = benchmark(lambda: User.insert_many(docs, validate=True))
```

---

## 4. Implementation Plan

- **Day 1**: Python API with `validate` param
- **Day 2**: Rust parallel BSON conversion (Rayon)
- **Day 3**: Benchmarks and testing

---

## 5. Dependencies

**Depends on**: 102 (Lazy Validation)
**Enables**: Phase 2 Query Engine optimizations

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Bulk insert speed | 5-10x faster (raw dicts) |
| Tests passing | 449 |
| API compatibility | 100% (add optional param) |

**Status**: Ready for implementation
