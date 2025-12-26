//! Type mapping between Python and PostgreSQL.
//!
//! This module handles conversion between Python objects and PostgreSQL types,
//! similar to data-bridge-mongodb's BSON type handling.

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::postgres::{PgArguments, PgRow};
use sqlx::{Arguments, Column, Row as SqlxRow, Type, TypeInfo, Postgres};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{DataBridgeError, Result};

/// Represents a value extracted from Python for PostgreSQL conversion.
///
/// This enum captures all Python types that can be mapped to PostgreSQL types.
/// The conversion happens entirely in Rust to avoid Python heap pressure.
#[derive(Debug, Clone)]
pub enum ExtractedValue {
    /// NULL value
    Null,
    /// Boolean (BOOLEAN)
    Bool(bool),
    /// Small integer (SMALLINT)
    SmallInt(i16),
    /// Integer (INTEGER)
    Int(i32),
    /// Big integer (BIGINT)
    BigInt(i64),
    /// Single-precision float (REAL)
    Float(f32),
    /// Double-precision float (DOUBLE PRECISION)
    Double(f64),
    /// Variable-length string (VARCHAR, TEXT)
    String(String),
    /// Binary data (BYTEA)
    Bytes(Vec<u8>),
    /// UUID (UUID)
    Uuid(Uuid),
    /// Date (DATE)
    Date(NaiveDate),
    /// Time (TIME)
    Time(NaiveTime),
    /// Timestamp without timezone (TIMESTAMP)
    Timestamp(NaiveDateTime),
    /// Timestamp with timezone (TIMESTAMPTZ)
    TimestampTz(DateTime<Utc>),
    /// JSON/JSONB (JSON, JSONB)
    Json(JsonValue),
    /// Array of values (ARRAY)
    Array(Vec<ExtractedValue>),
    /// Decimal/Numeric (NUMERIC, DECIMAL)
    Decimal(String), // Store as string to avoid precision loss
}

impl ExtractedValue {
    /// Converts Python object to ExtractedValue.
    ///
    /// # Arguments
    ///
    /// * `py` - Python GIL guard
    /// * `obj` - Python object to convert
    ///
    /// # Errors
    ///
    /// Returns error if Python type cannot be mapped to PostgreSQL type.
    pub fn from_python(/* py: Python, obj: &PyAny */) -> Result<Self> {
        // TODO: Implement Python to ExtractedValue conversion
        // - Handle None -> Null
        // - Handle bool -> Bool
        // - Handle int -> Int/BigInt (based on range)
        // - Handle float -> Double
        // - Handle str -> String
        // - Handle bytes -> Bytes
        // - Handle datetime -> TimestampTz
        // - Handle date -> Date
        // - Handle time -> Time
        // - Handle UUID -> Uuid
        // - Handle dict/list -> Json
        // - Handle list of same type -> Array
        // - Validate types and ranges
        todo!("Implement ExtractedValue::from_python")
    }

    /// Converts ExtractedValue to Python object.
    ///
    /// # Arguments
    ///
    /// * `py` - Python GIL guard
    ///
    /// # Errors
    ///
    /// Returns error if conversion fails.
    pub fn to_python(/* &self, py: Python */) -> Result<()> {
        // TODO: Implement ExtractedValue to Python conversion
        // - Null -> None
        // - Bool -> bool
        // - Int/BigInt -> int
        // - Double -> float
        // - String -> str
        // - Bytes -> bytes
        // - TimestampTz -> datetime
        // - Date -> date
        // - Time -> time
        // - Uuid -> UUID
        // - Json -> dict/list
        // - Array -> list
        todo!("Implement ExtractedValue::to_python")
    }

