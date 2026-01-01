//! ContentBlock model.

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// Content binding filter for queries.
#[derive(Debug, Clone)]
pub struct ContentBindingFilter {
    /// Binding ID (e.g., "urn:stix.mitre.org:xml:1.1.1").
    pub binding: String,
    /// Optional subtypes to filter on.
    pub subtypes: Vec<String>,
}

/// Filter parameters for content block queries.
#[derive(Debug, Default)]
pub struct ContentBlockFilter<'a> {
    /// Filter by collection ID.
    pub collection_id: Option<i32>,
    /// Filter to blocks after this time (exclusive).
    pub start_time: Option<DateTime<Utc>>,
    /// Filter to blocks before or at this time (inclusive).
    pub end_time: Option<DateTime<Utc>>,
    /// Filter by content bindings.
    pub bindings: Option<&'a [ContentBindingFilter]>,
    /// Offset for pagination.
    pub offset: i64,
    /// Limit results.
    pub limit: Option<i64>,
}

/// ContentBlock database row.
///
/// Table: content_blocks
#[derive(Debug, Clone, FromRow)]
pub struct ContentBlock {
    /// Primary key.
    pub id: i32,

    /// Optional message text.
    pub message: Option<String>,

    /// Timestamp label (indexed).
    pub timestamp_label: DateTime<Utc>,

    /// Foreign key to inbox_messages.id.
    pub inbox_message_id: Option<i32>,

    /// Binary content (BYTEA in PostgreSQL).
    pub content: Vec<u8>,

    /// Content binding ID (indexed).
    pub binding_id: Option<String>,

    /// Content binding subtype (indexed).
    pub binding_subtype: Option<String>,

    /// Row creation timestamp.
    pub date_created: DateTime<Utc>,
}

