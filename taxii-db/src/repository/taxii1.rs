//! TAXII 1.x Repository implementation.
//!
//! Provides database operations for TAXII 1.x entities including services,
//! collections, content blocks, inbox messages, result sets, and subscriptions.

use chrono::{DateTime, Utc};
use serde_json::json;
use tracing::debug;
use uuid::Uuid;

use crate::error::{DatabaseError, DatabaseResult};
use crate::models::taxii1::{
    ContentBlock, DataCollection, InboxMessage, ResultSet, Service, Subscription,
};
use crate::pool::TaxiiPool;
use crate::repository::traits::Taxii1Repository;

use taxii_core::{
    CollectionEntity, ContentBindingEntity, ContentBlockEntity, InboxMessageEntity,
    ResultSetEntity, ServiceEntity, SubscriptionEntity,
};

/// PostgreSQL implementation of [`Taxii1Repository`].
///
/// Wraps a database connection pool and provides all TAXII 1.x
/// database operations.
pub struct DbTaxii1Repository {
    pool: TaxiiPool,
}

impl DbTaxii1Repository {
    /// Create a new repository instance.
    pub fn new(pool: TaxiiPool) -> Self {
        Self { pool }
    }

    /// Get pool reference.
    pub fn pool(&self) -> &TaxiiPool {
        &self.pool
    }
}

impl Taxii1Repository for DbTaxii1Repository {
    // ========================================================================
    // Service Operations
    // ========================================================================

    async fn get_services(&self, collection_id: Option<i32>) -> DatabaseResult<Vec<ServiceEntity>> {
        let services = if let Some(coll_id) = collection_id {
            Service::find_by_collection(&self.pool, coll_id).await?
        } else {
            Service::find_all(&self.pool).await?
        };

        Ok(services.into_iter().map(Into::into).collect())
    }

    async fn get_service(&self, service_id: &str) -> DatabaseResult<Option<ServiceEntity>> {
        let service = Service::find(&self.pool, service_id).await?;
        Ok(service.map(Into::into))
    }

    async fn update_service(&self, entity: &ServiceEntity) -> DatabaseResult<ServiceEntity> {
        let service_id = entity
            .id
            .as_ref()
            .ok_or_else(|| DatabaseError::NotFound("Service ID required".to_string()))?;

        let properties_json = serde_json::to_string(&entity.properties).map_err(|e| {
            DatabaseError::InvalidData(format!("Failed to serialize properties: {e}"))
        })?;

        let service = Service::upsert(
            &self.pool,
            service_id,
            &entity.service_type,
            &properties_json,
        )
        .await?;

        Ok(service.into())
    }

    async fn create_service(&self, entity: &ServiceEntity) -> DatabaseResult<ServiceEntity> {
        self.update_service(entity).await
    }

    async fn delete_service(&self, service_id: &str) -> DatabaseResult<()> {
        Service::delete(&self.pool, service_id).await?;
        Ok(())
    }

    async fn get_domain(&self, service_id: &str) -> DatabaseResult<Option<String>> {
        let service = self.get_service(service_id).await?;

        let domain = service
            .and_then(|svc| svc.properties.get("domain").cloned())
            .and_then(|v| v.as_str().map(String::from))
            .filter(|s| !s.is_empty());

        Ok(domain)
    }

    async fn get_advertised_services(
        &self,
        discovery_service_id: &str,
    ) -> DatabaseResult<Vec<ServiceEntity>> {
        // Get the discovery service to check its advertised_services property
        let discovery_service = self.get_service(discovery_service_id).await?;

        let Some(service) = discovery_service else {
            return Ok(Vec::new());
        };

        // Get advertised_services from properties (array of service IDs)
        let advertised_ids: Vec<String> = service
            .properties
            .get("advertised_services")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        if advertised_ids.is_empty() {
            // If no advertised services configured, return all services
            return self.get_services(None).await;
        }

        // Fetch each advertised service
        let mut services = Vec::new();
        for service_id in advertised_ids {
            if let Ok(Some(svc)) = self.get_service(&service_id).await {
                services.push(svc);
            }
        }

        Ok(services)
    }

    async fn get_services_for_collection(
        &self,
        collection_id: i32,
        service_type: Option<&str>,
    ) -> DatabaseResult<Vec<ServiceEntity>> {
        let services = if let Some(svc_type) = service_type {
            Service::find_by_collection_and_type(&self.pool, collection_id, svc_type).await?
        } else {
            Service::find_by_collection(&self.pool, collection_id).await?
        };

        Ok(services.into_iter().map(Into::into).collect())
    }

    // ========================================================================
    // Collection Operations
    // ========================================================================

    async fn get_collections(
        &self,
        service_id: Option<&str>,
    ) -> DatabaseResult<Vec<CollectionEntity>> {
        let collections = if let Some(svc_id) = service_id {
            DataCollection::find_by_service(&self.pool, svc_id).await?
        } else {
            DataCollection::find_all(&self.pool).await?
        };

        Ok(collections.into_iter().map(Into::into).collect())
    }