    /// Returns the PostgreSQL type name for this value.
    pub fn pg_type_name(&self) -> &'static str {
        match self {
            ExtractedValue::Null => "NULL",
            ExtractedValue::Bool(_) => "BOOLEAN",
            ExtractedValue::SmallInt(_) => "SMALLINT",
            ExtractedValue::Int(_) => "INTEGER",
            ExtractedValue::BigInt(_) => "BIGINT",
            ExtractedValue::Float(_) => "REAL",
            ExtractedValue::Double(_) => "DOUBLE PRECISION",
            ExtractedValue::String(_) => "TEXT",
            ExtractedValue::Bytes(_) => "BYTEA",
            ExtractedValue::Uuid(_) => "UUID",
            ExtractedValue::Date(_) => "DATE",
            ExtractedValue::Time(_) => "TIME",
            ExtractedValue::Timestamp(_) => "TIMESTAMP",
            ExtractedValue::TimestampTz(_) => "TIMESTAMPTZ",
            ExtractedValue::Json(_) => "JSONB",
            ExtractedValue::Array(_) => "ARRAY",
            ExtractedValue::Decimal(_) => "NUMERIC",
        }
    }

    /// Bind this value to a sqlx query.
    ///
    /// This method adds the value as a parameter to the query, enabling
    /// GIL-free query construction.
    ///
    /// # Arguments
    ///
    /// * `arguments` - Mutable reference to PgArguments for binding
    ///
    /// # Errors
    ///
    /// Returns error if binding fails (e.g., type incompatibility).
    pub fn bind_to_arguments(&self, arguments: &mut PgArguments) -> Result<()> {
        match self {
            ExtractedValue::Null => {
                // For null, we need to bind as a typed null
                // Using Option<i32> as a generic nullable type
                arguments.add(Option::<i32>::None)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind NULL: {}", e)))?;
            }
            ExtractedValue::Bool(v) => {
                arguments.add(*v)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind BOOL: {}", e)))?;
            }
            ExtractedValue::SmallInt(v) => {
                arguments.add(*v)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind SMALLINT: {}", e)))?;
            }
            ExtractedValue::Int(v) => {
                arguments.add(*v)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind INT: {}", e)))?;
            }
            ExtractedValue::BigInt(v) => {
                arguments.add(*v)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind BIGINT: {}", e)))?;
            }
            ExtractedValue::Float(v) => {
                arguments.add(*v)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind FLOAT: {}", e)))?;
            }
            ExtractedValue::Double(v) => {
                arguments.add(*v)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind DOUBLE: {}", e)))?;
            }
            ExtractedValue::String(v) => {
                arguments.add(v.as_str())
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind STRING: {}", e)))?;
            }
            ExtractedValue::Bytes(v) => {
                arguments.add(v.as_slice())
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind BYTES: {}", e)))?;
            }
            ExtractedValue::Uuid(v) => {
                arguments.add(*v)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind UUID: {}", e)))?;
            }
            ExtractedValue::Date(v) => {
                arguments.add(*v)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind DATE: {}", e)))?;
            }
            ExtractedValue::Time(v) => {
                arguments.add(*v)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind TIME: {}", e)))?;
            }
            ExtractedValue::Timestamp(v) => {
                arguments.add(*v)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind TIMESTAMP: {}", e)))?;
            }
            ExtractedValue::TimestampTz(v) => {
                arguments.add(*v)
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind TIMESTAMPTZ: {}", e)))?;
            }
            ExtractedValue::Json(v) => {
                arguments.add(v.clone())
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind JSON: {}", e)))?;
            }
            ExtractedValue::Array(values) => {
                // For arrays, we need to determine the element type and bind accordingly
                // This is a simplified implementation - in production, you'd want to handle
                // homogeneous arrays more efficiently
                if values.is_empty() {
                    // Empty array - bind as NULL array
                    arguments.add(Option::<Vec<i32>>::None)
                        .map_err(|e| DataBridgeError::Query(format!("Failed to bind empty ARRAY: {}", e)))?;
                } else {
                    // For now, convert to JSON array as a fallback
                    // This handles heterogeneous arrays but may not be optimal
                    let json_array: Vec<JsonValue> = values
                        .iter()
                        .map(|v| extracted_to_json(v))
                        .collect::<Result<Vec<_>>>()?;
                    arguments.add(JsonValue::Array(json_array))
                        .map_err(|e| DataBridgeError::Query(format!("Failed to bind ARRAY: {}", e)))?;
                }
            }
            ExtractedValue::Decimal(v) => {
                // For now, bind decimals as strings
                // TODO: Enable rust_decimal feature in sqlx for native decimal support
                arguments.add(v.as_str())
                    .map_err(|e| DataBridgeError::Query(format!("Failed to bind DECIMAL: {}", e)))?;
            }
        }
        Ok(())
    }
}

