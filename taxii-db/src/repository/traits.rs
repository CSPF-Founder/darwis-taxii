//! Repository trait definitions for TAXII 1.x and 2.x.
//!
//! These traits define the interface for database operations, enabling:
//! - Mockability for unit testing
//! - Potential alternative implementations (e.g., in-memory, different databases)
//! - Clear API contracts

use chrono::{DateTime, Utc};

use crate::error::DatabaseResult;
use crate::models::taxii2::{PaginatedResult, PaginationCursor, Taxii2QueryParams};

use taxii_core::{
    ApiRoot, Collection, CollectionEntity, ContentBindingEntity, ContentBlockEntity,
    InboxMessageEntity, Job, ManifestRecord, ResultSetEntity, STIXObject, ServiceEntity,
    SubscriptionEntity, VersionRecord,
};

// ============================================================================
// TAXII 1.x Repository Trait
// ============================================================================

/// Repository trait for TAXII 1.x database operations.
///
/// Provides async methods for managing TAXII 1.x entities including services,
/// collections, content blocks, inbox messages, result sets, and subscriptions.
pub trait Taxii1Repository: Send + Sync {
    // ========================================================================
    // Service Operations
    // ========================================================================

    /// Get all services, optionally filtered by collection ID.
    fn get_services(
        &self,
        collection_id: Option<i32>,
    ) -> impl Future<Output = DatabaseResult<Vec<ServiceEntity>>> + Send;

    /// Get a service by ID.
    fn get_service(
        &self,
        service_id: &str,
    ) -> impl Future<Output = DatabaseResult<Option<ServiceEntity>>> + Send;

    /// Update or create a service (upsert).
    fn update_service(
        &self,
        entity: &ServiceEntity,
    ) -> impl Future<Output = DatabaseResult<ServiceEntity>> + Send;

    /// Create a service (alias for update_service).
    fn create_service(
        &self,
        entity: &ServiceEntity,
    ) -> impl Future<Output = DatabaseResult<ServiceEntity>> + Send;

    /// Delete a service by ID.
    fn delete_service(&self, service_id: &str) -> impl Future<Output = DatabaseResult<()>> + Send;

    /// Get configured domain for a service.
    fn get_domain(
        &self,
        service_id: &str,
    ) -> impl Future<Output = DatabaseResult<Option<String>>> + Send;

    /// Get advertised services for a discovery service.
    fn get_advertised_services(
        &self,
        discovery_service_id: &str,
    ) -> impl Future<Output = DatabaseResult<Vec<ServiceEntity>>> + Send;

    /// Get services of a specific type for a collection.
    fn get_services_for_collection(
        &self,
        collection_id: i32,
        service_type: Option<&str>,
    ) -> impl Future<Output = DatabaseResult<Vec<ServiceEntity>>> + Send;

    // ========================================================================
    // Collection Operations
    // ========================================================================

    /// Get all collections, optionally filtered by service ID.
    fn get_collections(
        &self,
        service_id: Option<&str>,
    ) -> impl Future<Output = DatabaseResult<Vec<CollectionEntity>>> + Send;

    /// Get a collection by name, optionally filtered by service ID.
    fn get_collection(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> impl Future<Output = DatabaseResult<Option<CollectionEntity>>> + Send;

    /// Create a new collection.
    fn create_collection(
        &self,
        entity: &CollectionEntity,
    ) -> impl Future<Output = DatabaseResult<CollectionEntity>> + Send;

    /// Update an existing collection.
    fn update_collection(
        &self,
        entity: &CollectionEntity,
    ) -> impl Future<Output = DatabaseResult<CollectionEntity>> + Send;

    /// Delete a collection by name.
    fn delete_collection(
        &self,
        collection_name: &str,
    ) -> impl Future<Output = DatabaseResult<()>> + Send;

    /// Set services for a collection.
    fn set_collection_services(
        &self,
        collection_id: i32,
        service_ids: &[String],
    ) -> impl Future<Output = DatabaseResult<()>> + Send;

    // ========================================================================
    // Content Block Operations
    // ========================================================================

    /// Get content blocks with filtering.
    fn get_content_blocks(
        &self,
        collection_id: Option<i32>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        bindings: Option<&[ContentBindingEntity]>,
        offset: i64,
        limit: Option<i64>,
    ) -> impl Future<Output = DatabaseResult<Vec<ContentBlockEntity>>> + Send;

    /// Get count of content blocks matching criteria.
    fn get_content_blocks_count(
        &self,
        collection_id: Option<i32>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        bindings: Option<&[ContentBindingEntity]>,
    ) -> impl Future<Output = DatabaseResult<i64>> + Send;

    /// Create a content block.
    fn create_content_block(
        &self,
        entity: &ContentBlockEntity,
        collection_ids: Option<&[i32]>,
        service_id: Option<&str>,
    ) -> impl Future<Output = DatabaseResult<ContentBlockEntity>> + Send;

    /// Delete content blocks within a timeframe.
    fn delete_content_blocks(
        &self,
        collection_name: &str,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
        with_messages: bool,
    ) -> impl Future<Output = DatabaseResult<i64>> + Send;

    // ========================================================================
    // Inbox Message Operations
    // ========================================================================

    /// Create an inbox message.
    fn create_inbox_message(
        &self,
        entity: &InboxMessageEntity,
    ) -> impl Future<Output = DatabaseResult<InboxMessageEntity>> + Send;

    // ========================================================================
    // Result Set Operations
    // ========================================================================

    /// Create a result set.
    fn create_result_set(
        &self,
        entity: &ResultSetEntity,
    ) -> impl Future<Output = DatabaseResult<ResultSetEntity>> + Send;

    /// Get a result set by ID.
    fn get_result_set(
        &self,
        result_set_id: &str,
    ) -> impl Future<Output = DatabaseResult<Option<ResultSetEntity>>> + Send;

    // ========================================================================
    // Subscription Operations
    // ========================================================================

    /// Get a subscription by ID.
    fn get_subscription(
        &self,
        subscription_id: &str,
    ) -> impl Future<Output = DatabaseResult<Option<SubscriptionEntity>>> + Send;

    /// Get subscriptions for a service.
    fn get_subscriptions(
        &self,
        service_id: &str,
    ) -> impl Future<Output = DatabaseResult<Vec<SubscriptionEntity>>> + Send;

    /// Update or create a subscription.
    fn update_subscription(
        &self,
        entity: &SubscriptionEntity,
    ) -> impl Future<Output = DatabaseResult<SubscriptionEntity>> + Send;

    /// Create a subscription (alias for update_subscription).
    fn create_subscription(
        &self,
        entity: &SubscriptionEntity,
    ) -> impl Future<Output = DatabaseResult<SubscriptionEntity>> + Send;
}

// ============================================================================
// TAXII 2.x Repository Trait
// ============================================================================

/// Repository trait for TAXII 2.x database operations.
///
/// Provides async methods for managing TAXII 2.x entities including API roots,
/// collections, STIX objects, and jobs.
pub trait Taxii2Repository: Send + Sync {
    // ========================================================================
    // API Root Operations
    // ========================================================================

