---
feature: type-schema-extraction
components:
  - data-bridge
lead: data-bridge
status: completed
created: 2025-12-15
completed: 2025-12-16
branch: feature/cross_data-bridge
---

# Specification: Type Schema Extraction

## 1. Problem Definition

### 1.1 Current State

data-bridge lacks runtime type validation. Need to extract Python type hints and pass to Rust for validation.

### 1.2 Proposed Solution

Extract `__annotations__` from Document classes → Build Rust schema → Cache for validation

### 1.3 Success Criteria

- ✅ Extract str, int, float, bool, Optional[T] types
- ✅ Build Rust schema from Python annotations
- ✅ Cache schema (one-time per Document class)
- ✅ No runtime performance impact (cached)

---

## 2. Technical Design

### 2.1 Python Type Extraction

```python
# data_bridge/type_extraction.py

from typing import Any, Dict, get_type_hints, get_origin, get_args

def extract_validation_schema(document_class: type) -> Dict[str, Any]:
    """Extract type schema from Document class."""
    type_hints = get_type_hints(document_class, include_extras=True)
    schema = {}

    for field_name, py_type in type_hints.items():
        if field_name.startswith("_"):
            continue

        origin = get_origin(py_type)
        args = get_args(py_type)

        # Determine if Optional
        is_optional = (origin is Union and type(None) in args)

        # Map to Rust type
        base_type = _map_python_to_rust_type(py_type)

        schema[field_name] = {
            "base_type": base_type,
            "is_optional": is_optional,
        }

    return schema
```

### 2.2 Rust Schema Registration

```rust
// crates/data-bridge/src/validation.rs

static SCHEMA_CACHE: OnceCell<Mutex<HashMap<String, DocumentSchema>>> = OnceCell::new();

#[pyfunction]
pub fn register_validation_schema(
    class_name: &str,
    schema_dict: Bound<'_, PyDict>,
) -> PyResult<()> {
    let mut cache = get_schema_cache().lock().unwrap();
    cache.insert(class_name.to_string(), parse_schema(schema_dict));
    Ok(())
}
```

---

## 3. Testing Strategy

```python
def test_extract_basic_types():
    """Verify basic type extraction."""
    class User(Document):
        name: str
        age: int
        email: Optional[str]

    schema = extract_validation_schema(User)
    assert schema["name"]["base_type"] == "string"
    assert schema["age"]["base_type"] == "integer"
    assert schema["email"]["is_optional"] is True
```

---

## 4. Implementation Plan

- **Day 1**: Python type extraction
- **Day 2**: Rust schema registration and caching
- **Day 3**: Integration tests

---

## 5. Dependencies

**Depends on**: None
**Enables**: 106 (Basic Type Validation)

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Types supported | str, int, float, bool, Optional[T] |
| Schema caching | 100% (one-time per class) |
| Performance impact | 0 (cached) |

**Status**: Ready for implementation
