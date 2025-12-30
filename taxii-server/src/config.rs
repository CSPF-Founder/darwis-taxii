//! Server configuration.
//!
//! Configuration is loaded from TOML file (primary) with environment
//! variable overrides (optional).

use serde::Deserialize;
use std::env;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use taxii_db::Taxii1Repository;

/// Global server configuration, initialized once.
/// Uses OnceLock with Result to handle initialization errors.
static CONFIG: OnceLock<Result<ServerConfig, String>> = OnceLock::new();

/// Mutex to ensure single initialization attempt.
static INIT_LOCK: Mutex<()> = Mutex::new(());

/// Environment variable prefix for overrides.
const ENV_PREFIX: &str = "DARWIS_TAXII_";

/// Config file paths to search (in order).
const CONFIG_PATHS: &[&str] = &["taxii.toml", "config/taxii.toml"];

/// Environment variable for config file path (overrides search).
const CONFIG_PATH_ENV_VAR: &str = "DARWIS_TAXII_CONFIG";

/// TOML configuration structure matching the config file layout.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct TomlConfig {
    pub bind_address: Option<String>,
    pub port: Option<u16>,
    pub domain: Option<String>,
    pub support_basic_auth: Option<bool>,
    pub return_server_error_details: Option<bool>,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub taxii1: Taxii1Config,
    pub taxii2: Taxii2Config,
}

/// Database configuration section.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct DatabaseConfig {
    pub url: Option<String>,
}

/// Auth configuration section.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct AuthConfig {
    pub secret: Option<String>,
    pub token_ttl_secs: Option<i64>,
}

/// TAXII 1.x configuration section.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Taxii1Config {
    pub save_raw_inbox_messages: Option<bool>,
    pub xml_parser_supports_huge_tree: Option<bool>,
    pub count_blocks_in_poll_responses: Option<bool>,
    pub unauthorized_status: Option<String>,
}

/// TAXII 2.x configuration section.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Taxii2Config {
    pub title: Option<String>,
    pub description: Option<String>,
    pub contact: Option<String>,
    pub max_content_length: Option<usize>,
    pub public_discovery: Option<bool>,
    pub allow_custom_properties: Option<bool>,
    /// Default pagination limit when client doesn't specify.
    pub default_pagination_limit: Option<i64>,
    /// Maximum pagination limit (hard cap).
    pub max_pagination_limit: Option<i64>,
}

/// Server configuration (flattened runtime config).
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Database connection string.
    pub db_connection: String,

    /// Auth secret for JWT.
    pub auth_secret: String,

    /// Token TTL in seconds.
    pub token_ttl_secs: i64,

    /// Server bind address.
    pub bind_address: String,

    /// Server port.
    pub port: u16,

    /// Domain for service URLs.
    /// Includes port, e.g., "localhost:9000"
    pub domain: Option<String>,

    /// Whether to support basic auth.
    pub support_basic_auth: bool,

    /// TAXII 2.x title.
    pub title: String,

    /// TAXII 2.x description.
    pub description: Option<String>,

    /// TAXII 2.x contact.
    pub contact: Option<String>,

    /// Maximum content length for TAXII 2.x.
    pub max_content_length: usize,

    /// Whether to allow public discovery.
    pub public_discovery: bool,

    /// Whether to allow custom STIX properties.
    pub allow_custom_properties: bool,

    /// Whether to return server error details.
    pub return_server_error_details: bool,

    /// Unauthorized status type for TAXII 1.x.
    pub unauthorized_status: String,

    // TAXII 1.x specific options
    /// Whether to save raw inbox messages (TAXII 1.x).
    /// When enabled, the original XML message is stored for later retrieval.
    pub save_raw_inbox_messages: bool,

    /// Whether XML parser supports huge tree (TAXII 1.x).
    /// When enabled, allows parsing of large XML documents.
    pub xml_parser_supports_huge_tree: bool,

    /// Whether to count blocks in poll responses (TAXII 1.x).
    /// When enabled, includes total count in poll responses (can be expensive).
    pub count_blocks_in_poll_responses: bool,

    /// Default pagination limit when client doesn't specify (TAXII 2.x).
    pub default_pagination_limit: i64,

    /// Maximum pagination limit, hard cap (TAXII 2.x).
    pub max_pagination_limit: i64,
}

/// Configuration loading error.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Failed to parse TOML: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Missing required configuration: {0}")]
    MissingRequired(String),
}

impl ServerConfig {
    /// Load configuration from TOML file with environment variable overrides.
    ///
    /// Configuration priority (highest to lowest):
    /// 1. Environment variables (DARWIS_TAXII_*)
    /// 2. TOML config file
    /// 3. Default values
    ///
    /// Config file search order:
    /// 1. DARWIS_TAXII_CONFIG env var (if set)
    /// 2. ./taxii.toml
    /// 3. ./config/taxii.toml
    pub fn load() -> Result<Self, ConfigError> {
        // Determine config file path
        let config_path = env::var(CONFIG_PATH_ENV_VAR).ok().or_else(|| {
            CONFIG_PATHS
                .iter()
                .find(|p| Path::new(p).exists())
                .map(|s| s.to_string())
        });

        // Load TOML config if file exists
        let toml_config = match &config_path {
            Some(path) if Path::new(path).exists() => {
                tracing::info!(path = %path, "Loading configuration from file");
                let content = std::fs::read_to_string(path)?;
                toml::from_str(&content)?
            }
            _ => {
                tracing::info!("No config file found, using defaults with env overrides");
                TomlConfig::default()
            }
        };

        // Build final config with env overrides
        Self::from_toml_with_env_overrides(toml_config)
    }

