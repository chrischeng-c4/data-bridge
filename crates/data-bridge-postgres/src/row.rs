//! PostgreSQL row representation.
//!
//! This module provides a row abstraction for query results,
//! similar to data-bridge-mongodb's document handling.
//!
//! # Examples
//!
//! ## Insert a row
//!
//! ```ignore
//! use data_bridge_postgres::{Connection, ExtractedValue, PoolConfig, Row};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let conn = Connection::new("postgresql://localhost/mydb", PoolConfig::default()).await?;
//! let pool = conn.pool();
//!
//! let values = vec![
//!     ("name".to_string(), ExtractedValue::String("Alice".to_string())),
//!     ("age".to_string(), ExtractedValue::Int(30)),
//! ];
//!
//! let row = Row::insert(pool, "users", &values).await?;
//! let id = row.get("id")?; // Auto-generated ID
//! # Ok(())
//! # }
//! ```
//!
//! ## Find rows
//!
//! ```ignore
//! use data_bridge_postgres::{QueryBuilder, Operator, ExtractedValue, Row};
//!
//! # async fn example(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! // Find by ID
//! let row = Row::find_by_id(pool, "users", 42).await?;
//!
//! // Find with filters
//! let query = QueryBuilder::new("users")?
//!     .where_clause("age", Operator::Gte, ExtractedValue::Int(18))?
//!     .limit(10);
//! let rows = Row::find_many(pool, "users", Some(&query)).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Batch insert
//!
//! ```ignore
//! use std::collections::HashMap;
//! use data_bridge_postgres::{Connection, ExtractedValue, PoolConfig, Row};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let conn = Connection::new("postgresql://localhost/mydb", PoolConfig::default()).await?;
//! let pool = conn.pool();
//!
//! let mut row1 = HashMap::new();
//! row1.insert("name".to_string(), ExtractedValue::String("Alice".to_string()));
//! row1.insert("age".to_string(), ExtractedValue::Int(30));
//!
//! let mut row2 = HashMap::new();
//! row2.insert("name".to_string(), ExtractedValue::String("Bob".to_string()));
//! row2.insert("age".to_string(), ExtractedValue::Int(25));
//!
//! // Batch insert is much faster than individual inserts
//! let rows = Row::insert_many(pool, "users", &[row1, row2]).await?;
//! assert_eq!(rows.len(), 2);
//! # Ok(())
//! # }
//! ```
//!
//! ## Update and delete
//!
//! ```ignore
//! use data_bridge_postgres::{ExtractedValue, Row};
//!
//! # async fn example(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error>> {
//! // Update
//! let updates = vec![
//!     ("name".to_string(), ExtractedValue::String("Bob".to_string())),
//! ];
//! Row::update(pool, "users", 42, &updates).await?;
//!
//! // Delete
//! Row::delete(pool, "users", 42).await?;
//! # Ok(())
//! # }
//! ```

use serde_json::Value as JsonValue;
use sqlx::postgres::{PgArguments, PgPool};
use sqlx::{Arguments, Row as SqlxRow};
use std::collections::HashMap;

use crate::{DataBridgeError, ExtractedValue, QueryBuilder, Result, row_to_extracted};

/// Represents a single row from a PostgreSQL query result.
///
/// This is the primary data structure returned from queries.
/// It wraps column names and values in a type-safe manner.
#[derive(Debug, Clone)]
pub struct Row {
    /// Column name to value mapping
    columns: HashMap<String, ExtractedValue>,
}

impl Row {
    /// Creates a new row from a column map.
    pub fn new(columns: HashMap<String, ExtractedValue>) -> Self {
        Self { columns }
    }

    /// Gets a value by column name.
    ///
    /// # Arguments
    ///
    /// * `column` - Column name
    ///
    /// # Errors
    ///
    /// Returns error if column doesn't exist.
    pub fn get(&self, column: &str) -> Result<&ExtractedValue> {
        self.columns
            .get(column)
            .ok_or_else(|| DataBridgeError::Query(format!("Column '{}' not found", column)))
    }

