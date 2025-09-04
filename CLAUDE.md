# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Data Bridge is a pure Python, type-based ODM (Object Document Mapper) that supports both synchronous and asynchronous operations. The project aims to provide a powerful, Django-style interface with SQLAlchemy-like operator support for multiple backends.

### Target Backends (Phase 1)
- MongoDB (pymongo and motor)
- Firestore (sync and async)
- Redis (redis and aioredis)

### Core Design Principles
- **Django-style query interface**: `User.objects().find(User.id == id, User.age >= 123)`
- **SQLAlchemy-like operators**: Class variables support Python operators for query building
- **Type-first approach**: Heavy reliance on Python typing for safety and IDE support
- **Dual sync/async support**: All backends must work in both synchronous and asynchronous contexts

## Development Setup

### Python Version
This project uses Python 3.12+ (specified in `.python-version`)

### Install Dependencies
```bash
# Install all dependencies including dev
uv sync --all-extras

# Install pre-commit hooks
uv run pre-commit install
```

### Docker Services
```bash
# Start all backend services (MongoDB, Redis, Firestore emulator)
docker-compose up -d

# Stop services
docker-compose down

# View logs
docker-compose logs -f
```

### Project Structure
```
src/
└── data_bridge/
    ├── __init__.py       # Main package entry
    └── py.typed          # PEP 561 marker for typed package
```

## Architecture Guidelines

### Query System Design
- Implement a query builder that captures Python expressions using operator overloading
- Class attributes should return query expression objects when used in comparisons
- The `.objects()` method should return a manager instance for the model
- The `.find()` method should accept multiple query expressions combined with logical operators

### Backend Abstraction
- Create a common interface that all backends must implement
- Use abstract base classes to enforce consistency across backends
- Each backend should have both sync and async implementations
- Consider using a factory pattern for backend selection

### Type Safety
- All public APIs must have complete type annotations
- Use generics for model definitions to preserve type information through queries
- Leverage mypy or pyright for static type checking
- Include py.typed marker (already present) to indicate typed package

### Async/Sync Dual Support Pattern
- Consider implementing sync versions as wrappers around async when possible
- Or maintain separate implementations if performance is critical
- Use consistent naming: `find()` for sync, `find_async()` or `afind()` for async

## Testing Strategy

### Test Structure
```
tests/
├── unit/           # Unit tests for individual components
├── integration/    # Tests with real backend connections
└── fixtures/       # Shared test data and utilities
```

### Running Tests
```bash
# Run all tests
uv run pytest

# Run specific test file
uv run pytest tests/unit/test_query_builder.py

# Run with coverage
uv run pytest --cov=data_bridge --cov-report=term-missing

# Run tests in multiple Python environments
uv run tox
```

## Code Quality

### Quick Commands
```bash
# Run all quality checks (lint, typecheck, test)
uv run ruff check src/ tests/ && uv run mypy src/ && uv run pytest

# Auto-format code
uv run black src/ tests/ && uv run isort src/ tests/ && uv run ruff check --fix src/ tests/

# Run all linters
uv run ruff check src/ tests/ && uv run pylint src/data_bridge
```

### Type Checking
```bash
# Run mypy with strict mode
uv run mypy src/
```

### Linting
```bash
# Run ruff (fast Python linter)
uv run ruff check src/ tests/

# Run pylint (comprehensive linter)
uv run pylint src/data_bridge

# Auto-fix issues with ruff
uv run ruff check --fix src/ tests/
```

### Formatting
```bash
# Format with black
uv run black src/ tests/

# Sort imports with isort
uv run isort src/ tests/

# Format with ruff (alternative to black)
uv run ruff format src/ tests/
```

### Pre-commit Hooks
Pre-commit hooks run automatically on git commit:
- trailing-whitespace
- end-of-file-fixer
- check-yaml, check-toml
- black formatting
- isort import sorting
- ruff linting
- mypy type checking

## Key Implementation Considerations

### Model Definition Pattern
Models should inherit from a base class that provides:
- The `objects()` class method returning a manager
- Attribute descriptors that return query expressions
- Serialization/deserialization logic for the backend

### Query Expression System
- Overload `__eq__`, `__ne__`, `__lt__`, `__le__`, `__gt__`, `__ge__` on field descriptors
- Return expression objects that can be combined with `&` (and) and `|` (or)
- The expression tree should be translatable to each backend's native query format

### Backend-Specific Translations
- MongoDB: Translate to MongoDB query syntax (`{"field": {"$gte": value}}`)
- Firestore: Translate to Firestore query methods
- Redis: Consider using RedisJSON or implement custom serialization

### Connection Management
- Implement connection pooling where applicable
- Support both explicit connection management and automatic lifecycle
- Consider context managers for transaction support