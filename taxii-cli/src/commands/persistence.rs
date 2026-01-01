//! Persistence management commands.

use chrono::{DateTime, Utc};
use clap::Subcommand;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use taxii_auth::AuthAPI;
use taxii_core::{CollectionEntity, ContentBindingEntity, PermissionValue, ServiceEntity};
use taxii_db::{
    DbTaxii1Repository, TAXII1_PERMISSIONS, TAXII2_PERMISSIONS, Taxii1Repository, TaxiiPool,
    validate_permissions,
};
use tracing::{debug, info};

/// Content block management actions.
#[derive(Subcommand)]
pub enum ContentAction {
    /// Delete content blocks from collections.
    Delete {
        /// Collection name(s) to delete from (can be specified multiple times).
        #[arg(short, long, required = true)]
        collection: Vec<String>,

        /// Exclusive beginning of time window (ISO8601).
        #[arg(long)]
        begin: String,

        /// Inclusive ending of time window (ISO8601).
        #[arg(long)]
        end: Option<String>,

        /// Also delete associated inbox messages.
        #[arg(short = 'm', long, default_value = "false")]
        with_messages: bool,
    },
}

/// YAML configuration structure.
#[derive(Debug, Deserialize)]
struct YamlConfig {
    #[serde(default)]
    services: Vec<ServiceConfig>,
    #[serde(default)]
    collections: Vec<CollectionConfig>,
    #[serde(default)]
    accounts: Vec<AccountConfig>,
}

#[derive(Debug, Deserialize)]
struct ServiceConfig {
    id: String,
    #[serde(rename = "type")]
    service_type: String,
    #[serde(flatten)]
    properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct CollectionConfig {
    name: String,
    /// ID field from YAML config (ignored - collections use auto-generated IDs
    /// or are matched by name to existing collections)
    #[serde(default)]
    #[allow(dead_code)]
    id: Option<String>,
    #[serde(default)]
    service_ids: Vec<String>,
    #[serde(default)]
    supported_content: Vec<ContentBindingConfig>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_true")]
    available: bool,
    #[serde(default = "default_true")]
    accept_all_content: bool,
    #[serde(rename = "type", default = "default_collection_type")]
    collection_type: String,
}

#[derive(Debug, Deserialize)]
struct ContentBindingConfig {
    binding: String,
    #[serde(default)]
    subtypes: Vec<String>,
}

/// Account configuration from YAML.
#[derive(Debug, Deserialize)]
struct AccountConfig {
    username: String,
    password: String,
    #[serde(default)]
    is_admin: bool,
    #[serde(default)]
    permissions: HashMap<String, PermissionInput>,
}

/// Permission input from YAML - supports both TAXII 1.x and 2.x formats.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum PermissionInput {
    /// TAXII 1.x style: single permission string ("read" or "modify")
    Single(String),
    /// TAXII 2.x style: list of permissions (["read", "write"])
    Multiple(Vec<String>),
}

fn default_true() -> bool {
    true
}

fn default_collection_type() -> String {
    "DATA_FEED".to_string()
}

/// Handle sync command.
pub async fn handle_sync(
    pool: TaxiiPool,
    auth_secret: &str,
    config_path: &str,
    force_delete: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load YAML configuration
    let yaml_content = fs::read_to_string(config_path)?;
    let config: YamlConfig = serde_yaml::from_str(&yaml_content)?;

    let persistence = DbTaxii1Repository::new(pool.clone());

    // Sync services
    sync_services(&persistence, &config.services).await?;

    // Sync collections
    sync_collections(&persistence, &config.collections, force_delete).await?;

    // Sync accounts
    if !config.accounts.is_empty() {
        sync_accounts(&pool, auth_secret, &config.accounts).await?;
    }

    println!("Configuration synchronized successfully");
    Ok(())
}

