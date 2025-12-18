# Plan: 902 Test Framework

## Overview

Build a custom Python test framework with Rust engine, following the data-bridge patterns established in 901 HTTP Client.

---

## Phase 1: Foundation (MVP)

### 1.1 Rust Crate Setup

**Create**: `crates/data-bridge-test/`

```
crates/data-bridge-test/
├── Cargo.toml
└── src/
    ├── lib.rs              # Module exports
    ├── runner.rs           # Test discovery and execution
    ├── assertions.rs       # expect() assertion engine
    └── reporter.rs         # Markdown report generation
```

**Cargo.toml**:
```toml
[package]
name = "data-bridge-test"
version.workspace = true
edition.workspace = true

[dependencies]
pyo3.workspace = true
pyo3-async-runtimes.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
```

### 1.2 Core Types

```rust
// Test metadata
pub struct TestMeta {
    pub name: String,
    pub test_type: TestType,
    pub timeout: Option<Duration>,
    pub tags: Vec<String>,
}

pub enum TestType {
    Unit,
    Profile,
    Stress,
    Security,
}

// Test result
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub metrics: Option<TestMetrics>,
}

pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

// Assertions
pub struct Expectation<T> {
    value: T,
    negated: bool,
}
```

### 1.3 Python Layer

**Create**: `python/data_bridge/test/`

```
python/data_bridge/test/
├── __init__.py           # Public API
├── decorators.py         # @test decorator
├── suite.py              # TestSuite base class
└── assertions.py         # expect() wrapper
```

---

## Phase 2: PyO3 Bindings

**Create**: `crates/data-bridge/src/test.rs`

### 2.1 Exported Classes

```rust
#[pyclass(name = "TestRunner")]
pub struct PyTestRunner { ... }

#[pyclass(name = "TestResult")]
pub struct PyTestResult { ... }

#[pyclass(name = "Expectation")]
pub struct PyExpectation { ... }
```

### 2.2 Module Registration

```rust
pub fn register_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyTestRunner>()?;
    m.add_class::<PyTestResult>()?;
    m.add_class::<PyExpectation>()?;
    Ok(())
}
```

---

## Phase 3: Profiling

**Add to**: `crates/data-bridge-test/src/profiler.rs`

### 3.1 Measurements

- CPU time (process time)
- Memory usage (peak allocation)
- Rust-Python boundary overhead (GIL acquire/release time)

### 3.2 Profile Decorator

```python
@profile(iterations=100, warmup=10, measure=["cpu", "memory", "rust_boundary"])
async def test_performance(self):
    return await self.client.post("/users", json={"name": "Test"})
```

---

## Phase 4: Stress Testing

**Add to**: `crates/data-bridge-test/src/stress.rs`

### 4.1 Tokio Scheduler

```rust
pub struct StressConfig {
    pub concurrent_users: usize,
    pub duration_secs: f64,
    pub target_rps: Option<usize>,
    pub ramp_up_secs: Option<f64>,
}

pub struct StressResult {
    pub total_requests: u64,
    pub successful: u64,
    pub failed: u64,
    pub rps: f64,
    pub latency_p50_ms: u64,
    pub latency_p95_ms: u64,
    pub latency_p99_ms: u64,
}
```

### 4.2 Stress Decorator

```python
@stress(concurrent_users=100, duration=60.0, target_rps=500)
async def high_load_test(self):
    response = await self.client.get("/health")
    return response.status_code == 200
```

---

## Phase 5: Security Testing

**Add to**: `crates/data-bridge-test/src/security.rs`

### 5.1 Fuzzing Engine

- SQL injection payloads
- NoSQL injection payloads
- XSS payloads
- Custom wordlist support

### 5.2 Security Decorator

```python
@security(fuzz_inputs=True, injection_types=["sql", "nosql", "xss"])
async def input_sanitization(self, payload: str):
    response = await self.client.post("/search", json={"q": payload})
    expect(response.status_code).to_not_equal(500)
```

---

## File Summary

### New Files to Create

**Rust Crate**:
- `crates/data-bridge-test/Cargo.toml`
- `crates/data-bridge-test/src/lib.rs`
- `crates/data-bridge-test/src/runner.rs`
- `crates/data-bridge-test/src/assertions.rs`
- `crates/data-bridge-test/src/reporter.rs`
- `crates/data-bridge-test/src/profiler.rs` (Phase 3)
- `crates/data-bridge-test/src/stress.rs` (Phase 4)
- `crates/data-bridge-test/src/security.rs` (Phase 5)

**PyO3 Bindings**:
- `crates/data-bridge/src/test.rs`

**Python Layer**:
- `python/data_bridge/test/__init__.py`
- `python/data_bridge/test/decorators.py`
- `python/data_bridge/test/suite.py`
- `python/data_bridge/test/assertions.py`
- `python/data_bridge/test/cli.py`

### Files to Modify

- `Cargo.toml` - Add data-bridge-test to workspace
- `crates/data-bridge/Cargo.toml` - Add test feature
- `crates/data-bridge/src/lib.rs` - Register test module
- `python/data_bridge/__init__.py` - Export test module
- `pyproject.toml` - Add CLI entry point

---

## Dependencies Graph

```
Phase 1 (Foundation)
    │
    ├── runner.rs
    ├── assertions.rs
    └── reporter.rs (Markdown)
          │
Phase 2 (PyO3) ─────┤
          │
Phase 3 (Profiling) │
    │               │
    └── profiler.rs │
          │         │
Phase 4 (Stress) ───┤
    │               │
    └── stress.rs   │
          │         │
Phase 5 (Security) ─┤
    │               │
    └── security.rs │
          │
    reporter.rs (HTML, JSON, JUnit)
```

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Complex async bridging | High | Follow 901 HTTP patterns |
| Profile accuracy | Medium | Use Rust std::time for precision |
| Stress test resource limits | High | Add configurable limits, graceful degradation |
| Security payload safety | Medium | Run in sandboxed environment |

---

## Success Criteria by Phase

### Phase 1 (MVP)
- [ ] `@test` decorator works
- [ ] `expect()` basic assertions work
- [ ] Test discovery finds decorated methods
- [ ] Markdown report generated

### Phase 2 (PyO3)
- [ ] All classes accessible from Python
- [ ] Async tests work with `future_into_py`

### Phase 3 (Profiling)
- [ ] CPU time measured accurately
- [ ] Memory usage tracked
- [ ] Boundary overhead < 1ms resolution

### Phase 4 (Stress)
- [ ] 1000+ concurrent users supported
- [ ] RPS metrics calculated
- [ ] Percentile latencies accurate

### Phase 5 (Security)
- [ ] SQL injection detected
- [ ] NoSQL injection detected
- [ ] XSS patterns detected
