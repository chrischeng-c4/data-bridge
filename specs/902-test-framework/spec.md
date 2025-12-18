---
feature: test-framework
components:
  - data-bridge
lead: data-bridge
status: planned
created: 2025-12-17
updated: 2025-12-17
branch: feature/data-bridge_mvp
---

# Specification: Test Framework

## 1. Problem Definition

### 1.1 Current State

Python testing relies on pytest, which:
- Is bound by GIL for test execution
- Has limited profiling of Rust-Python boundary overhead
- Lacks built-in stress testing capabilities
- Requires separate tools for security testing (fuzzing)

### 1.2 Proposed Solution

Custom Python test framework with Rust engine providing:
- **Unit Testing**: Decorator-based syntax with custom assertions
- **Profiling**: CPU, memory, and Rust-Python boundary overhead measurement
- **Stress Testing**: Tokio-powered concurrent load generation
- **Security Testing**: Input fuzzing and injection detection
- **Report Generation**: Markdown, HTML, JSON, JUnit XML formats

### 1.3 Success Criteria

- [ ] Custom test syntax with `@test`, `@profile`, `@stress`, `@security` decorators
- [ ] `expect()` assertion API (not Python `assert`)
- [ ] Test discovery and execution in Rust
- [ ] Profiling captures Rust-Python boundary overhead
- [ ] Stress tests generate 1000+ concurrent requests
- [ ] Security fuzzer detects basic injection vulnerabilities
- [ ] Reports in Markdown/HTML/JSON/JUnit formats
- [ ] CLI tool (`dbt`) for running tests

---

## 2. Technical Design

### 2.1 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Python Layer                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐│
│  │ decorators  │ │   suite     │ │     assertions          ││
│  │ @test       │ │ TestSuite   │ │ expect().to_equal()     ││
│  │ @profile    │ │ setup/tear  │ │ expect_http().status()  ││
│  │ @stress     │ │             │ │                         ││
│  │ @security   │ │             │ │                         ││
│  └─────────────┘ └─────────────┘ └─────────────────────────┘│
└───────────────────────────┬─────────────────────────────────┘
                            │ PyO3
┌───────────────────────────▼─────────────────────────────────┐
│                     Rust Engine                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐│
│  │  runner.rs  │ │ profiler.rs │ │    stress.rs            ││
│  │  Discovery  │ │ CPU/Memory  │ │ Tokio concurrent        ││
│  │  Execution  │ │ Boundary    │ │ load generation         ││
│  │  Scheduling │ │ overhead    │ │                         ││
│  └─────────────┘ └─────────────┘ └─────────────────────────┘│
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐│
│  │assertions.rs│ │ security.rs │ │    reporter.rs          ││
│  │ expect()    │ │ Fuzzing     │ │ MD/HTML/JSON/JUnit      ││
│  │ engine      │ │ Injection   │ │ generation              ││
│  └─────────────┘ └─────────────┘ └─────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### 2.2 Crate Structure

```
crates/data-bridge-test/
├── Cargo.toml
└── src/
    ├── lib.rs              # PyModule definition
    ├── runner.rs           # Test discovery and execution
    ├── assertions.rs       # expect() assertion engine
    ├── profiler.rs         # CPU/memory/boundary profiling
    ├── stress.rs           # Tokio stress test coordinator
    ├── security.rs         # Fuzzing and injection detection
    └── reporter.rs         # Report generation
```

### 2.3 Python Layer

```
python/data_bridge/test/
├── __init__.py           # Public API exports
├── decorators.py         # @test, @profile, @stress, @security
├── suite.py              # TestSuite base class
├── assertions.py         # expect(), expect_http() wrappers
├── config.py             # Test configuration
└── cli.py                # CLI entry point (dbt command)
```

### 2.4 Test Syntax

```python
from data_bridge.test import TestSuite, test, profile, stress, security, expect
from data_bridge.http import HttpClient

class UserAPITests(TestSuite):
    """API tests for user endpoints."""

    async def setup_suite(self):
        """Run once before all tests."""
        self.client = HttpClient(base_url="http://localhost:8000")

    async def teardown_suite(self):
        """Run once after all tests."""
        pass

    async def setup(self):
        """Run before each test."""
        pass

    async def teardown(self):
        """Run after each test."""
        pass

    # Unit Test
    @test(timeout=5.0, tags=["unit", "auth"])
    async def login_returns_token(self):
        """Verify login endpoint returns JWT token."""
        response = await self.client.post("/auth/login", json={
            "email": "test@example.com",
            "password": "secret"
        })
        expect(response.status_code).to_equal(200)
        expect(response.json()).to_have_key("token")
        expect(response.json()["token"]).to_match(r"^eyJ.*")

    # Profile Test
    @profile(iterations=100, warmup=10, measure=["cpu", "memory", "rust_boundary"])
    async def serialization_performance(self):
        """Profile JSON serialization overhead."""
        return await self.client.post("/users", json={"name": "Test"})

    # Stress Test
    @stress(concurrent_users=100, duration=60.0, target_rps=500)
    async def high_load_endpoint(self):
        """Stress test health endpoint."""
        response = await self.client.get("/health")
        return response.status_code == 200  # Return True for success

    # Security Test
    @security(fuzz_inputs=True, injection_types=["sql", "nosql", "xss"])
    async def input_sanitization(self, payload: str):
        """Test input sanitization against injections."""
        response = await self.client.post("/search", json={"q": payload})
        expect(response.status_code).to_not_equal(500)
```

