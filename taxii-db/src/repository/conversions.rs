//! Model-to-Entity conversions for TAXII types.
//!
//! This module provides `From` implementations to convert database models
//! to domain entities, ensuring consistent and type-safe transformations.

use taxii_core::{
    ApiRoot, Collection, CollectionEntity, ContentBindingEntity, ContentBlockEntity,
    InboxMessageEntity, ManifestRecord, ResultSetEntity, STIXObject, ServiceEntity,
    SubscriptionEntity, SubscriptionParameters, VersionRecord,
};

use crate::models::taxii1::{
    ContentBlock, DataCollection, InboxMessage, ResultSet, Service, Subscription,
};
use crate::models::taxii2;

// ============================================================================
// TAXII 1.x Conversions
// ============================================================================

impl From<Service> for ServiceEntity {
    fn from(model: Service) -> Self {
        let properties = model.properties().unwrap_or_default();
        Self {
            id: Some(model.id),
            service_type: model.service_type,
            properties,
        }
    }
}

impl From<DataCollection> for CollectionEntity {
    fn from(model: DataCollection) -> Self {
        let supported_content =
            ContentBindingEntity::deserialize_many(model.bindings.as_deref().unwrap_or("[]"));
        Self {
            id: Some(model.id),
            name: model.name,
            available: model.available,
            volume: Some(model.volume),
            description: model.description,
            accept_all_content: model.accept_all_content,
            collection_type: model.collection_type,
            supported_content,
        }
    }
}

impl From<ContentBlock> for ContentBlockEntity {
    fn from(model: ContentBlock) -> Self {
        let subtypes = model.binding_subtype.map(|s| vec![s]).unwrap_or_default();

        let content_binding = model
            .binding_id
            .map(|b| ContentBindingEntity::with_subtypes(b, subtypes));

        Self {
            id: Some(model.id),
            content: model.content,
            timestamp_label: model.timestamp_label,
            content_binding,
            message: model.message,
            inbox_message_id: model.inbox_message_id,
        }
    }
}

impl From<InboxMessage> for InboxMessageEntity {
    fn from(model: InboxMessage) -> Self {
        let destination_collections: Vec<String> = model
            .destination_collections
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        Self {
            id: Some(model.id),
            message_id: model.message_id,
            original_message: model.original_message,
            content_block_count: model.content_block_count,
            service_id: model.service_id,
            destination_collections,
            result_id: model.result_id,
            record_count: model.record_count,
            partial_count: model.partial_count,
            subscription_collection_name: model.subscription_collection_name,
            subscription_id: model.subscription_id,
            exclusive_begin_timestamp_label: model.exclusive_begin_timestamp_label,
            inclusive_end_timestamp_label: model.inclusive_end_timestamp_label,
        }
    }
}

impl From<ResultSet> for ResultSetEntity {
    fn from(model: ResultSet) -> Self {
        let content_bindings =
            ContentBindingEntity::deserialize_many(model.bindings.as_deref().unwrap_or("[]"));
        Self {
            id: model.id,
            collection_id: model.collection_id,
            content_bindings,
            timeframe: (model.begin_time, model.end_time),
        }
    }
}

impl From<Subscription> for SubscriptionEntity {
    fn from(model: Subscription) -> Self {
        let params = model.params.as_ref().and_then(|p| {
            let parsed: serde_json::Value = serde_json::from_str(p).ok()?;
            let response_type = parsed
                .get("response_type")
                .and_then(|v| v.as_str())
                .unwrap_or("FULL")
                .to_string();
            let content_bindings = parsed
                .get("content_bindings")
                .and_then(|v| v.as_str())
                .map(ContentBindingEntity::deserialize_many)
                .unwrap_or_default();

            Some(SubscriptionParameters {
                response_type,
                content_bindings,
            })
        });

        Self {
            service_id: model.service_id,
            collection_id: model.collection_id,
            subscription_id: Some(model.id),
            params,
            status: model.status,
        }
    }
}

// ============================================================================
// TAXII 2.x Conversions
// ============================================================================

impl From<taxii2::ApiRoot> for ApiRoot {
    fn from(model: taxii2::ApiRoot) -> Self {
        Self {
            id: model.id.to_string(),
            default: model.default,
            title: model.title,
            description: model.description,
            is_public: model.is_public,
        }
    }
}

impl From<taxii2::Collection> for Collection {
    fn from(model: taxii2::Collection) -> Self {
        Self {
            id: model.id.to_string(),
            api_root_id: model.api_root_id.to_string(),
            title: model.title,
            description: model.description,
            alias: model.alias,
            is_public: model.is_public,
            is_public_write: model.is_public_write,
        }
    }
}

impl From<taxii2::STIXObject> for STIXObject {
    fn from(model: taxii2::STIXObject) -> Self {
        Self {
            id: model.id,
            collection_id: model.collection_id.to_string(),
            stix_type: model.stix_type,
            spec_version: model.spec_version,
            date_added: model.date_added.and_utc(),
            version: model.version.and_utc(),
            serialized_data: model.serialized_data,
        }
    }
}

impl From<taxii2::STIXObject> for ManifestRecord {
    fn from(model: taxii2::STIXObject) -> Self {
        Self {
            id: model.id,
            date_added: model.date_added.and_utc(),
            version: model.version.and_utc(),
            spec_version: model.spec_version,
        }
    }
}

impl From<taxii2::VersionInfo> for VersionRecord {
    fn from(model: taxii2::VersionInfo) -> Self {
        Self {
            date_added: model.date_added.and_utc(),
            version: model.version.and_utc(),
        }
    }
}
