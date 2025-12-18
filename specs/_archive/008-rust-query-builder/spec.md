---
feature: rust-query-builder
components:
  - data-bridge
lead: data-bridge
status: specifying
created: 2025-12-15
branch: feature/cross_data-bridge
---

# Specification: Full Rust QueryBuilder & FieldProxy

## 1. Problem Definition

### 1.1 Current State

QueryBuilder and FieldProxy provide a pythonic query interface, but most logic is in Python:
- **Basic QueryBuilder** exists in Rust (filter, sort, skip, limit)
- **FieldProxy operators** are entirely Python-based
- **Query compilation** happens in Python before Rust execution

**Current Gaps:**
```python
# ❌ These operators are Python-only
User.age > 25                    # Comparison operators
User.email == "alice@example.com"  # Equality
User.tags.in_(["python", "rust"])  # Array operators
User.name.regex(r"^A")           # String operators
User.location.near(lon, lat)     # Geospatial operators
```

### 1.2 Proposed Solution

Implement full QueryBuilder and FieldProxy in Rust:
- **All operators in Rust** - comparison, logical, array, string, geospatial
- **Query compilation in Rust** - type-safe query building
- **Optimized query execution** - eliminate Python overhead

### 1.3 Success Criteria

- ✅ All FieldProxy operators work
- ✅ Type-safe query compilation
- ✅ 2-3x faster query building
- ✅ Backward-compatible API

---

## 2. Technical Approach

### 2.1 Architecture

```
Python Layer                    Rust Layer
─────────────────────────────   ───────────────────────────
FieldProxy expressions    ───►  RustQueryBuilder
  User.age > 25                  - Parse expression tree
  User.name.regex(r"^A")         - Type-safe compilation
                                 - MongoDB query generation

QueryBuilder chainable    ───►  RustQueryExecutor
  .filter().sort().limit()       - Optimized execution plan
  .to_list()                     - GIL-released queries
```

### 2.2 Operator Categories

**Comparison Operators:**
```python
User.age > 25, User.age >= 18, User.age < 65, User.age <= 100
User.email == "test@example.com", User.email != "spam@example.com"
```

**Logical Operators:**
```python
(User.age > 25) & (User.active == True)  # AND
(User.role == "admin") | (User.role == "moderator")  # OR
```

**Array Operators:**
```python
User.tags.in_(["python", "rust"])
User.tags.not_in_(["deprecated"])
User.tags.all(["python", "mongodb"])
User.tags.size(3)
User.tags.elem_match(Tag.category == "language")
```

**String Operators:**
```python
User.name.regex(r"^A")
User.email.regex(r".*@example\.com$", flags="i")
```

**Existence/Type:**
```python
User.nickname.exists(True)
User.age.type_("int")
```

**Geospatial:**
```python
Store.location.near(lon, lat, max_distance=1000)
Store.location.geo_within_box(sw, ne)
Store.location.geo_within_polygon(coords)
Store.location.geo_within_center_sphere(center, radius)
```

---

## 3. Implementation Phases

**Phase 1: Comparison & Logical** (Week 1)
- Basic comparison operators (==, !=, >, <, >=, <=)
- Logical operators (&, |)
- Type-safe expression trees

**Phase 2: Array & String** (Week 2)
- Array operators (in_, not_in, all, size, elem_match)
- String operators (regex with flags)
- Existence and type operators

**Phase 3: Geospatial** (Week 3)
- near() with max_distance
- geo_within_box(), geo_within_polygon()
- geo_within_center_sphere()
- 2dsphere index support

**Phase 4: Advanced Features** (Week 4)
- Query optimization hints
- Query caching
- Compiled query reuse
- Performance benchmarking

---

## 4. API Examples

### 4.1 Complex Queries

```python
# Multi-condition query
users = await User.find(
    (User.age >= 18) & (User.age <= 65) &
    (User.active == True) &
    User.tags.in_(["python", "rust"])
).sort(-User.created_at).limit(10).to_list()

# Geospatial query
nearby_stores = await Store.find(
    Store.location.near(lon=-122.4, lat=37.8, max_distance=5000)
).to_list()

# Regex search
users_starting_with_a = await User.find(
    User.name.regex(r"^A", flags="i")
).to_list()
```

---

## 5. Testing Strategy

- All operator combinations tested
- Type safety validation
- Performance vs Python implementation
- MongoDB query correctness

---

## 6. Dependencies

- Spec 002 (Performance) - Rust query foundation
- Spec 005 (Embedded Docs) - Dot notation support

---

## References

- Documentation:
  - `tech-docs/content/data-bridge/api/field-proxy/`
  - `tech-docs/content/data-bridge/api/query-builder/`
