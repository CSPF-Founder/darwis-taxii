//! Collection model (TAXII 2.x collections).

use sqlx::FromRow;
use uuid::Uuid;

use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// Collection database row (TAXII 2.x).
///
/// Table: opentaxii_collection
#[derive(Debug, Clone, FromRow)]
pub struct Collection {
    /// Primary key (UUID).
    pub id: Uuid,

    /// Foreign key to API root.
    pub api_root_id: Uuid,

    /// Collection title.
    pub title: String,

    /// Optional description.
    pub description: Option<String>,

    /// Optional alias for the collection.
    pub alias: Option<String>,

    /// Whether collection is publicly readable.
    pub is_public: bool,

    /// Whether collection is publicly writable.
    pub is_public_write: bool,
}

impl Collection {
    /// Find a collection by ID.
    pub async fn find(pool: &TaxiiPool, id: Uuid) -> DatabaseResult<Option<Self>> {
        let collection = sqlx::query_as!(
            Self,
            r#"SELECT id, api_root_id as "api_root_id!", title as "title!", description, alias,
                      is_public as "is_public!", is_public_write as "is_public_write!"
               FROM opentaxii_collection WHERE id = $1"#,
            id
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(collection)
    }

    /// Find collections by API root ID.
    pub async fn find_by_api_root(
        pool: &TaxiiPool,
        api_root_id: Uuid,
    ) -> DatabaseResult<Vec<Self>> {
        let collections = sqlx::query_as!(
            Self,
            r#"SELECT id, api_root_id as "api_root_id!", title as "title!", description, alias,
                      is_public as "is_public!", is_public_write as "is_public_write!"
               FROM opentaxii_collection WHERE api_root_id = $1 ORDER BY title"#,
            api_root_id
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(collections)
    }

    /// Find a collection by ID or alias within an API root.
    pub async fn find_by_id_or_alias(
        pool: &TaxiiPool,
        api_root_id: Uuid,
        id_or_alias: &str,
    ) -> DatabaseResult<Option<Self>> {
        // Try to parse as UUID first
        let collection_uuid = Uuid::parse_str(id_or_alias).ok();

        let collection = if let Some(coll_uuid) = collection_uuid {
            sqlx::query_as!(
                Self,
                r#"SELECT id, api_root_id as "api_root_id!", title as "title!", description, alias,
                          is_public as "is_public!", is_public_write as "is_public_write!"
                   FROM opentaxii_collection
                   WHERE api_root_id = $1 AND (id = $2 OR alias = $3)"#,
                api_root_id,
                coll_uuid,
                id_or_alias
            )
            .fetch_optional(pool.inner())
            .await?
        } else {
            sqlx::query_as!(
                Self,
                r#"SELECT id, api_root_id as "api_root_id!", title as "title!", description, alias,
                          is_public as "is_public!", is_public_write as "is_public_write!"
                   FROM opentaxii_collection
                   WHERE api_root_id = $1 AND alias = $2"#,
                api_root_id,
                id_or_alias
            )
            .fetch_optional(pool.inner())
            .await?
        };

        Ok(collection)
    }

    /// Create a new collection.
    pub async fn create(
        pool: &TaxiiPool,
        api_root_id: Uuid,
        title: &str,
        description: Option<&str>,
        alias: Option<&str>,
        is_public: bool,
        is_public_write: bool,
    ) -> DatabaseResult<Self> {
        let id = Uuid::new_v4();

        let collection = sqlx::query_as!(
            Self,
            r#"INSERT INTO opentaxii_collection (id, api_root_id, title, description, alias, is_public, is_public_write)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id, api_root_id as "api_root_id!", title as "title!", description, alias,
                         is_public as "is_public!", is_public_write as "is_public_write!""#,
            id,
            api_root_id,
            title,
            description,
            alias,
            is_public,
            is_public_write
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(collection)
    }

    /// Delete a collection by ID.
    pub async fn delete(pool: &TaxiiPool, id: Uuid) -> DatabaseResult<bool> {
        let result = sqlx::query!("DELETE FROM opentaxii_collection WHERE id = $1", id)
            .execute(pool.inner())
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
