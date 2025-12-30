//! Subscription request handlers.

use std::collections::HashMap;

use crate::constants::{
    ACT_PAUSE, ACT_RESUME, ACT_STATUS, ACT_SUBSCRIBE, ACT_TYPES_10, ACT_TYPES_11, ACT_UNSUBSCRIBE,
    RT_FULL, SD_SUPPORTED_CONTENT, SVC_POLL, StatusType, VID_TAXII_HTTP_10, VID_TAXII_XML_10,
    VID_TAXII_XML_11,
};
use crate::error::{Taxii1xError, Taxii1xResult};
use crate::messages::{tm10, tm11};
use taxii_db::Taxii1Repository;

use super::base::{HandlerContext, TaxiiHeaders, generate_id};

use taxii_core::{
    ContentBindingEntity, SubscriptionCreatedEvent, SubscriptionEntity, SubscriptionParameters,
    subscription_status,
};

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

/// Get poll instances for a collection (TAXII 1.1).
async fn get_poll_instances_11(
    ctx: &HandlerContext,
    collection_id: i32,
) -> Vec<tm11::PollInstance> {
    let polling_services = ctx
        .persistence
        .get_services_for_collection(collection_id, Some(SVC_POLL))
        .await
        .unwrap_or_default();

    polling_services
        .iter()
        .map(|svc| {
            let service_id = svc.id.as_deref().unwrap_or_default();
            let address = svc
                .properties
                .get("address")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| format!("/services/{}/", service_id));
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

            tm11::PollInstance {
                poll_protocol: protocol,
                poll_address: address,
                poll_message_bindings: message_bindings,
            }
        })
        .collect()
}

/// Get poll instances for a collection (TAXII 1.0).
async fn get_poll_instances_10(
    ctx: &HandlerContext,
    collection_id: i32,
) -> Vec<tm10::PollInstance> {
    let polling_services = ctx
        .persistence
        .get_services_for_collection(collection_id, Some(SVC_POLL))
        .await
        .unwrap_or_default();

    polling_services
        .iter()
        .map(|svc| {
            let service_id = svc.id.as_deref().unwrap_or_default();
            let address = svc
                .properties
                .get("address")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| format!("/services/{}/", service_id));
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

            tm10::PollInstance {
                poll_protocol: protocol,
                poll_address: address,
                poll_message_bindings: message_bindings,
            }
        })
        .collect()
}

/// TAXII 1.1 Subscription Request Handler.
pub struct SubscriptionRequest11Handler;

