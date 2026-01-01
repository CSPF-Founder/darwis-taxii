//! TAXII 2.x management commands.

use clap::Subcommand;
use taxii_db::{DbTaxii2Repository, Taxii2Repository, TaxiiPool};

/// API Root management actions.
#[derive(Subcommand)]
pub enum ApiRootAction {
    /// Add a new API root.
    Add {
        /// Title of the API root.
        #[arg(short, long)]
        title: String,

        /// Description of the API root.
        #[arg(short, long)]
        description: Option<String>,

        /// Set as default API root.
        #[arg(long, default_value = "false")]
        default: bool,

        /// Make the API root public.
        #[arg(long, default_value = "false")]
        public: bool,

        /// Custom UUID for the API root (auto-generated if not provided).
        #[arg(short, long)]
        id: Option<String>,
    },

    /// List all API roots.
    List,
}

/// Collection management actions (TAXII 2.x).
#[derive(Subcommand)]
pub enum CollectionAction {
    /// Add a new collection.
    Add {
        /// API root ID for the collection.
        #[arg(long)]
        api_root_id: String,

        /// Title of the collection.
        #[arg(short, long)]
        title: String,

        /// Description of the collection.
        #[arg(short, long)]
        description: Option<String>,

        /// Alias for the collection.
        #[arg(short, long)]
        alias: Option<String>,

        /// Allow public read access.
        #[arg(long, default_value = "false")]
        public: bool,

        /// Allow public write access.
        #[arg(long, default_value = "false")]
        public_write: bool,
    },

    /// List collections for an API root.
    List {
        /// API root ID.
        #[arg(long)]
        api_root_id: String,
    },
}

/// Job management actions.
#[derive(Subcommand)]
pub enum JobAction {
    /// Clean up old job logs (>24h).
    Cleanup,
}

/// Handle API root commands.
pub async fn handle_api_root(
    pool: TaxiiPool,
    action: ApiRootAction,
) -> Result<(), Box<dyn std::error::Error>> {
    let persistence = DbTaxii2Repository::new(pool);

    match action {
        ApiRootAction::Add {
            title,
            description,
            default,
            public,
            id,
        } => {
            // UUID format is validated by the persistence layer
            let api_root = persistence
                .add_api_root(
                    &title,
                    description.as_deref(),
                    default,
                    public,
                    id.as_deref(),
                )
                .await?;

            println!("API root created successfully:");
            println!("  ID: {}", api_root.id);
            println!("  Title: {}", api_root.title);
            if let Some(desc) = &api_root.description {
                println!("  Description: {desc}");
            }
            println!("  Default: {}", api_root.default);
            println!("  Public: {}", api_root.is_public);
        }
        ApiRootAction::List => {
            let api_roots = persistence.get_api_roots().await?;

            if api_roots.is_empty() {
                println!("No API roots found.");
                return Ok(());
            }

            println!(
                "{:<40} {:<30} {:<10} {:<10}",
                "ID", "Title", "Default", "Public"
            );
            println!("{}", "-".repeat(95));

            for root in api_roots {
                println!(
                    "{:<40} {:<30} {:<10} {:<10}",
                    root.id,
                    truncate(&root.title, 28),
                    if root.default { "Yes" } else { "No" },
                    if root.is_public { "Yes" } else { "No" }
                );
            }
        }
    }

    Ok(())
}

/// Handle collection commands (TAXII 2.x).
pub async fn handle_collection(
    pool: TaxiiPool,
    action: CollectionAction,
) -> Result<(), Box<dyn std::error::Error>> {
    let persistence = DbTaxii2Repository::new(pool);

    match action {
        CollectionAction::Add {
            api_root_id,
            title,
            description,
            alias,
            public,
            public_write,
        } => {
            // Verify API root exists
            let api_root = persistence.get_api_root(&api_root_id).await?;
            if api_root.is_none() {
                return Err(format!("API root '{api_root_id}' not found").into());
            }

            let collection = persistence
                .add_collection(
                    &api_root_id,
                    &title,
                    description.as_deref(),
                    alias.as_deref(),
                    public,
                    public_write,
                )
                .await?;

            println!("Collection created successfully:");
            println!("  ID: {}", collection.id);
            println!("  API Root: {}", collection.api_root_id);
            println!("  Title: {}", collection.title);
            if let Some(desc) = &collection.description {
                println!("  Description: {desc}");
            }
            if let Some(a) = &collection.alias {
                println!("  Alias: {a}");
            }
            println!("  Public Read: {}", collection.is_public);
            println!("  Public Write: {}", collection.is_public_write);
        }
        CollectionAction::List { api_root_id } => {
            let collections = persistence.get_collections(&api_root_id).await?;

            if collections.is_empty() {
                println!("No collections found for API root '{api_root_id}'.");
                return Ok(());
            }

            println!(
                "{:<40} {:<30} {:<15} {:<10} {:<10}",
                "ID", "Title", "Alias", "Public", "Writable"
            );
            println!("{}", "-".repeat(110));

            for coll in collections {
                println!(
                    "{:<40} {:<30} {:<15} {:<10} {:<10}",
                    coll.id,
                    truncate(&coll.title, 28),
                    coll.alias.as_deref().unwrap_or("-"),
                    if coll.is_public { "Yes" } else { "No" },
                    if coll.is_public_write { "Yes" } else { "No" }
                );
            }
        }
    }

    Ok(())
}

/// Handle job commands.
pub async fn handle_job(
    pool: TaxiiPool,
    action: JobAction,
) -> Result<(), Box<dyn std::error::Error>> {
    let persistence = DbTaxii2Repository::new(pool);

    match action {
        JobAction::Cleanup => {
            let removed = persistence.job_cleanup().await?;
            println!("{removed} job(s) removed");
        }
    }

    Ok(())
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
