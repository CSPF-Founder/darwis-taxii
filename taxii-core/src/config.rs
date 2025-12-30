//! Server configuration.

use serde::Deserialize;

/// Server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Database connection string.
    pub db_connection: String,

    /// Auth secret for JWT.
    pub auth_secret: Option<String>,

    /// Token TTL in seconds.
    pub token_ttl_secs: Option<u64>,
}
