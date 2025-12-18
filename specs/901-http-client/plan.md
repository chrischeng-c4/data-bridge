# Plan: 901 HTTP Client

## Overview

Complete the Rust-based HTTP client library with PyO3 Python bindings.

---

## Phase 1: Rust Core (DONE)

**Status**: Complete

Files created:
- `crates/data-bridge-http/Cargo.toml`
- `crates/data-bridge-http/src/lib.rs`
- `crates/data-bridge-http/src/client.rs`
- `crates/data-bridge-http/src/config.rs`
- `crates/data-bridge-http/src/request.rs`
- `crates/data-bridge-http/src/response.rs`
- `crates/data-bridge-http/src/error.rs`

Files modified:
- `Cargo.toml` (workspace members, reqwest dependency)
- `crates/data-bridge/Cargo.toml` (http feature, data-bridge-http dependency)

---

## Phase 2: PyO3 Bindings (IN PROGRESS)

**File**: `crates/data-bridge/src/http.rs`

### 2.1 Python Classes

```rust
#[pyclass(name = "HttpClient")]
pub struct PyHttpClient {
    inner: Arc<HttpClient>,
}

#[pyclass(name = "HttpResponse")]
pub struct PyHttpResponse {
    status_code: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    latency_ms: u64,
    url: String,
}
```

### 2.2 Methods to Implement

| Method | Signature | Status |
|--------|-----------|--------|
| `__new__` | `(base_url, timeout, ...)` | In Progress |
| `get` | `(path, headers?, params?, timeout?)` | In Progress |
| `post` | `(path, json?, form?, headers?, timeout?)` | Pending |
| `put` | `(path, json?, form?, headers?, timeout?)` | Pending |
| `patch` | `(path, json?, form?, headers?, timeout?)` | Pending |
| `delete` | `(path, headers?, params?, timeout?)` | Pending |
| `head` | `(path, headers?, params?, timeout?)` | Pending |
| `options` | `(path, headers?, timeout?)` | Pending |

### 2.3 Module Registration

```rust
pub fn register_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyHttpClient>()?;
    m.add_class::<PyHttpResponse>()?;
    Ok(())
}
```

---

## Phase 3: Python Wrapper

**File**: `python/data_bridge/http.py`

### 3.1 Exports

```python
from data_bridge._data_bridge.http import HttpClient, HttpResponse

__all__ = ["HttpClient", "HttpResponse"]
```

### 3.2 Type Stubs (Optional)

```python
# python/data_bridge/http.pyi
class HttpClient:
    def __init__(
        self,
        base_url: str | None = None,
        timeout: float = 30.0,
        connect_timeout: float = 10.0,
        pool_max_idle_per_host: int = 10,
    ) -> None: ...

    async def get(
        self,
        path: str,
        headers: dict[str, str] | None = None,
        params: dict[str, str] | None = None,
        timeout: float | None = None,
    ) -> HttpResponse: ...
    # ... other methods
```

---

## Phase 4: Integration

### 4.1 Update Exports

**File**: `python/data_bridge/__init__.py`

```python
from data_bridge import http
```

### 4.2 Build

```bash
cd data-bridge
maturin develop
```

---

## Phase 5: Testing

### 5.1 Unit Tests

```python
# tests/test_http_client.py
import pytest
from data_bridge.http import HttpClient

@pytest.mark.asyncio
async def test_get_request():
    client = HttpClient(base_url="https://httpbin.org")
    response = await client.get("/get")
    assert response.status_code == 200
    assert response.latency_ms > 0
```

### 5.2 Benchmarks

Compare against httpx:
- Single request latency
- Concurrent requests throughput
- Connection reuse efficiency

---

## File Summary

### Files to Create
- `python/data_bridge/http.py`

### Files to Complete
- `crates/data-bridge/src/http.rs` (PyO3 bindings)

### Files to Modify
- `crates/data-bridge/src/lib.rs` (register http module)
- `python/data_bridge/__init__.py` (export http)

---

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| reqwest | 0.12 | HTTP client backend |
| pyo3 | 0.24 | Python bindings |
| pyo3-async-runtimes | 0.24 | Async bridging |
| tokio | 1.40 | Async runtime |
| serde_json | 1.0 | JSON serialization |

---

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Async bridging complexity | Follow mongodb.rs pattern |
| GIL contention | Use ExtractedRequest for GIL-free processing |
| Error handling | Sanitize credentials from error messages |
| Memory leaks | Arc<> for shared state, proper cleanup |
