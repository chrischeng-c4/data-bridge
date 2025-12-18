---
feature: complex-type-validation
components:
  - data-bridge
lead: data-bridge
status: completed
created: 2025-12-15
completed: 2025-12-16
branch: feature/cross_data-bridge
---

# Specification: Complex Type Validation

## 1. Problem Definition

### 1.1 Current State

Basic types work (str, int, float, bool), but no support for List[T], Dict[K,V], nested documents, ObjectId, DateTime.

### 1.2 Proposed Solution

Extend Rust validation to handle complex types.

### 1.3 Success Criteria

- ✅ Validate List[T], Dict[K,V]
- ✅ Nested document validation
- ✅ ObjectId and DateTime types
- ✅ Maintain 10-100x speed vs Pydantic

---

## 2. Technical Design

### 2.1 List[T] Validation

```rust
fn validate_array(
    field_name: &str,
    array: &Bson::Array,
    item_type: &str,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    for (i, item) in array.iter().enumerate() {
        if let Err(err) = validate_single_type(item, item_type) {
            errors.push(ValidationError::new(
                &format!("{}[{}]", field_name, i),
                &err
            ));
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

### 2.2 Dict[K,V] Validation

```rust
fn validate_dict(
    field_name: &str,
    doc: &BsonDocument,
    key_type: &str,
    value_type: &str,
) -> Result<(), Vec<ValidationError>> {
    // Validate all keys and values match expected types
}
```

---

## 3. Testing Strategy

```python
def test_list_validation():
    """Verify List[T] validation."""
    class User(Document):
        tags: List[str]

    # Should raise: tags must be list of strings
    with pytest.raises(ValidationError):
        User(tags=["python", 123, "rust"])  # Mixed types
```

---

## 4. Implementation Plan

- **Day 1**: List[T] validation
- **Day 2**: Dict[K,V] and nested documents
- **Day 3**: ObjectId, DateTime, tests

---

## 5. Dependencies

**Depends on**: 106 (Basic Type Validation)
**Enables**: Full Pydantic-level type safety

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Types supported | List, Dict, nested docs, ObjectId, DateTime |
| Speed | 10-100x faster than Pydantic |

**Status**: Ready for implementation