impl SubscriptionRequest11Handler {
    /// Handle a TAXII 1.1 Subscription Management Request.
    // Nesting is inherent to TAXII protocol's action-based handling with
    // collection validation, subscription lookup, and action-specific logic.
    #[expect(
        clippy::excessive_nesting,
        reason = "TAXII protocol requires nested action handling"
    )]
    pub async fn handle_11(
        &self,
        ctx: &HandlerContext,
        _headers: &TaxiiHeaders,
        message: &tm11::Taxii11Message,
    ) -> Taxii1xResult<tm11::Taxii11Message> {
        let request = match message {
            tm11::Taxii11Message::ManageCollectionSubscriptionRequest(req) => req,
            _ => {
                return Err(Taxii1xError::failure(
                    "Expected Subscription Management Request message",
                    None,
                ));
            }
        };

        let collection_name = &request.collection_name;

        // Get collection
        let collection = ctx
            .persistence
            .get_collection(collection_name, Some(&ctx.service.id))
            .await?;

        let collection = collection.ok_or_else(|| {
            Taxii1xError::status_with_detail(
                StatusType::NotFound,
                "Requested collection was not found",
                Some(request.message_id.clone()),
                collection_name,
            )
        })?;

        let collection_id = collection.id.ok_or_else(|| {
            Taxii1xError::failure("Collection has no ID", Some(request.message_id.clone()))
        })?;

        // Get poll instances for this collection
        let poll_instances = get_poll_instances_11(ctx, collection_id).await;

        let mut response = tm11::ManageCollectionSubscriptionResponse::new(
            generate_id(),
            &request.message_id,
            collection_name,
        );

        // Set subscription message from service config
        response.message = ctx
            .service
            .get_property("subscription_message")
            .and_then(|v| v.as_str())
            .map(String::from);

        let action = request.action.as_str();

        // Validate action type
        if !ACT_TYPES_11.contains(&action) {
            return Err(Taxii1xError::StatusMessage {
                message: format!("Invalid action type: {}", action),
                in_response_to: Some(request.message_id.clone()),
                status_type: StatusType::BadMessage,
                status_detail: Some(action.to_string()),
            });
        }

        match action {
            ACT_SUBSCRIBE => {
                // Parse and validate content bindings against collection
                let params = if let Some(p) = request.subscription_parameters.as_ref() {
                    let response_type = p
                        .response_type
                        .clone()
                        .unwrap_or_else(|| RT_FULL.to_string());

                    let supported_contents = if p.content_bindings.is_empty() {
                        Vec::new()
                    } else {
                        // Parse requested bindings
                        let requested_bindings: Vec<ContentBindingEntity> = p
                            .content_bindings
                            .iter()
                            .map(|cb| ContentBindingEntity {
                                binding: cb.binding_id.clone(),
                                subtypes: cb
                                    .subtype_ids
                                    .iter()
                                    .map(|s| s.subtype_id.clone())
                                    .collect(),
                            })
                            .collect();

                        // Get matching bindings from collection
                        let matching = collection.get_matching_bindings(&requested_bindings);

                        // If bindings were requested but none matched, raise error
                        if !requested_bindings.is_empty() && matching.is_empty() {
                            // Build supported content for error details
                            let supported: Vec<String> = collection
                                .supported_content
                                .iter()
                                .map(|cb| cb.binding.clone())
                                .collect();
                            let details = HashMap::from([(
                                SD_SUPPORTED_CONTENT.to_owned(),
                                supported.join(", "),
                            )]);

                            return Err(Taxii1xError::StatusMessage {
                                message: "Content bindings not supported by collection".to_string(),
                                in_response_to: Some(request.message_id.clone()),
                                status_type: StatusType::UnsupportedContentBinding,
                                status_detail: Some(format!("{:?}", details)),
                            });
                        }

                        matching
                    };

                    Some(SubscriptionParameters {
                        response_type,
                        content_bindings: supported_contents,
                    })
                } else {
                    None
                };

                let subscription = SubscriptionEntity {
                    service_id: ctx.service.id.clone(),
                    collection_id,
                    subscription_id: Some(generate_id()),
                    params,
                    status: subscription_status::ACTIVE.to_string(),
                };

                let subscription = ctx.persistence.create_subscription(&subscription).await?;

                // Emit subscription created hook
                if let Some(ref hooks) = ctx.hooks {
                    let event = SubscriptionCreatedEvent {
                        subscription: subscription.clone(),
                        collection_name: collection_name.clone(),
                    };
                    hooks.emit_subscription_created(event);
                }

                let instance = tm11::SubscriptionInstance {
                    subscription_id: subscription.subscription_id.unwrap_or_default(),
                    status: Some(subscription.status),
                    subscription_parameters: request.subscription_parameters.clone(),
                    push_parameters: request.push_parameters.clone(),
                    poll_instances,
                };
                response.subscription_instances.push(instance);
            }

            ACT_UNSUBSCRIBE => {
                let subscription_id = request.subscription_id.as_ref().ok_or_else(|| {
                    Taxii1xError::StatusMessage {
                        message: format!("Action \"{}\" requires a subscription id", action),
                        in_response_to: Some(request.message_id.clone()),
                        status_type: StatusType::BadMessage,
                        status_detail: None,
                    }
                })?;

                let subscription = ctx.persistence.get_subscription(subscription_id).await?;

                if let Some(mut sub) = subscription {
                    // Validate subscription belongs to requested collection
                    if sub.collection_id != collection_id {
                        return Err(Taxii1xError::StatusMessage {
                            message: "Subscription does not belong to requested collection"
                                .to_string(),
                            in_response_to: Some(request.message_id.clone()),
                            status_type: StatusType::NotFound,
                            status_detail: Some(collection_name.clone()),
                        });
                    }

                    sub.status = subscription_status::UNSUBSCRIBED.to_string();
                    ctx.persistence.update_subscription(&sub).await?;

                    let instance = tm11::SubscriptionInstance {
                        subscription_id: sub.subscription_id.unwrap_or_default(),
                        status: Some(sub.status),
                        subscription_parameters: None,
                        push_parameters: None,
                        poll_instances,
                    };
                    response.subscription_instances.push(instance);
                } else {
                    // Spec says unsubscribe should be successful even if subscription doesn't exist
                    // Return a synthetic unsubscribed instance
                    let instance = tm11::SubscriptionInstance {
                        subscription_id: subscription_id.clone(),
                        status: Some(subscription_status::UNSUBSCRIBED.to_string()),
                        subscription_parameters: None,
                        push_parameters: None,
                        poll_instances,
                    };
                    response.subscription_instances.push(instance);
                }
            }

            ACT_PAUSE => {
                let subscription_id = request.subscription_id.as_ref().ok_or_else(|| {
                    Taxii1xError::StatusMessage {
                        message: format!("Action \"{}\" requires a subscription id", action),
                        in_response_to: Some(request.message_id.clone()),
                        status_type: StatusType::BadMessage,
                        status_detail: None,
                    }
                })?;

                let subscription = ctx.persistence.get_subscription(subscription_id).await?;

                // Raise ST_NOT_FOUND if subscription not found for PAUSE
                let mut sub = subscription.ok_or_else(|| Taxii1xError::StatusMessage {
                    message: "Subscription not found".to_string(),
                    in_response_to: Some(request.message_id.clone()),
                    status_type: StatusType::NotFound,
                    status_detail: Some(subscription_id.clone()),
                })?;

                // Validate subscription belongs to requested collection
                if sub.collection_id != collection_id {
                    return Err(Taxii1xError::StatusMessage {
                        message: "Subscription does not belong to requested collection".to_string(),
                        in_response_to: Some(request.message_id.clone()),
                        status_type: StatusType::NotFound,
                        status_detail: Some(collection_name.clone()),
                    });
                }

                // If already paused, just return it without update
                if sub.status != subscription_status::PAUSED {
                    sub.status = subscription_status::PAUSED.to_string();
                    ctx.persistence.update_subscription(&sub).await?;
                }

                let instance = tm11::SubscriptionInstance {
                    subscription_id: sub.subscription_id.unwrap_or_default(),
                    status: Some(sub.status),
                    subscription_parameters: None,
                    push_parameters: None,
                    poll_instances,
                };
                response.subscription_instances.push(instance);
            }

            ACT_RESUME => {
                let subscription_id = request.subscription_id.as_ref().ok_or_else(|| {
                    Taxii1xError::StatusMessage {
                        message: format!("Action \"{}\" requires a subscription id", action),
                        in_response_to: Some(request.message_id.clone()),
                        status_type: StatusType::BadMessage,
                        status_detail: None,
                    }
                })?;

                let subscription = ctx.persistence.get_subscription(subscription_id).await?;

                // Raise ST_NOT_FOUND if subscription not found for RESUME
                let mut sub = subscription.ok_or_else(|| Taxii1xError::StatusMessage {
                    message: "Subscription not found".to_string(),
                    in_response_to: Some(request.message_id.clone()),
                    status_type: StatusType::NotFound,
                    status_detail: Some(subscription_id.clone()),
                })?;

                // Validate subscription belongs to requested collection
                if sub.collection_id != collection_id {
                    return Err(Taxii1xError::StatusMessage {
                        message: "Subscription does not belong to requested collection".to_string(),
                        in_response_to: Some(request.message_id.clone()),
                        status_type: StatusType::NotFound,
                        status_detail: Some(collection_name.clone()),
                    });
                }

                // Only resume if currently in PAUSED state
                if sub.status == subscription_status::PAUSED {
                    sub.status = subscription_status::ACTIVE.to_string();
                    ctx.persistence.update_subscription(&sub).await?;
                }

                let instance = tm11::SubscriptionInstance {
                    subscription_id: sub.subscription_id.unwrap_or_default(),
                    status: Some(sub.status),
                    subscription_parameters: None,
                    push_parameters: None,
                    poll_instances,
                };
                response.subscription_instances.push(instance);
            }

            ACT_STATUS => {
                // Return status for all subscriptions or specific one
                if let Some(subscription_id) = &request.subscription_id {
                    let subscription = ctx.persistence.get_subscription(subscription_id).await?;

                    match subscription {
                        Some(sub) => {
                            // Validate subscription belongs to requested collection
                            if sub.collection_id != collection_id {
                                return Err(Taxii1xError::StatusMessage {
                                    message: "Subscription does not belong to requested collection"
                                        .to_string(),
                                    in_response_to: Some(request.message_id.clone()),
                                    status_type: StatusType::NotFound,
                                    status_detail: Some(subscription_id.clone()),
                                });
                            }

                            let instance = tm11::SubscriptionInstance {
                                subscription_id: sub.subscription_id.unwrap_or_default(),
                                status: Some(sub.status),
                                subscription_parameters: None,
                                push_parameters: None,
                                poll_instances,
                            };
                            response.subscription_instances.push(instance);
                        }
                        None => {
                            // Raise ST_NOT_FOUND when subscription_id provided but not found
                            return Err(Taxii1xError::StatusMessage {
                                message: "Subscription not found".to_string(),
                                in_response_to: Some(request.message_id.clone()),
                                status_type: StatusType::NotFound,
                                status_detail: Some(subscription_id.clone()),
                            });
                        }
                    }
                } else {
                    // Return all subscriptions for this service
                    let subscriptions = ctx.persistence.get_subscriptions(&ctx.service.id).await?;

                    response.subscription_instances = subscriptions
                        .into_iter()
                        .filter(|sub| sub.collection_id == collection_id)
                        .map(|sub| tm11::SubscriptionInstance {
                            subscription_id: sub.subscription_id.unwrap_or_default(),
                            status: Some(sub.status),
                            subscription_parameters: None,
                            push_parameters: None,
                            poll_instances: poll_instances.clone(),
                        })
                        .collect();
                }
            }

            _ => {
                return Err(Taxii1xError::failure(
                    format!("Unknown action: {}", action),
                    Some(request.message_id.clone()),
                ));
            }
        }

        Ok(tm11::Taxii11Message::ManageCollectionSubscriptionResponse(
            response,
        ))
    }
}

