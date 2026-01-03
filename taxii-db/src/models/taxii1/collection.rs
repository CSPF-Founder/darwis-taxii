//! DataCollection model (TAXII 1.x collections).

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// DataCollection database row.
///
/// Table: data_collections
#[derive(Debug, Clone, FromRow)]
pub struct DataCollection {
    /// Primary key.
    pub id: i32,

    /// Collection name (unique, indexed).
    pub name: String,

    /// Collection type (e.g., "DATA_FEED", "DATA_SET").
    #[sqlx(rename = "type")]
    pub collection_type: String,

    /// Optional description.
    pub description: Option<String>,

    /// Whether to accept all content types.
    pub accept_all_content: bool,

    /// Content bindings as JSON text.
    pub bindings: Option<String>,

    /// Whether collection is available.
    pub available: bool,

    /// Content block count.
    pub volume: i32,

    /// Row creation timestamp.
    pub date_created: DateTime<Utc>,
}

/// Parameters for updating a data collection.
#[derive(Debug, Clone)]
pub struct UpdateDataCollection<'a> {
    pub id: i32,
    pub name: &'a str,
    pub collection_type: &'a str,
    pub description: Option<&'a str>,
    pub available: bool,
    pub accept_all_content: bool,
    pub bindings: Option<&'a str>,
}

impl DataCollection {
    /// Collection type constant: Data Feed.
    pub const TYPE_FEED: &'static str = "DATA_FEED";

    /// Collection type constant: Data Set.
    pub const TYPE_SET: &'static str = "DATA_SET";

