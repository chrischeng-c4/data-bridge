//! PostgreSQL query builder.
//!
//! This module provides a type-safe query builder for constructing SQL queries,
//! similar to data-bridge-mongodb's query builder but for SQL.
//!
//! # Examples
//!
//! ## SELECT Query
//!
//! ```ignore
//! use data_bridge_postgres::{QueryBuilder, Operator, OrderDirection, ExtractedValue};
//!
//! let qb = QueryBuilder::new("users")?
//!     .select(vec!["id".to_string(), "name".to_string()])?
//!     .where_clause("age", Operator::Gte, ExtractedValue::Int(18))?
//!     .where_clause("active", Operator::Eq, ExtractedValue::Bool(true))?
//!     .order_by("name", OrderDirection::Asc)?
//!     .limit(10)
//!     .offset(20);
//!
//! let (sql, params) = qb.build();
//! // Result: "SELECT id, name FROM users WHERE age >= $1 AND active = $2 ORDER BY name ASC LIMIT $3 OFFSET $4"
//! ```
//!
//! ## INSERT Query
//!
//! ```ignore
//! use data_bridge_postgres::{QueryBuilder, ExtractedValue};
//!
//! let qb = QueryBuilder::new("users")?;
//! let values = vec![
//!     ("name".to_string(), ExtractedValue::String("Alice".to_string())),
//!     ("age".to_string(), ExtractedValue::Int(30)),
//! ];
//! let (sql, params) = qb.build_insert(&values)?;
//! // Result: "INSERT INTO users (name, age) VALUES ($1, $2) RETURNING *"
//! ```
//!
//! ## UPDATE Query
//!
//! ```ignore
//! use data_bridge_postgres::{QueryBuilder, Operator, ExtractedValue};
//!
//! let qb = QueryBuilder::new("users")?
//!     .where_clause("id", Operator::Eq, ExtractedValue::Int(42))?;
//! let values = vec![
//!     ("name".to_string(), ExtractedValue::String("Bob".to_string())),
//!     ("age".to_string(), ExtractedValue::Int(35)),
//! ];
//! let (sql, params) = qb.build_update(&values)?;
//! // Result: "UPDATE users SET name = $1, age = $2 WHERE id = $3"
//! ```
//!
//! ## DELETE Query
//!
//! ```ignore
//! use data_bridge_postgres::{QueryBuilder, Operator, ExtractedValue};
//!
//! let qb = QueryBuilder::new("users")?
//!     .where_clause("id", Operator::Eq, ExtractedValue::Int(42))?;
//! let (sql, params) = qb.build_delete();
//! // Result: "DELETE FROM users WHERE id = $1"
//! ```

use crate::{Connection, DataBridgeError, ExtractedValue, Result, Row};

/// Query comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// Equal (=)
    Eq,
    /// Not equal (!=)
    Ne,
    /// Greater than (>)
    Gt,
    /// Greater than or equal (>=)
    Gte,
    /// Less than (<)
    Lt,
    /// Less than or equal (<=)
    Lte,
    /// IN clause
    In,
    /// NOT IN clause
    NotIn,
    /// LIKE pattern matching
    Like,
    /// ILIKE case-insensitive pattern matching
    ILike,
    /// IS NULL
    IsNull,
    /// IS NOT NULL
    IsNotNull,
}

impl Operator {
    /// Returns the SQL operator string.
    pub fn to_sql(&self) -> &'static str {
        match self {
            Operator::Eq => "=",
            Operator::Ne => "!=",
            Operator::Gt => ">",
            Operator::Gte => ">=",
            Operator::Lt => "<",
            Operator::Lte => "<=",
            Operator::In => "IN",
            Operator::NotIn => "NOT IN",
            Operator::Like => "LIKE",
            Operator::ILike => "ILIKE",
            Operator::IsNull => "IS NULL",
            Operator::IsNotNull => "IS NOT NULL",
        }
    }
}

/// Sort order direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderDirection {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

