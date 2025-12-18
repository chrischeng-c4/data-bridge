# Tasks: 901 HTTP Client

## Completed Tasks

### T1: Rust Core (DONE)

- [x] T1.1: Create `crates/data-bridge-http/Cargo.toml`
- [x] T1.2: Create `crates/data-bridge-http/src/lib.rs`
- [x] T1.3: Implement `client.rs` - HttpClient with connection pool
- [x] T1.4: Implement `config.rs` - HttpClientConfig builder
- [x] T1.5: Implement `request.rs` - RequestBuilder, ExtractedRequest
- [x] T1.6: Implement `response.rs` - HttpResponse with latency
- [x] T1.7: Implement `error.rs` - HttpError with sanitization
- [x] T1.8: Update workspace `Cargo.toml` with reqwest dependency
- [x] T1.9: Add `data-bridge-http` to workspace members

---

## Remaining Tasks

### T2: PyO3 Bindings

**File**: `crates/data-bridge/src/http.rs`

- [ ] T2.1: Define `PyHttpClient` struct with `Arc<HttpClient>`
- [ ] T2.2: Implement `#[new]` constructor with config options
- [ ] T2.3: Implement `get()` method with `future_into_py`
- [ ] T2.4: Implement `post()` method with JSON/form body support
- [ ] T2.5: Implement `put()` method
- [ ] T2.6: Implement `patch()` method
- [ ] T2.7: Implement `delete()` method
- [ ] T2.8: Implement `head()` method
- [ ] T2.9: Implement `options()` method
- [ ] T2.10: Define `PyHttpResponse` with Python-accessible fields
- [ ] T2.11: Implement `json()` method for response
- [ ] T2.12: Implement `text()` method for response
- [ ] T2.13: Implement `is_success` property
- [ ] T2.14: Implement `register_module()` function

**Test**: `cargo check -p data-bridge`

---

### T3: Module Registration

**File**: `crates/data-bridge/src/lib.rs`

- [ ] T3.1: Add `#[cfg(feature = "http")] mod http;`
- [ ] T3.2: Register http module in `data_bridge()` function
- [ ] T3.3: Add `add_submodule()` for http

**Test**: `cargo check -p data-bridge --features http`

---

### T4: Python Wrapper

**File**: `python/data_bridge/http.py`

- [ ] T4.1: Create `python/data_bridge/http.py`
- [ ] T4.2: Import from `data_bridge._data_bridge.http`
- [ ] T4.3: Re-export `HttpClient`, `HttpResponse`
- [ ] T4.4: Add `__all__` exports

**Test**: `python -c "from data_bridge.http import HttpClient"`

---

### T5: Update Exports

**File**: `python/data_bridge/__init__.py`

- [ ] T5.1: Add `from data_bridge import http`
- [ ] T5.2: Add `http` to `__all__`

---

### T6: Build & Verify

- [ ] T6.1: Run `maturin develop`
- [ ] T6.2: Verify import: `python -c "from data_bridge.http import HttpClient; print('OK')"`
- [ ] T6.3: Test basic GET: `python -c "import asyncio; from data_bridge.http import HttpClient; ..."`

---

### T7: Integration Tests

**File**: `tests/test_http_client.py`

- [ ] T7.1: Test `HttpClient` instantiation
- [ ] T7.2: Test GET request to httpbin.org
- [ ] T7.3: Test POST with JSON body
- [ ] T7.4: Test POST with form data
- [ ] T7.5: Test custom headers
- [ ] T7.6: Test query parameters
- [ ] T7.7: Test timeout handling
- [ ] T7.8: Test error response handling
- [ ] T7.9: Test latency measurement

**Test**: `pytest tests/test_http_client.py -v`

---

### T8: Benchmarks (Optional)

- [ ] T8.1: Create benchmark script
- [ ] T8.2: Compare single request latency vs httpx
- [ ] T8.3: Compare concurrent requests throughput
- [ ] T8.4: Document results

---

## Task Dependencies

```
T1 (DONE) → T2 → T3 → T4 → T5 → T6 → T7 → T8
                 ↓
            T3 (parallel)
```

---

## Definition of Done

- [ ] All HTTP methods work (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
- [ ] Response includes latency_ms
- [ ] JSON/form body serialization works
- [ ] Error handling with sanitized messages
- [ ] All tests passing
- [ ] `maturin develop` succeeds
- [ ] Python import works

---

## Current Focus

**Next Task**: T2.1 - Define `PyHttpClient` struct

Start with completing the PyO3 bindings in `crates/data-bridge/src/http.rs`.