    async fn get_collection(
        &self,
        name: &str,
        service_id: Option<&str>,
    ) -> DatabaseResult<Option<CollectionEntity>> {
        let collection = if let Some(svc_id) = service_id {
            DataCollection::find_by_name_and_service(&self.pool, name, svc_id).await?
        } else {
            DataCollection::find_by_name(&self.pool, name).await?
        };

        Ok(collection.map(Into::into))
    }

    async fn create_collection(
        &self,
        entity: &CollectionEntity,
    ) -> DatabaseResult<CollectionEntity> {
        let bindings = ContentBindingEntity::serialize_many(&entity.supported_content);

        let collection = DataCollection::create(
            &self.pool,
            &entity.name,
            &entity.collection_type,
            entity.description.as_deref(),
            entity.available,
            entity.accept_all_content,
            Some(&bindings),
        )
        .await?;

        Ok(collection.into())
    }

    async fn update_collection(
        &self,
        entity: &CollectionEntity,
    ) -> DatabaseResult<CollectionEntity> {
        let id = entity
            .id
            .ok_or_else(|| DatabaseError::NotFound("Collection ID required".to_string()))?;

        let bindings = ContentBindingEntity::serialize_many(&entity.supported_content);

        let params = crate::models::taxii1::UpdateDataCollection {
            id,
            name: &entity.name,
            collection_type: &entity.collection_type,
            description: entity.description.as_deref(),
            available: entity.available,
            accept_all_content: entity.accept_all_content,
            bindings: Some(&bindings),
        };

        let collection = DataCollection::update(&self.pool, &params).await?;
        Ok(collection.into())
    }

    async fn delete_collection(&self, collection_name: &str) -> DatabaseResult<()> {
        DataCollection::delete_by_name(&self.pool, collection_name).await?;
        Ok(())
    }

    async fn set_collection_services(
        &self,
        collection_id: i32,
        service_ids: &[String],
    ) -> DatabaseResult<()> {
        DataCollection::set_services_validated(&self.pool, collection_id, service_ids).await?;

        debug!(
            id = collection_id,
            services = ?service_ids,
            "collection.services_set"
        );

        Ok(())
    }

    // ========================================================================
    // Content Block Operations
    // ========================================================================

    async fn get_content_blocks(
        &self,
        collection_id: Option<i32>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        bindings: Option<&[ContentBindingEntity]>,
        offset: i64,
        limit: Option<i64>,
    ) -> DatabaseResult<Vec<ContentBlockEntity>> {
        // Convert entity bindings to model bindings
        let model_bindings: Option<Vec<crate::models::taxii1::ContentBindingFilter>> = bindings
            .map(|binds| {
                binds
                    .iter()
                    .map(|b| crate::models::taxii1::ContentBindingFilter {
                        binding: b.binding.clone(),
                        subtypes: b.subtypes.clone(),
                    })
                    .collect()
            });

        let filter = crate::models::taxii1::ContentBlockFilter {
            collection_id,
            start_time,
            end_time,
            bindings: model_bindings.as_deref(),
            offset,
            limit,
        };

        let blocks = ContentBlock::find_filtered(&self.pool, &filter).await?;
        Ok(blocks.into_iter().map(Into::into).collect())
    }

    async fn get_content_blocks_count(
        &self,
        collection_id: Option<i32>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        bindings: Option<&[ContentBindingEntity]>,
    ) -> DatabaseResult<i64> {
        // Convert entity bindings to model bindings
        let model_bindings: Option<Vec<crate::models::taxii1::ContentBindingFilter>> = bindings
            .map(|binds| {
                binds
                    .iter()
                    .map(|b| crate::models::taxii1::ContentBindingFilter {
                        binding: b.binding.clone(),
                        subtypes: b.subtypes.clone(),
                    })
                    .collect()
            });

        let filter = crate::models::taxii1::ContentBlockFilter {
            collection_id,
            start_time,
            end_time,
            bindings: model_bindings.as_deref(),
            offset: 0,
            limit: None,
        };

        ContentBlock::count_filtered(&self.pool, &filter).await
    }

    async fn create_content_block(
        &self,
        entity: &ContentBlockEntity,
        collection_ids: Option<&[i32]>,
        _service_id: Option<&str>,
    ) -> DatabaseResult<ContentBlockEntity> {
        let (binding, subtype) = entity
            .content_binding
            .as_ref()
            .map(|cb| {
                let subtype = cb.subtypes.first().map(|s| s.as_str());
                (Some(cb.binding.as_str()), subtype)
            })
            .unwrap_or((None, None));

        let block = ContentBlock::create(
            &self.pool,
            entity.timestamp_label,
            entity.inbox_message_id,
            &entity.content,
            binding,
            subtype,
        )
        .await?;

        // Attach to collections and update volume (interleaved to match original semantics)
        if let Some(coll_ids) = collection_ids {
            for coll_id in coll_ids {
                ContentBlock::attach_to_collection(&self.pool, block.id, *coll_id).await?;
                DataCollection::increment_volume(&self.pool, *coll_id).await?;
            }

            debug!(
                content_block = block.id,
                collections = coll_ids.len(),
                "Content block added to collections"
            );
        }

        Ok(block.into())
    }

