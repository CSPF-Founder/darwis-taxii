//! InboxMessage model.

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// InboxMessage database row.
///
/// Table: inbox_messages
#[derive(Debug, Clone, FromRow)]
pub struct InboxMessage {
    /// Primary key.
    pub id: i32,

    /// TAXII message ID.
    pub message_id: String,

    /// Result ID for async responses.
    pub result_id: Option<String>,

    /// Record count.
    pub record_count: Option<i32>,

    /// Whether count is partial.
    pub partial_count: bool,

    /// Subscription collection name.
    pub subscription_collection_name: Option<String>,

    /// Subscription ID.
    pub subscription_id: Option<String>,

    /// Exclusive begin timestamp.
    pub exclusive_begin_timestamp_label: Option<DateTime<Utc>>,

    /// Inclusive end timestamp.
    pub inclusive_end_timestamp_label: Option<DateTime<Utc>>,

    /// Original message bytes.
    pub original_message: Vec<u8>,

    /// Content block count.
    pub content_block_count: i32,

    /// Destination collections as JSON text.
    pub destination_collections: Option<String>,

    /// Foreign key to services.id.
    pub service_id: String,

    /// Row creation timestamp.
    pub date_created: DateTime<Utc>,
}

/// Parameters for creating a new inbox message.
#[derive(Debug, Clone, Default)]
pub struct NewInboxMessage<'a> {
    pub message_id: &'a str,
    pub original_message: &'a [u8],
    pub content_block_count: i32,
    pub destination_collections: Option<&'a str>,
    pub service_id: &'a str,
    pub result_id: Option<&'a str>,
    pub record_count: Option<i32>,
    pub partial_count: bool,
    pub subscription_collection_name: Option<&'a str>,
    pub subscription_id: Option<&'a str>,
    pub exclusive_begin_timestamp_label: Option<DateTime<Utc>>,
    pub inclusive_end_timestamp_label: Option<DateTime<Utc>>,
}

impl InboxMessage {
    /// Find an inbox message by ID.
    pub async fn find(pool: &TaxiiPool, id: i32) -> DatabaseResult<Option<Self>> {
        let message = sqlx::query_as!(
            Self,
            r#"SELECT id, message_id as "message_id!", result_id, record_count,
                      partial_count as "partial_count!", subscription_collection_name, subscription_id,
                      exclusive_begin_timestamp_label, inclusive_end_timestamp_label,
                      original_message, content_block_count as "content_block_count!",
                      destination_collections, service_id as "service_id!", date_created as "date_created!"
               FROM inbox_messages WHERE id = $1"#,
            id
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(message)
    }

    /// Create a new inbox message.
    pub async fn create(pool: &TaxiiPool, params: &NewInboxMessage<'_>) -> DatabaseResult<Self> {
        let message = sqlx::query_as!(
            Self,
            r#"INSERT INTO inbox_messages (
                   message_id, original_message, content_block_count, destination_collections,
                   service_id, result_id, record_count, partial_count,
                   subscription_collection_name, subscription_id,
                   exclusive_begin_timestamp_label, inclusive_end_timestamp_label
               )
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               RETURNING id, message_id as "message_id!", result_id, record_count,
                         partial_count as "partial_count!", subscription_collection_name, subscription_id,
                         exclusive_begin_timestamp_label, inclusive_end_timestamp_label,
                         original_message, content_block_count as "content_block_count!",
                         destination_collections, service_id as "service_id!", date_created as "date_created!""#,
            params.message_id,
            params.original_message,
            params.content_block_count,
            params.destination_collections,
            params.service_id,
            params.result_id,
            params.record_count,
            params.partial_count,
            params.subscription_collection_name,
            params.subscription_id,
            params.exclusive_begin_timestamp_label,
            params.inclusive_end_timestamp_label
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(message)
    }

    /// Delete inbox messages by IDs.
    pub async fn delete_many(pool: &TaxiiPool, ids: &[i32]) -> DatabaseResult<u64> {
        let result = sqlx::query!("DELETE FROM inbox_messages WHERE id = ANY($1)", ids)
            .execute(pool.inner())
            .await?;

        Ok(result.rows_affected())
    }
}
