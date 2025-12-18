# data-bridge Roadmap

> Feature series from 1xx to 9xx for complete Rust-backed MongoDB ORM and infrastructure.

## Series Overview

| Series | Theme | Status | Features |
|--------|-------|--------|----------|
| **1xx** | Type Validation | ✅ Complete | Schema extraction, type validation, constraints |
| **2xx** | Performance | 🔜 Next | Bulk ops, query optimization, connection pooling |
| **3xx** | Relations | 📋 Planned | Link/BackLink in Rust, eager loading |
| **4xx** | Query Builder | 📋 Planned | FieldProxy operators, aggregation pipeline |
| **5xx** | Embedded Docs | 📋 Planned | Nested documents, dot notation |
| **6xx** | Inheritance | 📋 Planned | Polymorphic documents, discriminators |
| **7xx** | Migrations | 📋 Planned | Schema migrations, versioning |
| **8xx** | Tooling & DX | 📋 Planned | CLI, debugging, profiling |
| **9xx** | Infrastructure | ✅ Complete | HTTP client, test framework |

---

## 1xx Series: Type Validation ✅ COMPLETE

| ID | Feature | Status | Description |
|----|---------|--------|-------------|
| 101 | CoW State Management | ✅ Done | Copy-on-write for document state |
| 102 | Lazy Validation | ✅ Done | Deferred validation until save |
| 103 | Fast-Path Bulk Ops | ✅ Done | Optimized bulk insert/update |
| 104 | Rust Query Execution | ✅ Done | Query execution in Rust |
| 105 | Type Schema Extraction | ✅ Done | Python type hints → BSON descriptors |
| 106 | Basic Type Validation | ✅ Done | Primitive type validation in Rust |
| 107 | Complex Type Validation | ✅ Done | Optional, List, Dict, nested validation |
| 108 | Constraint Validation | ✅ Done | MinLen, MaxLen, Min, Max, Email, Url |

---

## 2xx Series: Performance Optimization

**Goal**: Achieve 2-5x performance improvement over MongoEngine.

| ID | Feature | Status | Description |
|----|---------|--------|-------------|
| 201 | Performance Optimization | 🔄 In Progress | Comprehensive performance tuning |

### 201: Performance Optimization (Consolidated)

**Current Benchmark Results (2024-12-18):**
```
Operation           data-bridge  MongoEngine  Speedup  Target  Status
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

**Priority Tasks:**
1. Add fast-path `insert()` that skips hooks/validation when not needed
2. Optimize find_many document creation - use Rust `find_as_documents` exclusively
3. Reduce Python ↔ Rust boundary crossings
4. Move validation and hooks to Rust where possible

**Implementation Plan:**
- Phase 1: Add `validate=False, hooks=False` options to save()
- Phase 2: Optimize document creation path
- Phase 3: Connection pool tuning and cursor streaming
- Phase 4: Parallel batch processing (Rayon)

---

## 3xx Series: Relations & References

**Goal**: Move Link/BackLink to Rust for performance and type safety.

| ID | Feature | Status | Description | Priority |
|----|---------|--------|-------------|----------|
| 301 | Rust Link[T] | 📋 Planned | Link resolution in Rust | P0 |
| 302 | Rust BackLink[T] | 📋 Planned | Reverse reference queries in Rust | P0 |
| 303 | Eager Loading | 📋 Planned | `.fetch_links()` with batching | P1 |
| 304 | Lazy Link Proxy | 📋 Planned | Auto-fetch on attribute access | P1 |
| 305 | Circular Reference Handling | 📋 Planned | Detect and handle circular refs | P2 |
| 306 | Link Validation | 📋 Planned | Validate referenced doc exists | P2 |
| 307 | Cascade Operations | 📋 Planned | Cascade delete/update through links | P2 |
| 308 | Link Prefetch Hints | 📋 Planned | Hint which links to prefetch | P3 |

**API Target:**
```python
class Author(Document):
    name: str

class Book(Document):
    title: str
    author: Link[Author]  # Rust-resolved