### 2.5 Assertion API

```python
from data_bridge.test import expect, expect_http

# Value assertions
expect(value).to_equal(expected)
expect(value).to_not_equal(expected)
expect(value).to_be_truthy()
expect(value).to_be_falsy()
expect(value).to_be_none()
expect(value).to_be_greater_than(n)
expect(value).to_be_less_than(n)
expect(value).to_be_between(low, high)
expect(string).to_match(r"pattern")
expect(string).to_contain("substring")
expect(string).to_start_with("prefix")
expect(string).to_end_with("suffix")
expect(collection).to_contain(item)
expect(collection).to_have_length(n)
expect(dict_val).to_have_key("key")
expect(dict_val).to_have_keys(["a", "b"])

# HTTP-specific assertions
expect_http(response).to_have_status(200)
expect_http(response).to_be_success()  # 2xx
expect_http(response).to_be_client_error()  # 4xx
expect_http(response).to_be_server_error()  # 5xx
expect_http(response).to_have_header("Content-Type", "application/json")
expect_http(response).to_have_json_path("$.user.id", 123)
expect_http(response).to_have_latency_under_ms(100)
```

### 2.6 CLI Design

```bash
# Run tests
dbt run                              # Run all tests
dbt run tests/api.py                 # Run specific file
dbt run tests/ -k "login"            # Filter by name pattern
dbt run --type unit                  # Run only unit tests
dbt run --type profile               # Run only profile tests
dbt run --type stress                # Run only stress tests
dbt run --type security              # Run only security tests
dbt run --tag auth                   # Run tests with tag

# Reports
dbt run --report markdown -o report.md
dbt run --report html -o report.html
dbt run --report json -o report.json
dbt run --report junit -o junit.xml  # CI integration

# Stress test options
dbt run --type stress --users 200 --duration 120 --ramp-up 10

# Profile options
dbt run --type profile --iterations 1000 --flamegraph

# Security options
dbt run --type security --wordlist payloads.txt --threads 10

# Verbose/debug
dbt run -v                           # Verbose output
dbt run --debug                      # Debug mode
```

### 2.7 Report Format

```markdown
# Test Report: UserAPITests

**Date**: 2025-12-17 | **Duration**: 45.2s | **Status**: PASSED (32/34)

## Summary
| Type     | Passed | Failed | Skipped | Duration |
|----------|--------|--------|---------|----------|
| Unit     | 20     | 1      | 0       | 5.3s     |
| Profile  | 5      | 0      | 0       | 12.1s    |
| Stress   | 4      | 1      | 0       | 25.0s    |
| Security | 3      | 0      | 1       | 2.8s     |

## Failed Tests

### login_returns_token (FAILED)
- **Error**: Expected status 200, got 401
- **Duration**: 0.45s
- **File**: tests/api.py:25

## Profile Metrics
| Test | Iterations | CPU Time | Memory | Rust Boundary |
|------|------------|----------|--------|---------------|
| serialization | 100 | 12.3ms | 2.1MB | 0.8ms |

## Stress Results
| Test | Duration | Total Requests | RPS | P50 | P95 | P99 | Error Rate |
|------|----------|----------------|-----|-----|-----|-----|------------|
| high_load | 60s | 29,100 | 485 | 45ms | 98ms | 142ms | 0.8% |

## Security Findings
| Test | Payloads Tested | Vulnerabilities | Severity |
|------|-----------------|-----------------|----------|
| input_sanitization | 1,500 | 0 | None |
```

---

## 3. Implementation Phases

### Phase 1: MVP (Foundation)
- TestSuite base class
- `@test` decorator
- `expect()` basic assertions
- Test runner (discovery, execution)
- Markdown report

### Phase 2: Profiling
- `@profile` decorator
- CPU time measurement
- Memory tracking
- Rust-Python boundary overhead

### Phase 3: Stress Testing
- `@stress` decorator
- Tokio concurrent scheduler
- RPS metrics
- Latency percentiles (P50, P95, P99)
- Ramp-up support

### Phase 4: Security Testing
- `@security` decorator
- Input fuzzing
- SQL/NoSQL/XSS injection detection
- Payload wordlists

### Phase 5: Advanced
- HTML report with charts
- JUnit XML for CI
- Flamegraph generation
- VS Code integration

---

## 4. Dependencies

**Depends on**:
- 901 HTTP Client (for API testing)

**Rust Dependencies**:
- `pyo3` - Python bindings
- `pyo3-async-runtimes` - Async bridging
- `tokio` - Async runtime, stress test scheduler
- `serde` / `serde_json` - Serialization
- `rayon` - Parallel test execution
- `criterion` (optional) - Benchmarking utilities

---

## 5. Success Metrics

| Metric | Target |
|--------|--------|
| Test Types | unit, profile, stress, security |
| Assertions | 20+ assertion methods |
| Report Formats | MD, HTML, JSON, JUnit |
| Concurrent Users | 1000+ for stress tests |
| Profile Accuracy | Rust boundary overhead < 1ms resolution |

**Status**: Ready for implementation
