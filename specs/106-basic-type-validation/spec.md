---
feature: basic-type-validation
components:
  - data-bridge
lead: data-bridge
status: completed
created: 2025-12-15
completed: 2025-12-16
branch: feature/cross_data-bridge
---

# Specification: Basic Type Validation

## 1. Problem Definition

### 1.1 Current State

No runtime type validation for str, int, float, bool.

### 1.2 Proposed Solution

Validate basic types in Rust using extracted schema.

### 1.3 Success Criteria

- ✅ Validate str, int, float, bool, Optional[T]
- ✅ Catch type mismatches at runtime
- ✅ 10-100x faster than Pydantic v2
- ✅ Required field checks

---

## 2. Technical Design

### 2.1 Rust Type Validation

```rust
// crates/data-bridge/src/validation.rs

pub fn validate_document(
    class_name: &str,
    doc: &BsonDocument,
) -> Result<(), Vec<ValidationError>> {
    let schema = get_schema_cache().lock().unwrap().get(class_name).cloned();

    let mut errors = Vec::new();

    for (field_name, field_schema) in schema.fields {
        let value = doc.get(&field_name);

        // Check required
        if value.is_none() && !field_schema.is_optional {
            errors.push(ValidationError::new(&field_name, "field required"));
            continue;
        }

        // Validate type
        if let Err(err) = validate_field_type(&field_name, value.unwrap(), &field_schema) {
            errors.push(err);
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn validate_field_type(
    field_name: &str,
    value: &Bson,
    schema: &FieldSchema,
) -> Result<(), ValidationError> {
    let actual_type = match value {
        Bson::String(_) => "string",
        Bson::Int32(_) | Bson::Int64(_) => "integer",
        Bson::Double(_) => "float",
        Bson::Boolean(_) => "boolean",
        _ => "unknown",
    };

    if actual_type != schema.base_type {
        return Err(ValidationError::type_mismatch(
            field_name, &schema.base_type, actual_type
        ));
    }

    Ok(())
}
```

---

## 3. Testing Strategy

```python
def test_type_validation():
    """Verify type mismatch detection."""
    class User(Document):
        age: int

    # Should raise: age must be int
    with pytest.raises(ValidationError):
        User(age="not_a_number")
```

---

## 4. Implementation Plan

- **Day 1**: Rust validation logic
- **Day 2**: Integration with Document.__init__
- **Day 3**: Tests and benchmarks

---

## 5. Dependencies

**Depends on**: 105 (Type Schema Extraction)
**Enables**: 107 (Complex Types), 108 (Constraints)

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Validation speed | 10-100x faster than Pydantic |
| Type coverage | str, int, float, bool, Optional |
| Error detection | 100% for basic types |

**Status**: Ready for implementation