/// Convert a PgRow to a HashMap of column name -> ExtractedValue.
///
/// This function enables GIL-free extraction of PostgreSQL rows into
/// our intermediate representation.
///
/// # Arguments
///
/// * `row` - PostgreSQL row from sqlx query result
///
/// # Errors
///
/// Returns error if column extraction or type conversion fails.
pub fn row_to_extracted(row: &PgRow) -> Result<HashMap<String, ExtractedValue>> {
    let mut columns = HashMap::new();

    // Iterate over all columns in the row
    for (idx, column) in row.columns().iter().enumerate() {
        let column_name = column.name().to_string();
        let type_info = column.type_info();
        let type_name = type_info.name();

        // Extract value based on PostgreSQL type
        let value = match type_name {
            "BOOL" | "BOOLEAN" => {
                match row.try_get::<Option<bool>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::Bool(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract BOOL from column '{}': {}", column_name, e)
                    )),
                }
            }
            "INT2" | "SMALLINT" => {
                match row.try_get::<Option<i16>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::SmallInt(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract SMALLINT from column '{}': {}", column_name, e)
                    )),
                }
            }
            "INT4" | "INTEGER" | "INT" => {
                match row.try_get::<Option<i32>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::Int(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract INT from column '{}': {}", column_name, e)
                    )),
                }
            }
            "INT8" | "BIGINT" => {
                match row.try_get::<Option<i64>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::BigInt(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract BIGINT from column '{}': {}", column_name, e)
                    )),
                }
            }
            "FLOAT4" | "REAL" => {
                match row.try_get::<Option<f32>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::Float(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract FLOAT from column '{}': {}", column_name, e)
                    )),
                }
            }
            "FLOAT8" | "DOUBLE PRECISION" | "DOUBLE" => {
                match row.try_get::<Option<f64>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::Double(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract DOUBLE from column '{}': {}", column_name, e)
                    )),
                }
            }
            "VARCHAR" | "TEXT" | "CHAR" | "BPCHAR" | "NAME" => {
                match row.try_get::<Option<String>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::String(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract STRING from column '{}': {}", column_name, e)
                    )),
                }
            }
            "BYTEA" => {
                match row.try_get::<Option<Vec<u8>>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::Bytes(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract BYTES from column '{}': {}", column_name, e)
                    )),
                }
            }
            "UUID" => {
                match row.try_get::<Option<Uuid>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::Uuid(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract UUID from column '{}': {}", column_name, e)
                    )),
                }
            }
            "DATE" => {
                match row.try_get::<Option<NaiveDate>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::Date(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract DATE from column '{}': {}", column_name, e)
                    )),
                }
            }
            "TIME" => {
                match row.try_get::<Option<NaiveTime>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::Time(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract TIME from column '{}': {}", column_name, e)
                    )),
                }
            }
            "TIMESTAMP" => {
                match row.try_get::<Option<NaiveDateTime>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::Timestamp(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract TIMESTAMP from column '{}': {}", column_name, e)
                    )),
                }
            }
            "TIMESTAMPTZ" | "TIMESTAMP WITH TIME ZONE" => {
                match row.try_get::<Option<DateTime<Utc>>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::TimestampTz(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract TIMESTAMPTZ from column '{}': {}", column_name, e)
                    )),
                }
            }
            "JSON" | "JSONB" => {
                match row.try_get::<Option<JsonValue>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::Json(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract JSON from column '{}': {}", column_name, e)
                    )),
                }
            }
            "NUMERIC" | "DECIMAL" => {
                // Extract as string for now
                // TODO: Enable rust_decimal feature in sqlx for native decimal support
                match row.try_get::<Option<String>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::Decimal(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract DECIMAL from column '{}': {}", column_name, e)
                    )),
                }
            }
            // Array types - handle common array types
            "_BOOL" => extract_array::<bool>(row, idx, &column_name, |v| ExtractedValue::Bool(v))?,
            "_INT2" => extract_array::<i16>(row, idx, &column_name, |v| ExtractedValue::SmallInt(v))?,
            "_INT4" => extract_array::<i32>(row, idx, &column_name, |v| ExtractedValue::Int(v))?,
            "_INT8" => extract_array::<i64>(row, idx, &column_name, |v| ExtractedValue::BigInt(v))?,
            "_FLOAT4" => extract_array::<f32>(row, idx, &column_name, |v| ExtractedValue::Float(v))?,
            "_FLOAT8" => extract_array::<f64>(row, idx, &column_name, |v| ExtractedValue::Double(v))?,
            "_TEXT" | "_VARCHAR" => extract_array::<String>(row, idx, &column_name, |v| ExtractedValue::String(v))?,
            "_UUID" => extract_array::<Uuid>(row, idx, &column_name, |v| ExtractedValue::Uuid(v))?,

            // Unknown type - try to extract as string as fallback
            unknown => {
                tracing::warn!("Unknown PostgreSQL type '{}' for column '{}', attempting string extraction", unknown, column_name);
                match row.try_get::<Option<String>, _>(idx) {
                    Ok(Some(v)) => ExtractedValue::String(v),
                    Ok(None) => ExtractedValue::Null,
                    Err(e) => return Err(DataBridgeError::Query(
                        format!("Failed to extract unknown type '{}' from column '{}': {}", unknown, column_name, e)
                    )),
                }
            }
        };

        columns.insert(column_name, value);
    }

    Ok(columns)
}

