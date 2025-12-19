# Data Flows

> Part of [dbtest Architecture Documentation](./README.md)

This document shows the sequence of operations and data flow through the dbtest system for different execution paths.

## Test Discovery & Execution Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI as Python CLI
    participant PyO3 as PyO3 Bridge
    participant Walk as Rust File Walker
    participant Reg as Rust Registry
    participant Lazy as Python Lazy Loader
    participant Run as Rust Runner
    participant Rep as Rust Reporter

    User->>CLI: dbtest unit
    CLI->>PyO3: discover_tests("tests/")

    PyO3->>Walk: walk_files(config)
    Walk->>Walk: walkdir crate (~2ms)
    Walk->>Walk: filter by pattern (test_*.py)
    Walk-->>PyO3: Vec<FileInfo>

    PyO3->>Reg: register file paths
    Reg->>Reg: apply tag filters
    Reg-->>PyO3: filtered FileInfo list

    PyO3->>Run: execute_tests()

    loop For each test file
        Run->>Lazy: lazy_load_test_suite(file_path)
        Lazy->>Lazy: importlib.util.spec_from_file
        Lazy->>Lazy: module_from_spec
        Lazy-->>Run: List[TestSuite classes]

        Run->>Run: setup_suite()
        Run->>Run: execute test functions
        Run->>Run: teardown_suite()
        Run->>Run: collect metrics
    end

    Run->>Rep: generate_report(results)
    Rep->>Rep: format (console/json/md)
    Rep-->>CLI: formatted report
    CLI-->>User: display results
```

**Key Points**:
1. **Fast Discovery**: Rust walkdir finds files in ~2ms (not Python glob)
2. **Lazy Loading**: Python modules only loaded when needed for execution
3. **Rust Execution**: Main runner logic in Rust, calls into Python for tests
4. **Single Pass**: No separate discovery and execution phases

## Benchmark Discovery & Execution Flow

```mermaid
sequenceDiagram
    participant User
    participant CLI as Python CLI
    participant PyO3 as PyO3 Bridge
    participant Walk as Rust File Walker
    participant Reg as Rust Registry
    participant Lazy as Python Lazy Loader
    participant BG as BenchmarkGroup
    participant Run as Rust Runner
    participant Rep as Rust Reporter

    User->>CLI: dbtest bench
    CLI->>PyO3: discover_benchmarks("tests/")

    PyO3->>Walk: walk_files(config)
    Walk->>Walk: walkdir crate (~2ms)
    Walk->>Walk: filter by pattern (bench_*.py)
    Walk-->>PyO3: Vec<FileInfo>

    PyO3->>Reg: register file paths
    Reg->>Reg: apply pattern filters
    Reg-->>PyO3: filtered FileInfo list

    PyO3->>Run: run_benchmarks()

    loop For each bench file
        Run->>Lazy: lazy_load_benchmark(file_path)
        Lazy->>Lazy: importlib.util.spec_from_file
        Lazy->>Lazy: module_from_spec

        Note over Lazy,BG: Module execution triggers<br/>@decorator and register_group()

        Lazy->>BG: BenchmarkGroup auto-registered
        Lazy-->>Run: BenchmarkGroup instances

        Run->>Run: calibrate iterations
        Run->>Run: run warmup rounds (3x)
        Run->>Run: execute timed rounds (5x)
        Run->>Run: calculate statistics
    end

    Run->>Rep: generate_comparison_report(results)
    Rep->>Rep: format comparison table
    Rep-->>CLI: formatted report
    CLI-->>User: display results
```

**Key Points**:
1. **Same Discovery**: Uses same Rust walkdir approach
2. **Lazy Loading**: Benchmark files loaded on-demand
3. **Auto-Registration**: BenchmarkGroup registers during module import
4. **Statistics**: Mean, median, stddev, percentiles calculated in Rust

## Filtering Flow

```mermaid
sequenceDiagram
    participant CLI as Python CLI
    participant Walk as Rust File Walker
    participant Reg as Rust Registry

    CLI->>Walk: walk_files(pattern="test_*.py")
    Walk-->>Walk: Find all test_*.py files

    Walk->>Reg: register(FileInfo { path, module_name })

    CLI->>Reg: filter_by_pattern("*crud*")
    Reg-->>Reg: Keep only files matching *crud*

    CLI->>Reg: filter_by_tags(["unit"])
    Note over Reg: Tags require loading module<br/>to inspect decorators
    Reg-->>Reg: Filtered FileInfo list

    Reg-->>CLI: Ready for execution
```

**Filtering Strategy**:
- **File Pattern**: Applied during walkdir (fast, no I/O)
- **Name Pattern**: Applied on FileInfo list (fast, string match)
- **Tags**: Requires lazy loading module to inspect decorators (slower)

## Error Handling Flow

```mermaid
sequenceDiagram
    participant CLI as Python CLI
    participant Walk as Rust File Walker
    participant Lazy as Python Lazy Loader
    participant Run as Rust Runner

    CLI->>Walk: walk_files("tests/")

    alt File system error
        Walk-->>CLI: Error: Permission denied
        CLI-->>User: Exit code 1
    else Files found
        Walk-->>CLI: Vec<FileInfo>
    end

    CLI->>Lazy: lazy_load_test_suite(file_path)

    alt Import error
        Lazy-->>Run: Error: Module import failed
        Run-->>Run: Mark test as Error
    else Module loaded
        Lazy-->>Run: TestSuite classes
    end

    Run->>Run: execute_test()

    alt Test fails
        Run-->>Run: Mark as Failed
    else Test passes
        Run-->>Run: Mark as Passed
    end

    Run-->>CLI: TestResults (including errors)
    CLI-->>User: Display all results + error summary
```

**Error Handling Principles**:
- **Fail Fast for Discovery**: File system errors exit immediately
- **Collect Test Errors**: Import/execution errors are collected, not fatal
- **Final Report**: Shows all results including errors
- **Exit Code**: Non-zero if any tests failed or errored

## Performance Optimization Points

### Critical Path (Discovery to Execution)

```
1. CLI startup              ~200-300ms  (Python import overhead)
   ↓
2. walkdir file discovery   ~2-3ms      (Rust walkdir, 100 files)
   ↓
3. Filtering                ~0.1-1ms    (Rust string matching)
   ↓
4. Lazy module loading      ~10-50ms    (Python importlib, per file)
   ↓
5. Test execution          Variable     (User test code)
   ↓
6. Report generation       ~10-50ms     (Rust formatting)
```

**Bottlenecks**:
- **CLI Startup**: Python import overhead (~200-300ms) - unavoidable
- **Module Loading**: Lazy loading mitigates by only loading needed files
- **Test Execution**: Dominated by actual test logic

**Optimizations**:
- ✅ Use Rust walkdir (10-50x faster than Python glob)
- ✅ Lazy loading (don't load filtered-out files)
- ✅ Filtering in Rust (faster than Python)
- ❌ Not caching (complexity not worth <3ms savings)

## See Also

- [Architecture](./architecture.md) - System architecture
- [State Machines](./state-machines.md) - Lifecycle states
- [Components](./components.md) - Component responsibilities