    /// Get the global configuration, initializing it if necessary.
    ///
    /// This is a lazy one-time initialization that caches the config.
    pub fn global() -> Result<&'static ServerConfig, ConfigError> {
        // Ensure we only try to initialize once
        let _guard = INIT_LOCK
            .lock()
            .map_err(|_| ConfigError::MissingRequired("Lock poisoned".to_string()))?;

        let result = CONFIG.get_or_init(|| Self::load().map_err(|e| e.to_string()));

        match result {
            Ok(config) => Ok(config),
            Err(msg) => Err(ConfigError::MissingRequired(msg.clone())),
        }
    }

    /// Initialize the global configuration explicitly.
    ///
    /// Returns Ok if already initialized (idempotent).
    pub fn init() -> Result<&'static ServerConfig, ConfigError> {
        Self::global()
    }

    /// Build ServerConfig from TOML config with environment variable overrides.
    fn from_toml_with_env_overrides(toml: TomlConfig) -> Result<Self, ConfigError> {
        // Database connection: env > toml, required
        let db_connection = env_var("DB_CONNECTION")
            .or(toml.database.url)
            .ok_or_else(|| {
                ConfigError::MissingRequired(
                    "database.url (or DARWIS_TAXII_DB_CONNECTION env var)".to_string(),
                )
            })?;

        // Auth secret: env > toml, required
        let auth_secret = env_var("AUTH_SECRET").or(toml.auth.secret).ok_or_else(|| {
            ConfigError::MissingRequired(
                "auth.secret (or DARWIS_TAXII_AUTH_SECRET env var)".to_string(),
            )
        })?;

        Ok(Self {
            db_connection,
            auth_secret,
            token_ttl_secs: env_var_parse("TOKEN_TTL_SECS")
                .or(toml.auth.token_ttl_secs)
                .unwrap_or(3600),
            bind_address: env_var("BIND_ADDRESS")
                .or(toml.bind_address)
                .unwrap_or_else(|| "0.0.0.0".to_string()),
            port: env_var_parse("PORT").or(toml.port).unwrap_or(9000),
            domain: env_var("DOMAIN")
                .or(toml.domain)
                .or_else(|| Some("localhost:9000".to_string())),
            support_basic_auth: env_var_parse("SUPPORT_BASIC_AUTH")
                .or(toml.support_basic_auth)
                .unwrap_or(true),
            title: env_var("TITLE")
                .or(toml.taxii2.title)
                .unwrap_or_else(|| "DARWIS TAXII".to_string()),
            description: env_var("DESCRIPTION").or(toml.taxii2.description),
            contact: env_var("CONTACT").or(toml.taxii2.contact),
            max_content_length: env_var_parse("MAX_CONTENT_LENGTH")
                .or(toml.taxii2.max_content_length)
                .unwrap_or(2048),
            public_discovery: env_var_parse("PUBLIC_DISCOVERY")
                .or(toml.taxii2.public_discovery)
                .unwrap_or(true),
            allow_custom_properties: env_var_parse("ALLOW_CUSTOM_PROPERTIES")
                .or(toml.taxii2.allow_custom_properties)
                .unwrap_or(true),
            return_server_error_details: env_var_parse("RETURN_SERVER_ERROR_DETAILS")
                .or(toml.return_server_error_details)
                .unwrap_or(false),
            unauthorized_status: env_var("UNAUTHORIZED_STATUS")
                .or(toml.taxii1.unauthorized_status)
                .unwrap_or_else(|| "UNAUTHORIZED".to_string()),
            save_raw_inbox_messages: env_var_parse("SAVE_RAW_INBOX_MESSAGES")
                .or(toml.taxii1.save_raw_inbox_messages)
                .unwrap_or(true),
            xml_parser_supports_huge_tree: env_var_parse("XML_PARSER_SUPPORTS_HUGE_TREE")
                .or(toml.taxii1.xml_parser_supports_huge_tree)
                .unwrap_or(true),
            count_blocks_in_poll_responses: env_var_parse("COUNT_BLOCKS_IN_POLL_RESPONSES")
                .or(toml.taxii1.count_blocks_in_poll_responses)
                .unwrap_or(false),
            default_pagination_limit: env_var_parse("DEFAULT_PAGINATION_LIMIT")
                .or(toml.taxii2.default_pagination_limit)
                .unwrap_or(1000),
            max_pagination_limit: env_var_parse("MAX_PAGINATION_LIMIT")
                .or(toml.taxii2.max_pagination_limit)
                .unwrap_or(1000),
        })
    }
}

/// Get environment variable with DARWIS_TAXII_ prefix.
fn env_var(name: &str) -> Option<String> {
    env::var(format!("{}{}", ENV_PREFIX, name)).ok()
}

/// Get and parse environment variable with DARWIS_TAXII_ prefix.
fn env_var_parse<T: std::str::FromStr>(name: &str) -> Option<T> {
    env_var(name).and_then(|s| s.parse().ok())
}

/// Get the domain for a service, checking persistence first, then falling back to config.
///
/// The domain resolution order is:
/// 1. Service-specific domain from persistence (service.properties.domain)
/// 2. Global domain from config
pub async fn get_domain(
    persistence: &taxii_db::DbTaxii1Repository,
    config: &ServerConfig,
    service_id: &str,
) -> Option<String> {
    // Try to get service-specific domain from persistence
    if let Ok(Some(domain)) = persistence.get_domain(service_id).await {
        return Some(domain);
    }

    // Fall back to config domain
    config.domain.clone()
}