    /// Gets all column names.
    pub fn columns(&self) -> Vec<&str> {
        self.columns.keys().map(|s| s.as_str()).collect()
    }

    /// Gets a reference to the column map.
    pub fn columns_map(&self) -> &HashMap<String, ExtractedValue> {
        &self.columns
    }

    /// Converts row to a JSON object.
    pub fn to_json(&self) -> Result<JsonValue> {
        let mut map = serde_json::Map::new();
        for (key, value) in &self.columns {
            let json_value = extracted_value_to_json(value)?;
            map.insert(key.clone(), json_value);
        }
        Ok(JsonValue::Object(map))
    }

    /// Converts row to Python dict.
    pub fn to_python(/* &self, py: Python */) -> Result<()> {
        // TODO: Implement row to Python dict conversion
        // - Create Python dict
        // - For each column, convert ExtractedValue to Python object
        // - Set dict items
        // - Return PyDict
        todo!("Implement Row::to_python - requires PyO3 integration")
    }

    /// Converts from SQLx row.
    pub fn from_sqlx(row: &sqlx::postgres::PgRow) -> Result<Self> {
        let columns = row_to_extracted(row)?;
        Ok(Self { columns })
    }

    /// Insert row into database, return generated ID.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `table` - Table name
    /// * `values` - Column name -> value mapping
    ///
    /// # Errors
    ///
    /// Returns error if insert fails or table is invalid.
    ///
    /// # Returns
    ///
    /// Returns the inserted row with all columns (including generated ID).
    pub async fn insert(
        pool: &PgPool,
        table: &str,
        values: &[(String, ExtractedValue)],
    ) -> Result<Self> {
        if values.is_empty() {
            return Err(DataBridgeError::Query("Cannot insert with no values".to_string()));
        }

        // Build INSERT query with RETURNING *
        let query_builder = QueryBuilder::new(table)?;
        let (sql, params) = query_builder.build_insert(values)?;

        // Bind parameters
        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        // Execute query with bound arguments
        let row = sqlx::query_with(&sql, args)
            .fetch_one(pool)
            .await
            .map_err(|e| DataBridgeError::Query(format!("Insert failed: {}", e)))?;

        // Convert PgRow to Row
        Self::from_sqlx(&row)
    }