/// Sync services from configuration.
async fn sync_services(
    persistence: &DbTaxii1Repository,
    services: &[ServiceConfig],
) -> Result<(), Box<dyn std::error::Error>> {
    let existing = persistence.get_services(None).await?;
    let existing_ids: std::collections::HashSet<_> =
        existing.iter().filter_map(|s| s.id.clone()).collect();

    let config_ids: std::collections::HashSet<_> = services.iter().map(|s| s.id.clone()).collect();

    let mut created = 0;
    let mut updated = 0;

    for svc_config in services {
        let entity = ServiceEntity {
            id: Some(svc_config.id.clone()),
            service_type: svc_config.service_type.clone(),
            properties: serde_json::to_value(&svc_config.properties)?,
        };

        if existing_ids.contains(&svc_config.id) {
            persistence.update_service(&entity).await?;
            updated += 1;
            debug!(id = %svc_config.id, "Service updated");
        } else {
            persistence.create_service(&entity).await?;
            created += 1;
            debug!(id = %svc_config.id, "Service created");
        }
    }

    // Delete services not in config
    let mut deleted = 0;
    for existing_id in existing_ids {
        if !config_ids.contains(&existing_id) {
            persistence.delete_service(&existing_id).await?;
            deleted += 1;
            debug!(id = %existing_id, "Service deleted");
        }
    }

    info!(created, updated, deleted, "Services synchronized");
    Ok(())
}

/// Sync collections from configuration.
async fn sync_collections(
    persistence: &DbTaxii1Repository,
    collections: &[CollectionConfig],
    force_delete: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let existing = persistence.get_collections(None).await?;
    let existing_by_name: HashMap<_, _> = existing
        .iter()
        .map(|c| (c.name.clone(), c.clone()))
        .collect();

    let config_names: std::collections::HashSet<_> =
        collections.iter().map(|c| c.name.clone()).collect();

    let mut created = 0;
    let mut updated = 0;

    for coll_config in collections {
        let supported_content: Vec<ContentBindingEntity> = coll_config
            .supported_content
            .iter()
            .map(|cb| ContentBindingEntity::with_subtypes(cb.binding.clone(), cb.subtypes.clone()))
            .collect();

        if let Some(existing_coll) = existing_by_name.get(&coll_config.name) {
            // Update existing collection
            let entity = CollectionEntity {
                id: existing_coll.id,
                name: coll_config.name.clone(),
                available: coll_config.available,
                volume: existing_coll.volume,
                description: coll_config.description.clone(),
                accept_all_content: coll_config.accept_all_content,
                collection_type: coll_config.collection_type.clone(),
                supported_content,
            };

            persistence.update_collection(&entity).await?;

            // Update service associations
            if let Some(coll_id) = existing_coll.id {
                persistence
                    .set_collection_services(coll_id, &coll_config.service_ids)
                    .await?;
            }

            updated += 1;
            debug!(name = %coll_config.name, "Collection updated");
        } else {
            // Create new collection
            let entity = CollectionEntity {
                id: None,
                name: coll_config.name.clone(),
                available: coll_config.available,
                volume: Some(0),
                description: coll_config.description.clone(),
                accept_all_content: coll_config.accept_all_content,
                collection_type: coll_config.collection_type.clone(),
                supported_content,
            };

            let created_coll = persistence.create_collection(&entity).await?;

            // Set service associations
            if let Some(coll_id) = created_coll.id {
                persistence
                    .set_collection_services(coll_id, &coll_config.service_ids)
                    .await?;
            }

            created += 1;
            debug!(name = %coll_config.name, "Collection created");
        }
    }

    // Handle collections not in config
    let mut deleted = 0;
    let mut disabled = 0;

    for (name, existing_coll) in &existing_by_name {
        if !config_names.contains(name) {
            if force_delete {
                persistence.delete_collection(name).await?;
                deleted += 1;
                debug!(name = %name, "Collection deleted");
            } else {
                // Disable collection
                let entity = CollectionEntity {
                    id: existing_coll.id,
                    name: existing_coll.name.clone(),
                    available: false,
                    volume: existing_coll.volume,
                    description: existing_coll.description.clone(),
                    accept_all_content: existing_coll.accept_all_content,
                    collection_type: existing_coll.collection_type.clone(),
                    supported_content: existing_coll.supported_content.clone(),
                };
                persistence.update_collection(&entity).await?;
                disabled += 1;
                debug!(name = %name, "Collection disabled");
            }
        }
    }

    info!(
        created,
        updated, deleted, disabled, "Collections synchronized"
    );
    Ok(())
}

