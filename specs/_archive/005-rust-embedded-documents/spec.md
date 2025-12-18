---
feature: rust-embedded-documents
components:
  - data-bridge
lead: data-bridge
status: specifying
created: 2025-12-15
branch: feature/cross_data-bridge
---

# Specification: Rust-Based Embedded Documents

## 1. Problem Definition

### 1.1 Current State

Embedded documents (nested Pydantic models) are currently handled entirely in Python. This means:

- **BSON serialization** happens in Python for nested structures
- **No Rust optimization** for deeply nested documents
- **FieldProxy dot notation** (`User.address.city`) not supported - must use raw MongoDB queries

**Current Limitations:**
```python
class Address(BaseModel):
    street: str
    city: str
    zip_code: str

class User(Document):
    name: str
    address: Address  # Nested document

# ❌ This doesn't work - no FieldProxy for embedded fields
await User.find(User.address.city == "San Francisco")

# ✅ Must use raw MongoDB query
await User.find({"address.city": "San Francisco"})
```

### 1.2 Proposed Solution

Implement Rust-side embedded document handling for:
- **Optimized BSON serialization** of nested structures
- **FieldProxy dot notation** support (`User.address.city`)
- **Type-safe nested queries** with full validation

### 1.3 Success Criteria

- ✅ FieldProxy dot notation works for embedded documents
- ✅ Nested BSON conversion in Rust (GIL-released)
- ✅ 2-3x faster for documents with deep nesting
- ✅ Maintain Pydantic validation compatibility

### 1.4 Out of Scope

- ❌ Lazy loading of embedded documents (always loaded with parent)
- ❌ Embedded document collections (use Link instead)
- ❌ Cross-document embedded validation

---

## 2. Technical Approach

### 2.1 Architecture

```
Python Layer                      Rust Layer
─────────────────────────────     ───────────────────────────
EmbeddedDocument base class ───►  RustEmbeddedSerializer
  - Pydantic BaseModel               - Recursive BSON conversion
  - Field validation                 - Nested structure handling
  - Type hints                       - GIL-released processing

FieldProxy.dot_notation     ───►  RustNestedFieldResolver
  User.address.city                 - Parse dot notation
  User.tags[0]                      - Build MongoDB query paths
  User.metadata.settings.theme      - Type checking
```

### 2.2 Key Features

1. **Dot Notation Support**
   ```python
   # Type-safe nested queries
   await User.find(User.address.city == "SF")
   await User.find(User.address.zip_code.regex(r"^94"))
   ```

2. **Nested BSON Optimization**
   - Serialize embedded documents in Rust
   - Parallel conversion for arrays of embedded docs
   - GIL-released for CPU-intensive work

3. **Validation**
   - Pydantic validation for embedded models
   - Rust-side type checking for query paths
   - Error messages show full field path

### 2.3 Implementation Phases

**Phase 1: Basic Support** (Week 1)
- Rust serialization for single-level embedded docs
- Pydantic integration maintained
- Test with existing embedded doc usage

**Phase 2: Dot Notation** (Week 2)
- FieldProxy dot notation parsing
- Nested field resolution
- Query path validation

**Phase 3: Arrays & Deep Nesting** (Week 3)
- Array of embedded documents
- Deep nesting (3+ levels)
- Performance optimization

**Phase 4: Advanced Features** (Week 4)
- elem_match for embedded arrays
- Projection for embedded fields
- Update operators for nested fields

---

## 3. API Examples

### 3.1 Type-Safe Nested Queries

```python
from pydantic import BaseModel

class Address(BaseModel):
    street: str
    city: str
    state: str
    zip_code: str

class User(Document):
    name: str
    address: Address
    emails: list[str]

# Dot notation queries (NEW)
users_in_ca = await User.find(User.address.state == "CA").to_list()
sf_users = await User.find(User.address.city == "San Francisco").to_list()
zip_search = await User.find(User.address.zip_code.regex(r"^94")).to_list()
```

### 3.2 Array of Embedded Documents

```python
class PhoneNumber(BaseModel):
    type: str  # "mobile", "home", "work"
    number: str

class Contact(Document):
    name: str
    phones: list[PhoneNumber]

# Query embedded array
has_mobile = await Contact.find(
    Contact.phones.elem_match(PhoneNumber.type == "mobile")
).to_list()
```

---

## 4. Testing Strategy

### 4.1 Compatibility Tests
- All existing embedded document tests pass
- Pydantic validation still works
- Backward compatibility verified

### 4.2 Performance Tests
- Benchmark nested BSON serialization
- Compare GIL-released vs Python implementation
- Test with varying nesting depths (1-5 levels)

### 4.3 Feature Tests
- Dot notation queries
- Array elem_match
- Deep nesting (3+ levels)
- Update operations on nested fields

---

## 5. Migration Path

**User Action Required:** None - transparent upgrade

Existing code continues to work, with new dot notation capability:
```python
# Before (still works)
await User.find({"address.city": "SF"})

# After (new capability)
await User.find(User.address.city == "SF")
```

---

## 6. Dependencies

- Spec 002 (Performance) - Foundation for Rust BSON handling
- Spec 008 (QueryBuilder/FieldProxy) - Dot notation parsing

---

## References

- Documentation: `tech-docs/content/data-bridge/guides/embedded-documents.mdx`
- Pydantic docs: https://docs.pydantic.dev/