/// TAXII 1.0 Subscription Request Handler.
pub struct SubscriptionRequest10Handler;

impl SubscriptionRequest10Handler {
    /// Handle a TAXII 1.0 Subscription Management Request.
    pub async fn handle_10(
        &self,
        ctx: &HandlerContext,
        _headers: &TaxiiHeaders,
        message: &tm10::Taxii10Message,
    ) -> Taxii1xResult<tm10::Taxii10Message> {
        let request = match message {
            tm10::Taxii10Message::ManageFeedSubscriptionRequest(req) => req,
            _ => {
                return Err(Taxii1xError::failure(
                    "Expected Subscription Management Request message",
                    None,
                ));
            }
        };

        let feed_name = &request.feed_name;

        // Get collection (feed)
        let collection = ctx
            .persistence
            .get_collection(feed_name, Some(&ctx.service.id))
            .await?;

        let collection = collection.ok_or_else(|| {
            Taxii1xError::status_with_detail(
                StatusType::NotFound,
                "Requested feed was not found",
                Some(request.message_id.clone()),
                feed_name,
            )
        })?;

        let collection_id = collection.id.ok_or_else(|| {
            Taxii1xError::failure("Feed has no ID", Some(request.message_id.clone()))
        })?;

        // Get poll instances for this collection
        let poll_instances = get_poll_instances_10(ctx, collection_id).await;

        let mut response = tm10::ManageFeedSubscriptionResponse::new(
            generate_id(),
            &request.message_id,
            feed_name,
        );

        // Set subscription message from service config
        response.message = ctx
            .service
            .get_property("subscription_message")
            .and_then(|v| v.as_str())
            .map(String::from);

        let action = request.action.as_str();

        // Validate action type for TAXII 1.0 (no PAUSE/RESUME)
        if !ACT_TYPES_10.contains(&action) {
            return Err(Taxii1xError::StatusMessage {
                message: format!("Invalid action type: {}", action),
                in_response_to: Some(request.message_id.clone()),
                status_type: StatusType::BadMessage,
                status_detail: Some(action.to_string()),
            });
        }

        match action {
            ACT_SUBSCRIBE => {
                let subscription = SubscriptionEntity {
                    service_id: ctx.service.id.clone(),
                    collection_id,
                    subscription_id: Some(generate_id()),
                    params: None,
                    status: subscription_status::ACTIVE.to_string(),
                };

                let subscription = ctx.persistence.create_subscription(&subscription).await?;

                // Emit subscription created hook
                if let Some(ref hooks) = ctx.hooks {
                    let event = SubscriptionCreatedEvent {
                        subscription: subscription.clone(),
                        collection_name: feed_name.clone(),
                    };
                    hooks.emit_subscription_created(event);
                }

                let instance = tm10::SubscriptionInstance {
                    subscription_id: subscription.subscription_id.unwrap_or_default(),
                    delivery_parameters: request.delivery_parameters.clone(),
                    poll_instances,
                };
                response.subscription_instances.push(instance);
            }

            ACT_UNSUBSCRIBE => {
                let subscription_id = request.subscription_id.as_ref().ok_or_else(|| {
                    Taxii1xError::failure(
                        "Subscription ID required for unsubscribe",
                        Some(request.message_id.clone()),
                    )
                })?;

                let subscription = ctx.persistence.get_subscription(subscription_id).await?;

                if let Some(mut sub) = subscription {
                    sub.status = subscription_status::UNSUBSCRIBED.to_string();
                    ctx.persistence.update_subscription(&sub).await?;
                }
            }

            ACT_STATUS => {
                if let Some(subscription_id) = &request.subscription_id {
                    let subscription = ctx.persistence.get_subscription(subscription_id).await?;

                    if let Some(sub) = subscription {
                        let instance = tm10::SubscriptionInstance {
                            subscription_id: sub.subscription_id.unwrap_or_default(),
                            delivery_parameters: None,
                            poll_instances,
                        };
                        response.subscription_instances.push(instance);
                    }
                } else {
                    let subscriptions = ctx.persistence.get_subscriptions(&ctx.service.id).await?;

                    response.subscription_instances = subscriptions
                        .into_iter()
                        .filter(|sub| sub.collection_id == collection_id)
                        .map(|sub| tm10::SubscriptionInstance {
                            subscription_id: sub.subscription_id.unwrap_or_default(),
                            delivery_parameters: None,
                            poll_instances: poll_instances.clone(),
                        })
                        .collect();
                }
            }

            _ => {
                return Err(Taxii1xError::failure(
                    format!("Unknown action: {}", action),
                    Some(request.message_id.clone()),
                ));
            }
        }

        Ok(tm10::Taxii10Message::ManageFeedSubscriptionResponse(
            response,
        ))
    }
}
