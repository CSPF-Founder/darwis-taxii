//! STIXObject model (TAXII 2.x STIX objects).

use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use super::query::{PaginationCursor, Taxii2QueryParams, get_next_param};
use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// Result of a filtered STIX object query.
#[derive(Debug)]
pub struct FilteredResult {
    /// The matching objects.
    pub objects: Vec<STIXObject>,
    /// Whether there are more results.
    pub more: bool,
    /// Pagination cursor for next page.
    pub next: Option<String>,
}

/// Version record from version query (model layer, uses NaiveDateTime).
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// Date added.
    pub date_added: NaiveDateTime,
    /// Object version (modified timestamp).
    pub version: NaiveDateTime,
}

/// Result of a versions query.
#[derive(Debug)]
pub struct VersionsResult {
    /// Version records (None if object doesn't exist).
    pub versions: Option<Vec<VersionInfo>>,
    /// Whether there are more results.
    pub more: bool,
    /// Pagination cursor for next page.
    pub next: Option<String>,
}

/// STIXObject database row.
///
/// Table: opentaxii_stixobject
#[derive(Debug, Clone, FromRow)]
pub struct STIXObject {
    /// Internal primary key (UUID).
    pub pk: Uuid,

    /// STIX object ID (e.g., "indicator--...").
    pub id: String,

    /// Foreign key to collection.
    pub collection_id: Uuid,

    /// STIX object type.
    #[sqlx(rename = "type")]
    pub stix_type: String,

    /// STIX spec version (e.g., "2.1").
    pub spec_version: String,

    /// When the object was added to this collection.
    /// Stored as timestamp without timezone in PostgreSQL.
    pub date_added: NaiveDateTime,

    /// Object version (from "modified" field).
    /// Stored as timestamp without timezone in PostgreSQL.
    pub version: NaiveDateTime,

    /// Serialized STIX data as JSON.
    pub serialized_data: Value,
}

/// Parameters for creating a new STIX object.
#[derive(Debug, Clone)]
pub struct NewSTIXObject<'a> {
    pub id: &'a str,
    pub collection_id: Uuid,
    pub stix_type: &'a str,
    pub spec_version: &'a str,
    /// Version timestamp. Stored as timestamp without timezone in PostgreSQL.
    pub version: NaiveDateTime,
    pub serialized_data: &'a Value,
}

