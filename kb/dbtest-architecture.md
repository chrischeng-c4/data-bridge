# dbtest System Architecture

## Overview

The `dbtest` CLI tool provides unified test and benchmark discovery and execution for the data-bridge project, powered by a Rust engine with Python bindings.

## High-Level Architecture

```mermaid
graph TB
    User[User: dbtest command]
    CLI[Python CLI Wrapper<br/>data_bridge.test.cli]
    Discovery[Python Discovery Layer<br/>discovery.py]
    Registry[Rust Registry<br/>discovery.rs]
    Runner[Rust Runner<br/>runner.rs]
    Reporter[Rust Reporter<br/>reporter.rs]

    User --> CLI
    CLI --> Discovery
    Discovery --> Registry
    Registry --> Runner
    Runner --> Reporter
    Reporter --> User

    style CLI fill:#e1f5ff
    style Discovery fill:#e1f5ff
    style Registry fill:#ffe1e1
    style Runner fill:#ffe1e1
    style Reporter fill:#ffe1e1
```

## Detailed Component Architecture

```mermaid
graph TB
    subgraph "User Interface Layer"
        CMD[Console Script: dbtest]
        ARGS[CLI Arguments<br/>--pattern, --tags, --format]
    end

    subgraph "Python Layer (Thin Wrapper)"
        CLI[cli.py<br/>Argument Parser]
        DISC[discovery.py<br/>File Discovery]
        GLOB[glob test_*.py<br/>glob bench_*.py]
        IMPORT[importlib<br/>Dynamic Import]
    end

    subgraph "PyO3 Bridge"
        PYO3[PyO3 Bindings<br/>crates/data-bridge/src/test.rs]
        PYTEST[PyTestRegistry]
        PYBENCH[PyBenchmarkRegistry]
        PYMETA[PyTestSuiteInfo<br/>PyBenchmarkGroupInfo]
    end

    subgraph "Rust Engine (Core Logic)"
        REG[discovery.rs<br/>Registries]
        TREG[TestRegistry]
        BREG[BenchmarkRegistry]
        FILTER[Filtering & Collection]

        RUN[runner.rs<br/>Test Runner]
        EXEC[Test Execution]
        METRICS[Metrics Collection]

        REP[reporter.rs<br/>Report Generator]
        CONSOLE[Console Format]
        JSON[JSON Format]
        MD[Markdown Format]
    end

    subgraph "Test Files"
        TFILES[test_*.py<br/>TestSuite classes]
        BFILES[bench_*.py<br/>BenchmarkGroup]
    end

    CMD --> CLI
    ARGS --> CLI
    CLI --> DISC
    DISC --> GLOB
    GLOB --> TFILES
    GLOB --> BFILES
    DISC --> IMPORT
    IMPORT --> TFILES
    IMPORT --> BFILES

    DISC --> PYO3
    PYO3 --> PYTEST
    PYO3 --> PYBENCH
    PYO3 --> PYMETA

    PYTEST --> TREG
    PYBENCH --> BREG
    PYMETA --> TREG
    PYMETA --> BREG

    TREG --> REG
    BREG --> REG
    REG --> FILTER
    FILTER --> RUN

    RUN --> EXEC
    EXEC --> METRICS
    METRICS --> REP

    REP --> CONSOLE
    REP --> JSON
    REP --> MD

    CONSOLE --> CMD
    JSON --> CMD
    MD --> CMD

    style CLI fill:#b3e5fc
    style DISC fill:#b3e5fc
    style GLOB fill:#b3e5fc
    style IMPORT fill:#b3e5fc

    style PYO3 fill:#fff9c4
    style PYTEST fill:#fff9c4
    style PYBENCH fill:#fff9c4
    style PYMETA fill:#fff9c4

    style REG fill:#ffccbc
    style TREG fill:#ffccbc
    style BREG fill:#ffccbc
    style FILTER fill:#ffccbc
    style RUN fill:#ffccbc
    style EXEC fill:#ffccbc
    style METRICS fill:#ffccbc
    style REP fill:#ffccbc
    style CONSOLE fill:#ffccbc
    style JSON fill:#ffccbc
    style MD fill:#ffccbc
```

