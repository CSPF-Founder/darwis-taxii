//! TAXII 2.x Repository implementation.
//!
//! Provides database operations for TAXII 2.x entities including API roots,
//! collections, STIX objects, and jobs.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::{DatabaseError, DatabaseResult};
use crate::models::taxii2::{PaginatedResult, PaginationCursor, Taxii2QueryParams};
use crate::pool::TaxiiPool;
use crate::repository::traits::Taxii2Repository;

use taxii_core::{
    ApiRoot, Collection, Job, JobDetail, JobDetails, ManifestRecord, STIXObject, VersionRecord,
};

// ============================================================================
// Utilities
// ============================================================================

/// TAXII 2.1 datetime format for parsing timestamps.
/// Uses %.6f for 6-digit microsecond precision.
const TAXII2_DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.6fZ";

/// Get the object version from a STIX object.
///
/// The TAXII version is inferred in this order:
/// 1. `modified` field, if present
/// 2. `created` field, if present
/// 3. 1970-01-01T00:00:00+00:00 (Unix epoch) otherwise
///
/// This is guided by TAXII 2.1 spec section 3.4.1 "Supported Fields for Match":
/// > If a STIX object is not versioned (and therefore does not have a modified
/// > timestamp) then this version parameter MUST use the created timestamp. If
/// > an object does not have a created or modified timestamp or any other
/// > version information that can be used, then the server should use a value for
/// > the version that is consistent to the server.
pub fn get_object_version(obj: &serde_json::Value) -> DateTime<Utc> {
    // Try modified first
    if let Some(modified) = obj.get("modified").and_then(|v| v.as_str()) {
        if let Ok(dt) = DateTime::parse_from_rfc3339(modified) {
            return dt.with_timezone(&Utc);
        }
        // Try alternative format without timezone
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(modified, TAXII2_DATETIME_FORMAT) {
            return dt.and_utc();
        }
    }

    // Try created as fallback
    if let Some(created) = obj.get("created").and_then(|v| v.as_str()) {
        if let Ok(dt) = DateTime::parse_from_rfc3339(created) {
            return dt.with_timezone(&Utc);
        }
        // Try alternative format without timezone
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(created, TAXII2_DATETIME_FORMAT) {
            return dt.and_utc();
        }
    }

    // Return Unix epoch as final fallback
    DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now)
}

// ============================================================================
// Repository Implementation
// ============================================================================

/// PostgreSQL implementation of [`Taxii2Repository`].
///
/// Wraps a database connection pool and provides all TAXII 2.x
/// database operations.
pub struct DbTaxii2Repository {
    pool: TaxiiPool,
}

impl DbTaxii2Repository {
    /// Create a new repository instance.
    pub fn new(pool: TaxiiPool) -> Self {
        Self { pool }
    }

    /// Get pool reference.
    pub fn pool(&self) -> &TaxiiPool {
        &self.pool
    }
}

impl Taxii2Repository for DbTaxii2Repository {
    // ========================================================================
    // API Root Operations
    // ========================================================================

    async fn get_api_roots(&self) -> DatabaseResult<Vec<ApiRoot>> {
        let api_roots = crate::models::taxii2::ApiRoot::find_all(&self.pool).await?;
        Ok(api_roots.into_iter().map(Into::into).collect())
    }

    async fn get_api_root(&self, api_root_id: &str) -> DatabaseResult<Option<ApiRoot>> {
        let uuid = Uuid::parse_str(api_root_id)
            .map_err(|_| DatabaseError::NotFound(format!("Invalid UUID: {api_root_id}")))?;

        let api_root = crate::models::taxii2::ApiRoot::find(&self.pool, uuid).await?;
        Ok(api_root.map(Into::into))
    }

    async fn add_api_root(
        &self,
        title: &str,
        description: Option<&str>,
        default: bool,
        is_public: bool,
        api_root_id: Option<&str>,
    ) -> DatabaseResult<ApiRoot> {
        let id = match api_root_id {
            Some(id_str) => Uuid::parse_str(id_str)
                .map_err(|e| DatabaseError::InvalidData(format!("Invalid UUID '{id_str}': {e}")))?,
            None => Uuid::new_v4(),
        };

        let r = crate::models::taxii2::ApiRoot::create(
            &self.pool,
            id,
            title,
            description,
            default,
            is_public,
        )
        .await?;

        Ok(r.into())
    }

    // ========================================================================
    // Collection Operations (TAXII 2.x)
    // ========================================================================

