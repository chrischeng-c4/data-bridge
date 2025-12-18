---
feature: rust-document-inheritance
components:
  - data-bridge
lead: data-bridge
status: specifying
created: 2025-12-15
branch: feature/cross_data-bridge
---

# Specification: Rust-Based Document Inheritance

## 1. Problem Definition

### 1.1 Current State

Document inheritance (polymorphic documents) is implemented in Python with:
- **`_class_id` field** added automatically to distinguish types
- **Single collection storage** for entire type hierarchy
- **Polymorphic loading** from root class
- **Python-based type discrimination** during deserialization

**Current Implementation:**
```python
class Animal(Document):
    name: str
    class Settings:
        is_root = True  # Root of hierarchy

class Dog(Animal):
    breed: str

class Cat(Animal):
    indoor: bool

# Python handles type discrimination
animals = await Animal.find().to_list()  # Returns mixed Dog/Cat instances
```

### 1.2 Proposed Solution

Move inheritance handling to Rust for:
- **Optimized polymorphic loading** - type discrimination in Rust
- **`_class_id` management in Rust** - automatic field handling
- **`with_children()` filter optimization** - query type hierarchies efficiently

### 1.3 Success Criteria

- ✅ Backward-compatible with existing inheritance code
- ✅ 2-3x faster polymorphic queries
- ✅ Type-safe inheritance validation
- ✅ Support all existing inheritance features

---

## 2. Technical Approach

### 2.1 Architecture

```
Python Layer                    Rust Layer
─────────────────────────────   ───────────────────────────
Document inheritance      ───►  RustInheritanceResolver
  - Class hierarchy info         - `_class_id` injection
  - is_root setting              - Type discrimination map
  - Type registry                - Polymorphic deserialization

QueryBuilder.with_children ───► RustTypeFilter
  - Filter by type hierarchy     - Efficient type queries
  - Include/exclude subtypes     - `_class_id` filters
```

### 2.2 Key Features

1. **Automatic `_class_id` Management**
   - Injected during save
   - Used during load for type resolution
   - Hidden from user code

2. **Polymorphic Loading**
   ```python
   # Load mixed types efficiently
   animals = await Animal.find().to_list()  # Rust discriminates types
   ```

3. **Type Filtering**
   ```python
   # Query specific subtypes
   dogs = await Animal.find(Animal._class_id == "Dog").to_list()
   dogs_only = await Animal.find().with_children(False).to_list()
   ```

---

## 3. Implementation Phases

**Phase 1: Foundation** (Week 1)
- Rust type registry
- `_class_id` injection/extraction
- Basic polymorphic loading

**Phase 2: Query Optimization** (Week 2)
- `with_children()` filtering
- Type-specific queries
- Performance benchmarks

**Phase 3: Validation** (Week 3)
- Inheritance validation (circular inheritance detection)
- Type hierarchy consistency checks
- Migration support for type changes

**Phase 4: Advanced Features** (Week 4)
- Multi-level inheritance (3+ levels)
- Abstract base documents
- Type coercion safety

---

## 4. Testing Strategy

- All existing inheritance tests pass
- Polymorphic loading correctness
- Performance vs Python implementation
- Edge cases (deep hierarchies, circular refs)

---

## 5. Dependencies

- Spec 002 (Performance) - Rust BSON foundation
- Spec 003 (Type Validation) - Type hierarchy validation

---

## References

- Documentation: `tech-docs/content/data-bridge/guides/document-inheritance.mdx`
