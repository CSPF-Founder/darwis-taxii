//! Service model.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;

use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// Service database row.
///
/// Table: services
#[derive(Debug, Clone, FromRow)]
pub struct Service {
    /// Primary key (string).
    pub id: String,

    /// Service type.
    #[sqlx(rename = "type")]
    pub service_type: String,

    /// Properties as JSON text.
    #[sqlx(rename = "_properties")]
    pub properties_json: String,

    /// Last update timestamp.
    pub date_updated: Option<DateTime<Utc>>,

    /// Row creation timestamp.
    pub date_created: DateTime<Utc>,
}

impl Service {
    /// Parse properties as JSON.
    pub fn properties(&self) -> DatabaseResult<Value> {
        let value = serde_json::from_str(&self.properties_json)?;
        Ok(value)
    }

    /// Find a service by ID.
    pub async fn find(pool: &TaxiiPool, id: &str) -> DatabaseResult<Option<Self>> {
        let service = sqlx::query_as!(
            Self,
            r#"SELECT id, type as "service_type!", _properties as "properties_json",
                      date_updated, date_created as "date_created!"
               FROM services WHERE id = $1"#,
            id
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(service)
    }

    /// Find all services.
    pub async fn find_all(pool: &TaxiiPool) -> DatabaseResult<Vec<Self>> {
        let services = sqlx::query_as!(
            Self,
            r#"SELECT id, type as "service_type!", _properties as "properties_json",
                      date_updated, date_created as "date_created!"
               FROM services"#
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(services)
    }

    /// Find services by collection ID.
    pub async fn find_by_collection(
        pool: &TaxiiPool,
        collection_id: i32,
    ) -> DatabaseResult<Vec<Self>> {
        let services = sqlx::query_as!(
            Self,
            r#"SELECT s.id, s.type as "service_type!", s._properties as "properties_json",
                      s.date_updated, s.date_created as "date_created!"
               FROM services s
               JOIN service_to_collection stc ON s.id = stc.service_id
               WHERE stc.collection_id = $1"#,
            collection_id
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(services)
    }

    /// Find services by collection ID and type.
    pub async fn find_by_collection_and_type(
        pool: &TaxiiPool,
        collection_id: i32,
        service_type: &str,
    ) -> DatabaseResult<Vec<Self>> {
        let services = sqlx::query_as!(
            Self,
            r#"SELECT s.id, s.type as "service_type!", s._properties as "properties_json",
                      s.date_updated, s.date_created as "date_created!"
               FROM services s
               JOIN service_to_collection stc ON s.id = stc.service_id
               WHERE stc.collection_id = $1 AND s.type = $2"#,
            collection_id,
            service_type
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(services)
    }

    /// Upsert a service (insert or update).
    /// Uses transaction with SELECT FOR UPDATE for atomicity.
    pub async fn upsert(
        pool: &TaxiiPool,
        id: &str,
        service_type: &str,
        properties_json: &str,
    ) -> DatabaseResult<Self> {
        // Use a transaction for atomicity
        let mut tx = pool.inner().begin().await?;

        // Check if exists with row lock
        let existing =
            sqlx::query_scalar!(r#"SELECT id FROM services WHERE id = $1 FOR UPDATE"#, id)
                .fetch_optional(&mut *tx)
                .await?;

        if existing.is_some() {
            // Update existing
            sqlx::query!(
                r#"UPDATE services SET type = $2, _properties = $3, date_updated = NOW() WHERE id = $1"#,
                id,
                service_type,
                properties_json
            )
            .execute(&mut *tx)
            .await?;
        } else {
            // Insert new
            sqlx::query!(
                r#"INSERT INTO services (id, type, _properties) VALUES ($1, $2, $3)"#,
                id,
                service_type,
                properties_json
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Self::find(pool, id)
            .await?
            .ok_or_else(|| crate::error::DatabaseError::not_found("Failed to upsert service"))
    }

    /// Delete a service by ID.
    pub async fn delete(pool: &TaxiiPool, id: &str) -> DatabaseResult<bool> {
        let result = sqlx::query!("DELETE FROM services WHERE id = $1", id)
            .execute(pool.inner())
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Count how many of the given service IDs exist.
    pub async fn count_existing(pool: &TaxiiPool, ids: &[String]) -> DatabaseResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM services WHERE id = ANY($1)")
            .bind(ids)
            .fetch_one(pool.inner())
            .await?;

        Ok(count)
    }

    /// Get configured domain from service properties.
    pub fn get_domain(&self) -> Option<String> {
        self.properties()
            .ok()
            .and_then(|p| p.get("domain").cloned())
            .and_then(|v| v.as_str().map(String::from))
            .filter(|s| !s.is_empty())
    }

    /// Get advertised service IDs from properties.
    pub fn get_advertised_service_ids(&self) -> Vec<String> {
        self.properties()
            .ok()
            .and_then(|p| p.get("advertised_services").cloned())
            .and_then(|v| v.as_array().cloned())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }
}
