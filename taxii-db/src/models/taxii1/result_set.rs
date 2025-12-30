//! ResultSet model.

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// ResultSet database row.
///
/// Table: result_sets
///
/// NOTE: id is a String, not an Integer!
#[derive(Debug, Clone, FromRow)]
pub struct ResultSet {
    /// Primary key (string).
    pub id: String,

    /// Foreign key to data_collections.id.
    pub collection_id: i32,

    /// Content bindings as JSON text.
    pub bindings: Option<String>,

    /// Begin time of the result set.
    pub begin_time: Option<DateTime<Utc>>,

    /// End time of the result set.
    pub end_time: Option<DateTime<Utc>>,

    /// Row creation timestamp.
    pub date_created: DateTime<Utc>,
}

impl ResultSet {
    /// Find a result set by ID.
    pub async fn find(pool: &TaxiiPool, id: &str) -> DatabaseResult<Option<Self>> {
        let result_set = sqlx::query_as!(
            Self,
            r#"SELECT id, collection_id as "collection_id!", bindings, begin_time, end_time,
                      date_created as "date_created!"
               FROM result_sets WHERE id = $1"#,
            id
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(result_set)
    }

    /// Create a new result set.
    pub async fn create(
        pool: &TaxiiPool,
        id: &str,
        collection_id: i32,
        bindings: Option<&str>,
        begin_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> DatabaseResult<Self> {
        let result_set = sqlx::query_as!(
            Self,
            r#"INSERT INTO result_sets (id, collection_id, bindings, begin_time, end_time)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id, collection_id as "collection_id!", bindings, begin_time, end_time,
                         date_created as "date_created!""#,
            id,
            collection_id,
            bindings,
            begin_time,
            end_time
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(result_set)
    }

    /// Delete a result set by ID.
    pub async fn delete(pool: &TaxiiPool, id: &str) -> DatabaseResult<bool> {
        let result = sqlx::query!("DELETE FROM result_sets WHERE id = $1", id)
            .execute(pool.inner())
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
