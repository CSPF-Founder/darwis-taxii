//! Inbox message handlers.

use chrono::Utc;

use crate::constants::{SD_ACCEPTABLE_DESTINATION, SD_ITEM, StatusType};
use crate::error::{Taxii1xError, Taxii1xResult};
use crate::messages::{tm10, tm11};
use taxii_db::Taxii1Repository;

use super::base::{HandlerContext, TaxiiHeaders, generate_id};

use taxii_core::{
    CollectionEntity, ContentBindingEntity, ContentBlockCreatedEvent, ContentBlockEntity,
    InboxMessageCreatedEvent, InboxMessageEntity,
};

/// Result of validating destination collections.
struct ValidatedDestinations {
    /// Destination collection names for the inbox message entity.
    destination_names: Vec<String>,
    /// Collections that passed validation and permission checks.
    valid_collections: Vec<CollectionEntity>,
}

/// Validate destination collections for TAXII 1.1 inbox messages.
///
/// Validates that:
/// - Destinations are provided if required (or not provided if prohibited)
/// - All specified destinations exist and are available
/// - User has permission to modify the collections
async fn validate_destinations_11(
    ctx: &HandlerContext,
    message_id: &str,
    destination_names: Vec<String>,
) -> Taxii1xResult<ValidatedDestinations> {
    // Get service configuration
    let destination_collection_required = ctx
        .service
        .get_property("destination_collection_required")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Get all service collections for validation
    let service_collections = ctx
        .persistence
        .get_collections(Some(&ctx.service.id))
        .await?;

    let available_names: Vec<String> = service_collections
        .iter()
        .filter(|c| c.available)
        .map(|c| c.name.clone())
        .collect();

    // Validate destination_collection_required logic
    if (destination_collection_required && destination_names.is_empty())
        || (!destination_collection_required && !destination_names.is_empty())
    {
        let message = if destination_names.is_empty() {
            "A Destination_Collection_Name is required and none were specified"
        } else {
            "Destination_Collection_Names are prohibited for this Inbox Service"
        };

        return Err(Taxii1xError::StatusMessage {
            message: message.to_string(),
            in_response_to: Some(message_id.to_string()),
            status_type: StatusType::DestinationCollectionError,
            status_detail: Some(format!(
                "{}: {:?}",
                SD_ACCEPTABLE_DESTINATION, available_names
            )),
        });
    }

    // If no destination specified and not required, use all service collections
    let user_specified = !destination_names.is_empty();
    let names_to_use = if destination_names.is_empty() {
        available_names.clone()
    } else {
        destination_names.clone()
    };

    // Build map of available collections for validation
    let available_set: std::collections::HashSet<&str> =
        available_names.iter().map(String::as_str).collect();

    // Get valid collections with permission check
    let mut valid_collections = Vec::new();
    for name in &names_to_use {
        // Only check against available set if user explicitly specified collections
        if user_specified && !available_set.contains(name.as_str()) {
            return Err(Taxii1xError::StatusMessage {
                message: format!("Collection {} was not found", name),
                in_response_to: Some(message_id.to_string()),
                status_type: StatusType::NotFound,
                status_detail: Some(format!("{}: {}", SD_ITEM, name)),
            });
        }

        let Some(c) = ctx
            .persistence
            .get_collection(name, Some(&ctx.service.id))
            .await?
        else {
            continue;
        };

        if !c.available {
            continue;
        }

        // Security check: verify user can modify this collection
        if let Some(ref account) = ctx.account {
            if !c.can_modify(account) {
                return Err(Taxii1xError::StatusMessage {
                    message: format!("User can not write to collection {}", c.name),
                    in_response_to: Some(message_id.to_string()),
                    status_type: StatusType::Unauthorized,
                    status_detail: None,
                });
            }
        }
        valid_collections.push(c);
    }

    if !names_to_use.is_empty() && valid_collections.is_empty() {
        return Err(Taxii1xError::StatusMessage {
            message: "No valid destination collections".to_string(),
            in_response_to: Some(message_id.to_string()),
            status_type: StatusType::DestinationCollectionError,
            status_detail: None,
        });
    }

    Ok(ValidatedDestinations {
        destination_names,
        valid_collections,
    })
}

/// Store an inbox message and emit the creation hook if configured.
async fn store_inbox_message(
    ctx: &HandlerContext,
    message: InboxMessageEntity,
    save_raw: bool,
) -> Taxii1xResult<InboxMessageEntity> {
    if save_raw {
        let saved = ctx.persistence.create_inbox_message(&message).await?;

        // Emit inbox message created hook
        if let Some(ref hooks) = ctx.hooks {
            let event = InboxMessageCreatedEvent {
                inbox_message: saved.clone(),
                service_id: Some(ctx.service.id.clone()),
            };
            hooks.emit_inbox_message_created(event);
        }

        Ok(saved)
    } else {
        Ok(message)
    }
}

