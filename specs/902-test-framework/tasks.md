# Tasks: 902 Test Framework

## Phase 1: Foundation (MVP)

### T1: Rust Crate Setup

- [ ] T1.1: Create `crates/data-bridge-test/Cargo.toml`
- [ ] T1.2: Create `crates/data-bridge-test/src/lib.rs` with module exports
- [ ] T1.3: Add `data-bridge-test` to workspace `Cargo.toml`
- [ ] T1.4: Add `test` feature to `crates/data-bridge/Cargo.toml`

**Test**: `cargo check -p data-bridge-test`

---

### T2: Core Types (runner.rs)

- [ ] T2.1: Define `TestMeta` struct (name, type, timeout, tags)
- [ ] T2.2: Define `TestType` enum (Unit, Profile, Stress, Security)
- [ ] T2.3: Define `TestResult` struct (name, status, duration, error, metrics)
- [ ] T2.4: Define `TestStatus` enum (Passed, Failed, Skipped, Error)
- [ ] T2.5: Implement `TestRunner` with discovery logic
- [ ] T2.6: Implement test execution scheduler

**Test**: Unit tests in `runner.rs`

---

### T3: Assertions (assertions.rs)

- [ ] T3.1: Define `Expectation<T>` struct
- [ ] T3.2: Implement `to_equal()`, `to_not_equal()`
- [ ] T3.3: Implement `to_be_truthy()`, `to_be_falsy()`, `to_be_none()`
- [ ] T3.4: Implement `to_be_greater_than()`, `to_be_less_than()`, `to_be_between()`
- [ ] T3.5: Implement `to_match()` (regex)
- [ ] T3.6: Implement `to_contain()`, `to_have_length()`
- [ ] T3.7: Implement `to_have_key()`, `to_have_keys()`
- [ ] T3.8: Define `AssertionError` with context

**Test**: Unit tests for each assertion method

---

### T4: Reporter (reporter.rs)

- [ ] T4.1: Define `ReportConfig` struct
- [ ] T4.2: Define `ReportFormat` enum (Markdown, Html, Json, Junit)
- [ ] T4.3: Implement `MarkdownReporter`
- [ ] T4.4: Generate summary table (type, passed, failed, duration)
- [ ] T4.5: Generate failed test details
- [ ] T4.6: Write report to file

**Test**: Generate sample report, verify format

---

## Phase 2: PyO3 Bindings

### T5: Python Classes (test.rs)

- [ ] T5.1: Create `crates/data-bridge/src/test.rs`
- [ ] T5.2: Define `PyTestRunner` class
- [ ] T5.3: Define `PyTestResult` class with getters
- [ ] T5.4: Define `PyExpectation` class with assertion methods
- [ ] T5.5: Implement `expect()` function
- [ ] T5.6: Implement `register_module()` function

**Test**: `cargo check -p data-bridge --features test`

---

### T6: Module Registration

- [ ] T6.1: Add `#[cfg(feature = "test")] mod test;` to lib.rs
- [ ] T6.2: Register test submodule in `data_bridge()` function
- [ ] T6.3: Update `crates/data-bridge/Cargo.toml` features

**Test**: `maturin develop && python -c "from data_bridge import test"`

---

## Phase 3: Python Layer

### T7: Test Package Setup

- [ ] T7.1: Create `python/data_bridge/test/__init__.py`
- [ ] T7.2: Import and re-export from Rust extension
- [ ] T7.3: Define `__all__` exports

---

### T8: Decorators (decorators.py)

- [ ] T8.1: Implement `@test(timeout, tags)` decorator
- [ ] T8.2: Store test metadata on function
- [ ] T8.3: Support async test functions
- [ ] T8.4: Implement `@skip(reason)` decorator
- [ ] T8.5: Implement `@skip_if(condition)` decorator

**Test**: Decorate sample functions, verify metadata

---

### T9: TestSuite (suite.py)