impl ContentBlock {
    /// Find a content block by ID.
    pub async fn find(pool: &TaxiiPool, id: i32) -> DatabaseResult<Option<Self>> {
        let block = sqlx::query_as!(
            Self,
            r#"SELECT id, message, timestamp_label as "timestamp_label!", inbox_message_id, content,
                      binding_id, binding_subtype, date_created as "date_created!"
               FROM content_blocks WHERE id = $1"#,
            id
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(block)
    }

    /// Create a new content block.
    pub async fn create(
        pool: &TaxiiPool,
        timestamp_label: DateTime<Utc>,
        inbox_message_id: Option<i32>,
        content: &[u8],
        binding_id: Option<&str>,
        binding_subtype: Option<&str>,
    ) -> DatabaseResult<Self> {
        let block = sqlx::query_as!(
            Self,
            r#"INSERT INTO content_blocks (timestamp_label, inbox_message_id, content, binding_id, binding_subtype)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id, message, timestamp_label as "timestamp_label!", inbox_message_id, content,
                         binding_id, binding_subtype, date_created as "date_created!""#,
            timestamp_label,
            inbox_message_id,
            content,
            binding_id,
            binding_subtype
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(block)
    }

    /// Attach content block to a single collection.
    pub async fn attach_to_collection(
        pool: &TaxiiPool,
        content_block_id: i32,
        collection_id: i32,
    ) -> DatabaseResult<()> {
        sqlx::query!(
            "INSERT INTO collection_to_content_block (collection_id, content_block_id) VALUES ($1, $2)",
            collection_id,
            content_block_id
        )
        .execute(pool.inner())
        .await?;

        Ok(())
    }

    /// Attach content block to multiple collections.
    pub async fn attach_to_collections(
        pool: &TaxiiPool,
        content_block_id: i32,
        collection_ids: &[i32],
    ) -> DatabaseResult<()> {
        for coll_id in collection_ids {
            Self::attach_to_collection(pool, content_block_id, *coll_id).await?;
        }

        Ok(())
    }

    /// Delete content blocks by IDs.
    pub async fn delete_many(pool: &TaxiiPool, ids: &[i32]) -> DatabaseResult<u64> {
        let result = sqlx::query!("DELETE FROM content_blocks WHERE id = ANY($1)", ids)
            .execute(pool.inner())
            .await?;

        Ok(result.rows_affected())
    }

    /// Get content block IDs for a collection within a time range.
    pub async fn find_ids_by_collection_and_time(
        pool: &TaxiiPool,
        collection_id: i32,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
    ) -> DatabaseResult<Vec<i32>> {
        let ids = if let Some(et) = end_time {
            sqlx::query_scalar!(
                r#"SELECT cb.id
                   FROM content_blocks cb
                   JOIN collection_to_content_block ctcb ON cb.id = ctcb.content_block_id
                   WHERE ctcb.collection_id = $1 AND cb.timestamp_label > $2 AND cb.timestamp_label <= $3"#,
                collection_id,
                start_time,
                et
            )
            .fetch_all(pool.inner())
            .await?
        } else {
            sqlx::query_scalar!(
                r#"SELECT cb.id
                   FROM content_blocks cb
                   JOIN collection_to_content_block ctcb ON cb.id = ctcb.content_block_id
                   WHERE ctcb.collection_id = $1 AND cb.timestamp_label > $2"#,
                collection_id,
                start_time
            )
            .fetch_all(pool.inner())
            .await?
        };

        Ok(ids)
    }

    /// Count content blocks in a collection.
    pub async fn count_by_collection(pool: &TaxiiPool, collection_id: i32) -> DatabaseResult<i64> {
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(cb.id) as "count!"
               FROM content_blocks cb
               JOIN collection_to_content_block ctcb ON cb.id = ctcb.content_block_id
               WHERE ctcb.collection_id = $1"#,
            collection_id
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(count)
    }

    /// Get inbox message IDs for content blocks.
    pub async fn get_inbox_message_ids(
        pool: &TaxiiPool,
        content_block_ids: &[i32],
    ) -> DatabaseResult<Vec<i32>> {
        let ids = sqlx::query_scalar!(
            r#"SELECT DISTINCT inbox_message_id as "inbox_message_id!"
               FROM content_blocks
               WHERE id = ANY($1) AND inbox_message_id IS NOT NULL"#,
            content_block_ids
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(ids)
    }

    /// Find content blocks with filtering.
    ///
    /// Supports filtering by collection, time range, and content bindings.
    pub async fn find_filtered(
        pool: &TaxiiPool,
        filter: &ContentBlockFilter<'_>,
    ) -> DatabaseResult<Vec<Self>> {
        // Build dynamic query
        let mut query = String::from(
            r#"SELECT cb.id, cb.message, cb.timestamp_label, cb.inbox_message_id,
                      cb.content, cb.binding_id, cb.binding_subtype, cb.date_created
               FROM content_blocks cb"#,
        );

        let mut conditions = Vec::new();
        let mut param_idx = 1;

        if filter.collection_id.is_some() {
            query.push_str(
                " JOIN collection_to_content_block ctcb ON cb.id = ctcb.content_block_id",
            );
            conditions.push(format!("ctcb.collection_id = ${param_idx}"));
            param_idx += 1;
        }

        if filter.start_time.is_some() {
            conditions.push(format!("cb.timestamp_label > ${param_idx}"));
            param_idx += 1;
        }

        if filter.end_time.is_some() {
            conditions.push(format!("cb.timestamp_label <= ${param_idx}"));
            param_idx += 1;
        }

        // Handle bindings filter
        if let Some(binds) = filter.bindings.filter(|b| !b.is_empty()) {
            let mut binding_conditions = Vec::new();
            for binding in binds {
                if binding.subtypes.is_empty() {
                    binding_conditions.push(format!("cb.binding_id = ${param_idx}"));
                    param_idx += 1;
                } else {
                    binding_conditions.push(format!(
                        "(cb.binding_id = ${} AND cb.binding_subtype = ANY(${}::text[]))",
                        param_idx,
                        param_idx + 1
                    ));
                    param_idx += 2;
                }
            }
            conditions.push(format!("({})", binding_conditions.join(" OR ")));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY cb.timestamp_label ASC");

        if let Some(lim) = filter.limit {
            query.push_str(&format!(" LIMIT {lim}"));
        }
        query.push_str(&format!(" OFFSET {}", filter.offset));

        // Execute query with dynamic bindings
        let mut q = sqlx::query_as::<_, Self>(&query);

        if let Some(coll_id) = filter.collection_id {
            q = q.bind(coll_id);
        }
        if let Some(st) = filter.start_time {
            q = q.bind(st);
        }
        if let Some(et) = filter.end_time {
            q = q.bind(et);
        }
        if let Some(binds) = filter.bindings {
            for binding in binds {
                q = q.bind(&binding.binding);
                if !binding.subtypes.is_empty() {
                    q = q.bind(&binding.subtypes);
                }
            }
        }

        let blocks = q.fetch_all(pool.inner()).await?;
        Ok(blocks)
    }

    /// Count content blocks with filtering.
    ///
    /// Supports filtering by collection, time range, and content bindings.
    pub async fn count_filtered(
        pool: &TaxiiPool,
        filter: &ContentBlockFilter<'_>,
    ) -> DatabaseResult<i64> {
        let mut query = String::from("SELECT COUNT(cb.id) FROM content_blocks cb");

        let mut conditions = Vec::new();
        let mut param_idx = 1;

        if filter.collection_id.is_some() {
            query.push_str(
                " JOIN collection_to_content_block ctcb ON cb.id = ctcb.content_block_id",
            );
            conditions.push(format!("ctcb.collection_id = ${param_idx}"));
            param_idx += 1;
        }

        if filter.start_time.is_some() {
            conditions.push(format!("cb.timestamp_label > ${param_idx}"));
            param_idx += 1;
        }

        if filter.end_time.is_some() {
            conditions.push(format!("cb.timestamp_label <= ${param_idx}"));
            param_idx += 1;
        }

        if let Some(binds) = filter.bindings.filter(|b| !b.is_empty()) {
            let mut binding_conditions = Vec::new();
            for binding in binds {
                if binding.subtypes.is_empty() {
                    binding_conditions.push(format!("cb.binding_id = ${param_idx}"));
                    param_idx += 1;
                } else {
                    binding_conditions.push(format!(
                        "(cb.binding_id = ${} AND cb.binding_subtype = ANY(${}::text[]))",
                        param_idx,
                        param_idx + 1
                    ));
                    param_idx += 2;
                }
            }
            conditions.push(format!("({})", binding_conditions.join(" OR ")));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        let mut q = sqlx::query_scalar::<_, i64>(&query);

        if let Some(coll_id) = filter.collection_id {
            q = q.bind(coll_id);
        }
        if let Some(st) = filter.start_time {
            q = q.bind(st);
        }
        if let Some(et) = filter.end_time {
            q = q.bind(et);
        }
        if let Some(binds) = filter.bindings {
            for binding in binds {
                q = q.bind(&binding.binding);
                if !binding.subtypes.is_empty() {
                    q = q.bind(&binding.subtypes);
                }
            }
        }

        let count = q.fetch_one(pool.inner()).await?;
        Ok(count)
    }
}