## Data Flow: Test Discovery & Execution

```mermaid
sequenceDiagram
    participant User
    participant CLI as Python CLI
    participant Disc as Python Discovery
    participant PyO3 as PyO3 Bridge
    participant Reg as Rust Registry
    participant Run as Rust Runner
    participant Rep as Rust Reporter

    User->>CLI: dbtest unit
    CLI->>Disc: discover_test_suites("tests/")

    Disc->>Disc: glob("test_*.py")
    Disc->>Disc: importlib.load(files)
    Disc->>Disc: inspect.getmembers()

    Disc->>PyO3: register TestSuite classes
    PyO3->>Reg: TestRegistry.register_suite()

    Reg->>Reg: apply filters (tags, pattern)
    Reg->>Run: execute_tests()

    Run->>Run: foreach test: execute()
    Run->>Run: collect metrics
    Run->>Rep: generate_report()

    Rep->>Rep: format (console/json/md)
    Rep->>CLI: return report
    CLI->>User: display results
```

## Data Flow: Benchmark Discovery & Execution

```mermaid
sequenceDiagram
    participant User
    participant CLI as Python CLI
    participant Disc as Python Discovery
    participant BM as BenchmarkGroup
    participant PyO3 as PyO3 Bridge
    participant Reg as Rust Registry
    participant Run as Rust Runner
    participant Rep as Rust Reporter

    User->>CLI: dbtest bench
    CLI->>Disc: discover_benchmarks("tests/")

    Disc->>Disc: glob("bench_*.py")
    Disc->>Disc: importlib.load(files)

    Note over Disc,BM: Module execution triggers<br/>@decorator and register_group()

    Disc->>BM: BenchmarkGroup registered
    BM->>PyO3: register_group()
    PyO3->>Reg: BenchmarkRegistry.register_group()

    Reg->>Reg: apply filters
    Reg->>Run: run_benchmarks()

    Run->>Run: foreach benchmark: calibrate
    Run->>Run: execute rounds + warmup
    Run->>Run: calculate statistics
    Run->>Rep: generate_report()

    Rep->>Rep: format comparison table
    Rep->>CLI: return report
    CLI->>User: display results
```

## File Structure

```mermaid
graph TB
    subgraph "Rust Engine"
        DBT[crates/data-bridge-test/]
        DBTSRC[src/]
        DISCOVERY[discovery.rs<br/>NEW]
        RUNNER[runner.rs<br/>MODIFY]
        BENCHMARK[benchmark.rs<br/>EXISTING]
        REPORTER[reporter.rs<br/>EXISTING]

        DBT --> DBTSRC
        DBTSRC --> DISCOVERY
        DBTSRC --> RUNNER
        DBTSRC --> BENCHMARK
        DBTSRC --> REPORTER
    end

    subgraph "PyO3 Bindings"
        DB[crates/data-bridge/]
        DBSRC[src/]
        TESTRS[test.rs<br/>MODIFY]

        DB --> DBSRC
        DBSRC --> TESTRS
    end

    subgraph "Python Layer"
        PYDB[python/data_bridge/test/]
        CLIPY[cli.py<br/>NEW]
        DISCPY[discovery.py<br/>NEW]
        MAINPY[__main__.py<br/>NEW]
        SUITEPY[suite.py<br/>EXISTING]
        BMPY[benchmark.py<br/>EXISTING]

        PYDB --> CLIPY
        PYDB --> DISCPY
        PYDB --> MAINPY
        PYDB --> SUITEPY
        PYDB --> BMPY
    end

    subgraph "Entry Points"
        CONSOLE[Console Script<br/>dbtest]
        MODULE[Python Module<br/>python -m data_bridge.test]
    end

    CONSOLE --> CLIPY
    MODULE --> MAINPY
    MAINPY --> CLIPY

    CLIPY --> DISCPY
    DISCPY --> SUITEPY
    DISCPY --> BMPY

    TESTRS -.PyO3.-> CLIPY
    TESTRS -.PyO3.-> DISCPY

    style DISCOVERY fill:#90EE90
    style CLIPY fill:#90EE90
    style DISCPY fill:#90EE90
    style MAINPY fill:#90EE90
    style TESTRS fill:#FFE4B5
    style RUNNER fill:#FFE4B5
```