    async fn get_collections(&self, api_root_id: &str) -> DatabaseResult<Vec<Collection>> {
        let uuid = Uuid::parse_str(api_root_id)
            .map_err(|_| DatabaseError::NotFound(format!("Invalid UUID: {api_root_id}")))?;

        let collections =
            crate::models::taxii2::Collection::find_by_api_root(&self.pool, uuid).await?;

        Ok(collections.into_iter().map(Into::into).collect())
    }

    async fn get_collection(
        &self,
        api_root_id: &str,
        collection_id_or_alias: &str,
    ) -> DatabaseResult<Option<Collection>> {
        let api_root_uuid = Uuid::parse_str(api_root_id).map_err(|_| {
            DatabaseError::NotFound(format!("Invalid API root UUID: {api_root_id}"))
        })?;

        let collection = crate::models::taxii2::Collection::find_by_id_or_alias(
            &self.pool,
            api_root_uuid,
            collection_id_or_alias,
        )
        .await?;

        Ok(collection.map(Into::into))
    }

    async fn add_collection(
        &self,
        api_root_id: &str,
        title: &str,
        description: Option<&str>,
        alias: Option<&str>,
        is_public: bool,
        is_public_write: bool,
    ) -> DatabaseResult<Collection> {
        let api_root_uuid = Uuid::parse_str(api_root_id).map_err(|_| {
            DatabaseError::NotFound(format!("Invalid API root UUID: {api_root_id}"))
        })?;

        let c = crate::models::taxii2::Collection::create(
            &self.pool,
            api_root_uuid,
            title,
            description,
            alias,
            is_public,
            is_public_write,
        )
        .await?;

        Ok(c.into())
    }

    // ========================================================================
    // STIX Object Operations
    // ========================================================================

    async fn get_manifest(
        &self,
        collection_id: &str,
        params: &Taxii2QueryParams<'_>,
    ) -> DatabaseResult<PaginatedResult<Vec<ManifestRecord>>> {
        let collection_uuid = Uuid::parse_str(collection_id).map_err(|_| {
            DatabaseError::NotFound(format!("Invalid collection UUID: {collection_id}"))
        })?;

        let result =
            crate::models::taxii2::STIXObject::find_filtered(&self.pool, collection_uuid, params)
                .await?;

        let records = result.objects.into_iter().map(Into::into).collect();

        Ok(PaginatedResult::new(records, result.more, result.next))
    }

    async fn get_objects(
        &self,
        collection_id: &str,
        params: &Taxii2QueryParams<'_>,
    ) -> DatabaseResult<PaginatedResult<Vec<STIXObject>>> {
        let collection_uuid = Uuid::parse_str(collection_id).map_err(|_| {
            DatabaseError::NotFound(format!("Invalid collection UUID: {collection_id}"))
        })?;

        let result =
            crate::models::taxii2::STIXObject::find_filtered(&self.pool, collection_uuid, params)
                .await?;

        let objects = result.objects.into_iter().map(Into::into).collect();

        Ok(PaginatedResult::new(objects, result.more, result.next))
    }