    async fn delete_content_blocks(
        &self,
        collection_name: &str,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
        with_messages: bool,
    ) -> DatabaseResult<i64> {
        // Get collection
        let collection = DataCollection::find_by_name(&self.pool, collection_name)
            .await?
            .ok_or_else(|| {
                DatabaseError::NotFound(format!(
                    "Collection with name '{collection_name}' does not exist"
                ))
            })?;

        // Get content block IDs
        let content_block_ids = ContentBlock::find_ids_by_collection_and_time(
            &self.pool,
            collection.id,
            start_time,
            end_time,
        )
        .await?;

        if content_block_ids.is_empty() {
            return Ok(0);
        }

        // Delete inbox messages if requested
        if with_messages {
            let inbox_message_ids =
                ContentBlock::get_inbox_message_ids(&self.pool, &content_block_ids).await?;
            if !inbox_message_ids.is_empty() {
                InboxMessage::delete_many(&self.pool, &inbox_message_ids).await?;
            }
        }

        // Delete content blocks
        let counter = ContentBlock::delete_many(&self.pool, &content_block_ids).await? as i64;

        // Update collection volume
        let volume = ContentBlock::count_by_collection(&self.pool, collection.id).await?;
        DataCollection::update_volume(&self.pool, collection.id, volume as i32).await?;

        Ok(counter)
    }

    // ========================================================================
    // Inbox Message Operations
    // ========================================================================

    async fn create_inbox_message(
        &self,
        entity: &InboxMessageEntity,
    ) -> DatabaseResult<InboxMessageEntity> {
        let destination_collections = if entity.destination_collections.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&entity.destination_collections).unwrap_or_default())
        };

        let params = crate::models::taxii1::NewInboxMessage {
            message_id: &entity.message_id,
            original_message: &entity.original_message,
            content_block_count: entity.content_block_count,
            destination_collections: destination_collections.as_deref(),
            service_id: &entity.service_id,
            result_id: entity.result_id.as_deref(),
            record_count: entity.record_count,
            partial_count: entity.partial_count,
            subscription_collection_name: entity.subscription_collection_name.as_deref(),
            subscription_id: entity.subscription_id.as_deref(),
            exclusive_begin_timestamp_label: entity.exclusive_begin_timestamp_label,
            inclusive_end_timestamp_label: entity.inclusive_end_timestamp_label,
        };

        let message = InboxMessage::create(&self.pool, &params).await?;
        Ok(message.into())
    }

    // ========================================================================
    // Result Set Operations
    // ========================================================================

    async fn create_result_set(&self, entity: &ResultSetEntity) -> DatabaseResult<ResultSetEntity> {
        let bindings = ContentBindingEntity::serialize_many(&entity.content_bindings);

        let result_set = ResultSet::create(
            &self.pool,
            &entity.id,
            entity.collection_id,
            Some(&bindings),
            entity.timeframe.0,
            entity.timeframe.1,
        )
        .await?;

        Ok(result_set.into())
    }

    async fn get_result_set(&self, result_set_id: &str) -> DatabaseResult<Option<ResultSetEntity>> {
        let result_set = ResultSet::find(&self.pool, result_set_id).await?;
        Ok(result_set.map(Into::into))
    }

    // ========================================================================
    // Subscription Operations
    // ========================================================================

    async fn get_subscription(
        &self,
        subscription_id: &str,
    ) -> DatabaseResult<Option<SubscriptionEntity>> {
        let subscription = Subscription::find(&self.pool, subscription_id).await?;
        Ok(subscription.map(Into::into))
    }

    async fn get_subscriptions(&self, service_id: &str) -> DatabaseResult<Vec<SubscriptionEntity>> {
        let subscriptions = Subscription::find_by_service(&self.pool, service_id).await?;
        Ok(subscriptions.into_iter().map(Into::into).collect())
    }

    async fn update_subscription(
        &self,
        entity: &SubscriptionEntity,
    ) -> DatabaseResult<SubscriptionEntity> {
        let params = entity.params.as_ref().map(|p| {
            json!({
                "response_type": p.response_type,
                "content_bindings": ContentBindingEntity::serialize_many(&p.content_bindings)
            })
            .to_string()
        });

        let subscription_id = entity
            .subscription_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let subscription = Subscription::upsert(
            &self.pool,
            &subscription_id,
            entity.collection_id,
            params.as_deref(),
            &entity.status,
            &entity.service_id,
        )
        .await?;

        debug!(
            subscription = %subscription_id,
            collection = entity.collection_id,
            status = %entity.status,
            "subscription.updated"
        );

        Ok(subscription.into())
    }

    async fn create_subscription(
        &self,
        entity: &SubscriptionEntity,
    ) -> DatabaseResult<SubscriptionEntity> {
        self.update_subscription(entity).await
    }
}
