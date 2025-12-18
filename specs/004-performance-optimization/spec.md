---
feature: performance-optimization
components:
  - data-bridge-mongodb
  - data-bridge
  - python
lead: data-bridge
status: in-progress
created: 2025-12-18
branch: feature/004-performance-optimization
---

# Specification: Performance Optimization

## 1. Problem Definition

### 1.1 Current State

**Current Benchmark Results (2024-12-18) vs Beanie:**
```
Operation           data-bridge  Beanie       Speedup  Target  Status
─────────────────────────────────────────────────────────────────────
insert_one          2.43ms       1.14ms       0.47x    2x      ❌ 2x SLOWER
bulk_insert(1000)   30.04ms      58.70ms      1.95x    5x      ⚠️ Below
find_many(100)      10.42ms      7.24ms       0.69x    3x      ❌ 1.4x SLOWER
bulk_update         14.84ms      21.12ms      1.42x    5x      ⚠️ Below
```

**Root Cause Analysis:**
1. **insert_one slowness**: Python `save()` has 4-5 async awaits (hooks, validation, state mgmt)
2. **find_many slowness**: Document creation overhead in Python `_from_db()` path
3. **Bulk ops**: Working well but can be optimized further

**Problems**:
- Single inserts slower than Beanie (2x slower)
- Find operations slower than Beanie (1.4x slower)
- Bulk operations below target (need 5x, only 1.95x for inserts)
- Excessive Python ↔ Rust boundary crossings
- Validation/hooks run even when not needed

**Impact**: Current performance does not meet the goal of 2-5x faster than Beanie.

### 1.2 Proposed Solution

Four-phase optimization approach:

**Phase 1: Fast-path Insert** (CURRENT FOCUS)
- Add `validate=False, hooks=False` options to save()
- Skip unnecessary async awaits when not needed
- Direct Rust insertion path

**Phase 2: Optimize Document Creation**
- Use Rust `find_as_documents` exclusively
- Reduce Python ↔ Rust boundary crossings
- Batch document construction

**Phase 3: Connection Pool Tuning**
- Cursor streaming optimization
- Connection pool configuration
- Query result caching

**Phase 4: Parallel Batch Processing**
- Use Rayon for parallelization
- Optimize batch sizes
- Parallel BSON conversion

### 1.3 Success Criteria

**Phase 1 Targets:**
- ✅ insert_one: <1.0ms (2x faster than Beanie)
- ✅ `validate=False` option added to save()
- ✅ `hooks=False` option added to save()
- ✅ All existing tests pass
- ✅ Backward compatible (defaults maintain current behavior)

**Overall Targets (All Phases):**
- ✅ insert_one: 2x faster than Beanie
- ✅ bulk_insert: 5x faster than Beanie
- ✅ find_many: 3x faster than Beanie
- ✅ bulk_update: 5x faster than Beanie

---

## 2. Technical Design - Phase 1: Fast-path Insert

### 2.1 Architecture

```
Current Flow (Slow):
Python save() → validate() → run_hooks() → state_mgmt() → Rust insert()
                 ↓             ↓              ↓
              awaits        awaits         awaits

Fast-path Flow (Target):
Python save(validate=False, hooks=False) → Rust insert()
                                           (single await)
```

### 2.2 API Changes

**Python Layer (`python/data_bridge/document.py`):**

```python
async def save(
    self,
    *,
    validate: bool = True,      # NEW: Skip validation when False
    hooks: bool = True,         # NEW: Skip hooks when False
    session: Optional[Any] = None
) -> "Document":
    """
    Save document to database.

    Args:
        validate: Run validation before save (default: True)
        hooks: Run before/after save hooks (default: True)
        session: MongoDB session for transactions

    Returns:
        Self for chaining
    """
    if validate:
        await self.validate()

    if hooks:
        await self._run_before_save_hooks()

    # Rust insert
    result = await self._save_to_db(session)

    if hooks:
        await self._run_after_save_hooks()

    return self
```

**Rust Layer (`crates/data-bridge/src/mongodb.rs`):**

No changes needed - existing `save_document` already optimized.

