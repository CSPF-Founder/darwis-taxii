//! ApiRoot model (TAXII 2.x API roots).

use sqlx::FromRow;
use uuid::Uuid;

use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// ApiRoot database row.
///
/// Table: opentaxii_api_root
#[derive(Debug, Clone, FromRow)]
pub struct ApiRoot {
    /// Primary key (UUID).
    pub id: Uuid,

    /// Whether this is the default API root.
    pub default: bool,

    /// API root title.
    pub title: String,

    /// Optional description.
    pub description: Option<String>,

    /// Whether this API root is publicly accessible.
    pub is_public: bool,
}

impl ApiRoot {
    /// Find an API root by ID.
    pub async fn find(pool: &TaxiiPool, id: Uuid) -> DatabaseResult<Option<Self>> {
        let api_root = sqlx::query_as!(
            Self,
            r#"SELECT id, "default", title, description, is_public
               FROM opentaxii_api_root WHERE id = $1"#,
            id
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(api_root)
    }

    /// Find all API roots.
    pub async fn find_all(pool: &TaxiiPool) -> DatabaseResult<Vec<Self>> {
        let api_roots = sqlx::query_as!(
            Self,
            r#"SELECT id, "default", title, description, is_public
               FROM opentaxii_api_root ORDER BY title"#
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(api_roots)
    }

    /// Create a new API root.
    pub async fn create(
        pool: &TaxiiPool,
        id: Uuid,
        title: &str,
        description: Option<&str>,
        default: bool,
        is_public: bool,
    ) -> DatabaseResult<Self> {
        let api_root = sqlx::query_as!(
            Self,
            r#"INSERT INTO opentaxii_api_root (id, title, description, "default", is_public)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id, "default", title, description, is_public"#,
            id,
            title,
            description,
            default,
            is_public
        )
        .fetch_one(pool.inner())
        .await?;

        // If this is default, unset other defaults
        if default {
            sqlx::query!(
                r#"UPDATE opentaxii_api_root SET "default" = false WHERE id != $1"#,
                id
            )
            .execute(pool.inner())
            .await?;
        }

        Ok(api_root)
    }

    /// Delete an API root by ID.
    pub async fn delete(pool: &TaxiiPool, id: Uuid) -> DatabaseResult<bool> {
        let result = sqlx::query!("DELETE FROM opentaxii_api_root WHERE id = $1", id)
            .execute(pool.inner())
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