## Component Responsibilities

### Python Layer (Thin Wrapper)

#### 1. CLI (cli.py)
- **Purpose**: Command-line interface and argument parsing
- **Responsibilities**:
  - Parse arguments (--pattern, --tags, --format)
  - Route to appropriate runner (unit/integration/bench)
  - Display results to user
- **Key Functions**:
  - `main()`: Entry point
  - `run_tests_only()`: Execute tests
  - `run_benchmarks_only()`: Execute benchmarks
  - `run_all()`: Execute both

#### 2. Discovery (discovery.py)
- **Purpose**: File system discovery and module loading
- **Responsibilities**:
  - Glob pattern matching (test_*.py, bench_*.py)
  - Dynamic module import via importlib
  - Class/function introspection via inspect
- **Key Functions**:
  - `discover_test_suites()`: Find TestSuite classes
  - `discover_benchmark_files()`: Wrapper for existing discover_benchmarks()

### Rust Layer (Core Engine)

#### 3. Discovery (discovery.rs) - NEW
- **Purpose**: Registry and metadata storage
- **Responsibilities**:
  - Store test/benchmark metadata
  - Filter by tags, patterns, types
  - Provide statistics
- **Key Types**:
  - `TestRegistry`: Test suite registry
  - `BenchmarkRegistry`: Benchmark registry
  - `TestSuiteInfo`: Test suite metadata
  - `BenchmarkGroupInfo`: Benchmark metadata

#### 4. Runner (runner.rs) - EXISTING
- **Purpose**: Test execution orchestration
- **Responsibilities**:
  - Execute tests/benchmarks
  - Collect metrics
  - Handle timeouts and errors
- **Key Types**:
  - `TestRunner`: Main runner
  - `TestResult`: Execution results
  - `TestMeta`: Test metadata

#### 5. Reporter (reporter.rs) - EXISTING
- **Purpose**: Report generation and formatting
- **Responsibilities**:
  - Format results (console/JSON/markdown)
  - Generate comparison tables
  - Save reports to files
- **Key Types**:
  - `Reporter`: Report generator
  - `TestReport`: Aggregated results
  - `BenchmarkReport`: Benchmark results

### PyO3 Bridge

#### 6. PyO3 Bindings (test.rs)
- **Purpose**: Expose Rust types to Python
- **Responsibilities**:
  - Wrap Rust types with #[pyclass]
  - Implement Python methods with #[pymethods]
  - Handle Python ↔ Rust conversions
- **Key Classes**:
  - `PyTestRegistry`
  - `PyBenchmarkRegistry`
  - `PyTestSuiteInfo`
  - `PyBenchmarkGroupInfo`

## Execution Flow

### Unit Test Execution

```mermaid
flowchart TD
    START([dbtest unit]) --> PARSE[Parse CLI Args]
    PARSE --> GLOB[Glob test_*.py]
    GLOB --> IMPORT[Import Modules]
    IMPORT --> INSPECT[Inspect for TestSuite]
    INSPECT --> REGISTER[Register in Rust Registry]
    REGISTER --> FILTER[Apply Filters]
    FILTER --> SETUP[Run setup_suite]
    SETUP --> EXEC[Execute Tests]
    EXEC --> TEARDOWN[Run teardown_suite]
    TEARDOWN --> COLLECT[Collect Results]
    COLLECT --> FORMAT[Format Report]
    FORMAT --> DISPLAY[Display to User]
    DISPLAY --> END([Exit])

    style START fill:#90EE90
    style END fill:#90EE90
    style REGISTER fill:#FFE4B5
    style FILTER fill:#FFE4B5
    style EXEC fill:#FFE4B5
    style COLLECT fill:#FFE4B5
    style FORMAT fill:#FFE4B5
```

