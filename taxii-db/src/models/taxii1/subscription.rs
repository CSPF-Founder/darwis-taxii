//! Subscription model.

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// Subscription database row.
///
/// Table: subscriptions
#[derive(Debug, Clone, FromRow)]
pub struct Subscription {
    /// Primary key (string).
    pub id: String,

    /// Foreign key to data_collections.id.
    pub collection_id: i32,

    /// Subscription parameters as JSON text.
    /// Contains: response_type, content_bindings
    pub params: Option<String>,

    /// Subscription status.
    pub status: String,

    /// Foreign key to services.id.
    pub service_id: String,

    /// Row creation timestamp.
    pub date_created: DateTime<Utc>,
}

/// Subscription status constants.
pub mod status {
    pub const ACTIVE: &str = "ACTIVE";
    pub const PAUSED: &str = "PAUSED";
    pub const UNSUBSCRIBED: &str = "UNSUBSCRIBED";
}

impl Subscription {
    /// Find a subscription by ID.
    pub async fn find(pool: &TaxiiPool, id: &str) -> DatabaseResult<Option<Self>> {
        let subscription = sqlx::query_as!(
            Self,
            r#"SELECT id, collection_id as "collection_id!", params, status as "status!",
                      service_id as "service_id!", date_created as "date_created!"
               FROM subscriptions WHERE id = $1"#,
            id
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(subscription)
    }

    /// Find all subscriptions for a service.
    pub async fn find_by_service(pool: &TaxiiPool, service_id: &str) -> DatabaseResult<Vec<Self>> {
        let subscriptions = sqlx::query_as!(
            Self,
            r#"SELECT id, collection_id as "collection_id!", params, status as "status!",
                      service_id as "service_id!", date_created as "date_created!"
               FROM subscriptions WHERE service_id = $1"#,
            service_id
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(subscriptions)
    }

    /// Upsert a subscription (insert or update).
    /// Uses transaction with SELECT FOR UPDATE for atomicity.
    pub async fn upsert(
        pool: &TaxiiPool,
        id: &str,
        collection_id: i32,
        params: Option<&str>,
        status: &str,
        service_id: &str,
    ) -> DatabaseResult<Self> {
        // Use a transaction for atomicity
        let mut tx = pool.inner().begin().await?;

        // Check if exists with row lock
        let existing = sqlx::query_scalar!(
            r#"SELECT id FROM subscriptions WHERE id = $1 FOR UPDATE"#,
            id
        )
        .fetch_optional(&mut *tx)
        .await?;

        if existing.is_some() {
            // Update existing
            sqlx::query!(
                r#"UPDATE subscriptions SET collection_id = $2, params = $3, status = $4, service_id = $5 WHERE id = $1"#,
                id,
                collection_id,
                params,
                status,
                service_id
            )
            .execute(&mut *tx)
            .await?;
        } else {
            // Insert new
            sqlx::query!(
                r#"INSERT INTO subscriptions (id, collection_id, params, status, service_id) VALUES ($1, $2, $3, $4, $5)"#,
                id,
                collection_id,
                params,
                status,
                service_id
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Self::find(pool, id)
            .await?
            .ok_or_else(|| crate::error::DatabaseError::not_found("Failed to upsert subscription"))
    }

    /// Delete a subscription by ID.
    pub async fn delete(pool: &TaxiiPool, id: &str) -> DatabaseResult<bool> {
        let result = sqlx::query!("DELETE FROM subscriptions WHERE id = $1", id)
            .execute(pool.inner())
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
