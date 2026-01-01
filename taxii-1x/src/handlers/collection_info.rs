//! Collection/Feed Information request handlers.

use crate::constants::{
    CT_DATA_FEED, SVC_COLLECTION_MANAGEMENT, SVC_FEED_MANAGEMENT, SVC_INBOX, SVC_POLL,
    VID_TAXII_HTTP_10, VID_TAXII_XML_10, VID_TAXII_XML_11,
};
use crate::error::{Taxii1xError, Taxii1xResult};
use crate::messages::{tm10, tm11};
use taxii_db::Taxii1Repository;

use super::base::{HandlerContext, TaxiiHeaders, generate_id};

/// Helper to extract string array from JSON value.
fn extract_string_array(value: Option<&serde_json::Value>, default: Vec<String>) -> Vec<String> {
    value
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or(default)
}

/// TAXII 1.1 Collection Information Request Handler.
pub struct CollectionInformationRequest11Handler;

impl CollectionInformationRequest11Handler {
    /// Handle a TAXII 1.1 Collection Information Request.
    pub async fn handle_11(
        &self,
        ctx: &HandlerContext,
        _headers: &TaxiiHeaders,
        message: &tm11::Taxii11Message,
    ) -> Taxii1xResult<tm11::Taxii11Message> {
        let request = match message {
            tm11::Taxii11Message::CollectionInformationRequest(req) => req,
            _ => {
                return Err(Taxii1xError::failure(
                    "Expected Collection Information Request message",
                    None,
                ));
            }
        };

        let mut response =
            tm11::CollectionInformationResponse::new(generate_id(), &request.message_id);

        // Get collections from database
        let collections = ctx.persistence.get_collections(Some(&ctx.service.id)).await;

        if let Ok(collections) = collections {
            for collection in collections {
                let collection_id = match collection.id {
                    Some(id) => id,
                    None => continue,
                };

                // Get polling services for this collection
                let polling_services = ctx
                    .persistence
                    .get_services_for_collection(collection_id, Some(SVC_POLL))
                    .await
                    .unwrap_or_default();

                let polling_service_instances: Vec<tm11::PollingServiceInstance> = polling_services
                    .iter()
                    .map(|svc| {
                        let service_id = svc.id.as_deref().unwrap_or_default();
                        let address = svc
                            .properties
                            .get("address")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| format!("/services/{service_id}/"));
                        let protocol = svc
                            .properties
                            .get("protocol_binding")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| VID_TAXII_HTTP_10.to_string());
                        let message_bindings = extract_string_array(
                            svc.properties.get("message_bindings"),
                            vec![VID_TAXII_XML_11.to_string()],
                        );

                        tm11::PollingServiceInstance {
                            poll_protocol: protocol,
                            poll_address: address,
                            poll_message_bindings: message_bindings,
                        }
                    })
                    .collect();

                // Get subscription services for this collection
                let subscription_services = ctx
                    .persistence
                    .get_services_for_collection(collection_id, Some(SVC_COLLECTION_MANAGEMENT))
                    .await
                    .unwrap_or_default();

                let subscription_methods: Vec<tm11::SubscriptionMethod> = subscription_services
                    .iter()
                    .map(|svc| {
                        let service_id = svc.id.as_deref().unwrap_or_default();
                        let address = svc
                            .properties
                            .get("address")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| format!("/services/{service_id}/"));
                        let protocol = svc
                            .properties
                            .get("protocol_binding")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| VID_TAXII_HTTP_10.to_string());
                        let message_bindings = extract_string_array(
                            svc.properties.get("message_bindings"),
                            vec![VID_TAXII_XML_11.to_string()],
                        );

                        tm11::SubscriptionMethod {
                            subscription_protocol: protocol,
                            subscription_address: address,
                            subscription_message_bindings: message_bindings,
                        }
                    })
                    .collect();

                // Get inbox services for this collection (TAXII 1.1 only)
                let inbox_services = ctx
                    .persistence
                    .get_services_for_collection(collection_id, Some(SVC_INBOX))
                    .await
                    .unwrap_or_default();

                let receiving_inbox_services: Vec<tm11::ReceivingInboxService> = inbox_services
                    .iter()
                    .map(|svc| {
                        let service_id = svc.id.as_deref().unwrap_or_default();
                        let address = svc
                            .properties
                            .get("address")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| format!("/services/{service_id}/"));
                        let protocol = svc
                            .properties
                            .get("protocol_binding")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| VID_TAXII_HTTP_10.to_string());
                        let message_bindings = extract_string_array(
                            svc.properties.get("message_bindings"),
                            vec![VID_TAXII_XML_11.to_string()],
                        );
                        let content_bindings: Vec<tm11::ContentBinding> =
                            extract_string_array(svc.properties.get("content_bindings"), vec![])
                                .into_iter()
                                .map(|s| tm11::ContentBinding::new(&s))
                                .collect();

                        tm11::ReceivingInboxService {
                            inbox_protocol: protocol,
                            inbox_address: address,
                            inbox_message_bindings: message_bindings,
                            supported_contents: content_bindings,
                        }
                    })
                    .collect();

                let collection_info = tm11::CollectionInformation {
                    collection_name: collection.name.clone(),
                    collection_type: Some(collection.collection_type.clone()),
                    available: Some(collection.available),
                    description: collection.description.clone(),
                    collection_volume: collection.volume,
                    supported_contents: collection
                        .supported_content
                        .iter()
                        .map(|cb| tm11::ContentBinding::new(&cb.binding))
                        .collect(),
                    polling_service_instances,
                    subscription_methods,
                    receiving_inbox_services,
                };
                response.collections.push(collection_info);
            }
        }

        Ok(tm11::Taxii11Message::CollectionInformationResponse(
            response,
        ))
    }
}

