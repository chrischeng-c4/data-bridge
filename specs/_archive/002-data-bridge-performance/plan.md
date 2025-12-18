---
feature: data-bridge-performance
spec_ref: .specify/specs/002-data-bridge-performance/spec.md
status: planning
updated: 2025-12-10
---

# Implementation Plan: data-bridge Performance Optimization

## Overview

Transform data-bridge from **2.2x slower** than MongoEngine to **2-5x faster** through:
1. Copy-on-Write (COW) state management
2. Fast path bulk operations with Rayon parallelization
3. Lazy validation for database loads
4. pytest-benchmark integration for CI/CD regression detection

**Current State**: 0.45x vs MongoEngine (2.2x slower)
**Target State**: 2-5x faster than MongoEngine

---

## Critical Files to Modify

| File | Lines | Changes |
|------|-------|---------|
| `python/data_bridge/document.py` | 338, 443 | Replace `copy.deepcopy()` with COW StateTracker |
| `python/data_bridge/document.py` | 502-566 | Add `from_db(validate=False)` fast path |
| `crates/data-bridge/src/mongodb.rs` | 1064-1102 | Add Rayon parallel BSON conversion to `insert_many` |
| `crates/data-bridge/src/mongodb.rs` | 1199-1288 | Optimize `find_as_documents` batch conversion |
| `crates/data-bridge/Cargo.toml` | - | Add `rayon` dependency |
| `pyproject.toml` | - | Add `pytest-benchmark` dependency |
| `tests/conftest.py` | - | Add benchmark fixtures |

---

## Phase 1: pytest-benchmark Foundation (Day 1)

**Goal**: Establish baseline measurements before any optimization.

### Tasks

| # | Task | File | Est. |
|---|------|------|------|
| 1.1 | Add pytest-benchmark to pyproject.toml | `pyproject.toml` | 15min |
| 1.2 | Create `tests/benchmarks/` directory structure | `tests/benchmarks/` | 15min |
| 1.3 | Create benchmark conftest.py with fixtures | `tests/benchmarks/conftest.py` | 30min |
| 1.4 | Create baseline insert benchmarks | `tests/benchmarks/test_insert.py` | 45min |
| 1.5 | Create baseline query benchmarks | `tests/benchmarks/test_query.py` | 45min |
| 1.6 | Create baseline update benchmarks | `tests/benchmarks/test_update.py` | 30min |
| 1.7 | Create comparison benchmarks (vs MongoEngine) | `tests/benchmarks/test_comparison.py` | 45min |
| 1.8 | Run baseline and save results | `.benchmarks/` | 15min |

**Deliverables**:
- `pytest tests/benchmarks/ --benchmark-only` works
- Baseline measurements saved for regression detection
- All existing 449 tests still pass

---

## Phase 2: COW State Management (Day 2)

**Goal**: Replace `copy.deepcopy()` with field-level change tracking.

**Expected Impact**:
- Memory: 50% reduction
- State operations: 10x faster (0.1ms vs 1-2ms)

---

## Phase 3: Lazy Validation (Day 2-3)

**Goal**: Skip Pydantic validation for database-loaded data.

**Expected Impact**:
- Find operations: 2-3x faster
- Memory: Additional reduction from skipped Pydantic model creation

---

## Phase 4: Rust Parallel BSON Conversion (Day 3-4)

**Goal**: Use Rayon for parallel BSON encoding/decoding.

**Expected Impact**:
- Bulk insert: 5-10x faster
- Find many: 3-5x faster
- Boundary crossing overhead: Significantly reduced

---

## Phase 5: Fast Path Bulk Operations (Day 4)

**Goal**: Skip Pydantic for raw dict bulk operations.

**Expected Impact**:
- Bulk operations: Additional 2-3x when using raw dicts

---

## Phase 6: CI/CD Integration (Day 5)

**Goal**: Automated performance regression detection.

---

## Success Criteria

### Performance Targets

| Operation | MongoEngine | Target | Speedup |
|-----------|-------------|--------|---------|
| bulk_insert | 37.87ms | <15ms | 2.5x |
| find_many | 2.11ms | <1ms | 2x |
| bulk_update | 24.36ms | <10ms | 2.4x |
| indexed_find | 1.09ms | <0.5ms | 2x |

### Quality Gates

- [ ] All 449+ existing tests pass
- [ ] 20+ new optimization tests pass
- [ ] 30+ pytest-benchmark tests pass
- [ ] Memory: 50% reduction verified
- [ ] No API breaking changes
- [ ] Security tests pass (validation on user input)

---

## Risk Mitigations

| Risk | Mitigation |
|------|------------|
| Breaking changes | 100% backward compatible, opt-in flags |
| Memory leaks in COW | Weak references, memory profiling tests |
| Validation bypass | Security tests, clear docs |
| Rayon complexity | Incremental rollout, fallback to sequential |

---

## Estimated Timeline

**Total: ~23 hours / 5 days**

Detailed implementation plan at: `/Users/chris.cheng/.claude/plans/frolicking-churning-ladybug.md`