/// Store a content block and emit the creation hook if configured.
async fn store_content_block(
    ctx: &HandlerContext,
    block: &ContentBlockEntity,
    collection_ids: Option<&[i32]>,
) -> Taxii1xResult<(taxii_core::ContentBlockEntity, Vec<i32>)> {
    let created = ctx
        .persistence
        .create_content_block(block, collection_ids, Some(&ctx.service.id))
        .await?;

    let ids = collection_ids.map(|s| s.to_vec()).unwrap_or_default();

    // Emit content block created hook
    if let Some(ref hooks) = ctx.hooks {
        let event = ContentBlockCreatedEvent {
            content_block: created.clone(),
            collection_ids: ids.clone(),
            service_id: Some(ctx.service.id.clone()),
        };
        hooks.emit_content_block_created(event);
    }

    Ok((created, ids))
}

/// Get accepted content bindings from service configuration.
fn get_service_accepted_content(ctx: &HandlerContext) -> Vec<String> {
    ctx.service
        .get_property("accepted_content")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// TAXII 1.1 Inbox Message Handler.
pub struct InboxMessage11Handler;

impl InboxMessage11Handler {
    /// Handle a TAXII 1.1 Inbox Message.
    pub async fn handle_11(
        &self,
        ctx: &HandlerContext,
        _headers: &TaxiiHeaders,
        message: &tm11::Taxii11Message,
    ) -> Taxii1xResult<tm11::Taxii11Message> {
        let request = match message {
            tm11::Taxii11Message::InboxMessage(req) => req,
            _ => return Err(Taxii1xError::failure("Expected Inbox Message", None)),
        };

        // Validate destination collections
        let validated = validate_destinations_11(
            ctx,
            &request.message_id,
            request.destination_collection_names.clone(),
        )
        .await?;

        // Get original XML message (log error if serialization fails)
        let original_message_xml = tm11::Taxii11Message::InboxMessage(request.clone())
            .to_xml()
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to serialize inbox message to XML");
                request.message_id.clone()
            });

        // Create and store inbox message entity
        let inbox_message = InboxMessageEntity {
            id: None,
            message_id: request.message_id.clone(),
            original_message: original_message_xml.into_bytes(),
            content_block_count: request.content_blocks.len() as i32,
            service_id: ctx.service.id.clone(),
            destination_collections: validated.destination_names,
            result_id: request.result_id.clone(),
            record_count: request
                .record_count
                .as_ref()
                .map(|rc| rc.record_count as i32),
            partial_count: request
                .record_count
                .as_ref()
                .is_some_and(|rc| rc.partial_count),
            subscription_collection_name: request
                .subscription_information
                .as_ref()
                .map(|si| si.collection_name.clone()),
            subscription_id: request
                .subscription_information
                .as_ref()
                .map(|si| si.subscription_id.clone()),
            exclusive_begin_timestamp_label: None,
            inclusive_end_timestamp_label: None,
        };

        let save_raw = ctx
            .service
            .get_property("save_raw_inbox_messages")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let inbox_message = store_inbox_message(ctx, inbox_message, save_raw).await?;

        // Store content blocks
        let service_accepted = get_service_accepted_content(ctx);
        for content_block in &request.content_blocks {
            let content_binding = ContentBindingEntity {
                binding: content_block.content_binding.binding_id.clone(),
                subtypes: content_block
                    .content_binding
                    .subtype_ids
                    .iter()
                    .map(|s| s.subtype_id.clone())
                    .collect(),
            };

            // Check if service supports this content binding
            if !service_accepted.is_empty()
                && !service_accepted
                    .iter()
                    .any(|b| b == &content_binding.binding)
            {
                tracing::warn!(
                    binding = %content_binding.binding,
                    "Content binding not supported by service, skipping block"
                );
                continue;
            }

            // Filter collections that support this content binding
            let matching_ids: Vec<i32> = validated
                .valid_collections
                .iter()
                .filter(|c| c.is_content_supported(&content_binding))
                .filter_map(|c| c.id)
                .collect();

            // Skip if no collections support this binding (when collections are configured)
            if matching_ids.is_empty() && !validated.valid_collections.is_empty() {
                tracing::warn!(
                    binding = %content_binding.binding,
                    "No collections support this content binding, skipping block"
                );
                continue;
            }

            let timestamp_label = content_block
                .timestamp_label
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);

            let block_entity = ContentBlockEntity {
                id: None,
                content: content_block.content.as_bytes().to_vec(),
                timestamp_label,
                content_binding: Some(content_binding),
                message: content_block.message.clone(),
                inbox_message_id: inbox_message.id,
            };

            let collection_ids = if validated.valid_collections.is_empty() {
                None
            } else {
                Some(matching_ids.as_slice())
            };

            store_content_block(ctx, &block_entity, collection_ids).await?;
        }

        Ok(tm11::Taxii11Message::StatusMessage(
            tm11::StatusMessage::success(generate_id(), &request.message_id),
        ))
    }
}