/// TAXII 1.0 Feed Information Request Handler.
pub struct FeedInformationRequest10Handler;

impl FeedInformationRequest10Handler {
    /// Handle a TAXII 1.0 Feed Information Request.
    pub async fn handle_10(
        &self,
        ctx: &HandlerContext,
        _headers: &TaxiiHeaders,
        message: &tm10::Taxii10Message,
    ) -> Taxii1xResult<tm10::Taxii10Message> {
        let request = match message {
            tm10::Taxii10Message::FeedInformationRequest(req) => req,
            _ => {
                return Err(Taxii1xError::failure(
                    "Expected Feed Information Request message",
                    None,
                ));
            }
        };

        let mut response = tm10::FeedInformationResponse::new(generate_id(), &request.message_id);

        // Get collections (feeds) from database
        // Only DATA_FEED type collections are valid for TAXII 1.0
        let collections = ctx.persistence.get_collections(Some(&ctx.service.id)).await;

        if let Ok(collections) = collections {
            for collection in collections {
                // In TAXII 1.0, only DATA_FEED type is supported
                if collection.collection_type != CT_DATA_FEED {
                    continue;
                }

                let collection_id = match collection.id {
                    Some(id) => id,
                    None => continue,
                };

                // Get polling services for this feed
                let polling_services = ctx
                    .persistence
                    .get_services_for_collection(collection_id, Some(SVC_POLL))
                    .await
                    .unwrap_or_default();

                let polling_service_instances: Vec<tm10::PollingServiceInstance> = polling_services
                    .iter()
                    .map(|svc| {
                        let service_id = svc.id.as_deref().unwrap_or_default();
                        let address = svc
                            .properties
                            .get("address")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| format!("/services/{service_id}/"));
                        let protocol = svc
                            .properties
                            .get("protocol_binding")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| VID_TAXII_HTTP_10.to_string());
                        let message_bindings = extract_string_array(
                            svc.properties.get("message_bindings"),
                            vec![VID_TAXII_XML_10.to_string()],
                        );

                        tm10::PollingServiceInstance {
                            poll_protocol: protocol,
                            poll_address: address,
                            poll_message_bindings: message_bindings,
                        }
                    })
                    .collect();

                // Get subscription services for this feed
                let subscription_services = ctx
                    .persistence
                    .get_services_for_collection(collection_id, Some(SVC_FEED_MANAGEMENT))
                    .await
                    .unwrap_or_default();

                let subscription_methods: Vec<tm10::SubscriptionMethod> = subscription_services
                    .iter()
                    .map(|svc| {
                        let service_id = svc.id.as_deref().unwrap_or_default();
                        let address = svc
                            .properties
                            .get("address")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| format!("/services/{service_id}/"));
                        let protocol = svc
                            .properties
                            .get("protocol_binding")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_else(|| VID_TAXII_HTTP_10.to_string());
                        let message_bindings = extract_string_array(
                            svc.properties.get("message_bindings"),
                            vec![VID_TAXII_XML_10.to_string()],
                        );

                        tm10::SubscriptionMethod {
                            subscription_protocol: protocol,
                            subscription_address: address,
                            subscription_message_bindings: message_bindings,
                        }
                    })
                    .collect();

                let feed_info = tm10::FeedInformation {
                    feed_name: collection.name.clone(),
                    available: Some(collection.available),
                    description: collection.description.clone(),
                    supported_contents: collection
                        .supported_content
                        .iter()
                        .map(|cb| cb.binding.clone())
                        .collect(),
                    polling_service_instances,
                    subscription_methods,
                };
                response.feeds.push(feed_info);
            }
        }

        Ok(tm10::Taxii10Message::FeedInformationResponse(response))
    }
}
