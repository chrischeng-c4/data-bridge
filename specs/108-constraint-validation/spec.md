---
feature: constraint-validation
components:
  - data-bridge
lead: data-bridge
status: implemented
created: 2025-12-15
updated: 2025-12-16
branch: feature/cross_data-bridge
---

# Specification: Constraint Validation

## 1. Problem Definition

### 1.1 Current State

Types are validated (str, int, List[T]) but no constraint checks (min_length, max, format).

### 1.2 Proposed Solution

Add constraint validation in Rust for:
- String: min_length, max_length
- Numeric: min, max
- Format: email, url

### 1.3 Success Criteria

- ✅ String length constraints
- ✅ Numeric range constraints
- ✅ Format validation (email, url)
- ✅ 10-100x faster than Pydantic

---

## 2. Technical Design

### 2.1 String Constraints

```rust
fn validate_string_constraints(
    field_name: &str,
    value: &str,
    constraints: &HashMap<String, Value>,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if let Some(min_len) = constraints.get("min_length") {
        if value.len() < min_len.as_u64().unwrap() as usize {
            errors.push(ValidationError::new(
                field_name,
                &format!("string too short (min: {})", min_len)
            ));
        }
    }

    if let Some(format) = constraints.get("format") {
        match format.as_str().unwrap() {
            "email" => {
                if !is_valid_email(value) {
                    errors.push(ValidationError::new(field_name, "invalid email"));
                }
            }
            _ => {}
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}
```

---

## 3. Testing Strategy

```python
def test_string_length():
    """Verify min_length constraint."""
    class User(Document):
        name: Annotated[str, MinLen(3)]

    with pytest.raises(ValidationError):
        User(name="AB")  # Too short
```

---

## 4. Implementation Plan

- **Day 1**: String constraints (min_length, max_length)
- **Day 2**: Numeric constraints (min, max), format (email, url)

---

## 5. Dependencies

**Depends on**: 107 (Complex Type Validation)
**Enables**: Full Pydantic feature parity (80%)

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Constraints | min_length, max_length, min, max, email, url |
| Speed | 10-100x faster than Pydantic |

**Status**: Ready for implementation