- [ ] T9.1: Define `TestSuite` base class
- [ ] T9.2: Implement `setup_suite()` / `teardown_suite()` hooks
- [ ] T9.3: Implement `setup()` / `teardown()` hooks (per-test)
- [ ] T9.4: Implement test discovery (find decorated methods)
- [ ] T9.5: Implement `run()` method

**Test**: Create sample TestSuite, run tests

---

### T10: Assertions (assertions.py)

- [ ] T10.1: Create `expect(value)` wrapper function
- [ ] T10.2: Create `expect_http(response)` wrapper function
- [ ] T10.3: Add HTTP-specific assertions:
  - `to_have_status(code)`
  - `to_be_success()` / `to_be_client_error()` / `to_be_server_error()`
  - `to_have_header(name, value)`
  - `to_have_latency_under_ms(ms)`

**Test**: Test all assertion methods

---

### T11: CLI (cli.py)

- [ ] T11.1: Create CLI entry point using argparse
- [ ] T11.2: Implement `dbt run` command
- [ ] T11.3: Add `--type` filter (unit, profile, stress, security)
- [ ] T11.4: Add `--tag` filter
- [ ] T11.5: Add `--report` option (markdown, html, json, junit)
- [ ] T11.6: Add `-o` output file option
- [ ] T11.7: Register CLI in `pyproject.toml`

**Test**: `dbt run tests/sample.py`

---

## Phase 4: Integration

### T12: Update Exports

- [ ] T12.1: Update `python/data_bridge/__init__.py` to export test
- [ ] T12.2: Update `pyproject.toml` with CLI entry point

---

### T13: Build & Verify

- [ ] T13.1: Run `maturin develop`
- [ ] T13.2: Test import: `from data_bridge.test import TestSuite, test, expect`
- [ ] T13.3: Create sample test file
- [ ] T13.4: Run with CLI: `dbt run tests/sample.py`
- [ ] T13.5: Verify Markdown report generated

---

## Phase 5: Profiling (Later)

### T14: Profiler

- [ ] T14.1: Create `crates/data-bridge-test/src/profiler.rs`
- [ ] T14.2: Implement CPU time measurement
- [ ] T14.3: Implement memory tracking
- [ ] T14.4: Implement Rust-Python boundary overhead measurement
- [ ] T14.5: Create `@profile` decorator

---

## Phase 6: Stress Testing (Later)

### T15: Stress Engine

- [ ] T15.1: Create `crates/data-bridge-test/src/stress.rs`
- [ ] T15.2: Implement Tokio-based concurrent scheduler
- [ ] T15.3: Implement RPS calculation
- [ ] T15.4: Implement latency percentiles (P50, P95, P99)
- [ ] T15.5: Implement ramp-up support
- [ ] T15.6: Create `@stress` decorator

---

## Phase 7: Security Testing (Later)

### T16: Security Engine

- [ ] T16.1: Create `crates/data-bridge-test/src/security.rs`
- [ ] T16.2: Implement SQL injection payload generator
- [ ] T16.3: Implement NoSQL injection payload generator
- [ ] T16.4: Implement XSS payload generator
- [ ] T16.5: Implement custom wordlist loading
- [ ] T16.6: Create `@security` decorator

---

## Task Dependencies

```
T1 → T2 → T3 → T4  (Rust Core)
          ↓
T5 → T6            (PyO3 Bindings)
          ↓
T7 → T8 → T9 → T10 → T11  (Python Layer)
                    ↓
T12 → T13          (Integration)
          ↓
T14 (Profiling) → T15 (Stress) → T16 (Security)
```

---

## Definition of Done (MVP)

- [ ] `@test` decorator works
- [ ] `expect()` assertions work (10+ methods)
- [ ] `TestSuite` discovers and runs tests
- [ ] Async tests supported
- [ ] Markdown report generated
- [ ] CLI `dbt run` works
- [ ] All tests passing

---

## Current Focus

**Next Task**: T1.1 - Create `crates/data-bridge-test/Cargo.toml`

Start implementation when ready to proceed with test framework development.