# Eager loading
books = await Book.find_all().fetch_links()
print(books[0].author.name)  # Already loaded, no extra query
```

---

## 4xx Series: Query Builder & Aggregation

**Goal**: Full FieldProxy operators and aggregation pipeline in Rust.

| ID | Feature | Status | Description | Priority |
|----|---------|--------|-------------|----------|
| 401 | FieldProxy Comparisons | 📋 Planned | `User.age > 25`, `User.name == "foo"` | P0 |
| 402 | FieldProxy Logical Ops | 📋 Planned | `(User.age > 18) & (User.active == True)` | P0 |
| 403 | FieldProxy String Ops | 📋 Planned | `User.name.startswith("A")`, regex | P1 |
| 404 | FieldProxy Array Ops | 📋 Planned | `User.tags.contains("python")` | P1 |
| 405 | Aggregation Pipeline | 📋 Planned | `$match`, `$group`, `$project` in Rust | P1 |
| 406 | Aggregation Stages | 📋 Planned | `$lookup`, `$unwind`, `$sort` | P2 |
| 407 | Pipeline Builder | 📋 Planned | Fluent API for aggregations | P2 |
| 408 | Query Explain | 📋 Planned | Get query execution plan | P3 |

**API Target:**
```python
# FieldProxy operators (compiled to Rust)
users = await User.find(
    (User.age > 25) & (User.status == "active")
).sort(User.created_at.desc()).to_list()

# Aggregation pipeline
result = await User.aggregate([
    Match(User.age > 18),
    Group(by=User.country, count=Count()),
    Sort(count=-1)
]).to_list()
```

---

## 5xx Series: Embedded Documents

**Goal**: Optimize nested document handling in Rust.

| ID | Feature | Status | Description | Priority |
|----|---------|--------|-------------|----------|
| 501 | Rust Embedded Serialization | 📋 Planned | BSON serialize nested docs in Rust | P0 |
| 502 | Dot Notation FieldProxy | 📋 Planned | `User.address.city == "NYC"` | P0 |
| 503 | Nested Validation | 📋 Planned | Validate embedded doc schemas | P1 |
| 504 | Partial Updates | 📋 Planned | `$set` only changed nested fields | P1 |
| 505 | Array of Embedded | 📋 Planned | `List[Address]` handling | P1 |
| 506 | Deep Merge Updates | 📋 Planned | Merge nested updates efficiently | P2 |
| 507 | Embedded Indexing | 📋 Planned | Index support for nested fields | P2 |
| 508 | Flattening/Unflattening | 📋 Planned | Convert nested ↔ flat for export | P3 |

**API Target:**
```python
class Address(EmbeddedDocument):
    street: str
    city: str
    zip: Annotated[str, MinLen(5), MaxLen(10)]

class User(Document):
    name: str
    address: Address
    addresses: List[Address]

# Dot notation queries
users = await User.find(User.address.city == "NYC").to_list()

# Partial nested update
await user.update({"$set": {"address.city": "LA"}})
```

---

## 6xx Series: Document Inheritance

**Goal**: Polymorphic documents with Rust-based type discrimination.

| ID | Feature | Status | Description | Priority |
|----|---------|--------|-------------|----------|
| 601 | Type Discriminator | 📋 Planned | `_class_id` field management | P0 |
| 602 | Polymorphic Queries | 📋 Planned | Query base class, get subclasses | P0 |
| 603 | Rust Type Resolution | 📋 Planned | Resolve correct Python class in Rust | P1 |
| 604 | Inheritance Validation | 📋 Planned | Validate based on actual type | P1 |
| 605 | Single Table Inheritance | 📋 Planned | All subclasses in one collection | P1 |
| 606 | Class Table Inheritance | 📋 Planned | Each class in own collection | P2 |
| 607 | Abstract Documents | 📋 Planned | Non-instantiable base documents | P2 |
| 608 | Mixin Support | 📋 Planned | Reusable field mixins | P3 |

**API Target:**
```python
class Animal(Document):
    name: str

class Dog(Animal):
    breed: str

class Cat(Animal):
    indoor: bool

# Query returns mixed Dog and Cat instances
animals = await Animal.find_all().to_list()
for animal in animals:
    if isinstance(animal, Dog):
        print(f"{animal.name} is a {animal.breed}")
```

---

## 7xx Series: Migrations

**Goal**: Rust-powered schema migrations for large collections.

| ID | Feature | Status | Description | Priority |
|----|---------|--------|-------------|----------|
| 701 | Migration Runner | 📋 Planned | Execute migrations in Rust | P0 |
| 702 | Version Tracking | 📋 Planned | Track applied migrations | P0 |
| 703 | Batch Processing | 📋 Planned | Process docs in configurable batches | P1 |
| 704 | Rollback Support | 📋 Planned | Reverse migrations | P1 |
| 705 | Dry Run Mode | 📋 Planned | Preview changes without applying | P1 |
| 706 | Parallel Migrations | 📋 Planned | Multi-threaded doc processing | P2 |
| 707 | Schema Diff | 📋 Planned | Auto-detect schema changes | P2 |
| 708 | Migration Generator | 📋 Planned | Generate migration from model diff | P3 |

**API Target:**
```python
class AddStatusField(Migration):
    version = "20250101_001"

    async def forward(self):
        # Runs in Rust with batching
        await self.update_many(
            User,
            filter={},
            update={"$set": {"status": "active"}}
        )

    async def backward(self):
        await self.update_many(
            User,
            filter={},
            update={"$unset": {"status": ""}}
        )