### Benchmark Execution

```mermaid
flowchart TD
    START([dbtest bench]) --> PARSE[Parse CLI Args]
    PARSE --> GLOB[Glob bench_*.py]
    GLOB --> IMPORT[Import Modules]
    IMPORT --> AUTODISCOVER[Auto-register via Decorators]
    AUTODISCOVER --> REGISTER[Register in Rust Registry]
    REGISTER --> FILTER[Apply Filters]
    FILTER --> CALIBRATE[Auto-calibrate Iterations]
    CALIBRATE --> WARMUP[Run Warmup Rounds]
    WARMUP --> EXEC[Execute Benchmark Rounds]
    EXEC --> STATS[Calculate Statistics]
    STATS --> COMPARE[Generate Comparisons]
    COMPARE --> FORMAT[Format Report]
    FORMAT --> SAVE[Save Report Files]
    SAVE --> DISPLAY[Display to User]
    DISPLAY --> END([Exit])

    style START fill:#90EE90
    style END fill:#90EE90
    style REGISTER fill:#FFE4B5
    style FILTER fill:#FFE4B5
    style CALIBRATE fill:#FFE4B5
    style EXEC fill:#FFE4B5
    style STATS fill:#FFE4B5
    style COMPARE fill:#FFE4B5
    style FORMAT fill:#FFE4B5
```

## State Machines

### Discovery State Machine

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Scanning: Start discovery
    Scanning --> Loading: Files found
    Loading --> Registering: Modules loaded
    Registering --> Filtering: Registered in Rust
    Filtering --> Ready: Filters applied
    Scanning --> Empty: No files found
    Loading --> Failed: Import error
    Empty --> [*]
    Failed --> [*]
    Ready --> [*]

    note right of Scanning
        Python: glob patterns
        find test_*.py, bench_*.py
    end note

    note right of Loading
        Python: importlib
        Dynamic module import
    end note

    note right of Registering
        Rust: TestRegistry
        Store metadata
    end note

    note right of Filtering
        Rust: Apply filters
        tags, patterns, types
    end note
```

### Test Execution State Machine

```mermaid
stateDiagram-v2
    [*] --> Discovered
    Discovered --> Filtered: Apply filters
    Filtered --> Queued: Selected for run
    Queued --> SettingUp: Start test suite
    SettingUp --> Ready: Setup complete
    Ready --> Running: Execute test
    Running --> TearingDown: Test complete
    TearingDown --> Completed: Teardown done

    Filtered --> Skipped: Filtered out
    SettingUp --> Failed: Setup error
    Running --> Passed: Assertions pass
    Running --> Failed: Assertions fail
    Running --> Error: Exception/timeout
    TearingDown --> Error: Teardown error

    Skipped --> [*]
    Passed --> [*]
    Failed --> [*]
    Error --> [*]
    Completed --> [*]

    note right of SettingUp
        Run setup_suite()
        Initialize fixtures
    end note

    note right of Running
        Execute test function
        Collect metrics
        Check assertions
    end note

    note right of TearingDown
        Run teardown_suite()
        Cleanup resources
    end note
```

**States:**
- **Discovered**: Test found during discovery
- **Filtered**: After tag/pattern filtering
- **Queued**: Scheduled for execution
- **SettingUp**: Running setup_suite()
- **Ready**: Setup complete, ready to run
- **Running**: Test executing
- **TearingDown**: Running teardown_suite()
- **Completed**: All phases done
- **Passed/Failed/Error/Skipped**: Final states (TestStatus)

### Benchmark Execution State Machine

```mermaid
stateDiagram-v2
    [*] --> Discovered
    Discovered --> Filtered: Apply filters
    Filtered --> Queued: Selected for run
    Queued --> Calibrating: Start benchmark
    Calibrating --> WarmingUp: Iterations determined
    WarmingUp --> Running: Warmup complete
    Running --> Analyzing: Rounds complete
    Analyzing --> Completed: Stats calculated

    Filtered --> Skipped: Filtered out
    Calibrating --> Failed: Calibration error
    WarmingUp --> Failed: Warmup error
    Running --> Failed: Execution error

    Skipped --> [*]
    Failed --> [*]
    Completed --> [*]

    note right of Calibrating
        Auto-calibrate iterations
        Target: 100ms per round
    end note

    note right of WarmingUp
        Run 3 warmup rounds
        Prime caches
    end note

    note right of Running
        Execute 5 rounds
        Collect timing data
    end note

    note right of Analyzing
        Calculate statistics:
        - Mean, median, stddev
        - Percentiles (P25, P50, P75, P95, P99)
        - Outlier detection
        - Confidence intervals
    end note