    async fn add_objects(
        &self,
        api_root_id: &str,
        collection_id: &str,
        objects: &[serde_json::Value],
    ) -> DatabaseResult<Job> {
        let api_root_uuid = Uuid::parse_str(api_root_id).map_err(|_| {
            DatabaseError::NotFound(format!("Invalid API root UUID: {api_root_id}"))
        })?;
        let collection_uuid = Uuid::parse_str(collection_id).map_err(|_| {
            DatabaseError::NotFound(format!("Invalid collection UUID: {collection_id}"))
        })?;

        // Create job using model
        let job = crate::models::taxii2::Job::create(
            &self.pool,
            &crate::models::taxii2::NewJob {
                api_root_id: api_root_uuid,
            },
        )
        .await?;
        let job_id = job.id;
        let now = job
            .request_timestamp
            .unwrap_or_else(|| Utc::now().naive_utc());

        let mut job_details = Vec::new();
        let mut total_count = 0;
        let mut success_count = 0;

        for obj in objects {
            let stix_id = obj["id"].as_str().unwrap_or_default();
            let spec_version = obj["spec_version"].as_str().unwrap_or("2.1");

            // Parse version using TAXII 2.1 fallback logic (modified -> created -> epoch)
            let version = get_object_version(obj);
            let version_naive = version.naive_utc();

            // Check if object already exists using model
            let exists = crate::models::taxii2::STIXObject::exists(
                &self.pool,
                stix_id,
                collection_uuid,
                version_naive,
            )
            .await?;

            if !exists {
                let stix_type = stix_id.split("--").next().unwrap_or_default();
                let serialized_data: serde_json::Value = obj
                    .as_object()
                    .map(|o| {
                        let filtered: serde_json::Map<String, serde_json::Value> = o
                            .iter()
                            .filter(|(k, _)| !["id", "type", "spec_version"].contains(&k.as_str()))
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();
                        serde_json::Value::Object(filtered)
                    })
                    .unwrap_or_default();

                // Create STIX object using model
                let new_obj = crate::models::taxii2::NewSTIXObject {
                    id: stix_id,
                    collection_id: collection_uuid,
                    stix_type,
                    spec_version,
                    version: version_naive,
                    serialized_data: &serialized_data,
                };
                crate::models::taxii2::STIXObject::create(&self.pool, &new_obj).await?;
            }

            // Create job detail using model
            let detail = crate::models::taxii2::JobDetail::create(
                &self.pool,
                job_id,
                stix_id,
                version_naive,
                crate::models::taxii2::job_detail_status::SUCCESS,
                None,
            )
            .await?;

            job_details.push(JobDetail {
                id: detail.id.to_string(),
                job_id: detail.job_id.to_string(),
                stix_id: detail.stix_id,
                version,
                message: String::new(),
                status: "success".to_string(),
            });

            total_count += 1;
            success_count += 1;
        }

        // Complete job using model
        crate::models::taxii2::Job::complete(&self.pool, job_id, total_count, success_count, 0)
            .await?;

        // Build job entity
        let mut details = JobDetails::default();
        details.success.extend(job_details);

        Ok(Job {
            id: job_id.to_string(),
            api_root_id: api_root_id.to_string(),
            status: "complete".to_string(),
            request_timestamp: now.and_utc(),
            completed_timestamp: Some(Utc::now()),
            total_count,
            success_count,
            failure_count: 0,
            pending_count: 0,
            details,
        })
    }

    async fn get_job_and_details(
        &self,
        api_root_id: &str,
        job_id: &str,
    ) -> DatabaseResult<Option<Job>> {
        let api_root_uuid = Uuid::parse_str(api_root_id).map_err(|_| {
            DatabaseError::NotFound(format!("Invalid API root UUID: {api_root_id}"))
        })?;
        let job_uuid = Uuid::parse_str(job_id)
            .map_err(|_| DatabaseError::NotFound(format!("Invalid job UUID: {job_id}")))?;

        let job = crate::models::taxii2::Job::find_by_api_root(&self.pool, api_root_uuid, job_uuid)
            .await?;

        let job = match job {
            Some(j) => j,
            None => return Ok(None),
        };

        let details = crate::models::taxii2::JobDetail::find_by_job(&self.pool, job_uuid).await?;

        let mut job_details = JobDetails::default();
        for detail in details {
            let jd = JobDetail {
                id: detail.id.to_string(),
                job_id: detail.job_id.to_string(),
                stix_id: detail.stix_id,
                version: detail.version.and_utc(),
                message: detail.message.unwrap_or_default(),
                status: detail.status.clone(),
            };

            match detail.status.as_str() {
                "success" => job_details.success.push(jd),
                "failure" => job_details.failure.push(jd),
                "pending" => job_details.pending.push(jd),
                _ => {}
            }
        }

        Ok(Some(Job {
            id: job.id.to_string(),
            api_root_id: job.api_root_id.to_string(),
            status: job.status,
            request_timestamp: job
                .request_timestamp
                .map(|t| t.and_utc())
                .unwrap_or_else(Utc::now),
            completed_timestamp: job.completed_timestamp.map(|t| t.and_utc()),
            total_count: job.total_count.unwrap_or(0),
            success_count: job.success_count.unwrap_or(0),
            failure_count: job.failure_count.unwrap_or(0),
            pending_count: job.pending_count.unwrap_or(0),
            details: job_details,
        }))
    }

    async fn get_object(
        &self,
        collection_id: &str,
        object_id: &str,
        params: &Taxii2QueryParams<'_>,
    ) -> DatabaseResult<PaginatedResult<Vec<STIXObject>>> {
        let collection_uuid = Uuid::parse_str(collection_id).map_err(|_| {
            DatabaseError::NotFound(format!("Invalid collection UUID: {collection_id}"))
        })?;

        // Check if object exists in collection
        let exists = crate::models::taxii2::STIXObject::exists_any_version(
            &self.pool,
            object_id,
            collection_uuid,
        )
        .await?;

        if !exists {
            return Ok(PaginatedResult::empty());
        }

        // Use find_filtered with match_id filter
        let match_id = vec![object_id.to_string()];
        let params_with_id = Taxii2QueryParams {
            match_id: Some(&match_id),
            ..*params
        };

        let result = crate::models::taxii2::STIXObject::find_filtered(
            &self.pool,
            collection_uuid,
            &params_with_id,
        )
        .await?;

        let objects: Vec<STIXObject> = result.objects.into_iter().map(Into::into).collect();

        Ok(PaginatedResult::new(objects, result.more, result.next))
    }

