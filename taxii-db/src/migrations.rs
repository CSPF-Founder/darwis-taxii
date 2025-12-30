//! Database migrations for TAXII server.
//!
//! Migrations are managed via SQLx and stored in the `migrations/` directory
//! at the project root.

use sqlx::PgPool;
use sqlx::migrate::{MigrateError, Migrator};

/// Static migrator loaded from `migrations/` directory at compile time.
static MIGRATOR: Migrator = sqlx::migrate!("../migrations");

/// Run all pending migrations.
///
/// This is idempotent - migrations that have already been applied will be skipped.
/// For existing OpenTAXII databases, the initial migration uses `IF NOT EXISTS`
/// to avoid conflicts with existing tables.
pub async fn run(pool: &PgPool) -> Result<(), MigrateError> {
    MIGRATOR.run(pool).await
}

/// Information about a migration.
#[derive(Debug, Clone)]
pub struct MigrationInfo {
    /// Migration version (timestamp).
    pub version: i64,
    /// Migration description.
    pub description: String,
}

/// Get list of all migrations defined in the migrations directory.
pub fn list() -> Vec<MigrationInfo> {
    MIGRATOR
        .iter()
        .map(|m| MigrationInfo {
            version: m.version,
            description: m.description.to_string(),
        })
        .collect()
}

/// Get the number of migrations.
pub fn count() -> usize {
    MIGRATOR.iter().count()
}

/// Get information about applied migrations by querying the database.
///
/// Returns version numbers of all applied migrations.
pub async fn applied(pool: &PgPool) -> Result<Vec<i64>, sqlx::Error> {
    // Query the SQLx migrations table directly
    let rows: Vec<(i64,)> = sqlx::query_as("SELECT version FROM _sqlx_migrations ORDER BY version")
        .fetch_all(pool)
        .await
        .unwrap_or_default();

    Ok(rows.into_iter().map(|(v,)| v).collect())
}
