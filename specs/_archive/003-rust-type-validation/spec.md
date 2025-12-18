---
feature: rust-type-validation
components:
  - data-bridge
lead: data-bridge
status: specifying
created: 2025-12-10
branch: feature/cross_data-bridge
---

# Rust-Side Type Validation for data-bridge

## Problem Statement

### Current State: Type Safety Gap

data-bridge currently lacks strong runtime type validation for document fields.

**Note**: The existing `validation.rs` in the Rust crates provides **query validation** (blocking dangerous operators like `$where`, `$function`) and **security validation** (collection names, field names, ObjectId parsing). This spec addresses **document type validation** (field types, constraints, required fields), which is a separate concern.

**Performance Reality**:
- data-bridge is **faster** than Beanie in bulk operations (3.7x) and queries (30%+)
- BUT Beanie is 32% faster in single insert (most common operation)
- More critically: **Beanie has Pydantic v2 type safety**, data-bridge doesn't

**Type Safety Comparison**:

```python
# Beanie with Pydantic v2 - Strong validation
class BeanieUser(Document):
    email: str
    age: int

user = BeanieUser(email="test@example.com", age="not_a_number")
# ❌ Raises ValidationError: age must be int

# data-bridge - Weak validation (Python's dynamic typing)
class DataBridgeUser(Document):
    email: str
    age: int

user = DataBridgeUser(email="test@example.com", age="not_a_number")
# ✅ Silently accepts - age stores "not_a_number" as string
```

**User Feedback**:
> "no, I think we are lose, since beanie with the typesafe(actually pydantic2)"
> "can we no python validate, all run in rust?"

### Goal: Rust-Side Validation

Move validation entirely to Rust to achieve **BOTH**:
1. **Type Safety**: Pydantic v2 level runtime validation
2. **Performance**: Rust-level speed (10-100x faster than Pydantic)

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| **Validate always** | Match Beanie behavior - validate on ALL operations (user input + DB reads) |
| **No Pydantic integration** | Users can use `pydantic_model.model_validate(obj)` if needed |
| **Custom validators in Python** | Rust validates types, Python handles custom business logic |
| **Simple error format** | Easy to read for humans and AI agents (no need to match Pydantic exactly) |
| **Schema evolution handled naturally** | Strict validation forces proper migration (add fields as Optional first) |

## Architecture Overview

### High-Level Design

```
┌─────────────────────────────────────────────────────────────────┐
│ Python Layer                                                    │
│                                                                 │
│  class User(Document):                                          │
│      email: str            ← Extract type hints at runtime     │
│      age: int              ← Pass to Rust as validation schema │
│      city: Optional[str]                                        │
│                                                                 │
│      class Settings:                                            │
│          use_validation = True  # Opt-in strict validation     │
│                                                                 │
│      def validate_on_save(self):  # Custom Python validation   │
│          if self.age < 18:                                      │
│              raise ValueError("Must be 18+")                    │
│                                                                 │
│  # ALWAYS validates (like Beanie)                               │
│  user = User(...)          ──────┐  Rust validates             │
│  user = await User.find_one(...) ┼──┐  Rust validates          │
│  await user.save()         ──────┘  └─ Rust validates          │
└──────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│ Rust Layer (PyO3)                                               │
│                                                                 │
│  1. Extract Python type hints (one-time per class)              │
│     - Use Python's __annotations__ dict                         │
│     - Parse typing.* (Optional, List, Dict, Annotated, etc.)    │
│                                                                 │
│  2. Build Rust validation schema (cached)                       │
│     - Map Python types to Rust validators                       │
│     - Extract constraints from Annotated types                  │
│                                                                 │
│  3. Validate BSON on ALL operations                             │
│     - __init__: Validate user input                             │
│     - find/find_one: Validate DB documents (STRICT like Beanie) │
│     - save/update: Validate before write                        │
│                                                                 │
│  4. Return clear validation errors                              │
│     - Field-level error messages                                │
│     - Easy to read for humans and AI                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Validation Flow (Match Beanie)

```python
# User creates document
user = User(email="test@example.com", age="not_int")
# → Rust validates immediately
# → Raises: ValidationError: age: expected integer, got string

# User loads old document with missing field
user = await User.find_one(User.email == "old@example.com")
# → Rust validates DB document (STRICT)
# → If 'age' field missing: Raises ValidationError: age: field required
# → Forces migration: add field as Optional first, migrate data, then make required