    async fn delete_object(
        &self,
        collection_id: &str,
        object_id: &str,
        match_version: Option<&[String]>,
        match_spec_version: Option<&[String]>,
    ) -> DatabaseResult<()> {
        let collection_uuid = Uuid::parse_str(collection_id).map_err(|_| {
            DatabaseError::NotFound(format!("Invalid collection UUID: {collection_id}"))
        })?;

        crate::models::taxii2::STIXObject::delete_filtered(
            &self.pool,
            collection_uuid,
            object_id,
            match_version,
            match_spec_version,
        )
        .await?;

        Ok(())
    }

    async fn get_versions(
        &self,
        collection_id: &str,
        object_id: &str,
        limit: Option<i64>,
        added_after: Option<DateTime<Utc>>,
        next_kwargs: Option<PaginationCursor>,
        match_spec_version: Option<&[String]>,
    ) -> DatabaseResult<PaginatedResult<Vec<VersionRecord>>> {
        let collection_uuid = Uuid::parse_str(collection_id).map_err(|_| {
            DatabaseError::NotFound(format!("Invalid collection UUID: {collection_id}"))
        })?;

        let result = crate::models::taxii2::STIXObject::find_versions(
            &self.pool,
            collection_uuid,
            object_id,
            limit,
            added_after,
            next_kwargs.as_ref(),
            match_spec_version,
        )
        .await?;

        let records = result
            .versions
            .map(|versions| versions.into_iter().map(Into::into).collect())
            .unwrap_or_default();

        Ok(PaginatedResult::new(records, result.more, result.next))
    }

    async fn job_cleanup(&self) -> DatabaseResult<i32> {
        let count = crate::models::taxii2::Job::cleanup_old(&self.pool).await?;
        Ok(count as i32)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use serde_json::json;

    /// Test get_object_version with modified field present.
    #[test]
    fn test_get_object_version_with_modified() {
        let obj = json!({
            "id": "indicator--12345678-1234-1234-1234-123456789012",
            "type": "indicator",
            "spec_version": "2.1",
            "created": "2020-01-01T00:00:00.000Z",
            "modified": "2021-06-15T10:30:00.000Z"
        });

        let version = get_object_version(&obj);
        assert_eq!(version.year(), 2021);
        assert_eq!(version.month(), 6);
        assert_eq!(version.day(), 15);
    }

    /// Test get_object_version fallback to created when modified is absent.
    #[test]
    fn test_get_object_version_fallback_to_created() {
        let obj = json!({
            "id": "marking-definition--12345678-1234-1234-1234-123456789012",
            "type": "marking-definition",
            "spec_version": "2.1",
            "created": "2020-03-20T12:00:00.000Z"
        });

        let version = get_object_version(&obj);
        assert_eq!(version.year(), 2020);
        assert_eq!(version.month(), 3);
        assert_eq!(version.day(), 20);
    }

    /// Test get_object_version fallback to Unix epoch when both fields are absent.
    #[test]
    fn test_get_object_version_fallback_to_epoch() {
        let obj = json!({
            "id": "x-custom--12345678-1234-1234-1234-123456789012",
            "type": "x-custom"
        });

        let version = get_object_version(&obj);
        assert_eq!(version.year(), 1970);
        assert_eq!(version.month(), 1);
        assert_eq!(version.day(), 1);
        assert_eq!(version.timestamp(), 0);
    }

    /// Test get_object_version with RFC3339 format (with timezone).
    #[test]
    fn test_get_object_version_rfc3339_format() {
        let obj = json!({
            "id": "indicator--12345678-1234-1234-1234-123456789012",
            "type": "indicator",
            "modified": "2023-10-05T14:30:00+00:00"
        });

        let version = get_object_version(&obj);
        assert_eq!(version.year(), 2023);
        assert_eq!(version.month(), 10);
        assert_eq!(version.day(), 5);
    }

    /// Test get_object_version with TAXII datetime format.
    #[test]
    fn test_get_object_version_taxii_format() {
        let obj = json!({
            "id": "indicator--12345678-1234-1234-1234-123456789012",
            "type": "indicator",
            "created": "2022-08-15T09:45:30.123456Z"
        });

        let version = get_object_version(&obj);
        assert_eq!(version.year(), 2022);
        assert_eq!(version.month(), 8);
        assert_eq!(version.day(), 15);
    }
}