/// Helper function to extract arrays from PostgreSQL rows.
///
/// # Type Parameters
///
/// * `T` - Element type that implements sqlx Type and Clone
/// * `F` - Function to convert T to ExtractedValue
fn extract_array<T>(
    row: &PgRow,
    idx: usize,
    column_name: &str,
    convert: impl Fn(T) -> ExtractedValue
) -> Result<ExtractedValue>
where
    T: for<'a> sqlx::Decode<'a, Postgres> + Type<Postgres> + sqlx::postgres::PgHasArrayType,
{
    match row.try_get::<Option<Vec<T>>, _>(idx) {
        Ok(Some(vec)) => {
            let values: Vec<ExtractedValue> = vec.into_iter()
                .map(convert)
                .collect();
            Ok(ExtractedValue::Array(values))
        }
        Ok(None) => Ok(ExtractedValue::Null),
        Err(e) => Err(DataBridgeError::Query(
            format!("Failed to extract array from column '{}': {}", column_name, e)
        )),
    }
}

/// Helper function to convert ExtractedValue to JSON for array binding.
fn extracted_to_json(value: &ExtractedValue) -> Result<JsonValue> {
    Ok(match value {
        ExtractedValue::Null => JsonValue::Null,
        ExtractedValue::Bool(v) => JsonValue::Bool(*v),
        ExtractedValue::SmallInt(v) => JsonValue::Number((*v).into()),
        ExtractedValue::Int(v) => JsonValue::Number((*v).into()),
        ExtractedValue::BigInt(v) => JsonValue::Number((*v).into()),
        ExtractedValue::Float(v) => {
            serde_json::Number::from_f64(*v as f64)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null)
        }
        ExtractedValue::Double(v) => {
            serde_json::Number::from_f64(*v)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null)
        }
        ExtractedValue::String(v) => JsonValue::String(v.clone()),
        ExtractedValue::Bytes(v) => {
            // Encode bytes as hex string
            let hex_string = v.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            JsonValue::String(hex_string)
        }
        ExtractedValue::Uuid(v) => JsonValue::String(v.to_string()),
        ExtractedValue::Date(v) => JsonValue::String(v.to_string()),
        ExtractedValue::Time(v) => JsonValue::String(v.to_string()),
        ExtractedValue::Timestamp(v) => JsonValue::String(v.to_string()),
        ExtractedValue::TimestampTz(v) => JsonValue::String(v.to_rfc3339()),
        ExtractedValue::Json(v) => v.clone(),
        ExtractedValue::Array(values) => {
            let json_values: Vec<JsonValue> = values
                .iter()
                .map(|v| extracted_to_json(v))
                .collect::<Result<Vec<_>>>()?;
            JsonValue::Array(json_values)
        }
        ExtractedValue::Decimal(v) => JsonValue::String(v.clone()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Arguments;

    #[test]
    fn test_extracted_value_type_names() {
        assert_eq!(ExtractedValue::Null.pg_type_name(), "NULL");
        assert_eq!(ExtractedValue::Bool(true).pg_type_name(), "BOOLEAN");
        assert_eq!(ExtractedValue::SmallInt(42).pg_type_name(), "SMALLINT");
        assert_eq!(ExtractedValue::Int(42).pg_type_name(), "INTEGER");
        assert_eq!(ExtractedValue::BigInt(42).pg_type_name(), "BIGINT");
        assert_eq!(ExtractedValue::Float(3.14).pg_type_name(), "REAL");
        assert_eq!(ExtractedValue::Double(3.14).pg_type_name(), "DOUBLE PRECISION");
        assert_eq!(ExtractedValue::String("test".to_string()).pg_type_name(), "TEXT");
        assert_eq!(ExtractedValue::Bytes(vec![1, 2, 3]).pg_type_name(), "BYTEA");
        assert_eq!(ExtractedValue::Uuid(Uuid::nil()).pg_type_name(), "UUID");
        assert_eq!(ExtractedValue::Json(JsonValue::Null).pg_type_name(), "JSONB");
        assert_eq!(ExtractedValue::Array(vec![]).pg_type_name(), "ARRAY");
        assert_eq!(ExtractedValue::Decimal("123.45".to_string()).pg_type_name(), "NUMERIC");
    }

    #[test]
    fn test_bind_to_arguments() {
        let mut args = PgArguments::default();

        // Test binding various types
        let value = ExtractedValue::Int(42);
        assert!(value.bind_to_arguments(&mut args).is_ok());

        let value = ExtractedValue::String("test".to_string());
        assert!(value.bind_to_arguments(&mut args).is_ok());

        let value = ExtractedValue::Bool(true);
        assert!(value.bind_to_arguments(&mut args).is_ok());

        let value = ExtractedValue::Null;
        assert!(value.bind_to_arguments(&mut args).is_ok());

        let value = ExtractedValue::Uuid(Uuid::nil());
        assert!(value.bind_to_arguments(&mut args).is_ok());
    }

    #[test]
    fn test_extracted_to_json() {
        use serde_json::json;

        // Test basic types
        let result = extracted_to_json(&ExtractedValue::Null).unwrap();
        assert_eq!(result, json!(null));

        let result = extracted_to_json(&ExtractedValue::Bool(true)).unwrap();
        assert_eq!(result, json!(true));

        let result = extracted_to_json(&ExtractedValue::Int(42)).unwrap();
        assert_eq!(result, json!(42));

        let result = extracted_to_json(&ExtractedValue::String("test".to_string())).unwrap();
        assert_eq!(result, json!("test"));

        // Test bytes to hex conversion
        let result = extracted_to_json(&ExtractedValue::Bytes(vec![0xff, 0x00, 0xab])).unwrap();
        assert_eq!(result, json!("ff00ab"));

        // Test nested arrays
        let result = extracted_to_json(&ExtractedValue::Array(vec![
            ExtractedValue::Int(1),
            ExtractedValue::Int(2),
            ExtractedValue::Int(3),
        ])).unwrap();
        assert_eq!(result, json!([1, 2, 3]));
    }

    #[test]
    fn test_bind_array_values() {
        let mut args = PgArguments::default();

        // Empty array
        let value = ExtractedValue::Array(vec![]);
        assert!(value.bind_to_arguments(&mut args).is_ok());

        // Array with values (will be converted to JSON)
        let value = ExtractedValue::Array(vec![
            ExtractedValue::Int(1),
            ExtractedValue::Int(2),
        ]);
        assert!(value.bind_to_arguments(&mut args).is_ok());
    }
}