/// TAXII 1.0 Inbox Message Handler.
pub struct InboxMessage10Handler;

impl InboxMessage10Handler {
    /// Get destination collections from service configuration (TAXII 1.0).
    async fn get_destination_collections(
        ctx: &HandlerContext,
        message_id: &str,
    ) -> Taxii1xResult<Vec<CollectionEntity>> {
        // In TAXII 1.0, destination collections come from service config
        let names: Vec<String> = ctx
            .service
            .get_property("destination_collection_names")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let mut collections = Vec::new();
        for name in &names {
            let Some(c) = ctx
                .persistence
                .get_collection(name, Some(&ctx.service.id))
                .await?
            else {
                continue;
            };

            if !c.available {
                continue;
            }

            // Check write permission
            if let Some(ref account) = ctx.account {
                if !c.can_modify(account) {
                    return Err(Taxii1xError::StatusMessage {
                        message: format!("User can not write to collection {}", c.name),
                        in_response_to: Some(message_id.to_string()),
                        status_type: StatusType::Unauthorized,
                        status_detail: None,
                    });
                }
            }
            collections.push(c);
        }

        Ok(collections)
    }

    /// Handle a TAXII 1.0 Inbox Message.
    pub async fn handle_10(
        &self,
        ctx: &HandlerContext,
        _headers: &TaxiiHeaders,
        message: &tm10::Taxii10Message,
    ) -> Taxii1xResult<tm10::Taxii10Message> {
        let request = match message {
            tm10::Taxii10Message::InboxMessage(req) => req,
            _ => return Err(Taxii1xError::failure("Expected Inbox Message", None)),
        };

        // Get destination collections from service config
        let destination_collections =
            Self::get_destination_collections(ctx, &request.message_id).await?;

        // Get original XML message (log error if serialization fails)
        let original_message_xml = tm10::Taxii10Message::InboxMessage(request.clone())
            .to_xml()
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to serialize inbox message to XML");
                request.message_id.clone()
            });

        // Create and store inbox message
        let inbox_message = InboxMessageEntity {
            id: None,
            message_id: request.message_id.clone(),
            original_message: original_message_xml.into_bytes(),
            content_block_count: request.content_blocks.len() as i32,
            service_id: ctx.service.id.clone(),
            destination_collections: Vec::new(),
            result_id: None,
            record_count: None,
            partial_count: false,
            subscription_collection_name: request
                .subscription_information
                .as_ref()
                .map(|si| si.feed_name.clone()),
            subscription_id: request
                .subscription_information
                .as_ref()
                .map(|si| si.subscription_id.clone()),
            exclusive_begin_timestamp_label: None,
            inclusive_end_timestamp_label: None,
        };

        let save_raw = ctx
            .service
            .get_property("save_raw_inbox_messages")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let inbox_message = store_inbox_message(ctx, inbox_message, save_raw).await?;

        // Store content blocks
        let service_accepted = get_service_accepted_content(ctx);
        for content_block in &request.content_blocks {
            let content_binding = ContentBindingEntity {
                binding: content_block.content_binding.clone(),
                subtypes: Vec::new(),
            };

            // Check if service supports this content binding
            if !service_accepted.is_empty()
                && !service_accepted
                    .iter()
                    .any(|b| b == &content_binding.binding)
            {
                tracing::warn!(
                    binding = %content_binding.binding,
                    "Content block binding is not supported by service, skipping"
                );
                continue;
            }

            let timestamp_label = content_block
                .timestamp_label
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now);

            let block_entity = ContentBlockEntity {
                id: None,
                content: content_block.content.as_bytes().to_vec(),
                timestamp_label,
                content_binding: Some(content_binding.clone()),
                message: None,
                inbox_message_id: inbox_message.id,
            };

            // Filter collections that support this content binding
            let matching_ids: Vec<i32> = destination_collections
                .iter()
                .filter(|c| c.is_content_supported(&content_binding))
                .filter_map(|c| c.id)
                .collect();

            let collection_ids = if matching_ids.is_empty() {
                None
            } else {
                Some(matching_ids.as_slice())
            };

            store_content_block(ctx, &block_entity, collection_ids).await?;
        }

        Ok(tm10::Taxii10Message::StatusMessage(
            tm10::StatusMessage::success(generate_id(), &request.message_id),
        ))
    }
}
