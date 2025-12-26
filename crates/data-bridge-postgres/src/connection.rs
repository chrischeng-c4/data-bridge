//! PostgreSQL connection management with connection pooling.
//!
//! This module provides connection pooling using SQLx's built-in pool manager.
//! Similar to data-bridge-mongodb's connection management, but optimized for PostgreSQL.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

use crate::{DataBridgeError, Result};

/// Connection pool configuration.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of connections in the pool.
    pub min_connections: u32,
    /// Maximum number of connections in the pool.
    pub max_connections: u32,
    /// Connection timeout in seconds.
    pub connect_timeout: u64,
    /// Maximum lifetime of a connection in seconds.
    pub max_lifetime: Option<u64>,
    /// Idle timeout in seconds.
    pub idle_timeout: Option<u64>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            connect_timeout: 30,
            max_lifetime: Some(1800), // 30 minutes
            idle_timeout: Some(600),   // 10 minutes
        }
    }
}

/// PostgreSQL connection wrapper with connection pooling.
pub struct Connection {
    pool: PgPool,
}

impl Connection {
    /// Creates a new connection pool.
    ///
    /// # Arguments
    ///
    /// * `uri` - PostgreSQL connection URI (e.g., "postgresql://user:password@localhost/db")
    /// * `config` - Pool configuration
    ///
    /// # Errors
    ///
    /// Returns error if connection fails or URI is invalid.
    pub async fn new(uri: &str, config: PoolConfig) -> Result<Self> {
        // Validate URI format (basic check)
        if uri.is_empty() {
            return Err(DataBridgeError::Connection(
                "Connection URI cannot be empty".to_string(),
            ));
        }

        // Build pool options with configuration
        let mut pool_options = PgPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout));

        // Add optional timeouts
        if let Some(max_lifetime_secs) = config.max_lifetime {
            pool_options = pool_options.max_lifetime(Duration::from_secs(max_lifetime_secs));
        }

        if let Some(idle_timeout_secs) = config.idle_timeout {
            pool_options = pool_options.idle_timeout(Duration::from_secs(idle_timeout_secs));
        }

        // Connect to the database and create pool
        let pool = pool_options.connect(uri).await?;

        // Test the connection with a simple ping
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .map_err(|e| DataBridgeError::Connection(format!("Failed to verify connection: {}", e)))?;

        Ok(Self { pool })
    }

    /// Gets a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Closes the connection pool.
    pub async fn close(&self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }

    /// Pings the database to verify connectivity.
    pub async fn ping(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