# CLI
$ data-bridge migrate --up
$ data-bridge migrate --down 1
$ data-bridge migrate --dry-run
```

---

## 8xx Series: Tooling & Developer Experience

**Goal**: CLI tools, debugging, profiling for productivity.

| ID | Feature | Status | Description | Priority |
|----|---------|--------|-------------|----------|
| 801 | CLI Tool | 📋 Planned | `data-bridge` command for common ops | P0 |
| 802 | Query Profiler | 📋 Planned | Profile slow queries | P1 |
| 803 | Schema Inspector | 📋 Planned | Inspect collection schemas | P1 |
| 804 | Index Analyzer | 📋 Planned | Suggest missing indexes | P1 |
| 805 | REPL/Shell | 📋 Planned | Interactive data-bridge shell | P2 |
| 806 | VS Code Extension | 📋 Planned | Syntax highlighting, autocomplete | P2 |
| 807 | Query Logging | 📋 Planned | Log all queries with timing | P2 |
| 808 | Health Check | 📋 Planned | Connection and performance health | P3 |

**CLI Target:**
```bash
# Schema operations
$ data-bridge schema show User
$ data-bridge schema diff User --collection users

# Index operations
$ data-bridge index list users
$ data-bridge index suggest users

# Query profiling
$ data-bridge profile "User.find(User.age > 25)"

# Migrations
$ data-bridge migrate --up
$ data-bridge migrate --status

# Health
$ data-bridge health
$ data-bridge benchmark
```

---

## 9xx Series: Infrastructure & Testing

**Goal**: Rust-based infrastructure libraries for HTTP and testing operations.

| ID | Feature | Status | Description | Priority |
|----|---------|--------|-------------|----------|
| 901 | HTTP Client | ✅ Done | Async HTTP client with connection pooling | P0 |
| 902 | Test Framework | ✅ Done | Custom Rust-based test framework engine | P1 |

### 901: HTTP Client

High-performance async HTTP client using reqwest:
- Connection pooling with configurable limits
- Built-in latency measurement
- All HTTP methods (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
- Request/response serialization in Rust
- PyO3 Python bindings

**Python API:**
```python
from data_bridge.http import HttpClient

client = HttpClient(base_url="https://api.example.com", timeout=30.0)
response = await client.get("/users")
print(response.status_code, response.latency_ms)
```

### 902: Test Framework

Custom Python test framework with Rust engine:
- Decorator-based test syntax (`@test`, `@profile`, `@stress`, `@security`)
- Custom assertion engine (`expect().to_equal()`)
- Profiling (CPU, memory, Rust-Python boundary overhead)
- Stress testing (Tokio-powered concurrent load)
- Security testing (fuzzing, injection detection)
- Report generation (Markdown, HTML, JSON, JUnit XML)

**Python API:**
```python
from data_bridge.test import TestSuite, test, expect

class UserAPITests(TestSuite):
    @test(timeout=5.0, tags=["unit"])
    async def login_returns_token(self):
        response = await self.client.post("/auth/login", json={...})
        expect(response.status_code).to_equal(200)
```

---

## Implementation Order (Recommended)

### Phase 1: Performance (Q1)
```
201 → 202 → 207 → 203
```
Fix critical performance issues first.

### Phase 2: Query Power (Q1-Q2)
```
401 → 402 → 403 → 404
```
Enable pythonic query syntax.

### Phase 3: Relations (Q2)
```
301 → 302 → 303 → 304
```
Rust-powered link resolution.

### Phase 4: Nested Data (Q2-Q3)
```
501 → 502 → 503 → 505
```
Embedded document optimization.

### Phase 5: Advanced (Q3-Q4)
```
601 → 602 → 701 → 702 → 801
```
Inheritance, migrations, tooling.

---

## Quick Reference

```
1xx - Type Validation     ✅ COMPLETE (101-108)
2xx - Performance         🔄 IN PROGRESS (201)
3xx - Relations           📋 PLANNED (301-308)
4xx - Query Builder       📋 PLANNED (401-408)
5xx - Embedded Docs       📋 PLANNED (501-508)
6xx - Inheritance         📋 PLANNED (601-608)
7xx - Migrations          📋 PLANNED (701-708)
8xx - Tooling             📋 PLANNED (801-808)
9xx - Infrastructure      ✅ COMPLETE (901-902)
```
