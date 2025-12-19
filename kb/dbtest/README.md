# dbtest System Architecture

## Overview

The `dbtest` CLI tool provides unified test and benchmark discovery and execution for the data-bridge project, powered by a Rust engine with Python bindings.

## User Requirements

- **CLI**: Rust runner engine, Python CLI wrapper
- **Discovery**: Rust-powered (walkdir + lazy loading)
- **Benchmark**: Keep current BenchmarkGroup pattern
- **Standalone**: No pytest integration

## Optimized Architecture

```
Rust File Walker (walkdir)
  ↓ finds test_*.py, bench_*.py (~2ms for 100 files)
  ↓ stores paths in
Rust Registry (discovery.rs)
  ↓ applies filters (tags, patterns)
  ↓ lazy-loads modules via
Python importlib (on-demand)
  ↓ only for tests that will execute
  ↓ executes via
Rust Runner (runner.rs)
  ↓ collects results
  ↓ generates
Rust Reporter (reporter.rs)
```

**Key Innovation**: Rust-first discovery with lazy loading
- **Rust**: File walking (walkdir crate), registry, filtering, execution, reporting
- **Python**: Lazy module loading (only load what we execute)
- **Performance**: <3ms discovery, module loading only for executed tests

## Documentation Structure

This architecture documentation is split into focused files for easier navigation:

### 1. [architecture.md](./architecture.md)
High-level system diagrams showing:
- Overall architecture flow
- Detailed component architecture
- Layer responsibilities (Python, PyO3, Rust)

### 2. [state-machines.md](./state-machines.md)
State machine definitions for:
- Discovery lifecycle
- Test execution lifecycle
- Benchmark execution lifecycle
- CLI execution lifecycle
- State data structures (Rust types)

### 3. [data-flows.md](./data-flows.md)
Sequence diagrams showing:
- Test discovery and execution flow
- Benchmark discovery and execution flow

### 4. [components.md](./components.md)
Component responsibilities and integration:
- Python layer (CLI, lazy loading)
- Rust layer (discovery, runner, reporter)
- PyO3 bridge
- Integration with existing systems

### 5. [implementation.md](./implementation.md)
Implementation details:
- File structure and organization
- Execution flow diagrams
- Key design patterns
- Performance characteristics
- Future extensions

## Quick Start

### Commands

```bash
dbtest              # Run all tests and benchmarks
dbtest unit         # Unit tests only
dbtest integration  # Integration tests
dbtest bench        # Benchmarks only
```

### Options

```bash
--pattern PATTERN   # Filter by file/test pattern
--tags TAGS         # Filter by tags
--verbose           # Detailed output
--fail-fast         # Stop on first failure
--format FORMAT     # Output format (console/json/markdown)
```

## Success Criteria

- ✅ `dbtest` command available after install
- ✅ Auto-discovers test_*.py and bench_*.py files
- ✅ Filters by pattern, tags, test type
- ✅ Runs tests and benchmarks separately or together
- ✅ Generates reports (console/JSON/markdown)
- ✅ <200ms discovery for 100 files
- ✅ All existing tests still pass

## Implementation Plan

Detailed implementation plan: `/Users/chris.cheng/.claude/plans/enumerated-foraging-lighthouse.md`

**Phases**:
- **Phase 0**: Documentation reorganization (CURRENT)
- **Phase 1**: Rust foundation (discovery.rs)
- **Phase 2**: PyO3 bindings
- **Phase 3**: Python lazy loading
- **Phase 4**: Python CLI
- **Phase 5**: Console script
- **Phase 6**: Documentation & templates

## References

- **Rust Crate**: `crates/data-bridge-test/`
- **Python Module**: `python/data_bridge/test/`
- **PyO3 Bindings**: `crates/data-bridge/src/test.rs`
- **Existing Benchmark Discovery**: `python/data_bridge/test/benchmark.py:449-563`
