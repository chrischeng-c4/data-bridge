# Components

> Part of [dbtest Architecture Documentation](./README.md)

This document details the responsibilities of each component in the dbtest system.

## Python Layer (Thin Wrapper)

### 1. CLI (cli.py)

**Purpose**: Command-line interface and argument parsing

**Responsibilities**:
- Parse CLI arguments using argparse
- Route to appropriate runner (unit/integration/bench/all)
- Display formatted results to user
- Handle exit codes

**Key Functions**:
```python
def main() -> int:
    """Entry point for dbtest CLI"""

def run_tests_only(config: CliConfig) -> TestReport:
    """Execute tests only (unit + integration)"""

def run_benchmarks_only(config: CliConfig) -> BenchmarkReport:
    """Execute benchmarks only"""

def run_all(config: CliConfig) -> CombinedReport:
    """Execute both tests and benchmarks"""
```

**CLI Arguments**:
- Positional: `{unit,integration,bench}` - Execution mode
- `--pattern PATTERN` - File/test pattern filter
- `--tags TAGS` - Tag filter (comma-separated)
- `--verbose` - Detailed output
- `--fail-fast` - Stop on first failure
- `--format {console,json,markdown}` - Output format

### 2. Lazy Loader (lazy_loader.py)

**Purpose**: On-demand Python module loading

**Responsibilities**:
- Load Python modules only when needed for execution
- Use importlib.util for dynamic loading
- Return TestSuite classes or BenchmarkGroup instances
- Handle import errors gracefully

**Key Functions**:
```python
def lazy_load_test_suite(file_path: Path) -> List[Type[TestSuite]]:
    """Load a single test file and return TestSuite classes.

    Called by Rust during test execution phase.
    Uses importlib.util to load module without adding to sys.modules.
    """

def lazy_load_benchmark(file_path: Path) -> List[BenchmarkGroup]:
    """Load a single benchmark file and return registered groups.

    Called by Rust during benchmark execution phase.
    Module execution triggers @decorator and register_group().
    """
```

**Performance**:
- ~10-50ms per module (Python import overhead)
- Only called for files that pass filters
- Deferred until execution phase (not during discovery)

## Rust Layer (Core Engine)

### 3. Discovery (discovery.rs) - NEW

**Purpose**: Fast file discovery and metadata registry

**Responsibilities**:
- Walk file system using walkdir crate (~2ms for 100 files)
- Create FileInfo structs with paths and module names
- Store test/benchmark metadata in registries
- Filter by patterns, tags, and types
- Provide discovery statistics

**Key Types**:
```rust
pub struct DiscoveryConfig {
    pub root_path: PathBuf,
    pub patterns: Vec<String>,    // e.g., ["test_*.py", "bench_*.py"]
    pub exclusions: Vec<String>,  // e.g., ["__pycache__"]
}

pub enum FileType {
    Test,
    Benchmark,
}

pub struct FileInfo {
    pub path: PathBuf,
    pub module_name: String,
    pub file_type: FileType,
}

pub struct TestRegistry {
    files: Vec<FileInfo>,
}

pub struct BenchmarkRegistry {
    files: Vec<FileInfo>,
}

pub struct DiscoveryStats {
    pub files_found: usize,
    pub filtered_count: usize,
    pub discovery_time_ms: u64,
}
```

**Key Functions**:
```rust
pub fn walk_files(config: &DiscoveryConfig) -> Result<Vec<FileInfo>>;
pub fn filter_files(files: Vec<FileInfo>, filters: &Filters) -> Vec<FileInfo>;
pub fn lazy_load_module(py: Python, file_info: &FileInfo) -> PyResult<PyModule>;
```

### 4. Runner (runner.rs) - EXISTING

**Purpose**: Test/benchmark execution orchestration

**Responsibilities**:
- Execute tests and benchmarks
- Call into Python for test logic via lazy-loaded modules
- Collect execution metrics (timing, memory, etc.)
- Handle timeouts and errors
- Manage test lifecycle (setup → execute → teardown)

**Key Types** (existing):
```rust
pub struct TestRunner {
    // Implementation details
}

pub struct TestResult {
    pub status: TestStatus,
    pub duration: Duration,
    pub error: Option<String>,
}

pub struct TestMeta {
    pub name: String,
    pub tags: Vec<String>,
    pub timeout: Option<Duration>,
}
```

**Key Functions**:
```rust
pub fn execute_tests(files: Vec<FileInfo>) -> Vec<TestResult>;
pub fn run_benchmarks(files: Vec<FileInfo>) -> Vec<BenchmarkStats>;
```

### 5. Reporter (reporter.rs) - EXISTING

**Purpose**: Report generation and formatting

**Responsibilities**:
- Format results in multiple formats (console/JSON/markdown)
- Generate comparison tables for benchmarks
- Save reports to files
- Provide summary statistics

**Key Types** (existing):
```rust
pub struct Reporter {
    format: OutputFormat,
}

pub enum OutputFormat {
    Console,
    Json,
    Markdown,
}

pub struct TestReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub errors: usize,
    pub duration: Duration,
}

pub struct BenchmarkReport {
    pub groups: Vec<BenchmarkGroupResult>,
    pub comparisons: Vec<Comparison>,
}
```

