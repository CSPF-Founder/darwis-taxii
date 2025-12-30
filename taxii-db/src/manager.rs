//! Database connection manager for lifecycle management.

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// Database connection manager.
///
/// Provides connection lifecycle management including health checks
/// and graceful shutdown.
#[derive(Debug, Clone)]
pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    /// Create a new database manager with the given connection string.
    pub async fn new(db_connection: &str) -> DatabaseResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(db_connection)
            .await?;

        Ok(Self { pool })
    }

    /// Create a new database manager with custom pool options.
    pub async fn with_options(
        db_connection: &str,
        max_connections: u32,
        min_connections: u32,
    ) -> DatabaseResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(db_connection)
            .await?;

        Ok(Self { pool })
    }

    /// Get a TaxiiPool for database operations.
    pub fn pool(&self) -> TaxiiPool {
        TaxiiPool::new(self.pool.clone())
    }

    /// Get a reference to the inner PgPool.
    pub fn inner(&self) -> &PgPool {
        &self.pool
    }

    /// Check database connectivity.
    ///
    /// Returns Ok(true) if the database is reachable, Ok(false) if not.
    pub async fn health_check(&self) -> DatabaseResult<bool> {
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("Database health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Close all database connections gracefully.
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Check if the pool is closed.
    pub fn is_closed(&self) -> bool {
        self.pool.is_closed()
    }

    /// Get current pool statistics.
    pub fn pool_size(&self) -> u32 {
        self.pool.size()
    }

    /// Get number of idle connections.
    pub fn idle_connections(&self) -> usize {
        self.pool.num_idle()
    }
}

// Note: Migrations are handled externally.
// This implementation uses the existing database schema.