/// Sync accounts from configuration.
async fn sync_accounts(
    pool: &TaxiiPool,
    auth_secret: &str,
    accounts: &[AccountConfig],
) -> Result<(), Box<dyn std::error::Error>> {
    let auth = AuthAPI::new(pool.clone(), auth_secret.to_string(), None)?;

    let existing = auth.get_accounts().await?;
    let existing_by_name: HashMap<_, _> = existing
        .iter()
        .map(|a| (a.username.clone(), a.clone()))
        .collect();

    let mut created = 0;
    let mut updated = 0;

    for account_config in accounts {
        // Convert permissions from YAML format to PermissionValue
        let permissions = convert_permissions(&account_config.permissions)?;

        // Validate permissions
        // Note: TAXII 1.x uses collection name, TAXII 2.x uses collection UUID directly
        // No normalization is performed - users must use the correct identifier format
        validate_permissions(&permissions)?;

        if let Some(existing_account) = existing_by_name.get(&account_config.username) {
            // Update existing account
            let updated_account = taxii_core::Account {
                id: existing_account.id,
                username: account_config.username.clone(),
                is_admin: account_config.is_admin,
                permissions: permissions.clone(),
                details: existing_account.details.clone(),
            };

            auth.update_account(&updated_account, Some(&account_config.password))
                .await?;
            updated += 1;
            debug!(username = %account_config.username, "Account updated");
        } else {
            // Create new account
            let new_account = auth
                .create_account(
                    &account_config.username,
                    &account_config.password,
                    account_config.is_admin,
                )
                .await?;

            // If permissions are set, update the account with them
            if !permissions.is_empty() {
                let account_with_perms = taxii_core::Account {
                    id: new_account.id,
                    username: new_account.username,
                    is_admin: new_account.is_admin,
                    permissions,
                    details: new_account.details,
                };
                auth.update_account(&account_with_perms, None).await?;
            }

            created += 1;
            debug!(username = %account_config.username, "Account created");
        }
    }

    info!(created, updated, "Accounts synchronized");
    Ok(())
}

/// Convert YAML permissions to PermissionValue format.
fn convert_permissions(
    input: &HashMap<String, PermissionInput>,
) -> Result<HashMap<String, PermissionValue>, String> {
    let mut result = HashMap::new();

    for (collection, perm_input) in input {
        let perm_value = match perm_input {
            PermissionInput::Single(s) => {
                // Validate TAXII 1.x permission
                if !TAXII1_PERMISSIONS.contains(&s.as_str()) {
                    return Err(format!(
                        "Invalid TAXII 1.x permission '{s}' for collection '{collection}'. Valid: {TAXII1_PERMISSIONS:?}"
                    ));
                }
                PermissionValue::Taxii1(s.clone())
            }
            PermissionInput::Multiple(list) => {
                // Validate TAXII 2.x permissions
                for p in list {
                    if !TAXII2_PERMISSIONS.contains(&p.as_str()) {
                        return Err(format!(
                            "Invalid TAXII 2.x permission '{p}' for collection '{collection}'. Valid: {TAXII2_PERMISSIONS:?}"
                        ));
                    }
                }
                PermissionValue::Taxii2(list.clone())
            }
        };
        result.insert(collection.clone(), perm_value);
    }

    Ok(result)
}

/// Handle content block commands.
pub async fn handle_content(
    pool: TaxiiPool,
    action: ContentAction,
) -> Result<(), Box<dyn std::error::Error>> {
    let persistence = DbTaxii1Repository::new(pool);

    match action {
        ContentAction::Delete {
            collection,
            begin,
            end,
            with_messages,
        } => {
            let start_time: DateTime<Utc> = DateTime::parse_from_rfc3339(&begin)
                .map_err(|e| format!("Invalid begin timestamp: {e}"))?
                .with_timezone(&Utc);

            let end_time: Option<DateTime<Utc>> = end
                .map(|e| {
                    DateTime::parse_from_rfc3339(&e)
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(|err| format!("Invalid end timestamp: {err}"))
                })
                .transpose()?;

            let mut total_deleted = 0;

            for coll_name in &collection {
                let deleted = persistence
                    .delete_content_blocks(coll_name, start_time, end_time, with_messages)
                    .await?;

                println!("Deleted {deleted} content blocks from '{coll_name}'");
                total_deleted += deleted;
            }

            println!("Total deleted: {total_deleted} content blocks");
        }
    }

    Ok(())
}