### 2.3 Implementation Plan

**Step 1: Update Python save() signature**
- Add `validate` and `hooks` parameters with defaults `True`
- Maintain backward compatibility

**Step 2: Add conditional execution**
- Wrap validation in `if validate:`
- Wrap hooks in `if hooks:`
- Direct path to Rust when both False

**Step 3: Update insert_many() for bulk**
- Add same parameters to `insert_many()`
- Allow fast bulk inserts

**Step 4: Documentation**
- Update docstrings
- Add usage examples
- Performance guidelines

### 2.4 Testing Strategy

**Unit Tests:**
```python
# tests/unit/test_fast_path_insert.py
async def test_save_with_validate_false():
    """Validation should be skipped when validate=False"""
    user = User(name="Test", age=999)  # Invalid age
    await user.save(validate=False)  # Should succeed

async def test_save_with_hooks_false():
    """Hooks should be skipped when hooks=False"""
    hook_called = False
    # Test that hooks don't run

async def test_save_defaults_unchanged():
    """Default behavior should validate and run hooks"""
    # Ensure backward compatibility
```

**Integration Tests:**
```python
# tests/integration/test_fast_path_performance.py
async def test_fast_path_performance():
    """Fast-path should be significantly faster"""
    # Benchmark with validate=True, hooks=True
    # Benchmark with validate=False, hooks=False
    # Assert speedup
```

**Benchmark Tests:**
```python
# tests/mongo/benchmarks/bench_insert.py
@insert_one.add("data-bridge-fast")
async def db_insert_one_fast():
    await DBUser(name="Test", email="test@test.com", age=30).save(
        validate=False,
        hooks=False
    )
```

### 2.5 Migration Guide

**For users who want maximum performance:**
```python
# Before
await user.save()

# After (fast-path)
await user.save(validate=False, hooks=False)
```

**For bulk operations:**
```python
# Before
await User.insert_many(users)

# After (fast-path)
await User.insert_many(users, validate=False, hooks=False)
```

---

## 3. Implementation Phases

### Phase 1: Fast-path Insert (Week 1)
- [ ] Update save() signature
- [ ] Add conditional validation/hooks
- [ ] Update insert_many()
- [ ] Write tests
- [ ] Run benchmarks
- [ ] Update documentation

### Phase 2: Optimize Document Creation (Week 2)
- [ ] Profile _from_db() path
- [ ] Move to Rust find_as_documents
- [ ] Batch document construction
- [ ] Benchmark find_many

### Phase 3: Connection Pool Tuning (Week 3)
- [ ] Optimize cursor streaming
- [ ] Connection pool config
- [ ] Query result caching

### Phase 4: Parallel Batch Processing (Week 4)
- [ ] Rayon integration
- [ ] Optimize batch sizes
- [ ] Parallel BSON conversion

---

## 4. Performance Targets

### Phase 1 Expected Results

```
Operation           Before   After    Speedup  vs Beanie
─────────────────────────────────────────────────────────
insert_one          2.43ms   0.8ms    3.0x     1.4x faster
bulk_insert(1000)   30.04ms  15.0ms   2.0x     3.9x faster
```

### All Phases Expected Results

```
Operation           Current  Target   vs Beanie
──────────────────────────────────────────────
insert_one          2.43ms   0.5ms    2.3x faster
bulk_insert(1000)   30.04ms  12.0ms   4.9x faster
find_many(100)      10.42ms  2.4ms    3.0x faster
bulk_update         14.84ms  4.2ms    5.0x faster
```

---

## 5. Security & Compatibility

### 5.1 Security Considerations
- Skipping validation is opt-in (safe by default)
- Document validation requirements when using fast-path
- Warning in docs about skipping validation

### 5.2 Backward Compatibility
- Default behavior unchanged (validate=True, hooks=True)
- Existing code works without changes
- New parameters are keyword-only

### 5.3 Breaking Changes
None - fully backward compatible.

---

## 6. References

- ROADMAP.md: Feature 201
- CLAUDE.md: Performance targets and benchmarks
- Beanie benchmark comparison