impl STIXObject {
    /// Check if an object exists by ID, collection, and version.
    pub async fn exists(
        pool: &TaxiiPool,
        stix_id: &str,
        collection_id: Uuid,
        version: NaiveDateTime,
    ) -> DatabaseResult<bool> {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                   SELECT 1 FROM opentaxii_stixobject
                   WHERE id = $1 AND collection_id = $2 AND version = $3
               ) as "exists!""#,
            stix_id,
            collection_id,
            version
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(exists)
    }

    /// Check if any version of an object exists in a collection.
    pub async fn exists_any_version(
        pool: &TaxiiPool,
        stix_id: &str,
        collection_id: Uuid,
    ) -> DatabaseResult<bool> {
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(
                   SELECT 1 FROM opentaxii_stixobject
                   WHERE id = $1 AND collection_id = $2
               ) as "exists!""#,
            stix_id,
            collection_id
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(exists)
    }

    /// Create a new STIX object.
    pub async fn create(pool: &TaxiiPool, params: &NewSTIXObject<'_>) -> DatabaseResult<Self> {
        let pk = Uuid::new_v4();
        let date_added = Utc::now().naive_utc();

        let obj = sqlx::query_as!(
            Self,
            r#"INSERT INTO opentaxii_stixobject (pk, id, collection_id, type, spec_version, date_added, version, serialized_data)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8::json)
               RETURNING pk, id as "id!", collection_id as "collection_id!", type as "stix_type!",
                         spec_version as "spec_version!", date_added as "date_added!", version as "version!",
                         serialized_data as "serialized_data!""#,
            pk,
            params.id,
            params.collection_id,
            params.stix_type,
            params.spec_version,
            date_added,
            params.version,
            params.serialized_data
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(obj)
    }

    /// Delete all versions of an object.
    pub async fn delete_all_versions(
        pool: &TaxiiPool,
        collection_id: Uuid,
        stix_id: &str,
    ) -> DatabaseResult<u64> {
        let result = sqlx::query!(
            "DELETE FROM opentaxii_stixobject WHERE collection_id = $1 AND id = $2",
            collection_id,
            stix_id
        )
        .execute(pool.inner())
        .await?;

        Ok(result.rows_affected())
    }

    /// Delete the first (oldest) version of an object.
    pub async fn delete_first_version(
        pool: &TaxiiPool,
        collection_id: Uuid,
        stix_id: &str,
    ) -> DatabaseResult<u64> {
        let result = sqlx::query!(
            r#"DELETE FROM opentaxii_stixobject
               WHERE pk IN (
                   SELECT pk FROM opentaxii_stixobject
                   WHERE collection_id = $1 AND id = $2
                   ORDER BY version ASC
                   LIMIT 1
               )"#,
            collection_id,
            stix_id
        )
        .execute(pool.inner())
        .await?;

        Ok(result.rows_affected())
    }

    /// Delete the last (newest) version of an object.
    pub async fn delete_last_version(
        pool: &TaxiiPool,
        collection_id: Uuid,
        stix_id: &str,
    ) -> DatabaseResult<u64> {
        let result = sqlx::query!(
            r#"DELETE FROM opentaxii_stixobject
               WHERE pk IN (
                   SELECT pk FROM opentaxii_stixobject
                   WHERE collection_id = $1 AND id = $2
                   ORDER BY version DESC
                   LIMIT 1
               )"#,
            collection_id,
            stix_id
        )
        .execute(pool.inner())
        .await?;

        Ok(result.rows_affected())
    }

    /// Delete objects with flexible filtering.
    ///
    /// Supports:
    /// - Delete all versions (match_version: "all")
    /// - Delete first version (match_version: "first")
    /// - Delete last version (match_version: "last")
    /// - Delete specific versions by timestamp
    /// - Filter by spec_version
    ///
    /// Note: For simple cases without spec_version filtering, this delegates to
    /// the cached sqlx::query! macro methods. For complex filtering, dynamic queries are used.
    pub async fn delete_filtered(
        pool: &TaxiiPool,
        collection_id: Uuid,
        stix_id: &str,
        match_version: Option<&[String]>,
        match_spec_version: Option<&[String]>,
    ) -> DatabaseResult<u64> {
        let default_version = vec!["all".to_string()];
        let effective_version = match_version.unwrap_or(&default_version);

        let has_all = effective_version.iter().any(|v| v == "all");
        let has_first = effective_version.iter().any(|v| v == "first");
        let has_last = effective_version.iter().any(|v| v == "last");

        // Collect specific datetime versions
        let specific_versions: Vec<&str> = effective_version
            .iter()
            .filter(|v| *v != "all" && *v != "first" && *v != "last")
            .map(|s| s.as_str())
            .collect();

        if has_all {
            if match_spec_version.is_none() {
                // Simple case: use cached sqlx::query! macro
                return Self::delete_all_versions(pool, collection_id, stix_id).await;
            }
            // Complex case: delete all with spec_version filter (dynamic query)
            let result = sqlx::query(
                "DELETE FROM opentaxii_stixobject WHERE collection_id = $1 AND id = $2 AND spec_version = ANY($3)",
            )
            .bind(collection_id)
            .bind(stix_id)
            .bind(match_spec_version)
            .execute(pool.inner())
            .await?;
            Ok(result.rows_affected())
        } else if has_first {
            // Use cached sqlx::query! macro
            // Note: spec_version filter not supported for first/last - matches TAXII spec
            Self::delete_first_version(pool, collection_id, stix_id).await
        } else if has_last {
            // Use cached sqlx::query! macro
            Self::delete_last_version(pool, collection_id, stix_id).await
        } else if !specific_versions.is_empty() {
            // Delete specific versions by timestamp (dynamic query)
            let version_strings: Vec<String> =
                specific_versions.iter().map(|s| s.to_string()).collect();

            if let Some(spec_versions) = match_spec_version {
                let result = sqlx::query(
                    "DELETE FROM opentaxii_stixobject WHERE collection_id = $1 AND id = $2 AND version = ANY($3::timestamptz[]) AND spec_version = ANY($4)",
                )
                .bind(collection_id)
                .bind(stix_id)
                .bind(&version_strings)
                .bind(spec_versions)
                .execute(pool.inner())
                .await?;
                Ok(result.rows_affected())
            } else {
                let result = sqlx::query(
                    "DELETE FROM opentaxii_stixobject WHERE collection_id = $1 AND id = $2 AND version = ANY($3::timestamptz[])",
                )
                .bind(collection_id)
                .bind(stix_id)
                .bind(&version_strings)
                .execute(pool.inner())
                .await?;
                Ok(result.rows_affected())
            }
        } else {
            // No matching criteria - nothing to delete
            Ok(0)
        }
    }

    /// Find STIX objects with filtering and pagination.
    ///
    /// Supports filtering by ID, type, version, spec_version, and pagination
    /// with cursor-based next parameter.
    pub async fn find_filtered(
        pool: &TaxiiPool,
        collection_id: Uuid,
        params: &Taxii2QueryParams<'_>,
    ) -> DatabaseResult<FilteredResult> {
        let Taxii2QueryParams {
            limit,
            added_after,
            next: next_kwargs,
            match_id,
            match_type,
            match_version,
            match_spec_version,
        } = params;

        // Build base query
        let mut query = String::from(
            r#"SELECT pk, id, collection_id, type, spec_version, date_added, version, serialized_data
               FROM opentaxii_stixobject
               WHERE collection_id = $1"#,
        );

        let mut param_idx = 2;

        if added_after.is_some() {
            query.push_str(&format!(" AND date_added > ${param_idx}"));
            param_idx += 1;
        }

        if next_kwargs.is_some() {
            query.push_str(&format!(
                " AND (date_added > ${} OR (date_added = ${} AND id > ${}))",
                param_idx,
                param_idx,
                param_idx + 1
            ));
            param_idx += 2;
        }

        if match_id.is_some() {
            query.push_str(&format!(" AND id = ANY(${param_idx})"));
            param_idx += 1;
        }

        if match_type.is_some() {
            query.push_str(&format!(" AND type = ANY(${param_idx})"));
            param_idx += 1;
        }

        if match_spec_version.is_some() {
            query.push_str(&format!(" AND spec_version = ANY(${param_idx})"));
            param_idx += 1;
        }

        // Handle match_version - default to "last"
        let default_version = vec!["last".to_string()];
        let effective_version = match_version.unwrap_or(&default_version);

        let has_all = effective_version.iter().any(|v| v == "all");
        let has_first = effective_version.iter().any(|v| v == "first");
        let has_last = effective_version.iter().any(|v| v == "last");

        // Collect specific datetime versions
        let specific_versions: Vec<&str> = effective_version
            .iter()
            .filter(|v| *v != "all" && *v != "first" && *v != "last")
            .map(|s| s.as_str())
            .collect();

        if !has_all {
            if has_first {
                // Get first version using DISTINCT ON with ASC ordering
                query = r#"SELECT DISTINCT ON (id) pk, id, collection_id, type, spec_version, date_added, version, serialized_data
                       FROM opentaxii_stixobject
                       WHERE collection_id = $1"#
                    .to_string();
                param_idx = 2;
                if added_after.is_some() {
                    query.push_str(&format!(" AND date_added > ${param_idx}"));
                    param_idx += 1;
                }
                if next_kwargs.is_some() {
                    query.push_str(&format!(
                        " AND (date_added > ${} OR (date_added = ${} AND id > ${}))",
                        param_idx,
                        param_idx,
                        param_idx + 1
                    ));
                    param_idx += 2;
                }
                if match_id.is_some() {
                    query.push_str(&format!(" AND id = ANY(${param_idx})"));
                    param_idx += 1;
                }
                if match_type.is_some() {
                    query.push_str(&format!(" AND type = ANY(${param_idx})"));
                    param_idx += 1;
                }
                if match_spec_version.is_some() {
                    query.push_str(&format!(" AND spec_version = ANY(${param_idx})"));
                }
                query.push_str(" ORDER BY id, version ASC");
            } else if has_last {
                // Get last version using DISTINCT ON with DESC ordering
                query = r#"SELECT DISTINCT ON (id) pk, id, collection_id, type, spec_version, date_added, version, serialized_data
                       FROM opentaxii_stixobject
                       WHERE collection_id = $1"#
                    .to_string();
                param_idx = 2;
                if added_after.is_some() {
                    query.push_str(&format!(" AND date_added > ${param_idx}"));
                    param_idx += 1;
                }
                if next_kwargs.is_some() {
                    query.push_str(&format!(
                        " AND (date_added > ${} OR (date_added = ${} AND id > ${}))",
                        param_idx,
                        param_idx,
                        param_idx + 1
                    ));
                    param_idx += 2;
                }
                if match_id.is_some() {
                    query.push_str(&format!(" AND id = ANY(${param_idx})"));
                    param_idx += 1;
                }
                if match_type.is_some() {
                    query.push_str(&format!(" AND type = ANY(${param_idx})"));
                    param_idx += 1;
                }
                if match_spec_version.is_some() {
                    query.push_str(&format!(" AND spec_version = ANY(${param_idx})"));
                }
                query.push_str(" ORDER BY id, version DESC");
            } else if !specific_versions.is_empty() {
                // Filter by specific version timestamps
                query.push_str(&format!(" AND version = ANY(${param_idx}::timestamptz[])"));
            }
        }

        // Wrap DISTINCT ON query for final ordering
        if has_first || has_last {
            query = format!("SELECT * FROM ({query}) AS subq ORDER BY date_added, id");
        } else {
            query.push_str(" ORDER BY date_added, id");
        }

        // Apply limit + 1 for efficient "more" detection
        let fetch_limit = limit.map(|lim| lim + 1);
        if let Some(lim) = fetch_limit {
            query.push_str(&format!(" LIMIT {lim}"));
        }

        // Bind parameters
        let mut q = sqlx::query_as::<_, Self>(&query);
        q = q.bind(collection_id);

        if let Some(aa) = added_after {
            q = q.bind(aa);
        }

        if let Some(cursor) = next_kwargs {
            q = q.bind(cursor.date_added);
            q = q.bind(&cursor.object_id);
        }

        if let Some(ids) = match_id {
            q = q.bind(ids);
        }

        if let Some(types) = match_type {
            q = q.bind(types);
        }

        if let Some(versions) = match_spec_version {
            q = q.bind(versions);
        }

        // Bind specific version timestamps if provided
        if !specific_versions.is_empty() {
            let version_strings: Vec<String> =
                specific_versions.iter().map(|s| s.to_string()).collect();
            q = q.bind(version_strings);
        }

        let mut items: Vec<Self> = q.fetch_all(pool.inner()).await?;

        // Determine if more results
        let more = if let Some(lim) = *limit {
            items.len() as i64 > lim
        } else {
            false
        };

        // Truncate to actual limit
        if let Some(lim) = *limit {
            items.truncate(lim as usize);
        }

        // Get next param for pagination
        let next = if more {
            items
                .last()
                .map(|last| get_next_param(&last.date_added, &last.id))
        } else {
            None
        };

        Ok(FilteredResult {
            objects: items,
            more,
            next,
        })
    }

    /// Get versions of a specific object.
    ///
    /// Returns None for versions if the object doesn't exist in the collection.
    pub async fn find_versions(
        pool: &TaxiiPool,
        collection_id: Uuid,
        object_id: &str,
        limit: Option<i64>,
        added_after: Option<DateTime<Utc>>,
        next_kwargs: Option<&PaginationCursor>,
        match_spec_version: Option<&[String]>,
    ) -> DatabaseResult<VersionsResult> {
        // Check if object exists
        let exists = Self::exists_any_version(pool, object_id, collection_id).await?;

        if !exists {
            return Ok(VersionsResult {
                versions: None,
                more: false,
                next: None,
            });
        }

        let mut query = String::from(
            r#"SELECT date_added, version, id
               FROM opentaxii_stixobject
               WHERE collection_id = $1 AND id = $2"#,
        );

        let mut param_idx = 3;

        if added_after.is_some() {
            query.push_str(&format!(" AND date_added > ${param_idx}"));
            param_idx += 1;
        }

        if next_kwargs.is_some() {
            query.push_str(&format!(
                " AND (date_added > ${} OR (date_added = ${} AND id > ${}))",
                param_idx,
                param_idx,
                param_idx + 1
            ));
            param_idx += 2;
        }

        if match_spec_version.is_some() {
            query.push_str(&format!(" AND spec_version = ANY(${param_idx})"));
        }

        query.push_str(" ORDER BY date_added, id");

        // Apply limit + 1 for efficient "more" detection
        let fetch_limit = limit.map(|lim| lim + 1);
        if let Some(lim) = fetch_limit {
            query.push_str(&format!(" LIMIT {lim}"));
        }

        // Bind parameters
        let mut q = sqlx::query(&query);
        q = q.bind(collection_id).bind(object_id);

        if let Some(aa) = added_after {
            q = q.bind(aa);
        }

        if let Some(cursor) = next_kwargs {
            q = q.bind(cursor.date_added);
            q = q.bind(&cursor.object_id);
        }

        if let Some(spec_versions) = match_spec_version {
            q = q.bind(spec_versions);
        }

        let rows = q.fetch_all(pool.inner()).await?;

        // Determine if more results
        let more = if let Some(lim) = limit {
            rows.len() as i64 > lim
        } else {
            false
        };

        // Truncate to actual limit
        let rows: Vec<_> = if let Some(lim) = limit {
            rows.into_iter().take(lim as usize).collect()
        } else {
            rows
        };

        // Generate next_param for pagination
        let next = if more {
            rows.last().map(|row| {
                use sqlx::Row;
                let date_added: NaiveDateTime = row.get("date_added");
                let id: String = row.get("id");
                get_next_param(&date_added, &id)
            })
        } else {
            None
        };

        let versions: Vec<VersionInfo> = rows
            .iter()
            .map(|row| {
                use sqlx::Row;
                VersionInfo {
                    date_added: row.get("date_added"),
                    version: row.get("version"),
                }
            })
            .collect();

        Ok(VersionsResult {
            versions: Some(versions),
            more,
            next,
        })
    }
}