impl OrderDirection {
    /// Returns the SQL order direction string.
    pub fn to_sql(&self) -> &'static str {
        match self {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    }
}

/// Type-safe SQL query builder.
///
/// Provides a fluent API for constructing SELECT, INSERT, UPDATE, and DELETE queries
/// with parameter binding and security validation.
#[derive(Debug)]
pub struct QueryBuilder {
    table: String,
    /// SELECT columns (empty means SELECT *)
    select_columns: Vec<String>,
    /// WHERE conditions (field, operator, value)
    where_conditions: Vec<WhereCondition>,
    /// ORDER BY clauses (field, direction)
    order_by_clauses: Vec<(String, OrderDirection)>,
    /// LIMIT clause
    limit_value: Option<i64>,
    /// OFFSET clause
    offset_value: Option<i64>,
}

/// Represents a WHERE condition.
#[derive(Debug, Clone)]
struct WhereCondition {
    field: String,
    operator: Operator,
    value: Option<ExtractedValue>, // None for IS NULL / IS NOT NULL
}

impl QueryBuilder {
    /// Creates a new query builder for a table.
    ///
    /// # Arguments
    ///
    /// * `table` - Table name (validated for SQL injection)
    ///
    /// # Errors
    ///
    /// Returns error if table name is invalid.
    pub fn new(table: &str) -> Result<Self> {
        Self::validate_identifier(table)?;
        Ok(Self {
            table: table.to_string(),
            select_columns: Vec::new(),
            where_conditions: Vec::new(),
            order_by_clauses: Vec::new(),
            limit_value: None,
            offset_value: None,
        })
    }

    /// Specifies which columns to SELECT.
    ///
    /// # Arguments
    ///
    /// * `columns` - Column names to select
    pub fn select(mut self, columns: Vec<String>) -> Result<Self> {
        for col in &columns {
            Self::validate_identifier(col)?;
        }
        self.select_columns = columns;
        Ok(self)
    }

    /// Adds a WHERE condition.
    ///
    /// # Arguments
    ///
    /// * `field` - Column name
    /// * `operator` - Comparison operator
    /// * `value` - Value to compare against
    pub fn where_clause(mut self, field: &str, operator: Operator, value: ExtractedValue) -> Result<Self> {
        Self::validate_identifier(field)?;

        // For IS NULL and IS NOT NULL, we don't need a value
        let condition_value = match operator {
            Operator::IsNull | Operator::IsNotNull => None,
            _ => Some(value),
        };

        self.where_conditions.push(WhereCondition {
            field: field.to_string(),
            operator,
            value: condition_value,
        });
        Ok(self)
    }

    /// Adds a WHERE condition for IS NULL.
    pub fn where_null(self, field: &str) -> Result<Self> {
        self.where_clause(field, Operator::IsNull, ExtractedValue::Null)
    }

    /// Adds a WHERE condition for IS NOT NULL.
    pub fn where_not_null(self, field: &str) -> Result<Self> {
        self.where_clause(field, Operator::IsNotNull, ExtractedValue::Null)
    }

    /// Adds an ORDER BY clause.
    pub fn order_by(mut self, field: &str, direction: OrderDirection) -> Result<Self> {
        Self::validate_identifier(field)?;
        self.order_by_clauses.push((field.to_string(), direction));
        Ok(self)
    }

