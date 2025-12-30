//! Database connection pool.

use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::error::DatabaseResult;

/// Default maximum number of connections in the pool.
pub const DEFAULT_MAX_CONNECTIONS: u32 = 10;

/// Default minimum number of connections in the pool.
pub const DEFAULT_MIN_CONNECTIONS: u32 = 1;

/// Default connection acquire timeout in seconds.
pub const DEFAULT_ACQUIRE_TIMEOUT_SECS: u64 = 30;

/// Configuration options for the database connection pool.
#[derive(Debug, Clone)]
pub struct PoolOptions {
    /// Maximum number of connections in the pool.
    pub max_connections: u32,
    /// Minimum number of connections to maintain.
    pub min_connections: u32,
    /// Timeout for acquiring a connection from the pool.
    pub acquire_timeout: Duration,
}

impl Default for PoolOptions {
    fn default() -> Self {
        Self {
            max_connections: DEFAULT_MAX_CONNECTIONS,
            min_connections: DEFAULT_MIN_CONNECTIONS,
            acquire_timeout: Duration::from_secs(DEFAULT_ACQUIRE_TIMEOUT_SECS),
        }
    }
}

impl PoolOptions {
    /// Create new pool options with specified max connections.
    #[must_use]
    pub fn with_max_connections(max_connections: u32) -> Self {
        Self {
            max_connections,
            ..Default::default()
        }
    }
}

/// Database connection pool wrapper.
#[derive(Debug, Clone)]
pub struct TaxiiPool {
    pool: PgPool,
}

impl TaxiiPool {
    /// Create a new pool from an existing PgPool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Connect to database with connection string using default options.
    pub async fn connect(db_connection: &str) -> DatabaseResult<Self> {
        Self::connect_with_options(db_connection, PoolOptions::default()).await
    }

    /// Connect to database with connection string and custom options.
    pub async fn connect_with_options(
        db_connection: &str,
        options: PoolOptions,
    ) -> DatabaseResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(options.max_connections)
            .min_connections(options.min_connections)
            .acquire_timeout(options.acquire_timeout)
            .connect(db_connection)
            .await?;

        Ok(Self { pool })
    }

    /// Get reference to inner pool.
    #[must_use]
    pub fn inner(&self) -> &PgPool {
        &self.pool
    }
}
