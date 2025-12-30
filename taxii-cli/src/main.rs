//! TAXII CLI - Command-line interface for TAXII server management.
//!
//! This tool provides commands for managing accounts, collections,
//! services, and content in a TAXII server.
//!
//! Configuration is loaded from TOML file (if exists) with CLI args/env override.

mod commands;

use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::env;
use std::path::Path;
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use taxii_db::TaxiiPool;

/// Environment variable prefix for overrides.
const ENV_PREFIX: &str = "DARWIS_TAXII_";

/// Config file paths to search (in order).
const CONFIG_PATHS: &[&str] = &["taxii.toml", "config/taxii.toml"];

/// Environment variable for config file path (overrides search).
const CONFIG_PATH_ENV_VAR: &str = "DARWIS_TAXII_CONFIG";

/// TOML configuration structure (subset needed for CLI).
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct TomlConfig {
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
}

/// Database configuration section.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct DatabaseConfig {
    pub url: Option<String>,
}

/// Auth configuration section.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct AuthConfig {
    pub secret: Option<String>,
}

/// TAXII CLI - Command-line interface for TAXII server management.
#[derive(Parser)]
#[command(name = "taxii-cli")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Database connection string (PostgreSQL).
    /// If not provided, reads from TOML config or DARWIS_TAXII_DB_CONNECTION env.
    #[arg(long, env = "DARWIS_TAXII_DB_CONNECTION")]
    database_url: Option<String>,

    /// Auth secret for JWT tokens.
    /// If not provided, reads from TOML config or DARWIS_TAXII_AUTH_SECRET env.
    #[arg(long, env = "DARWIS_TAXII_AUTH_SECRET")]
    auth_secret: Option<String>,

    /// Path to TOML configuration file.
    #[arg(long, short, env = "DARWIS_TAXII_CONFIG")]
    config: Option<String>,

    /// Enable verbose output.
    #[arg(short, long, default_value = "false")]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage user accounts.
    Account {
        #[command(subcommand)]
        action: commands::account::AccountAction,
    },

    /// Run database migrations.
    Migrate {
        #[command(subcommand)]
        action: commands::migrate::MigrateAction,
    },

    /// Synchronize configuration from YAML file.
    Sync {
        /// Path to YAML configuration file.
        config: String,

        /// Force deletion of collections not in config.
        #[arg(short, long, default_value = "false")]
        force_delete: bool,
    },

    /// Delete content blocks from collections.
    #[command(name = "content")]
    Content {
        #[command(subcommand)]
        action: commands::persistence::ContentAction,
    },

    /// Manage TAXII 2.x API roots.
    #[command(name = "api-root")]
    ApiRoot {
        #[command(subcommand)]
        action: commands::taxii2::ApiRootAction,
    },

    /// Manage TAXII 2.x collections.
    Collection {
        #[command(subcommand)]
        action: commands::taxii2::CollectionAction,
    },

    /// Clean up old job logs.
    #[command(name = "job")]
    Job {
        #[command(subcommand)]
        action: commands::taxii2::JobAction,
    },
}

/// Resolved configuration after merging TOML, env, and CLI args.
struct ResolvedConfig {
    database_url: String,
    auth_secret: String,
}

impl ResolvedConfig {
    /// Load configuration with priority: CLI args > env vars > TOML config.
    fn load(cli: &Cli) -> Result<Self, String> {
        // Determine config file path
        let config_path = cli
            .config
            .clone()
            .or_else(|| env::var(CONFIG_PATH_ENV_VAR).ok())
            .or_else(|| {
                CONFIG_PATHS
                    .iter()
                    .find(|p| Path::new(p).exists())
                    .map(|s| s.to_string())
            });

        // Load TOML config if file exists
        let toml_config = match &config_path {
            Some(path) if Path::new(path).exists() => {
                match std::fs::read_to_string(path) {
                    Ok(content) => match toml::from_str(&content) {
                        Ok(config) => {
                            if cli.verbose {
                                eprintln!("Loaded config from: {}", path);
                            }
                            config
                        }
                        Err(e) => {
                            return Err(format!("Failed to parse TOML config: {}", e));
                        }
                    },
                    Err(e) => {
                        return Err(format!("Failed to read config file: {}", e));
                    }
                }
            }
            _ => TomlConfig::default(),
        };

        // Resolve database_url: CLI > env > TOML
        let database_url = cli
            .database_url
            .clone()
            .or_else(|| env::var(format!("{}DB_CONNECTION", ENV_PREFIX)).ok())
            .or(toml_config.database.url)
            .ok_or_else(|| {
                "Database URL required. Provide via --database-url, DARWIS_TAXII_DB_CONNECTION env, or database.url in taxii.toml".to_string()
            })?;

        // Resolve auth_secret: CLI > env > TOML
        let auth_secret = cli
            .auth_secret
            .clone()
            .or_else(|| env::var(format!("{}AUTH_SECRET", ENV_PREFIX)).ok())
            .or(toml_config.auth.secret)
            .ok_or_else(|| {
                "Auth secret required. Provide via --auth-secret, DARWIS_TAXII_AUTH_SECRET env, or auth.secret in taxii.toml".to_string()
            })?;

        Ok(Self {
            database_url,
            auth_secret,
        })
    }
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Load and resolve configuration
    let config = ResolvedConfig::load(&cli)?;

    // Connect to database
    let pool = TaxiiPool::connect(&config.database_url).await?;

    match cli.command {
        Commands::Account { action } => {
            commands::account::handle(pool, &config.auth_secret, action).await?;
        }
        Commands::Migrate { action } => {
            commands::migrate::handle(pool, action).await?;
        }
        Commands::Sync {
            config: yaml_config,
            force_delete,
        } => {
            commands::persistence::handle_sync(
                pool,
                &config.auth_secret,
                &yaml_config,
                force_delete,
            )
            .await?;
        }
        Commands::Content { action } => {
            commands::persistence::handle_content(pool, action).await?;
        }
        Commands::ApiRoot { action } => {
            commands::taxii2::handle_api_root(pool, action).await?;
        }
        Commands::Collection { action } => {
            commands::taxii2::handle_collection(pool, action).await?;
        }
        Commands::Job { action } => {
            commands::taxii2::handle_job(pool, action).await?;
        }
    }

    Ok(())
}