# User saves document
await user.save()
# → Rust validates before write
# → Then Python custom validation (validate_on_save)
```

## Technical Implementation

### Phase 1: Type Extraction (Python → Rust)

**Goal**: Extract Python type annotations and pass to Rust

#### Python Type Extraction Helper

```python
# python/data_bridge/type_extraction.py

from typing import Any, Dict, Optional, get_type_hints, get_origin, get_args
import typing

class TypeSchema:
    """Schema representation for Rust validation."""

    def __init__(self, field_name: str, py_type: Any):
        self.field_name = field_name
        self.base_type = self._extract_base_type(py_type)
        self.is_optional = self._is_optional(py_type)
        self.constraints = self._extract_constraints(py_type)

    def _extract_base_type(self, py_type: Any) -> str:
        """Extract base type (str, int, float, etc.)."""
        origin = get_origin(py_type)

        # Handle Optional[T] → T
        if origin is typing.Union:
            args = get_args(py_type)
            non_none_types = [t for t in args if t is not type(None)]
            if len(non_none_types) == 1:
                return self._type_to_rust_name(non_none_types[0])

        # Handle List[T], Dict[K, V]
        if origin is list:
            return "list"
        if origin is dict:
            return "dict"

        # Simple types
        return self._type_to_rust_name(py_type)

    def _type_to_rust_name(self, py_type: Any) -> str:
        """Map Python type to Rust validator type."""
        type_map = {
            str: "string",
            int: "integer",
            float: "float",
            bool: "boolean",
            bytes: "bytes",
        }
        return type_map.get(py_type, "any")

    def _is_optional(self, py_type: Any) -> bool:
        """Check if type is Optional[T]."""
        origin = get_origin(py_type)
        if origin is typing.Union:
            args = get_args(py_type)
            return type(None) in args
        return False

    def _extract_constraints(self, py_type: Any) -> Dict[str, Any]:
        """Extract validation constraints from Annotated types."""
        if get_origin(py_type) is typing.Annotated:
            args = get_args(py_type)
            # Parse constraints: Annotated[str, MinLen(5), MaxLen(100)]
            constraints = {}
            for constraint in args[1:]:
                # Simple constraint parsing
                # Can be extended to support custom constraint objects
                if hasattr(constraint, '__dict__'):
                    constraints.update(constraint.__dict__)
            return constraints
        return {}

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dict for Rust FFI."""
        return {
            "field_name": self.field_name,
            "base_type": self.base_type,
            "is_optional": self.is_optional,
            "constraints": self.constraints,
        }


def extract_validation_schema(document_class: type) -> Dict[str, TypeSchema]:
    """Extract validation schema from Document class annotations."""
    try:
        type_hints = get_type_hints(document_class, include_extras=True)
    except Exception:
        # Fallback to __annotations__ if get_type_hints fails
        type_hints = getattr(document_class, '__annotations__', {})

    schema = {}
    for field_name, py_type in type_hints.items():
        # Skip private fields and class vars
        if field_name.startswith("_"):
            continue

        schema[field_name] = TypeSchema(field_name, py_type)

    return schema
```

#### Integrate with Document Metaclass

```python
# python/data_bridge/document.py (DocumentMeta.__new__)

class DocumentMeta(type):
    def __new__(mcs, name, bases, namespace, **kwargs):
        cls = super().__new__(mcs, name, bases, namespace, **kwargs)

        # ... existing metaclass logic ...

        # Extract validation schema (one-time per class)
        if name != "Document" and cls.Settings.use_validation:
            from .type_extraction import extract_validation_schema
            cls._validation_schema = extract_validation_schema(cls)

            # Convert to dict for Rust FFI
            cls._validation_schema_dict = {
                field_name: schema.to_dict()
                for field_name, schema in cls._validation_schema.items()
            }

            # Register schema with Rust immediately
            from . import _engine
            _engine.register_validation_schema(name, cls._validation_schema_dict)

        return cls
```

### Phase 2: Rust Validation Engine

**Goal**: Implement type validation in Rust using extracted schema

#### Add Dependencies

```toml
# data-bridge/Cargo.toml

[workspace.dependencies]
# Validation
validator = { version = "0.18", features = ["derive"] }
once_cell = "1.20"  # For schema caching
```

#### Validation Schema Cache (Rust)

```rust
// crates/data-bridge/src/validation.rs (add to existing file)

use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Mutex;
use bson::{Bson, Document as BsonDocument};
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyDict;

/// Validation schema for a document field
#[derive(Debug, Clone)]
pub struct FieldSchema {
    pub field_name: String,
    pub base_type: String,  // "string", "integer", "float", etc.
    pub is_optional: bool,
    pub constraints: HashMap<String, serde_json::Value>,
}

/// Validation schema for a document class
#[derive(Debug, Clone)]
pub struct DocumentSchema {
    pub class_name: String,
    pub fields: HashMap<String, FieldSchema>,
}

/// Global schema cache (one schema per document class)
static SCHEMA_CACHE: OnceCell<Mutex<HashMap<String, DocumentSchema>>> = OnceCell::new();

pub fn get_schema_cache() -> &'static Mutex<HashMap<String, DocumentSchema>> {
    SCHEMA_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Register validation schema for a document class (called from Python metaclass)
#[pyfunction]
pub fn register_validation_schema(
    class_name: &str,
    schema_dict: &Bound<'_, PyDict>,
) -> PyResult<()> {
    let mut fields = HashMap::new();

    for (field_name, field_schema) in schema_dict.iter() {
        let field_name: String = field_name.extract()?;
        let field_schema: &Bound<'_, PyDict> = field_schema.downcast()?;

        // Extract field schema
        let base_type: String = field_schema.get_item("base_type")?.unwrap().extract()?;
        let is_optional: bool = field_schema.get_item("is_optional")?.unwrap().extract()?;

        // Parse constraints JSON
        let constraints_dict = field_schema.get_item("constraints")?.unwrap();
        let constraints_str = constraints_dict.str()?.to_string();
        let constraints: HashMap<String, serde_json::Value> =
            serde_json::from_str(&constraints_str).unwrap_or_default();

        fields.insert(field_name.clone(), FieldSchema {
            field_name,
            base_type,
            is_optional,
            constraints,
        });
    }

    let schema = DocumentSchema {
        class_name: class_name.to_string(),
        fields,
    };

    // Cache the schema
    let mut cache = get_schema_cache().lock().unwrap();
    cache.insert(class_name.to_string(), schema);

    Ok(())
}
```

#### Validation Logic

```rust
// crates/data-bridge/src/validation.rs (continued)

/// Validation error with field-level details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field_name: String,
    pub error_message: String,
    pub expected: String,
    pub got: String,
}

impl ValidationError {
    pub fn new(field_name: &str, message: &str) -> Self {
        Self {
            field_name: field_name.to_string(),
            error_message: message.to_string(),
            expected: String::new(),
            got: String::new(),
        }
    }

    pub fn type_mismatch(field_name: &str, expected: &str, got: &str) -> Self {
        Self {
            field_name: field_name.to_string(),
            error_message: format!("expected {}, got {}", expected, got),
            expected: expected.to_string(),
            got: got.to_string(),
        }
    }

    /// Format error for Python (easy to read)
    pub fn format(&self) -> String {
        if !self.expected.is_empty() {
            format!("{}: {} (expected: {}, got: {})",
                self.field_name, self.error_message, self.expected, self.got)
        } else {
            format!("{}: {}", self.field_name, self.error_message)
        }
    }
}

/// Validate a BSON document against a schema (STRICT - always validate)
pub fn validate_document(
    class_name: &str,
    doc: &BsonDocument,
) -> Result<(), Vec<ValidationError>> {
    // Get cached schema
    let cache = get_schema_cache().lock().unwrap();
    let schema = match cache.get(class_name) {
        Some(s) => s.clone(),
        None => {
            // No schema registered - skip validation
            return Ok(());
        }
    };
    drop(cache);  // Release lock

    let mut errors = Vec::new();

    // Validate each field in schema
    for (field_name, field_schema) in &schema.fields {
        let value = doc.get(field_name);

        // Check required fields (STRICT)
        if value.is_none() {
            if !field_schema.is_optional {
                errors.push(ValidationError::new(field_name, "field required"));
            }
            continue;
        }

        let value = value.unwrap();

        // Validate type (STRICT)
        if let Err(err) = validate_field_type(field_name, value, field_schema) {
            errors.push(err);
            continue;
        }

        // Validate constraints
        if let Err(constraint_errors) = validate_field_constraints(field_name, value, field_schema) {
            errors.extend(constraint_errors);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validate field type matches schema
fn validate_field_type(
    field_name: &str,
    value: &Bson,
    schema: &FieldSchema,
) -> Result<(), ValidationError> {
    let expected_type = &schema.base_type;

    let actual_type = match value {
        Bson::String(_) => "string",
        Bson::Int32(_) | Bson::Int64(_) => "integer",
        Bson::Double(_) => "float",
        Bson::Boolean(_) => "boolean",
        Bson::Binary(_) => "bytes",
        Bson::Array(_) => "list",
        Bson::Document(_) => "dict",
        Bson::ObjectId(_) => "objectid",
        Bson::DateTime(_) => "datetime",
        Bson::Null => {
            if schema.is_optional {
                return Ok(());
            } else {
                return Err(ValidationError::new(field_name, "field cannot be null"));
            }
        }
        _ => "unknown",
    };

    if actual_type != expected_type && expected_type != "any" {
        return Err(ValidationError::type_mismatch(field_name, expected_type, actual_type));
    }

    Ok(())
}

/// Validate field constraints (length, range, format, etc.)
fn validate_field_constraints(
    field_name: &str,
    value: &Bson,
    schema: &FieldSchema,
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // String constraints
    if let Bson::String(s) = value {
        // min_length
        if let Some(min_len) = schema.constraints.get("min_length") {
            if let Some(min) = min_len.as_u64() {
                if s.len() < min as usize {
                    errors.push(ValidationError::new(
                        field_name,
                        &format!("string too short (min: {} chars)", min),
                    ));
                }
            }
        }

        // max_length
        if let Some(max_len) = schema.constraints.get("max_length") {
            if let Some(max) = max_len.as_u64() {
                if s.len() > max as usize {
                    errors.push(ValidationError::new(
                        field_name,
                        &format!("string too long (max: {} chars)", max),
                    ));
                }
            }
        }

        // format validation (email, url, etc.)
        if let Some(format) = schema.constraints.get("format") {
            if let Some(format_str) = format.as_str() {
                match format_str {
                    "email" => {
                        if !is_valid_email(s) {
                            errors.push(ValidationError::new(field_name, "invalid email format"));
                        }
                    }
                    "url" => {
                        if !is_valid_url(s) {
                            errors.push(ValidationError::new(field_name, "invalid URL format"));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Numeric constraints
    if let Bson::Int32(n) = value {
        validate_numeric_constraints(field_name, *n as i64, schema, &mut errors);
    }
    if let Bson::Int64(n) = value {
        validate_numeric_constraints(field_name, *n, schema, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_numeric_constraints(
    field_name: &str,
    value: i64,
    schema: &FieldSchema,
    errors: &mut Vec<ValidationError>,
) {
    // min
    if let Some(min_val) = schema.constraints.get("min") {
        if let Some(min) = min_val.as_i64() {
            if value < min {
                errors.push(ValidationError::new(
                    field_name,
                    &format!("value too small (min: {})", min),
                ));
            }
        }
    }

    // max
    if let Some(max_val) = schema.constraints.get("max") {
        if let Some(max) = max_val.as_i64() {
            if value > max {
                errors.push(ValidationError::new(
                    field_name,
                    &format!("value too large (max: {})", max),
                ));
            }
        }
    }
}

// Email validation using regex
fn is_valid_email(email: &str) -> bool {
    use regex::Regex;
    let email_regex = Regex::new(
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
    ).unwrap();
    email_regex.is_match(email)
}

// URL validation
fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// Format multiple validation errors into a readable message
pub fn format_validation_errors(errors: Vec<ValidationError>) -> String {
    let count = errors.len();
    let error_list = errors
        .iter()
        .map(|e| format!("  {}", e.format()))
        .collect::<Vec<_>>()
        .join("\n");

    format!("Validation failed ({} error{}):\n{}",
        count,
        if count == 1 { "" } else { "s" },
        error_list)
}
```

### Phase 3: Integration with MongoDB Operations

**Goal**: Call validation on ALL operations (like Beanie)

#### Update MongoDB Operations

```rust
// crates/data-bridge/src/mongodb.rs

/// Insert a single document (WITH VALIDATION)
#[pyfunction]
pub fn insert_one(
    py: Python,
    collection_name: &str,
    document: &Bound<'_, PyDict>,
    class_name: Option<&str>,
) -> PyResult<Bound<'_, PyAny>> {
    future_into_py(py, async move {
        let bson_doc = py_dict_to_bson(py, document)?;

        // VALIDATE: Always validate user input
        if let Some(class_name) = class_name {
            if let Err(validation_errors) = validate_document(class_name, &bson_doc) {
                return Err(PyValueError::new_err(format_validation_errors(validation_errors)));
            }
        }

        // ... existing MongoDB insert logic ...
    })
}

/// Find documents and convert to Python dicts (WITH VALIDATION)
#[pyfunction]
pub fn find_as_documents(
    py: Python,
    collection_name: &str,
    filter: &Bound<'_, PyDict>,
    class_name: Option<&str>,  // NEW: for validation
    // ... other params ...
) -> PyResult<Bound<'_, PyAny>> {
    future_into_py(py, async move {
        // ... existing find logic ...

        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await? {
            // VALIDATE: Strict validation on DB documents (like Beanie)
            if let Some(class_name) = class_name {
                if let Err(validation_errors) = validate_document(class_name, &doc) {
                    return Err(PyValueError::new_err(format_validation_errors(validation_errors)));
                }
            }

            results.push(bson_to_py_dict(py, &doc)?);
        }

        Ok(results)
    })
}
```

#### Update Document Methods

```python
# python/data_bridge/document.py

class Document:
    def __init__(self, **data: Any):
        """Create document instance (VALIDATES if use_validation=True)."""
        # Set data first
        self._data = data
        self._id = data.get("_id")
        self._revision_id = None
        self._original_data = None
        self._previous_changes = None

        # Validate user input (Rust validation)
        if self.Settings.use_validation:
            from . import _engine
            # Convert to BSON for validation
            bson_dict = {k: v for k, v in data.items() if not k.startswith("_")}
            _engine.validate_document_dict(self.__class__.__name__, bson_dict)

    @classmethod
    def _from_db(cls: Type[T], data: Dict[str, Any]) -> T:
        """Create instance from database document (VALIDATES if use_validation=True)."""
        # ... polymorphic class selection ...

        # STRICT VALIDATION (like Beanie): validate DB documents
        if cls.Settings.use_validation:
            from . import _engine
            _engine.validate_document_dict(cls.__name__, data)

        # Create instance (fast path - skip __init__ overhead)
        instance = object.__new__(cls)
        instance._id = data.get("_id")
        instance._data = data
        instance._revision_id = None
        instance._original_data = None
        instance._previous_changes = None

        return instance

    async def save(self) -> T:
        """Save document to database (VALIDATES if use_validation=True)."""
        # Validate before save
        if self.Settings.use_validation:
            from . import _engine
            _engine.validate_document_dict(self.__class__.__name__, self._data)

        # Custom Python validation
        if hasattr(self, 'validate_on_save'):
            self.validate_on_save()

        # ... rest of save logic ...
```

### Phase 4: Custom Python Validators

**Goal**: Support custom business logic validation in Python

```python
# python/data_bridge/document.py

class Document:
    def validate_on_save(self):
        """Override this method to add custom validation logic.

        Called AFTER Rust type validation, BEFORE database write.
        Raise ValueError/ValidationError for validation failures.

        Example:
            def validate_on_save(self):
                if self.age < 18 and "admin" in self.email:
                    raise ValueError("Admins must be 18+")
        """
        pass

# Example usage:
class User(Document):
    email: str
    age: int
    role: str

    class Settings:
        name = "users"
        use_validation = True

    def validate_on_save(self):
        """Custom business logic validation."""
        # Rust already validated types (str, int)
        # Now validate business rules
        if self.age < 18 and self.role == "admin":
            raise ValueError("Admins must be at least 18 years old")

        if self.role not in ["user", "admin", "moderator"]:
            raise ValueError(f"Invalid role: {self.role}")
```

## Performance Analysis

### Expected Performance

Based on validator crate benchmarks and Pydantic v2 comparisons:

| Operation | Pydantic v2 | Rust Validation | Speedup |
|-----------|-------------|-----------------|---------|
| Simple types (str, int) | ~10μs | ~0.1μs | 100x |
| Complex nested objects | ~100μs | ~1μs | 100x |
| Email validation | ~5μs | ~0.5μs | 10x |
| List[int] 100 items | ~50μs | ~0.5μs | 100x |

### Target Performance

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| **Single insert** | 1.667ms | <1.2ms | Beat Beanie (1.129ms) |
| **Validation overhead** | N/A | <0.1ms | 10-100x faster than Pydantic |
| **Type safety** | None | Pydantic-level | Catch 80%+ type errors |

## Migration Path

### Phase 1: Proof of Concept (Week 1)
- [ ] Type extraction helper (Python)
- [ ] Basic type validation (str, int, float, bool) in Rust
- [ ] Schema cache and registration
- [ ] Integration with __init__ and _from_db
- [ ] Benchmarks: Rust vs Pydantic v2

**Success Criteria**: 10-100x faster than Pydantic v2, catches basic type errors

### Phase 2: Core Features (Week 2)
- [ ] Optional types (Optional[T])
- [ ] Collections (List[T], Dict[K, V])
- [ ] Nested documents
- [ ] ObjectId validation
- [ ] Integration with all DB operations (find, update, delete)

**Success Criteria**: All basic Pydantic features supported, strict validation works

### Phase 3: Advanced Constraints (Week 3)
- [ ] String constraints (min_length, max_length)
- [ ] Numeric constraints (min, max)
- [ ] Format validation (email, url, uuid)
- [ ] Custom Python validators (validate_on_save)
- [ ] Error message formatting

**Success Criteria**: 80% Pydantic feature parity

### Phase 4: Production Readiness (Week 4)
- [ ] Performance optimization
- [ ] Comprehensive tests (100+ tests)
- [ ] Documentation and migration guide
- [ ] Schema evolution examples
- [ ] Benchmark suite

**Success Criteria**: Production-ready, beats Beanie in single insert performance

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Type extraction complexity** | High | Start with simple types, incrementally add complex types |
| **Breaking changes** | Critical | Backward compatible via `use_validation=False` (default) |
| **Performance regression** | High | Extensive benchmarking, validation overhead must be <0.1ms |
| **Schema evolution issues** | Medium | Clear migration guide (add as Optional first, migrate data, then required) |
| **Error message clarity** | Medium | Iterative user testing, AI-agent friendly format |

## Success Metrics

### Performance Targets

| Operation | Current | Target | Status |
|-----------|---------|--------|--------|
| Single insert | 1.667ms | <1.2ms | Beat Beanie |
| Bulk insert 100 | 1.877ms | <1.5ms | Maintain advantage |
| Find one | 0.803ms | <0.8ms | Maintain advantage |
| Validation overhead | N/A | <0.1ms | 10-100x faster than Pydantic |

### Quality Gates

- [ ] Zero breaking changes (use_validation=False by default)
- [ ] 100% backward compatibility
- [ ] 10-100x faster validation than Pydantic v2
- [ ] 80%+ Pydantic feature parity
- [ ] All existing 449+ tests pass
- [ ] 100+ new validation tests
- [ ] Schema evolution documented with examples

## References

### Rust Validation Libraries

- [validator crate](https://docs.rs/validator)
- [serde_valid](https://docs.rs/serde_valid/latest/serde_valid/)
- [Leapcell validation guide](https://leapcell.io/blog/ensuring-robustness-in-rust-web-services-type-safe-request-body-parsing-and-validation-with-serde-and-validator)

### PyO3 Type Conversion

- [PyO3 type conversions](https://pyo3.rs/main/conversions/tables.html)
- [PyO3 conversion traits](https://pyo3.rs/main/conversions/traits.html)
- [Python typing hints](https://pyo3.rs/v0.25.0/python-typing-hints.html)

### MongoDB Schema Validation

- [MongoDB Rust driver schema validation](https://www.mongodb.com/docs/drivers/rust/current/fundamentals/schema-validation/)
- [BSON crate](https://docs.rs/bson)

## Open Questions Resolved

1. ~~**Pydantic v2 Integration**~~ → Users can use `pydantic_model.model_validate(obj)` if needed
2. ~~**Custom Validators**~~ → Python `validate_on_save()` method for custom business logic
3. ~~**Error Formatting**~~ → Simple, easy-to-read format (no need to match Pydantic exactly)
4. ~~**Schema Evolution**~~ → Strict validation forces proper migration workflow (handled naturally)

## Next Steps

1. **User Approval**: Confirm this approach aligns with project goals
2. **Proof of Concept**: Implement Phase 1 with basic type validation
3. **Benchmark**: Compare Rust validation vs Pydantic v2 and current performance
4. **Iterate**: Refine based on performance results and user feedback