```

**States:**
- **Discovered**: Benchmark found during discovery
- **Filtered**: After pattern filtering
- **Queued**: Scheduled for execution
- **Calibrating**: Determining iteration count
- **WarmingUp**: Running warmup rounds
- **Running**: Executing timed rounds
- **Analyzing**: Computing statistics
- **Completed**: All phases done, stats ready
- **Skipped/Failed**: Terminal states

### CLI Execution State Machine

```mermaid
stateDiagram-v2
    [*] --> Parsing
    Parsing --> Discovering: Args parsed
    Discovering --> Registering: Files discovered
    Registering --> Filtering: Metadata in Rust
    Filtering --> Executing: Filtered set ready
    Executing --> Reporting: Execution complete
    Reporting --> [*]: Report displayed

    Parsing --> Error: Invalid args
    Discovering --> Error: Path not found
    Registering --> Error: Import failed
    Executing --> Error: Execution failed

    Error --> [*]: Exit code 1

    note right of Parsing
        argparse
        Validate options
    end note

    note right of Discovering
        discover_test_suites()
        or discover_benchmarks()
    end note

    note right of Filtering
        Rust: TestRegistry
        Apply filters
    end note

    note right of Executing
        run_tests() or
        run_benchmarks()
    end note

    note right of Reporting
        Format: console/json/md
        Display results
    end note
```

### State Transitions & Triggers

#### Test Lifecycle

| From State | Trigger | To State | Action |
|------------|---------|----------|--------|
| Discovered | Filter match | Filtered | Add to filtered set |
| Discovered | Filter mismatch | Skipped | Mark as skipped |
| Filtered | Execution start | Queued | Add to execution queue |
| Queued | Runner picks up | SettingUp | Call setup_suite() |
| SettingUp | Setup succeeds | Ready | Mark ready |
| SettingUp | Setup fails | Failed | Record error |
| Ready | Test starts | Running | Execute test function |
| Running | All assertions pass | Passed | Record success |
| Running | Assertion fails | Failed | Record failure |
| Running | Exception raised | Error | Record error |
| Running | Timeout | Error | Record timeout |
| Passed/Failed/Error | Cleanup needed | TearingDown | Call teardown_suite() |
| TearingDown | Teardown succeeds | Completed | Finalize |
| TearingDown | Teardown fails | Error | Record teardown error |

#### Benchmark Lifecycle

| From State | Trigger | To State | Action |
|------------|---------|----------|--------|
| Discovered | Filter match | Filtered | Add to filtered set |
| Discovered | Filter mismatch | Skipped | Mark as skipped |
| Filtered | Execution start | Queued | Add to execution queue |
| Queued | Runner picks up | Calibrating | Determine iterations |
| Calibrating | Iterations found | WarmingUp | Run warmup rounds |
| Calibrating | Calibration fails | Failed | Record error |
| WarmingUp | Warmup done | Running | Run timed rounds |
| WarmingUp | Warmup fails | Failed | Record error |
| Running | All rounds done | Analyzing | Calculate statistics |
| Running | Round fails | Failed | Record error |
| Analyzing | Stats computed | Completed | Finalize results |

### State Data Structures (Rust)

```rust
/// Overall execution state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionState {
    Idle,
    Discovering,
    Filtering,
    Executing,
    Reporting,
    Completed,
    Failed(String),
}