    /// Get all API roots.
    fn get_api_roots(&self) -> impl Future<Output = DatabaseResult<Vec<ApiRoot>>> + Send;

    /// Get an API root by ID.
    fn get_api_root(
        &self,
        api_root_id: &str,
    ) -> impl Future<Output = DatabaseResult<Option<ApiRoot>>> + Send;

    /// Add a new API root.
    fn add_api_root(
        &self,
        title: &str,
        description: Option<&str>,
        default: bool,
        is_public: bool,
        api_root_id: Option<&str>,
    ) -> impl Future<Output = DatabaseResult<ApiRoot>> + Send;

    // ========================================================================
    // Collection Operations (TAXII 2.x)
    // ========================================================================

    /// Get collections for an API root.
    fn get_collections(
        &self,
        api_root_id: &str,
    ) -> impl Future<Output = DatabaseResult<Vec<Collection>>> + Send;

    /// Get a collection by ID or alias.
    fn get_collection(
        &self,
        api_root_id: &str,
        collection_id_or_alias: &str,
    ) -> impl Future<Output = DatabaseResult<Option<Collection>>> + Send;

    /// Add a new collection.
    fn add_collection(
        &self,
        api_root_id: &str,
        title: &str,
        description: Option<&str>,
        alias: Option<&str>,
        is_public: bool,
        is_public_write: bool,
    ) -> impl Future<Output = DatabaseResult<Collection>> + Send;

    // ========================================================================
    // STIX Object Operations
    // ========================================================================

    /// Get manifest records.
    fn get_manifest(
        &self,
        collection_id: &str,
        params: &Taxii2QueryParams<'_>,
    ) -> impl Future<Output = DatabaseResult<PaginatedResult<Vec<ManifestRecord>>>> + Send;

    /// Get STIX objects.
    fn get_objects(
        &self,
        collection_id: &str,
        params: &Taxii2QueryParams<'_>,
    ) -> impl Future<Output = DatabaseResult<PaginatedResult<Vec<STIXObject>>>> + Send;

    /// Add STIX objects.
    fn add_objects(
        &self,
        api_root_id: &str,
        collection_id: &str,
        objects: &[serde_json::Value],
    ) -> impl Future<Output = DatabaseResult<Job>> + Send;

    /// Get a single object (returns empty items if object doesn't exist).
    fn get_object(
        &self,
        collection_id: &str,
        object_id: &str,
        params: &Taxii2QueryParams<'_>,
    ) -> impl Future<Output = DatabaseResult<PaginatedResult<Vec<STIXObject>>>> + Send;

    /// Delete an object.
    fn delete_object(
        &self,
        collection_id: &str,
        object_id: &str,
        match_version: Option<&[String]>,
        match_spec_version: Option<&[String]>,
    ) -> impl Future<Output = DatabaseResult<()>> + Send;

    /// Get versions of an object.
    ///
    /// Returns empty items if the object doesn't exist in the collection.
    fn get_versions(
        &self,
        collection_id: &str,
        object_id: &str,
        limit: Option<i64>,
        added_after: Option<DateTime<Utc>>,
        next_kwargs: Option<PaginationCursor>,
        match_spec_version: Option<&[String]>,
    ) -> impl Future<Output = DatabaseResult<PaginatedResult<Vec<VersionRecord>>>> + Send;

    // ========================================================================
    // Job Operations
    // ========================================================================

    /// Get job and details.
    fn get_job_and_details(
        &self,
        api_root_id: &str,
        job_id: &str,
    ) -> impl Future<Output = DatabaseResult<Option<Job>>> + Send;

    /// Cleanup old jobs.
    fn job_cleanup(&self) -> impl Future<Output = DatabaseResult<i32>> + Send;
}

use std::future::Future;
