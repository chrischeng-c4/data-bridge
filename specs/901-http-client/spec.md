---
feature: http-client
components:
  - data-bridge
lead: data-bridge
status: done
created: 2025-12-17
updated: 2025-12-17
branch: feature/data-bridge_mvp
---

# Specification: HTTP Client

## 1. Problem Definition

### 1.1 Current State

Python HTTP clients (httpx, aiohttp, requests) are limited by the GIL, which restricts throughput in high-concurrency scenarios. For testing and API integration with the data-bridge ecosystem, we need a high-performance HTTP client that bypasses GIL limitations.

### 1.2 Proposed Solution

Rust-based async HTTP client using reqwest with:
- Connection pooling with configurable limits
- Built-in latency measurement on every request
- All standard HTTP methods
- Request/response serialization fully in Rust
- PyO3 bindings with `future_into_py` for async support

### 1.3 Success Criteria

- [x] Connection pooling with configurable pool size
- [x] All HTTP methods (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
- [x] Request/response latency tracking built-in
- [x] Error handling with sanitization (no credentials in error messages)
- [ ] PyO3 Python bindings with async support
- [ ] Python wrapper with type hints
- [ ] Higher throughput than httpx in benchmarks

---

## 2. Technical Design

### 2.1 Crate Structure

```
crates/data-bridge-http/
├── Cargo.toml
└── src/
    ├── lib.rs           # Module exports
    ├── client.rs        # HttpClient with connection pool
    ├── config.rs        # HttpClientConfig builder pattern
    ├── request.rs       # RequestBuilder, ExtractedRequest (GIL-free)
    ├── response.rs      # HttpResponse with latency_ms
    └── error.rs         # HttpError with sanitization
```

### 2.2 Key Types

```rust
// HttpClient - Thread-safe client with connection pool
pub struct HttpClient {
    inner: Arc<HttpClientInner>,
}

// HttpClientConfig - Builder pattern for configuration
pub struct HttpClientConfig {
    pub base_url: Option<String>,
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub pool_max_idle_per_host: usize,
    pub pool_idle_timeout: Option<Duration>,
    // ... security options
}

// HttpResponse - Response with built-in latency
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub latency_ms: u64,
    pub url: String,
}

// ExtractedRequest - GIL-free intermediate representation
pub struct ExtractedRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub query_params: Vec<(String, String)>,
    pub body: ExtractedBody,
    pub auth: ExtractedAuth,
    pub timeout_ms: Option<u64>,
}
```

### 2.3 Python API

```python
from data_bridge.http import HttpClient, HttpResponse

# Create client with configuration
client = HttpClient(
    base_url="https://api.example.com",
    timeout=30.0,
    connect_timeout=10.0,
    pool_max_idle_per_host=10,
)

# Async HTTP methods
response = await client.get("/users", headers={"X-Api-Key": "..."})
response = await client.post("/users", json={"name": "Alice"})
response = await client.put("/users/1", json={"name": "Bob"})
response = await client.patch("/users/1", json={"status": "active"})
response = await client.delete("/users/1")

# Response handling
response.status_code    # 200
response.latency_ms     # 45
response.json()         # {"id": 123, "name": "Alice"}
response.text()         # Raw text response
response.headers        # dict of headers
response.is_success     # True if 2xx
```

### 2.4 GIL-Release Pattern

Following data-bridge-mongodb pattern:

```rust
#[pymethods]
impl PyHttpClient {
    fn get<'py>(
        &self,
        py: Python<'py>,
        path: String,
        headers: Option<HashMap<String, String>>,
        params: Option<HashMap<String, String>>,
        timeout: Option<f64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        // Phase 1: Extract with GIL
        let extracted = ExtractedRequest {
            method: HttpMethod::Get,
            url: path,
            headers: headers.unwrap_or_default().into_iter().collect(),
            query_params: params.unwrap_or_default().into_iter().collect(),
            body: ExtractedBody::None,
            auth: ExtractedAuth::None,
            timeout_ms: timeout.map(|t| (t * 1000.0) as u64),
        };
        let inner = self.inner.clone();

        // Phase 2: Release GIL, execute async
        future_into_py(py, async move {
            let response = inner.execute(extracted).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            // Phase 3: Convert back to Python
            Python::with_gil(|py| response_to_py(py, response))
        })
    }
}
```

---

## 3. Implementation Status

### 3.1 Completed

- [x] `data-bridge-http` crate created
- [x] `HttpClient` with reqwest + connection pooling
- [x] `HttpClientConfig` builder pattern
- [x] `RequestBuilder` and `ExtractedRequest`
- [x] `HttpResponse` with latency tracking
- [x] `HttpError` with credential sanitization
- [x] Workspace Cargo.toml updated

### 3.2 In Progress

- [ ] PyO3 bindings (`crates/data-bridge/src/http.rs`)
- [ ] Python wrapper (`python/data_bridge/http.py`)

### 3.3 Remaining

- [ ] Integration tests
- [ ] Benchmark vs httpx
- [ ] Documentation

---

## 4. Dependencies

**Depends on**: None (standalone library)

**Enables**:
- 902 Test Framework (uses HTTP client for API testing)
- Backend API testing infrastructure

---

## 5. Success Metrics

| Metric | Target | Current |
|--------|--------|---------|
| HTTP methods | GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS | Done |
| Connection pooling | Configurable pool size | Done |
| Latency tracking | Built-in on every request | Done |
| PyO3 bindings | Full async support | In Progress |
| Throughput vs httpx | Higher RPS | Pending |

**Status**: Implementation in progress