/// Test lifecycle state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestLifecycleState {
    Discovered,
    Filtered,
    Queued,
    SettingUp,
    Ready,
    Running,
    TearingDown,
    Completed(TestStatus),  // Passed/Failed/Error/Skipped
}

/// Benchmark lifecycle state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BenchmarkLifecycleState {
    Discovered,
    Filtered,
    Queued,
    Calibrating,
    WarmingUp,
    Running { current_round: usize, total_rounds: usize },
    Analyzing,
    Completed,
    Failed(String),
}

/// Test status (final outcome) - EXISTING
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}
```

### State Persistence

**In-Memory Only:**
- States are tracked during execution
- No persistence to disk
- State machine resets on each CLI invocation

**State Tracking:**
```rust
pub struct TestExecution {
    meta: TestMeta,
    state: TestLifecycleState,
    started_at: Option<Instant>,
    completed_at: Option<Instant>,
    result: Option<TestResult>,
}

pub struct BenchmarkExecution {
    meta: BenchmarkMeta,
    state: BenchmarkLifecycleState,
    calibration_iterations: Option<usize>,
    warmup_results: Vec<Duration>,
    round_results: Vec<Duration>,
    stats: Option<BenchmarkStats>,
}
```

## Key Design Patterns

### 1. Hybrid Discovery Pattern
```
Python (glob + importlib) → Rust (registry + filtering)
```
- Leverages Python's file system and dynamic loading
- Leverages Rust's performance for filtering and execution

### 2. Registry Pattern
```
TestRegistry/BenchmarkRegistry
  ├─ register(info)
  ├─ get_all()
  ├─ filter_by_pattern()
  └─ filter_by_tags()
```
- Centralized metadata storage
- Efficient filtering before execution

### 3. PyO3 Wrapper Pattern
```rust
#[pyclass(name = "TestRegistry")]
pub struct PyTestRegistry {
    inner: Arc<Mutex<TestRegistry>>,
}

#[pymethods]
impl PyTestRegistry {
    fn register_suite(&self, suite: PyTestSuiteInfo) {
        // Delegate to Rust
    }
}
```
- Thin Python wrapper over Rust types
- Thread-safe with Arc<Mutex<>>

## Performance Characteristics

| Operation | Target | Implementation |
|-----------|--------|----------------|
| Discovery | <200ms for 100 files | Python glob + importlib |
| Filtering | <10ms for 1000 tests | Rust registry operations |
| Execution | Variable | Rust runner with Python callbacks |
| Reporting | <50ms | Rust reporter with formatting |
| CLI Startup | <500ms cold | Python argparse |

## Integration Points

### With Existing Systems

```mermaid
graph LR
    subgraph "Existing"
        PYTEST[pytest tests/]
        BENCH[Benchmark Runner]
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

- `dbtest` is **standalone** - does not interfere with pytest
- Existing benchmark discovery pattern is **reused**
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

## Future Extensions

### Phase 2 Features (Not in Current Plan)

```mermaid
graph TB
    CURRENT[Current: dbtest CLI]

    CURRENT --> COV[Coverage Integration]
    CURRENT --> PAR[Parallel Execution]
    CURRENT --> CI[CI/CD Integration]
    CURRENT --> WATCH[Watch Mode]
    CURRENT --> HTML[HTML Dashboard]

    style CURRENT fill:#90EE90
    style COV fill:#E0E0E0
    style PAR fill:#E0E0E0
    style CI fill:#E0E0E0
    style WATCH fill:#E0E0E0
    style HTML fill:#E0E0E0
```

These are potential future enhancements, not included in the current implementation plan.

## References

- **Rust Crate**: `crates/data-bridge-test/`
- **Python Module**: `python/data_bridge/test/`
- **PyO3 Bindings**: `crates/data-bridge/src/test.rs`
- **Existing Benchmark Discovery**: `python/data_bridge/test/benchmark.py:449-563`
- **Implementation Plan**: `/Users/chris.cheng/.claude/plans/enumerated-foraging-lighthouse.md`