**Key Functions**:
```rust
pub fn generate_report(results: &[TestResult]) -> String;
pub fn format_comparison_table(groups: &[BenchmarkGroupResult]) -> String;
```

## PyO3 Bridge Layer

### 6. PyO3 Bindings (test.rs)

**Purpose**: Expose Rust types to Python

**Responsibilities**:
- Wrap Rust types with #[pyclass] macro
- Implement Python methods with #[pymethods]
- Handle Python ↔ Rust type conversions
- Provide thread-safe access with Arc<Mutex<>>

**Key Classes** (to be added):
```rust
#[pyclass(name = "TestRegistry")]
pub struct PyTestRegistry {
    inner: Arc<Mutex<TestRegistry>>,
}

#[pyclass(name = "BenchmarkRegistry")]
pub struct PyBenchmarkRegistry {
    inner: Arc<Mutex<BenchmarkRegistry>>,
}

#[pyclass(name = "FileInfo")]
pub struct PyFileInfo {
    inner: FileInfo,
}

#[pyclass(name = "DiscoveryStats")]
pub struct PyDiscoveryStats {
    inner: DiscoveryStats,
}

#[pymethods]
impl PyTestRegistry {
    #[new]
    fn new() -> Self { ... }

    fn discover(&self, path: String) -> PyResult<PyDiscoveryStats> { ... }

    fn filter_by_pattern(&self, pattern: String) -> PyResult<Vec<PyFileInfo>> { ... }

    fn filter_by_tags(&self, tags: Vec<String>) -> PyResult<Vec<PyFileInfo>> { ... }
}
```

**Thread Safety**:
- All registries wrapped in Arc<Mutex<>> for thread-safe access
- Python GIL released during Rust operations where possible

## Integration Points

### With Existing Systems

```mermaid
graph LR
    subgraph "Existing"
        PYTEST[pytest tests/]
        BENCH[Manual benchmarks]
        MANUAL[Manual test execution]
    end

    subgraph "New: dbtest"
        DB[dbtest CLI]
    end

    subgraph "Coexistence"
        BOTH[Both frameworks<br/>can be used]
    end

    PYTEST -.-> BOTH
    BENCH -.-> BOTH
    MANUAL -.-> BOTH
    DB --> BOTH

    style DB fill:#90EE90
    style BOTH fill:#FFE4B5
```

**Key Points**:
- `dbtest` is **standalone** - does not interfere with pytest
- Existing test files work with both pytest and dbtest
- BenchmarkGroup pattern is reused from existing framework
- Can run both pytest and dbtest in same project

### With justfile

```mermaid
graph TB
    JUST[justfile commands]

    JUST --> DBTEST[just dbtest]
    JUST --> UNIT[just dbtest-unit]
    JUST --> BENCH[just dbtest-bench]

    DBTEST --> CLI[uv run dbtest]
    UNIT --> CLI2[uv run dbtest unit]
    BENCH --> CLI3[uv run dbtest bench]

    style JUST fill:#e1f5ff
    style CLI fill:#90EE90
    style CLI2 fill:#90EE90
    style CLI3 fill:#90EE90
```

**justfile Integration**:
```justfile
# Run all tests and benchmarks
dbtest:
    uv run dbtest

# Run unit tests only
dbtest-unit:
    uv run dbtest unit

# Run integration tests only
dbtest-integration:
    uv run dbtest integration

# Run benchmarks only
dbtest-bench:
    uv run dbtest bench

# Run with specific pattern
dbtest-pattern PATTERN:
    uv run dbtest --pattern "{{PATTERN}}"
```

### With Existing Test Framework

**Existing Framework** (`data_bridge.test`):
- TestSuite base class
- @test decorator
- BenchmarkGroup class
- @benchmark decorator
- Assertions (assert_eq, assert_raises, etc.)

**dbtest Integration**:
- **Discovers**: Finds test_*.py and bench_*.py files
- **Loads**: Uses lazy_loader.py to import modules
- **Executes**: Calls TestSuite.run_tests() via Rust runner
- **Reports**: Formats results with Rust reporter

**No Changes Required**: Existing test files work as-is

## Component Communication

### Request Flow (Test Execution)

```
User → CLI → PyO3 → Rust Walker → Rust Registry → Python Lazy Loader → Rust Runner → Rust Reporter → CLI → User
```

### Data Flow

```
CLI Args → DiscoveryConfig → Vec<FileInfo> → Filtered FileInfo → Lazy-loaded Modules → Test Results → Formatted Report
```

### Error Propagation

```
Rust Error → PyO3 PyErr → Python Exception → CLI Exit Code
```

## Performance Characteristics

| Component | Operation | Performance | Implementation |
|-----------|-----------|-------------|----------------|
| CLI | Startup | ~200-300ms | Python argparse |
| Discovery | File walk | ~2-3ms (100 files) | Rust walkdir |
| Registry | Filtering | <10ms (1000 files) | Rust string matching |
| Lazy Loader | Module import | ~10-50ms per file | Python importlib |
| Runner | Execution | Variable | User test code |
| Reporter | Formatting | ~10-50ms | Rust templating |

**Total Overhead** (excluding test execution): ~250-400ms

## See Also

- [Architecture](./architecture.md) - High-level architecture
- [State Machines](./state-machines.md) - Lifecycle states
- [Data Flows](./data-flows.md) - Sequence diagrams
- [Implementation](./implementation.md) - File structure and patterns
