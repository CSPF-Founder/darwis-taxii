//! DARWIS TAXII server binary.

use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use taxii_auth::AuthAPI;
use taxii_db::{DbTaxii1Repository, DbTaxii2Repository, TaxiiPool, migrations};
use taxii_server::{ServerConfig, create_router};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting DARWIS TAXII server...");

    if let Err(e) = run().await {
        error!("Server error: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration (TOML primary, env overrides)
    // Uses lazy initialization - config is loaded once and cached
    let config = ServerConfig::init()?;

    info!(
        bind = %config.bind_address,
        port = %config.port,
        "Configuration loaded"
    );

    // Create database pool
    let pool = TaxiiPool::connect(&config.db_connection).await?;
    info!("Database connection established");

    // Run migrations (idempotent - skips already applied)
    info!("Running database migrations...");
    migrations::run(pool.inner()).await?;
    info!("Database migrations completed");

    // Create repository instances
    let taxii1_persistence = DbTaxii1Repository::new(pool.clone());
    let taxii2_persistence = DbTaxii2Repository::new(pool.clone());

    // Create auth API
    let auth = AuthAPI::new(
        pool,
        config.auth_secret.clone(),
        Some(config.token_ttl_secs),
    )?;
    info!("Auth API initialized");

    // Create listener address before moving config
    let addr: SocketAddr = format!("{}:{}", config.bind_address, config.port).parse()?;

    // Create router
    let app = create_router(taxii1_persistence, taxii2_persistence, auth, config);
    info!("Router created");

    // Bind listener
    let listener = TcpListener::bind(addr).await?;
    info!(address = %addr, "Server listening");

    // Run server
    axum::serve(listener, app).await?;

    Ok(())
}
