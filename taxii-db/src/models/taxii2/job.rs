//! Job and JobDetail models (TAXII 2.x async jobs).

use chrono::{NaiveDateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// Job database row.
///
/// Table: opentaxii_job
#[derive(Debug, Clone, FromRow)]
pub struct Job {
    /// Primary key (UUID).
    pub id: Uuid,

    /// Foreign key to API root.
    pub api_root_id: Uuid,

    /// Job status: "pending" or "complete".
    pub status: String,

    /// When the request was received.
    /// Stored as timestamp without timezone in PostgreSQL.
    pub request_timestamp: Option<NaiveDateTime>,

    /// When the job completed.
    /// Stored as timestamp without timezone in PostgreSQL.
    pub completed_timestamp: Option<NaiveDateTime>,

    /// Total number of objects in the request.
    pub total_count: Option<i32>,

    /// Number of successfully processed objects.
    pub success_count: Option<i32>,

    /// Number of failed objects.
    pub failure_count: Option<i32>,

    /// Number of pending objects.
    pub pending_count: Option<i32>,
}

/// Parameters for creating a new job.
#[derive(Debug, Clone)]
pub struct NewJob {
    pub api_root_id: Uuid,
}

impl Job {
    /// Find a job by ID.
    pub async fn find(pool: &TaxiiPool, id: Uuid) -> DatabaseResult<Option<Self>> {
        let job = sqlx::query_as!(
            Self,
            r#"SELECT id, api_root_id as "api_root_id!", status::text as "status!",
                      request_timestamp, completed_timestamp,
                      total_count, success_count, failure_count, pending_count
               FROM opentaxii_job WHERE id = $1"#,
            id
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(job)
    }

    /// Find a job by API root and job ID.
    pub async fn find_by_api_root(
        pool: &TaxiiPool,
        api_root_id: Uuid,
        job_id: Uuid,
    ) -> DatabaseResult<Option<Self>> {
        let job = sqlx::query_as!(
            Self,
            r#"SELECT id, api_root_id as "api_root_id!", status::text as "status!",
                      request_timestamp, completed_timestamp,
                      total_count, success_count, failure_count, pending_count
               FROM opentaxii_job WHERE api_root_id = $1 AND id = $2"#,
            api_root_id,
            job_id
        )
        .fetch_optional(pool.inner())
        .await?;

        Ok(job)
    }

    /// Create a new pending job.
    pub async fn create(pool: &TaxiiPool, params: &NewJob) -> DatabaseResult<Self> {
        let id = Uuid::new_v4();
        let now = Utc::now().naive_utc();

        let job = sqlx::query_as!(
            Self,
            r#"INSERT INTO opentaxii_job (id, api_root_id, status, request_timestamp, total_count, success_count, failure_count, pending_count)
               VALUES ($1, $2, 'pending', $3, 0, 0, 0, 0)
               RETURNING id, api_root_id as "api_root_id!", status::text as "status!", request_timestamp, completed_timestamp,
                         total_count, success_count, failure_count, pending_count"#,
            id,
            params.api_root_id,
            now
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(job)
    }

    /// Complete a job with counts.
    pub async fn complete(
        pool: &TaxiiPool,
        id: Uuid,
        total_count: i32,
        success_count: i32,
        failure_count: i32,
    ) -> DatabaseResult<()> {
        sqlx::query!(
            r#"UPDATE opentaxii_job
               SET status = 'complete', completed_timestamp = $2, total_count = $3,
                   success_count = $4, failure_count = $5, pending_count = 0
               WHERE id = $1"#,
            id,
            Utc::now().naive_utc(),
            total_count,
            success_count,
            failure_count
        )
        .execute(pool.inner())
        .await?;

        Ok(())
    }

    /// Cleanup old completed jobs (older than 24 hours).
    pub async fn cleanup_old(pool: &TaxiiPool) -> DatabaseResult<i64> {
        let cutoff = (Utc::now() - chrono::Duration::hours(24)).naive_utc();

        let result = sqlx::query!(
            "DELETE FROM opentaxii_job WHERE completed_timestamp < $1",
            cutoff
        )
        .execute(pool.inner())
        .await?;

        Ok(result.rows_affected() as i64)
    }
}

/// JobDetail database row.
///
/// Table: opentaxii_job_detail
#[derive(Debug, Clone, FromRow)]
pub struct JobDetail {
    /// Primary key (UUID).
    pub id: Uuid,

    /// Foreign key to job.
    pub job_id: Uuid,

    /// STIX object ID.
    pub stix_id: String,

    /// Object version.
    /// Stored as timestamp without timezone in PostgreSQL.
    pub version: NaiveDateTime,

    /// Status message.
    pub message: Option<String>,

    /// Detail status: "success", "failure", or "pending".
    pub status: String,
}

impl JobDetail {
    /// Find all details for a job.
    pub async fn find_by_job(pool: &TaxiiPool, job_id: Uuid) -> DatabaseResult<Vec<Self>> {
        let details = sqlx::query_as!(
            Self,
            r#"SELECT id, job_id as "job_id!", stix_id as "stix_id!", version as "version!",
                      message, status::text as "status!"
               FROM opentaxii_job_detail WHERE job_id = $1"#,
            job_id
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(details)
    }

    /// Create a new job detail.
    pub async fn create(
        pool: &TaxiiPool,
        job_id: Uuid,
        stix_id: &str,
        version: NaiveDateTime,
        status: &str,
        message: Option<&str>,
    ) -> DatabaseResult<Self> {
        let id = Uuid::new_v4();

        // Use raw query to handle enum type casting
        sqlx::query(
            r#"INSERT INTO opentaxii_job_detail (id, job_id, stix_id, version, status, message)
               VALUES ($1, $2, $3, $4, $5::job_detail_status_enum, $6)"#,
        )
        .bind(id)
        .bind(job_id)
        .bind(stix_id)
        .bind(version)
        .bind(status)
        .bind(message)
        .execute(pool.inner())
        .await?;

        // Fetch the created record
        let detail = sqlx::query_as!(
            Self,
            r#"SELECT id, job_id as "job_id!", stix_id as "stix_id!", version as "version!",
                      message, status::text as "status!"
               FROM opentaxii_job_detail WHERE id = $1"#,
            id
        )
        .fetch_one(pool.inner())
        .await?;

        Ok(detail)
    }
}

/// Job status constants.
pub mod job_status {
    pub const PENDING: &str = "pending";
    pub const COMPLETE: &str = "complete";
}

/// Job detail status constants.
pub mod job_detail_status {
    pub const SUCCESS: &str = "success";
    pub const FAILURE: &str = "failure";
    pub const PENDING: &str = "pending";
}
