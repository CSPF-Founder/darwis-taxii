//! Account model.

use std::collections::HashMap;

use sqlx::FromRow;
use uuid::Uuid;

use crate::error::{DatabaseError, DatabaseResult};
use crate::models::taxii1::DataCollection;
use crate::models::taxii2::Collection as Taxii2Collection;
use crate::pool::TaxiiPool;
use taxii_core::PermissionValue;

/// Valid TAXII 1.x permission values.
pub const TAXII1_PERMISSIONS: &[&str] = &["read", "modify"];

/// Valid TAXII 2.x permission values.
pub const TAXII2_PERMISSIONS: &[&str] = &["read", "write"];

/// Account database row.
///
/// Table: accounts
#[derive(Debug, Clone, FromRow)]
pub struct Account {
    /// Primary key.
    pub id: i32,

    /// Username (unique).
    pub username: String,

    /// Password hash (scrypt format).
    pub password_hash: String,

    /// Whether account is admin.
    pub is_admin: bool,

    /// Permissions as JSON text.
    #[sqlx(rename = "_permissions")]
    pub permissions_json: String,
}

impl Account {
    /// Get permissions as HashMap.
    ///
    /// Supports both TAXII 1.x format (string values) and TAXII 2.x format (list values).
    pub fn permissions(&self) -> HashMap<String, PermissionValue> {
        serde_json::from_str(&self.permissions_json).unwrap_or_default()
    }

    /// Find an account by ID.
    pub async fn find(pool: &TaxiiPool, id: i32) -> DatabaseResult<Option<Self>> {
        let account = sqlx::query_as!(
            Self,
            r#"SELECT id, username as "username!", password_hash as "password_hash!",
                      is_admin as "is_admin!", _permissions as "permissions_json!"
               FROM accounts WHERE id = $1"#,
            id
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(account)
    }

    /// Find an account by username.
    pub async fn find_by_username(
        pool: &TaxiiPool,
        username: &str,
    ) -> DatabaseResult<Option<Self>> {
        let account = sqlx::query_as!(
            Self,
            r#"SELECT id, username as "username!", password_hash as "password_hash!",
                      is_admin as "is_admin!", _permissions as "permissions_json!"
               FROM accounts WHERE username = $1"#,
            username
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(account)
    }

    /// Find all accounts.
    pub async fn find_all(pool: &TaxiiPool) -> DatabaseResult<Vec<Self>> {
        let accounts = sqlx::query_as!(
            Self,
            r#"SELECT id, username as "username!", password_hash as "password_hash!",
                      is_admin as "is_admin!", _permissions as "permissions_json!"
               FROM accounts"#
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(accounts)
    }

    /// Create a new account.
    pub async fn create(
        pool: &TaxiiPool,
        username: &str,
        password_hash: &str,
        is_admin: bool,
    ) -> DatabaseResult<Self> {
        let permissions_json = "{}";

        let id = sqlx::query_scalar!(
            r#"INSERT INTO accounts (username, password_hash, is_admin, _permissions)
               VALUES ($1, $2, $3, $4)
               RETURNING id"#,
            username,
            password_hash,
            is_admin,
            permissions_json
        )
        .fetch_one(pool.inner())
        .await?;

        Self::find(pool, id)
            .await?
            .ok_or_else(|| DatabaseError::not_found("Failed to create account"))
    }

    /// Update an existing account (without password).
    pub async fn update(
        pool: &TaxiiPool,
        id: i32,
        is_admin: bool,
        permissions_json: &str,
    ) -> DatabaseResult<Self> {
        sqlx::query!(
            r#"UPDATE accounts SET is_admin = $2, _permissions = $3 WHERE id = $1"#,
            id,
            is_admin,
            permissions_json
        )
        .execute(pool.inner())
        .await?;

        Self::find(pool, id)
            .await?
            .ok_or_else(|| DatabaseError::not_found("Account not found"))
    }

    /// Update an existing account with password.
    pub async fn update_with_password(
        pool: &TaxiiPool,
        id: i32,
        password_hash: &str,
        is_admin: bool,
        permissions_json: &str,
    ) -> DatabaseResult<Self> {
        sqlx::query!(
            r#"UPDATE accounts SET password_hash = $2, is_admin = $3, _permissions = $4 WHERE id = $1"#,
            id,
            password_hash,
            is_admin,
            permissions_json
        )
        .execute(pool.inner())
        .await?;

        Self::find(pool, id)
            .await?
            .ok_or_else(|| DatabaseError::not_found("Account not found"))
    }

    /// Delete an account by username.
    pub async fn delete_by_username(pool: &TaxiiPool, username: &str) -> DatabaseResult<bool> {
        let result = sqlx::query!("DELETE FROM accounts WHERE username = $1", username)
            .execute(pool.inner())
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Validate permissions.
///
/// Validates both TAXII 1.x (string) and TAXII 2.x (list) permission formats.
pub fn validate_permissions(permissions: &HashMap<String, PermissionValue>) -> Result<(), String> {
    for (collection_name, permission) in permissions {
        match permission {
            PermissionValue::Taxii1(s) => {
                if !TAXII1_PERMISSIONS.contains(&s.as_str()) {
                    return Err(format!(
                        "Unknown TAXII1 permission '{s}' specified for collection '{collection_name}'"
                    ));
                }
            }
            PermissionValue::Taxii2(list) => {
                for p in list {
                    if !TAXII2_PERMISSIONS.contains(&p.as_str()) {
                        return Err(format!(
                            "Unknown TAXII2 permission '{p}' specified for collection '{collection_name}'"
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

/// An invalid collection reference in account permissions.
#[derive(Debug)]
pub struct InvalidCollectionRef {
    /// The collection reference (name for TAXII 1.x, UUID for TAXII 2.x).
    pub collection_ref: String,
    /// The permission type ("TAXII 1.x" or "TAXII 2.x").
    pub permission_type: &'static str,
}

/// Validate that all collections referenced in permissions exist in the database.
///
/// For TAXII 1.x permissions: validates collection names exist in data_collections table.
/// For TAXII 2.x permissions: validates collection UUIDs exist in opentaxii_collection table.
///
/// Returns a list of invalid collection references if any are found.
pub async fn validate_collection_references(
    pool: &TaxiiPool,
    permissions: &HashMap<String, PermissionValue>,
) -> DatabaseResult<Vec<InvalidCollectionRef>> {
    let mut invalid_refs = Vec::new();

    for (collection_ref, permission) in permissions {
        let exists = match permission {
            PermissionValue::Taxii1(_) => {
                // TAXII 1.x: collection_ref is a collection name
                DataCollection::exists_by_name(pool, collection_ref).await?
            }
            PermissionValue::Taxii2(_) => {
                // TAXII 2.x: collection_ref should be a UUID
                match Uuid::parse_str(collection_ref) {
                    Ok(uuid) => Taxii2Collection::exists(pool, uuid).await?,
                    Err(_) => false, // Invalid UUID format counts as non-existent
                }
            }
        };

        if !exists {
            let permission_type = match permission {
                PermissionValue::Taxii1(_) => "TAXII 1.x",
                PermissionValue::Taxii2(_) => "TAXII 2.x",
            };
            invalid_refs.push(InvalidCollectionRef {
                collection_ref: collection_ref.clone(),
                permission_type,
            });
        }
    }

    Ok(invalid_refs)
}