    /// Sets LIMIT.
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit_value = Some(limit);
        self
    }

    /// Sets OFFSET.
    pub fn offset(mut self, offset: i64) -> Self {
        self.offset_value = Some(offset);
        self
    }

    /// Builds a SELECT SQL query string with parameter placeholders.
    ///
    /// Returns the SQL string with $1, $2, etc. placeholders.
    pub fn build_select(&self) -> (String, Vec<ExtractedValue>) {
        let mut sql = String::from("SELECT ");

        // SELECT columns or *
        if self.select_columns.is_empty() {
            sql.push('*');
        } else {
            sql.push_str(&self.select_columns.join(", "));
        }

        sql.push_str(" FROM ");
        sql.push_str(&self.table);

        // WHERE clause
        let mut params = Vec::new();
        if !self.where_conditions.is_empty() {
            sql.push_str(" WHERE ");
            let where_parts: Vec<String> = self.where_conditions.iter().map(|cond| {
                match cond.operator {
                    Operator::IsNull | Operator::IsNotNull => {
                        format!("{} {}", cond.field, cond.operator.to_sql())
                    }
                    Operator::In | Operator::NotIn => {
                        if let Some(ref value) = cond.value {
                            params.push(value.clone());
                            format!("{} {} (${})", cond.field, cond.operator.to_sql(), params.len())
                        } else {
                            format!("{} {} (NULL)", cond.field, cond.operator.to_sql())
                        }
                    }
                    _ => {
                        if let Some(ref value) = cond.value {
                            params.push(value.clone());
                            format!("{} {} ${}", cond.field, cond.operator.to_sql(), params.len())
                        } else {
                            format!("{} {} NULL", cond.field, cond.operator.to_sql())
                        }
                    }
                }
            }).collect();
            sql.push_str(&where_parts.join(" AND "));
        }

        // ORDER BY clause
        if !self.order_by_clauses.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_parts: Vec<String> = self.order_by_clauses.iter()
                .map(|(field, dir)| format!("{} {}", field, dir.to_sql()))
                .collect();
            sql.push_str(&order_parts.join(", "));
        }

        // LIMIT clause
        if let Some(limit) = self.limit_value {
            params.push(ExtractedValue::BigInt(limit));
            sql.push_str(&format!(" LIMIT ${}", params.len()));
        }

        // OFFSET clause
        if let Some(offset) = self.offset_value {
            params.push(ExtractedValue::BigInt(offset));
            sql.push_str(&format!(" OFFSET ${}", params.len()));
        }

        (sql, params)
    }

    /// Builds an INSERT SQL query string with parameter placeholders.
    ///
    /// Returns the SQL string with $1, $2, etc. placeholders and the parameter values.
    pub fn build_insert(&self, values: &[(String, ExtractedValue)]) -> Result<(String, Vec<ExtractedValue>)> {
        if values.is_empty() {
            return Err(DataBridgeError::Query("Cannot insert with no values".to_string()));
        }

        // Validate column names
        for (col, _) in values {
            Self::validate_identifier(col)?;
        }

        let mut sql = format!("INSERT INTO {} (", self.table);
        let columns: Vec<&str> = values.iter().map(|(col, _)| col.as_str()).collect();
        sql.push_str(&columns.join(", "));
        sql.push_str(") VALUES (");

        let placeholders: Vec<String> = (1..=values.len()).map(|i| format!("${}", i)).collect();
        sql.push_str(&placeholders.join(", "));
        sql.push_str(") RETURNING *");

        let params: Vec<ExtractedValue> = values.iter().map(|(_, val)| val.clone()).collect();

        Ok((sql, params))
    }

    /// Builds an UPDATE SQL query string with parameter placeholders.
    ///
    /// Returns the SQL string with $1, $2, etc. placeholders and the parameter values.
    pub fn build_update(&self, values: &[(String, ExtractedValue)]) -> Result<(String, Vec<ExtractedValue>)> {
        if values.is_empty() {
            return Err(DataBridgeError::Query("Cannot update with no values".to_string()));
        }

        // Validate column names
        for (col, _) in values {
            Self::validate_identifier(col)?;
        }

        let mut sql = format!("UPDATE {} SET ", self.table);
        let mut params: Vec<ExtractedValue> = Vec::new();

        // SET clause
        let set_parts: Vec<String> = values.iter().map(|(col, val)| {
            params.push(val.clone());
            format!("{} = ${}", col, params.len())
        }).collect();
        sql.push_str(&set_parts.join(", "));

        // WHERE clause
        if !self.where_conditions.is_empty() {
            sql.push_str(" WHERE ");
            let where_parts: Vec<String> = self.where_conditions.iter().map(|cond| {
                match cond.operator {
                    Operator::IsNull | Operator::IsNotNull => {
                        format!("{} {}", cond.field, cond.operator.to_sql())
                    }
                    _ => {
                        if let Some(ref value) = cond.value {
                            params.push(value.clone());
                            format!("{} {} ${}", cond.field, cond.operator.to_sql(), params.len())
                        } else {
                            format!("{} {} NULL", cond.field, cond.operator.to_sql())
                        }
                    }
                }
            }).collect();
            sql.push_str(&where_parts.join(" AND "));
        }

        Ok((sql, params))
    }

    /// Builds a DELETE SQL query string with parameter placeholders.
    ///
    /// Returns the SQL string with $1, $2, etc. placeholders and the parameter values.
    pub fn build_delete(&self) -> (String, Vec<ExtractedValue>) {
        let mut sql = format!("DELETE FROM {}", self.table);
        let mut params: Vec<ExtractedValue> = Vec::new();

        // WHERE clause
        if !self.where_conditions.is_empty() {
            sql.push_str(" WHERE ");
            let where_parts: Vec<String> = self.where_conditions.iter().map(|cond| {
                match cond.operator {
                    Operator::IsNull | Operator::IsNotNull => {
                        format!("{} {}", cond.field, cond.operator.to_sql())
                    }
                    _ => {
                        if let Some(ref value) = cond.value {
                            params.push(value.clone());
                            format!("{} {} ${}", cond.field, cond.operator.to_sql(), params.len())
                        } else {
                            format!("{} {} NULL", cond.field, cond.operator.to_sql())
                        }
                    }
                }
            }).collect();
            sql.push_str(&where_parts.join(" AND "));
        }

        (sql, params)
    }

    /// Builds and executes a SELECT query.
    pub async fn execute_select(&self, _conn: &Connection) -> Result<Vec<Row>> {
        // TODO: Implement SELECT query execution with SQLx
        // This requires implementing Row::from_sqlx and binding ExtractedValue to SQLx parameters
        todo!("Implement QueryBuilder::execute_select - requires SQLx integration")
    }

    /// Builds and executes an INSERT query.
    pub async fn execute_insert(&self, _conn: &Connection, _values: Vec<(String, ExtractedValue)>) -> Result<Row> {
        // TODO: Implement INSERT query execution with SQLx
        todo!("Implement QueryBuilder::execute_insert - requires SQLx integration")
    }

    /// Builds and executes an UPDATE query.
    pub async fn execute_update(&self, _conn: &Connection, _values: Vec<(String, ExtractedValue)>) -> Result<u64> {
        // TODO: Implement UPDATE query execution with SQLx
        todo!("Implement QueryBuilder::execute_update - requires SQLx integration")
    }

    /// Builds and executes a DELETE query.
    pub async fn execute_delete(&self, _conn: &Connection) -> Result<u64> {
        // TODO: Implement DELETE query execution with SQLx
        todo!("Implement QueryBuilder::execute_delete - requires SQLx integration")
    }

    /// Validates a SQL identifier (table/column name).
    ///
    /// Supports both simple identifiers and schema-qualified names (e.g., "public.users").
    pub fn validate_identifier(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(DataBridgeError::Query("Identifier cannot be empty".to_string()));
        }

        // Check if this is a schema-qualified name (e.g., "public.users")
        if name.contains('.') {
            let parts: Vec<&str> = name.split('.').collect();

            // Only allow schema.table format (two parts)
            if parts.len() != 2 {
                return Err(DataBridgeError::Query(
                    format!("Invalid schema-qualified identifier '{}': must be in format 'schema.table'", name)
                ));
            }

            // Validate each part separately
            for part in parts {
                Self::validate_identifier_part(part)?;
            }

            return Ok(());
        }

        // Simple identifier - validate as a single part
        Self::validate_identifier_part(name)
    }

    /// Validates a single part of an identifier (no dots allowed).
    fn validate_identifier_part(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(DataBridgeError::Query("Identifier part cannot be empty".to_string()));
        }

        // Check length (PostgreSQL limit is 63 bytes per part)
        if name.len() > 63 {
            return Err(DataBridgeError::Query(
                format!("Identifier '{}' exceeds maximum length of 63", name)
            ));
        }

        // Must start with letter or underscore
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return Err(DataBridgeError::Query(
                format!("Identifier '{}' must start with a letter or underscore", name)
            ));
        }

        // Rest must be alphanumeric or underscore
        for ch in name.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '_' {
                return Err(DataBridgeError::Query(
                    format!("Identifier '{}' contains invalid character '{}'", name, ch)
                ));
            }
        }

        // Prevent system schema access
        let name_lower = name.to_lowercase();
        if name_lower.starts_with("pg_") {
            return Err(DataBridgeError::Query(
                format!("Access to PostgreSQL system catalog '{}' is not allowed", name)
            ));
        }

        if name_lower == "information_schema" {
            return Err(DataBridgeError::Query(
                "Access to information_schema is not allowed".to_string()
            ));
        }

        // Prevent SQL keywords
        const SQL_KEYWORDS: &[&str] = &[
            "select", "insert", "update", "delete", "drop", "create", "alter",
            "truncate", "grant", "revoke", "exec", "execute", "union", "declare",
            "table", "index", "view", "schema", "database", "user", "role",
            "from", "where", "join", "inner", "outer", "left", "right",
            "on", "using", "and", "or", "not", "in", "exists", "between",
            "like", "ilike", "is", "null", "true", "false", "case", "when",
            "then", "else", "end", "as", "order", "by", "group", "having",
            "limit", "offset", "distinct", "all", "any", "some",
        ];

        if SQL_KEYWORDS.contains(&name_lower.as_str()) {
            return Err(DataBridgeError::Query(
                format!("Identifier '{}' is a reserved SQL keyword", name)
            ));
        }

        Ok(())
    }

    /// Builds a query and returns (SQL, parameters) tuple.
    ///
    /// This is a convenience method for SELECT queries.
    pub fn build(&self) -> (String, Vec<ExtractedValue>) {
        self.build_select()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_select() {
        let qb = QueryBuilder::new("users").unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM users");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_select_with_columns() {
        let qb = QueryBuilder::new("users").unwrap()
            .select(vec!["id".to_string(), "name".to_string()]).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT id, name FROM users");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_select_with_where() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("id", Operator::Eq, ExtractedValue::Int(42)).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM users WHERE id = $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_select_with_multiple_where() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("age", Operator::Gt, ExtractedValue::Int(18)).unwrap()
            .where_clause("status", Operator::Eq, ExtractedValue::String("active".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM users WHERE age > $1 AND status = $2");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_select_with_order_by() {
        let qb = QueryBuilder::new("users").unwrap()
            .order_by("created_at", OrderDirection::Desc).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM users ORDER BY created_at DESC");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_select_with_limit_offset() {
        let qb = QueryBuilder::new("users").unwrap()
            .limit(10)
            .offset(20);
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM users LIMIT $1 OFFSET $2");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_complex_select() {
        let qb = QueryBuilder::new("users").unwrap()
            .select(vec!["id".to_string(), "name".to_string(), "email".to_string()]).unwrap()
            .where_clause("age", Operator::Gte, ExtractedValue::Int(18)).unwrap()
            .where_clause("active", Operator::Eq, ExtractedValue::Bool(true)).unwrap()
            .order_by("name", OrderDirection::Asc).unwrap()
            .limit(50)
            .offset(100);
        let (sql, params) = qb.build_select();
        assert_eq!(
            sql,
            "SELECT id, name, email FROM users WHERE age >= $1 AND active = $2 ORDER BY name ASC LIMIT $3 OFFSET $4"
        );
        assert_eq!(params.len(), 4);
    }

    #[test]
    fn test_insert_query() {
        let qb = QueryBuilder::new("users").unwrap();
        let values = vec![
            ("name".to_string(), ExtractedValue::String("Alice".to_string())),
            ("age".to_string(), ExtractedValue::Int(30)),
        ];
        let (sql, params) = qb.build_insert(&values).unwrap();
        assert_eq!(sql, "INSERT INTO users (name, age) VALUES ($1, $2) RETURNING *");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_update_query() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("id", Operator::Eq, ExtractedValue::Int(42)).unwrap();
        let values = vec![
            ("name".to_string(), ExtractedValue::String("Bob".to_string())),
            ("age".to_string(), ExtractedValue::Int(35)),
        ];
        let (sql, params) = qb.build_update(&values).unwrap();
        assert_eq!(sql, "UPDATE users SET name = $1, age = $2 WHERE id = $3");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_delete_query() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("id", Operator::Eq, ExtractedValue::Int(42)).unwrap();
        let (sql, params) = qb.build_delete();
        assert_eq!(sql, "DELETE FROM users WHERE id = $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_where_is_null() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_null("email").unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM users WHERE email IS NULL");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_where_is_not_null() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_not_null("email").unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM users WHERE email IS NOT NULL");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_validate_identifier_valid() {
        assert!(QueryBuilder::validate_identifier("users").is_ok());
        assert!(QueryBuilder::validate_identifier("user_table").is_ok());
        assert!(QueryBuilder::validate_identifier("_private").is_ok());
        assert!(QueryBuilder::validate_identifier("table123").is_ok());
    }

    #[test]
    fn test_validate_identifier_invalid() {
        // Empty
        assert!(QueryBuilder::validate_identifier("").is_err());

        // Starts with number
        assert!(QueryBuilder::validate_identifier("123table").is_err());

        // Contains special characters
        assert!(QueryBuilder::validate_identifier("user-table").is_err());
        assert!(QueryBuilder::validate_identifier("user$table").is_err());

        // SQL keywords
        assert!(QueryBuilder::validate_identifier("select").is_err());
        assert!(QueryBuilder::validate_identifier("drop").is_err());
        assert!(QueryBuilder::validate_identifier("DELETE").is_err());

        // System catalogs
        assert!(QueryBuilder::validate_identifier("pg_catalog").is_err());
        assert!(QueryBuilder::validate_identifier("information_schema").is_err());

        // Too long (>63 characters)
        let long_name = "a".repeat(64);
        assert!(QueryBuilder::validate_identifier(&long_name).is_err());
    }

    #[test]
    fn test_validate_schema_qualified_identifiers() {
        // Valid schema-qualified names
        assert!(QueryBuilder::validate_identifier("public.users").is_ok());
        assert!(QueryBuilder::validate_identifier("public.bench_insert_one_db").is_ok());
        assert!(QueryBuilder::validate_identifier("myschema.mytable").is_ok());
        assert!(QueryBuilder::validate_identifier("_private._internal").is_ok());

        // Invalid: too many dots
        assert!(QueryBuilder::validate_identifier("schema.table.column").is_err());
        assert!(QueryBuilder::validate_identifier("a.b.c.d").is_err());

        // Invalid: empty parts
        assert!(QueryBuilder::validate_identifier(".table").is_err());
        assert!(QueryBuilder::validate_identifier("schema.").is_err());
        assert!(QueryBuilder::validate_identifier(".").is_err());

        // Invalid: system schema in qualified name
        assert!(QueryBuilder::validate_identifier("pg_catalog.users").is_err());
        assert!(QueryBuilder::validate_identifier("public.pg_internal").is_err());

        // Invalid: SQL keyword in qualified name
        assert!(QueryBuilder::validate_identifier("public.select").is_err());
        assert!(QueryBuilder::validate_identifier("drop.users").is_err());

        // Invalid: starts with number
        assert!(QueryBuilder::validate_identifier("public.123table").is_err());
        assert!(QueryBuilder::validate_identifier("123schema.table").is_err());

        // Invalid: special characters in parts
        assert!(QueryBuilder::validate_identifier("public.user-table").is_err());
        assert!(QueryBuilder::validate_identifier("my-schema.users").is_err());
    }

    #[test]
    fn test_new_with_invalid_table() {
        assert!(QueryBuilder::new("drop").is_err());
        assert!(QueryBuilder::new("pg_catalog").is_err());
        assert!(QueryBuilder::new("123table").is_err());
    }

    #[test]
    fn test_new_with_schema_qualified_table() {
        // Valid schema-qualified table names
        assert!(QueryBuilder::new("public.users").is_ok());
        assert!(QueryBuilder::new("public.bench_insert_one_db").is_ok());
        assert!(QueryBuilder::new("myschema.mytable").is_ok());

        // Test that queries work with schema-qualified names
        let qb = QueryBuilder::new("public.users").unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM public.users");
        assert_eq!(params.len(), 0);

        // Test with WHERE clause
        let qb = QueryBuilder::new("public.bench_insert_one_db").unwrap()
            .where_clause("id", Operator::Eq, ExtractedValue::Int(1)).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM public.bench_insert_one_db WHERE id = $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_operators() {
        assert_eq!(Operator::Eq.to_sql(), "=");
        assert_eq!(Operator::Ne.to_sql(), "!=");
        assert_eq!(Operator::Gt.to_sql(), ">");
        assert_eq!(Operator::Gte.to_sql(), ">=");
        assert_eq!(Operator::Lt.to_sql(), "<");
        assert_eq!(Operator::Lte.to_sql(), "<=");
        assert_eq!(Operator::In.to_sql(), "IN");
        assert_eq!(Operator::NotIn.to_sql(), "NOT IN");
        assert_eq!(Operator::Like.to_sql(), "LIKE");
        assert_eq!(Operator::ILike.to_sql(), "ILIKE");
        assert_eq!(Operator::IsNull.to_sql(), "IS NULL");
        assert_eq!(Operator::IsNotNull.to_sql(), "IS NOT NULL");
    }

    #[test]
    fn test_order_direction() {
        assert_eq!(OrderDirection::Asc.to_sql(), "ASC");
        assert_eq!(OrderDirection::Desc.to_sql(), "DESC");
    }

    #[test]
    fn test_like_operator() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("name", Operator::Like, ExtractedValue::String("%John%".to_string())).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM users WHERE name LIKE $1");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_in_operator() {
        let qb = QueryBuilder::new("users").unwrap()
            .where_clause("status", Operator::In,
                ExtractedValue::Array(vec![
                    ExtractedValue::String("active".to_string()),
                    ExtractedValue::String("pending".to_string())
                ])
            ).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM users WHERE status IN ($1)");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_empty_insert_fails() {
        let qb = QueryBuilder::new("users").unwrap();
        let values: Vec<(String, ExtractedValue)> = vec![];
        assert!(qb.build_insert(&values).is_err());
    }

    #[test]
    fn test_empty_update_fails() {
        let qb = QueryBuilder::new("users").unwrap();
        let values: Vec<(String, ExtractedValue)> = vec![];
        assert!(qb.build_update(&values).is_err());
    }

    #[test]
    fn test_insert_with_invalid_column_name() {
        let qb = QueryBuilder::new("users").unwrap();
        let values = vec![
            ("drop".to_string(), ExtractedValue::String("value".to_string())),
        ];
        assert!(qb.build_insert(&values).is_err());
    }

    #[test]
    fn test_update_with_invalid_column_name() {
        let qb = QueryBuilder::new("users").unwrap();
        let values = vec![
            ("select".to_string(), ExtractedValue::String("value".to_string())),
        ];
        assert!(qb.build_update(&values).is_err());
    }

    #[test]
    fn test_multiple_order_by() {
        let qb = QueryBuilder::new("users").unwrap()
            .order_by("created_at", OrderDirection::Desc).unwrap()
            .order_by("name", OrderDirection::Asc).unwrap();
        let (sql, params) = qb.build_select();
        assert_eq!(sql, "SELECT * FROM users ORDER BY created_at DESC, name ASC");
        assert_eq!(params.len(), 0);
    }
}