    /// Find a collection by ID.
    pub async fn find(pool: &TaxiiPool, id: i32) -> DatabaseResult<Option<Self>> {
        let collection = sqlx::query_as!(
            Self,
            r#"SELECT id, name as "name!", type as "collection_type!", description,
                      accept_all_content as "accept_all_content!", bindings,
                      available as "available!", volume as "volume!", date_created as "date_created!"
               FROM data_collections WHERE id = $1"#,
            id
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(collection)
    }

    /// Find a collection by name.
    pub async fn find_by_name(pool: &TaxiiPool, name: &str) -> DatabaseResult<Option<Self>> {
        let collection = sqlx::query_as!(
            Self,
            r#"SELECT id, name as "name!", type as "collection_type!", description,
                      accept_all_content as "accept_all_content!", bindings,
                      available as "available!", volume as "volume!", date_created as "date_created!"
               FROM data_collections WHERE name = $1"#,
            name
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(collection)
    }

    /// Find all collections.
    pub async fn find_all(pool: &TaxiiPool) -> DatabaseResult<Vec<Self>> {
        let collections = sqlx::query_as!(
            Self,
            r#"SELECT id, name as "name!", type as "collection_type!", description,
                      accept_all_content as "accept_all_content!", bindings,
                      available as "available!", volume as "volume!", date_created as "date_created!"
               FROM data_collections"#
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(collections)
    }

    /// Find collections by service ID.
    pub async fn find_by_service(pool: &TaxiiPool, service_id: &str) -> DatabaseResult<Vec<Self>> {
        let collections = sqlx::query_as!(
            Self,
            r#"SELECT dc.id, dc.name as "name!", dc.type as "collection_type!", dc.description,
                      dc.accept_all_content as "accept_all_content!", dc.bindings,
                      dc.available as "available!", dc.volume as "volume!", dc.date_created as "date_created!"
               FROM data_collections dc
               JOIN service_to_collection stc ON dc.id = stc.collection_id
               WHERE stc.service_id = $1"#,
            service_id
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(collections)
    }

    /// Find a collection by name and service ID.
    pub async fn find_by_name_and_service(
        pool: &TaxiiPool,
        name: &str,
        service_id: &str,
    ) -> DatabaseResult<Option<Self>> {
        let collection = sqlx::query_as!(
            Self,
            r#"SELECT dc.id, dc.name as "name!", dc.type as "collection_type!", dc.description,
                      dc.accept_all_content as "accept_all_content!", dc.bindings,
                      dc.available as "available!", dc.volume as "volume!", dc.date_created as "date_created!"
               FROM data_collections dc
               JOIN service_to_collection stc ON dc.id = stc.collection_id
               WHERE stc.service_id = $1 AND dc.name = $2"#,
            service_id,
            name
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(collection)
    }

    /// Create a new collection.
    pub async fn create(
        pool: &TaxiiPool,
        name: &str,
        collection_type: &str,
        description: Option<&str>,
        available: bool,
        accept_all_content: bool,
        bindings: Option<&str>,
    ) -> DatabaseResult<Self> {
        let collection = sqlx::query_as!(
            Self,
            r#"INSERT INTO data_collections (name, type, description, available, accept_all_content, bindings)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, name as "name!", type as "collection_type!", description,
                         accept_all_content as "accept_all_content!", bindings,
                         available as "available!", volume as "volume!", date_created as "date_created!""#,
            name,
            collection_type,
            description,
            available,
            accept_all_content,
            bindings
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(collection)
    }

    /// Update a collection.
    pub async fn update(
        pool: &TaxiiPool,
        params: &UpdateDataCollection<'_>,
    ) -> DatabaseResult<Self> {
        let collection = sqlx::query_as!(
            Self,
            r#"UPDATE data_collections
               SET name = $2, type = $3, description = $4, available = $5,
                   accept_all_content = $6, bindings = $7
               WHERE id = $1
               RETURNING id, name as "name!", type as "collection_type!", description,
                         accept_all_content as "accept_all_content!", bindings,
                         available as "available!", volume as "volume!", date_created as "date_created!""#,
            params.id,
            params.name,
            params.collection_type,
            params.description,
            params.available,
            params.accept_all_content,
            params.bindings
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(collection)
    }

    /// Delete a collection by name.
    pub async fn delete_by_name(pool: &TaxiiPool, name: &str) -> DatabaseResult<bool> {
        let result = sqlx::query!("DELETE FROM data_collections WHERE name = $1", name)
            .execute(pool.inner())
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Set services for a collection (without validation).
    pub async fn set_services(
        pool: &TaxiiPool,
        collection_id: i32,
        service_ids: &[String],
    ) -> DatabaseResult<()> {
        // Delete existing links
        sqlx::query!(
            "DELETE FROM service_to_collection WHERE collection_id = $1",
            collection_id
        )
        .execute(pool.inner())
        .await?;

        // Insert new links
        for service_id in service_ids {
            sqlx::query!(
                "INSERT INTO service_to_collection (service_id, collection_id) VALUES ($1, $2)",
                service_id,
                collection_id
            )
            .execute(pool.inner())
            .await?;
        }

        Ok(())
    }

    /// Set services for a collection with validation.
    ///
    /// Validates that the collection exists and all services exist before setting.
    pub async fn set_services_validated(
        pool: &TaxiiPool,
        collection_id: i32,
        service_ids: &[String],
    ) -> DatabaseResult<()> {
        use super::Service;
        use crate::error::DatabaseError;

        // Verify collection exists
        Self::find(pool, collection_id).await?.ok_or_else(|| {
            DatabaseError::NotFound(format!("Collection with id {collection_id} does not exist"))
        })?;

        // Verify all services exist
        if !service_ids.is_empty() {
            let count = Service::count_existing(pool, service_ids).await?;
            if count != service_ids.len() as i64 {
                return Err(DatabaseError::NotFound(
                    "Some services do not exist".to_string(),
                ));
            }
        }

        // Set the services
        Self::set_services(pool, collection_id, service_ids).await
    }

    /// Update volume count for a collection.
    pub async fn update_volume(pool: &TaxiiPool, id: i32, volume: i32) -> DatabaseResult<()> {
        sqlx::query!(
            "UPDATE data_collections SET volume = $2 WHERE id = $1",
            id,
            volume
        )
        .execute(pool.inner())
        .await?;

        Ok(())
    }

    /// Increment volume count for a collection by 1.
    pub async fn increment_volume(pool: &TaxiiPool, id: i32) -> DatabaseResult<()> {
        sqlx::query("UPDATE data_collections SET volume = COALESCE(volume, 0) + 1 WHERE id = $1")
            .bind(id)
            .execute(pool.inner())
            .await?;

        Ok(())
    }

    /// Check if a collection with the given name exists.
    pub async fn exists_by_name(pool: &TaxiiPool, name: &str) -> DatabaseResult<bool> {
        let result = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM data_collections WHERE name = $1) as "exists!""#,
            name
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(result)
    }
}