    /// Insert multiple rows with a single batch INSERT statement.
    ///
    /// This is much faster than individual inserts for large batches because
    /// it generates a single INSERT with multiple VALUES clauses:
    /// `INSERT INTO table (col1, col2) VALUES ($1, $2), ($3, $4), ... RETURNING *`
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `table` - Table name
    /// * `rows` - Vector of rows, where each row is a HashMap of column -> value
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Any row is empty
    /// - Rows have different columns
    /// - Insert fails
    /// - Table is invalid
    ///
    /// # Returns
    ///
    /// Returns vector of inserted rows with all columns (including generated IDs).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::collections::HashMap;
    /// use data_bridge_postgres::{Connection, ExtractedValue, PoolConfig, Row};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let conn = Connection::new("postgresql://localhost/mydb", PoolConfig::default()).await?;
    /// let pool = conn.pool();
    ///
    /// let mut row1 = HashMap::new();
    /// row1.insert("name".to_string(), ExtractedValue::String("Alice".to_string()));
    /// row1.insert("age".to_string(), ExtractedValue::Int(30));
    ///
    /// let mut row2 = HashMap::new();
    /// row2.insert("name".to_string(), ExtractedValue::String("Bob".to_string()));
    /// row2.insert("age".to_string(), ExtractedValue::Int(25));
    ///
    /// let rows = Row::insert_many(pool, "users", &[row1, row2]).await?;
    /// assert_eq!(rows.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn insert_many(
        pool: &PgPool,
        table: &str,
        rows: &[HashMap<String, ExtractedValue>],
    ) -> Result<Vec<Self>> {
        if rows.is_empty() {
            return Ok(vec![]);
        }

        // Get column names from first row and validate
        let first_row = &rows[0];
        if first_row.is_empty() {
            return Err(DataBridgeError::Query("Cannot insert with no columns".to_string()));
        }

        // Collect and sort column names for consistent ordering
        let mut column_names: Vec<&String> = first_row.keys().collect();
        column_names.sort();

        // Validate all rows have the same columns
        for (idx, row) in rows.iter().enumerate().skip(1) {
            let mut row_columns: Vec<&String> = row.keys().collect();
            row_columns.sort();
            if row_columns != column_names {
                return Err(DataBridgeError::Query(format!(
                    "Row {} has different columns than first row. Expected: {:?}, Got: {:?}",
                    idx, column_names, row_columns
                )));
            }
        }

        // Validate table name
        let _query_builder = QueryBuilder::new(table)?;

        // Validate column names
        for col in &column_names {
            QueryBuilder::validate_identifier(col)?;
        }

        // Build batch INSERT SQL
        // INSERT INTO table (col1, col2) VALUES ($1, $2), ($3, $4), ... RETURNING *
        let mut sql = format!(
            "INSERT INTO {} ({}) VALUES ",
            table,
            column_names.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
        );

        let mut param_num = 1;
        let mut values_clauses = Vec::with_capacity(rows.len());
        let mut params = Vec::with_capacity(rows.len() * column_names.len());

        for row in rows {
            let placeholders: Vec<String> = (0..column_names.len())
                .map(|_| {
                    let p = format!("${}", param_num);
                    param_num += 1;
                    p
                })
                .collect();
            values_clauses.push(format!("({})", placeholders.join(", ")));

            // Collect parameter values in the same order as column names
            for col in &column_names {
                params.push(row.get(*col).unwrap().clone());
            }
        }

        sql.push_str(&values_clauses.join(", "));
        sql.push_str(" RETURNING *");

        // Bind all parameters
        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        // Execute query and fetch all returned rows
        let pg_rows = sqlx::query_with(&sql, args)
            .fetch_all(pool)
            .await
            .map_err(|e| DataBridgeError::Query(format!("Batch insert failed: {}", e)))?;

        // Convert all PgRows to Rows
        pg_rows.iter()
            .map(|row| Self::from_sqlx(row))
            .collect()
    }

    /// Find single row by primary key.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `table` - Table name
    /// * `id` - Primary key value
    ///
    /// # Errors
    ///
    /// Returns error if query fails.
    ///
    /// # Returns
    ///
    /// Returns Some(Row) if found, None if not found.
    pub async fn find_by_id(pool: &PgPool, table: &str, id: i64) -> Result<Option<Self>> {
        // Build SELECT * WHERE id = $1 query
        let query_builder = QueryBuilder::new(table)?
            .where_clause("id", crate::Operator::Eq, ExtractedValue::BigInt(id))?;
        let (sql, params) = query_builder.build_select();

        // Bind parameters
        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        // Execute query
        let result = sqlx::query_with(&sql, args)
            .fetch_optional(pool)
            .await
            .map_err(|e| DataBridgeError::Query(format!("Find by ID failed: {}", e)))?;

        match result {
            Some(row) => Ok(Some(Self::from_sqlx(&row)?)),
            None => Ok(None),
        }
    }

    /// Find multiple rows with filters.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `table` - Table name
    /// * `query` - Query builder with filters (optional)
    ///
    /// # Errors
    ///
    /// Returns error if query fails.
    ///
    /// # Returns
    ///
    /// Returns vector of matching rows.
    pub async fn find_many(
        pool: &PgPool,
        table: &str,
        query: Option<&QueryBuilder>,
    ) -> Result<Vec<Self>> {
        let (sql, params) = if let Some(qb) = query {
            qb.build_select()
        } else {
            let qb = QueryBuilder::new(table)?;
            qb.build_select()
        };

        // Bind parameters
        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        // Execute query
        let rows = sqlx::query_with(&sql, args)
            .fetch_all(pool)
            .await
            .map_err(|e| DataBridgeError::Query(format!("Find many failed: {}", e)))?;

        // Convert all PgRows to Rows
        rows.iter()
            .map(|row| Self::from_sqlx(row))
            .collect()
    }

    /// Update row in database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `table` - Table name
    /// * `id` - Primary key value
    /// * `values` - Column name -> value mapping for updates
    ///
    /// # Errors
    ///
    /// Returns error if update fails.
    ///
    /// # Returns
    ///
    /// Returns true if row was updated, false if not found.
    pub async fn update(
        pool: &PgPool,
        table: &str,
        id: i64,
        values: &[(String, ExtractedValue)],
    ) -> Result<bool> {
        if values.is_empty() {
            return Err(DataBridgeError::Query("Cannot update with no values".to_string()));
        }

        // Build UPDATE SET ... WHERE id = $N query
        let query_builder = QueryBuilder::new(table)?
            .where_clause("id", crate::Operator::Eq, ExtractedValue::BigInt(id))?;
        let (sql, params) = query_builder.build_update(values)?;

        // Bind parameters
        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        // Execute query
        let result = sqlx::query_with(&sql, args)
            .execute(pool)
            .await
            .map_err(|e| DataBridgeError::Query(format!("Update failed: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete row from database.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `table` - Table name
    /// * `id` - Primary key value
    ///
    /// # Errors
    ///
    /// Returns error if delete fails.
    ///
    /// # Returns
    ///
    /// Returns true if row was deleted, false if not found.
    pub async fn delete(pool: &PgPool, table: &str, id: i64) -> Result<bool> {
        // Build DELETE WHERE id = $1 query
        let query_builder = QueryBuilder::new(table)?
            .where_clause("id", crate::Operator::Eq, ExtractedValue::BigInt(id))?;
        let (sql, params) = query_builder.build_delete();

        // Bind parameters
        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        // Execute query
        let result = sqlx::query_with(&sql, args)
            .execute(pool)
            .await
            .map_err(|e| DataBridgeError::Query(format!("Delete failed: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    /// Count rows matching query.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `table` - Table name
    /// * `query` - Query builder with filters (optional)
    ///
    /// # Errors
    ///
    /// Returns error if query fails.
    ///
    /// # Returns
    ///
    /// Returns count of matching rows.
    pub async fn count(
        pool: &PgPool,
        table: &str,
        query: Option<&QueryBuilder>,
    ) -> Result<i64> {
        // Build SELECT COUNT(*) query
        let mut sql = format!("SELECT COUNT(*) FROM {}", table);
        let mut params = Vec::new();

        if let Some(qb) = query {
            // Extract WHERE clause from the SELECT query
            let (select_sql, select_params) = qb.build_select();
            params = select_params;

            // Find WHERE clause in the generated SQL
            if let Some(where_pos) = select_sql.find(" WHERE ") {
                let where_clause = &select_sql[where_pos..];
                // Find the end of WHERE clause (before ORDER BY, LIMIT, etc.)
                let end_pos = where_clause
                    .find(" ORDER BY ")
                    .or_else(|| where_clause.find(" LIMIT "))
                    .or_else(|| where_clause.find(" OFFSET "))
                    .unwrap_or(where_clause.len());
                sql.push_str(&where_clause[..end_pos]);
            }
        }

        // Bind parameters
        let mut args = PgArguments::default();
        for param in &params {
            param.bind_to_arguments(&mut args)?;
        }

        // Execute query
        let row = sqlx::query_with(&sql, args)
            .fetch_one(pool)
            .await
            .map_err(|e| DataBridgeError::Query(format!("Count failed: {}", e)))?;

        let count: i64 = row.try_get(0)
            .map_err(|e| DataBridgeError::Query(format!("Failed to extract count: {}", e)))?;

        Ok(count)
    }
}

/// Helper function to convert ExtractedValue to JSON.
fn extracted_value_to_json(value: &ExtractedValue) -> Result<JsonValue> {
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
                .map(|v| extracted_value_to_json(v))
                .collect::<Result<Vec<_>>>()?;
            JsonValue::Array(json_values)
        }
        ExtractedValue::Decimal(v) => JsonValue::String(v.clone()),
    })
}
