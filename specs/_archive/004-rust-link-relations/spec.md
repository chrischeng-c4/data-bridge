---
feature: rust-link-relations
components:
  - data-bridge
lead: data-bridge
status: specifying
created: 2025-12-15
branch: feature/cross_data-bridge
---

# Specification: Rust-Based Link/BackLink Relations

## 1. Problem Definition

### 1.1 Current State

Link and BackLink relations are currently implemented entirely in Python (`python/data_bridge/relations.py`). While functional, this means:

- **No Rust performance benefits** for link resolution
- **Python GIL limitations** during link fetching
- **Inefficient batched loading** compared to what Rust could achieve

**Current Implementation:**
```python
class Link[T]:
    """Forward reference to another document"""
    # Implemented in pure Python
    # Fetches via Python MongoDB calls
    # Subject to GIL during resolution

class BackLink[T]:
    """Reverse reference from related documents"""
    # Implemented in pure Python
    # Batched fetching in Python
```

### 1.2 Proposed Solution

Move Link/BackLink resolution to Rust for:
- **GIL-released link fetching** - parallel resolution across CPU cores
- **Optimized batched loading** - intelligent query batching in Rust
- **Cascade operations in Rust** - write/delete cascades without Python overhead

### 1.3 Success Criteria

- ✅ Maintain backward-compatible Python API
- ✅ 2-5x faster link resolution for large batches
- ✅ GIL-released during link fetching
- ✅ Support all current cascade rules (WriteRules, DeleteRules)

### 1.4 Out of Scope

- ❌ Changing Link/BackLink API
- ❌ Adding new relationship types (focus on performance only)
- ❌ Graph traversal optimization (deferred)

---

## 2. Technical Approach

### 2.1 Architecture

```
Python Layer                    Rust Layer
─────────────────────────────   ───────────────────────────
Link[T] class wrapper    ───►   RustLinkResolver
  - Type hints                    - Extract ObjectIds
  - Field configuration           - Batch query planning
  - API compatibility             - GIL-released fetching
                                  - Cascade rule execution
BackLink[T] wrapper      ───►   RustBackLinkResolver
  - Reverse resolution            - Reverse query optimization
  - Batching hints                - Parallel loading
```

### 2.2 Key Features

1. **Batched Link Fetching**
   - Collect all Link fields to resolve
   - Single Rust call for batch resolution
   - Parallel queries across collections

2. **Cascade Operations**
   - WriteRules.WRITE - Cascade saves in Rust
   - DeleteRules.DELETE_LINKS - Cascade deletes in Rust
   - Transaction support (future)

3. **Lazy Loading**
   - Links resolve on first access
   - Cached after resolution
   - Configurable eager loading

### 2.3 Implementation Phases

**Phase 1: Foundation** (Week 1)
- Rust link resolver structure
- Basic forward link resolution
- Test compatibility with existing API

**Phase 2: Performance** (Week 2)
- Batched fetching optimization
- GIL-released parallel loading
- Benchmark vs Python implementation

**Phase 3: Cascades** (Week 3)
- Cascade write rules
- Cascade delete rules
- Safety validations

**Phase 4: BackLinks** (Week 4)
- Reverse link resolution
- Batch reverse queries
- Integration tests

---

## 3. API Compatibility

### 3.1 No Breaking Changes

```python
# Existing API remains unchanged
class Post(Document):
    title: str
    author: Link["User"]  # Still works

class User(Document):
    name: str
    posts: BackLink["Post"] = BackLink(original_field="author")  # Still works

# Fetch with links (same API, faster implementation)
post = await Post.find_one(Post.id == post_id, fetch_links=True)
print(post.author.name)  # Resolved in Rust
```

### 3.2 Performance Configuration

```python
class Settings:
    # New optional settings for Rust optimization
    eager_load_links = False  # Lazy by default
    max_link_batch_size = 100  # Batch size hint
```

---

## 4. Testing Strategy

### 4.1 Compatibility Tests
- All existing Link/BackLink tests must pass
- Cascade rule behavior verified
- Edge cases (circular references, missing links)

### 4.2 Performance Tests
- Benchmark link resolution speed
- Measure GIL release impact
- Compare batch vs individual fetching

### 4.3 Integration Tests
- End-to-end document relations
- Cascade operations
- Complex relationship graphs

---

## 5. Migration Path

### 5.1 Gradual Rollout
1. Implement Rust link resolver (feature-flagged)
2. Run in parallel with Python implementation
3. Verify correctness with A/B testing
4. Switch default to Rust
5. Deprecate Python implementation (v2.0)

### 5.2 User Action Required
**None** - Transparent upgrade with same API

---

## 6. Dependencies

- Spec 002 (Performance Optimization) - Rust BSON foundation
- Spec 003 (Type Validation) - Type-safe link targets

---

## 7. Future Enhancements

- Graph traversal optimization
- Link prefetching strategies
- Relationship caching
- Transaction support for cascades

---

## References

- Current implementation: `python/data_bridge/relations.py`
- Documentation: `tech-docs/content/data-bridge/guides/embedded-documents.mdx`
