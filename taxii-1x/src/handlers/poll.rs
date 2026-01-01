//! Poll request handlers.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::constants::{
    CT_DATA_FEED, RT_COUNT_ONLY, RT_FULL, SD_ESTIMATED_WAIT, SD_RESULT_ID, SD_SUPPORTED_CONTENT,
    SD_WILL_PUSH, ST_PENDING, StatusType,
};
use crate::error::{Taxii1xError, Taxii1xResult};
use crate::messages::{tm10, tm11};
use taxii_core::{CollectionEntity, ContentBindingEntity};
use taxii_db::{DatabaseError, Taxii1Repository};

use super::base::{HandlerContext, TaxiiHeaders, generate_id};

/// Parse an RFC3339 timestamp string into a DateTime<Utc>.
fn parse_timestamp(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

/// Resolved poll parameters including bindings and response type.
struct ResolvedPollParams {
    content_bindings: Vec<ContentBindingEntity>,
    response_type: String,
    allow_async: bool,
}

/// Resolve content bindings from subscription or poll parameters.
///
/// Returns the content bindings, response type, and allow_async flag.
async fn resolve_poll_bindings_11(
    ctx: &HandlerContext,
    request: &tm11::PollRequest,
    collection: &CollectionEntity,
) -> Taxii1xResult<ResolvedPollParams> {
    let collection_id = collection.id.unwrap_or(0);

    if let Some(ref subscription_id) = request.subscription_id {
        // Get bindings from subscription
        let subscription = ctx.persistence.get_subscription(subscription_id).await?;

        let sub = subscription.ok_or_else(|| Taxii1xError::StatusMessage {
            message: "Requested subscription was not found".to_string(),
            in_response_to: Some(request.message_id.clone()),
            status_type: StatusType::NotFound,
            status_detail: Some(subscription_id.clone()),
        })?;

        // Validate subscription belongs to collection
        if sub.collection_id != collection_id {
            return Err(Taxii1xError::StatusMessage {
                message: "Subscription does not belong to requested collection".to_string(),
                in_response_to: Some(request.message_id.clone()),
                status_type: StatusType::NotFound,
                status_detail: Some(collection.name.clone()),
            });
        }

        let bindings = sub
            .params
            .as_ref()
            .map(|p| p.content_bindings.clone())
            .unwrap_or_default();
        let resp_type = sub
            .params
            .as_ref()
            .map(|p| p.response_type.clone())
            .unwrap_or_else(|| RT_FULL.to_string());

        Ok(ResolvedPollParams {
            content_bindings: bindings,
            response_type: resp_type,
            allow_async: false,
        })
    } else if let Some(ref params) = request.poll_parameters {
        // Parse content bindings from poll_parameters
        let requested_bindings: Vec<ContentBindingEntity> = params
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
        let content_bindings = if requested_bindings.is_empty() {
            Vec::new()
        } else {
            let matching = collection.get_matching_bindings(&requested_bindings);

            // Error if requested but none match
            if !requested_bindings.is_empty() && matching.is_empty() {
                let supported: Vec<String> = collection
                    .supported_content
                    .iter()
                    .map(|cb| cb.binding.clone())
                    .collect();
                let details =
                    HashMap::from([(SD_SUPPORTED_CONTENT.to_owned(), supported.join(", "))]);

                return Err(Taxii1xError::StatusMessage {
                    message: "Content bindings not supported by collection".to_string(),
                    in_response_to: Some(request.message_id.clone()),
                    status_type: StatusType::UnsupportedContentBinding,
                    status_detail: Some(format!("{details:?}")),
                });
            }
            matching
        };

        let resp_type = params
            .response_type
            .clone()
            .unwrap_or_else(|| RT_FULL.to_string());
        let allow_async = params.allow_asynch.unwrap_or(false);

        Ok(ResolvedPollParams {
            content_bindings,
            response_type: resp_type,
            allow_async,
        })
    } else {
        Ok(ResolvedPollParams {
            content_bindings: Vec::new(),
            response_type: RT_FULL.to_string(),
            allow_async: false,
        })
    }
}

/// Resolve content bindings for TAXII 1.0 poll requests.
async fn resolve_poll_bindings_10(
    ctx: &HandlerContext,
    request: &tm10::PollRequest,
    collection: &CollectionEntity,
) -> Taxii1xResult<Vec<ContentBindingEntity>> {
    let collection_id = collection.id.unwrap_or(0);

    if let Some(ref subscription_id) = request.subscription_id {
        // Get bindings from subscription
        let subscription = ctx.persistence.get_subscription(subscription_id).await?;

        let sub = subscription.ok_or_else(|| Taxii1xError::StatusMessage {
            message: "Requested subscription was not found".to_string(),
            in_response_to: Some(request.message_id.clone()),
            status_type: StatusType::NotFound,
            status_detail: Some(subscription_id.clone()),
        })?;

        // Validate subscription belongs to collection
        if sub.collection_id != collection_id {
            return Err(Taxii1xError::StatusMessage {
                message: "Subscription does not belong to requested collection".to_string(),
                in_response_to: Some(request.message_id.clone()),
                status_type: StatusType::NotFound,
                status_detail: Some(collection.name.clone()),
            });
        }

        Ok(sub
            .params
            .as_ref()
            .map(|p| p.content_bindings.clone())
            .unwrap_or_default())
    } else {
        // Parse bindings from request
        let requested_bindings: Vec<ContentBindingEntity> = request
            .content_bindings
            .iter()
            .map(|b| ContentBindingEntity {
                binding: b.clone(),
                subtypes: Vec::new(),
            })
            .collect();

        if requested_bindings.is_empty() {
            return Ok(Vec::new());
        }

        let matching = collection.get_matching_bindings(&requested_bindings);

        // Error if requested but none match
        if !requested_bindings.is_empty() && matching.is_empty() {
            let supported: Vec<String> = collection
                .supported_content
                .iter()
                .map(|cb| cb.binding.clone())
                .collect();
            let details = HashMap::from([(SD_SUPPORTED_CONTENT.to_owned(), supported.join(", "))]);

            return Err(Taxii1xError::StatusMessage {
                message: "Content bindings not supported by collection".to_string(),
                in_response_to: Some(request.message_id.clone()),
                status_type: StatusType::UnsupportedContentBinding,
                status_detail: Some(format!("{details:?}")),
            });
        }

        Ok(matching)
    }
}

/// Handler for TAXII 1.1 Poll Request messages.
///
/// The Poll service allows clients to request cyber threat intelligence (CTI)
/// content from a collection. Polling supports both synchronous and asynchronous
/// modes:
///
/// - **Synchronous**: Content is returned immediately in the response
/// - **Asynchronous**: Server returns `ST_PENDING` status; client must use
///   Poll Fulfillment to retrieve results later
///
/// # Message Flow
///
/// ```text
/// Client                          Server
///   |                               |
///   |--- Poll Request ------------->|
///   |                               |
///   |<-- Poll Response (content) ---|  (sync)
///   |     or                        |
///   |<-- Status (ST_PENDING) -------|  (async)
///   |                               |
///   |--- Poll Fulfillment --------->|  (if async)
///   |<-- Poll Response (content) ---|
/// ```
pub struct PollRequest11Handler;

/// Result of processing a TAXII 1.1 Poll Request.
///
/// A poll request can result in either:
/// - A [`PollResponse`](tm11::PollResponse) containing content blocks
/// - A [`StatusMessage`](tm11::StatusMessage) indicating async processing
///
/// # Synchronous vs Asynchronous
///
/// When content is immediately available, the handler returns `Response` with
/// the requested content blocks.
///
/// When results require preparation (e.g., large datasets), the handler returns
/// `Status` with `ST_PENDING` and a result ID. The client then uses Poll
/// Fulfillment to retrieve content in parts.
pub enum PollResult {
    /// Successful poll response containing content blocks.
    ///
    /// May include pagination info (`more`, `result_id`) if there are
    /// additional results to retrieve.
    Response(tm11::PollResponse),

    /// Status message for asynchronous polling.
    ///
    /// Typically `ST_PENDING` with status details including:
    /// - `ESTIMATED_WAIT`: Suggested seconds before retry
    /// - `RESULT_ID`: ID for Poll Fulfillment requests
    /// - `WILL_PUSH`: Whether server will push results
    Status(tm11::StatusMessage),
}

/// Internal parameters for poll response preparation.
///
/// Encapsulates all the options that affect how a poll response is built,
/// extracted from either subscription parameters or explicit poll parameters.
struct PollParams<'a> {
    /// Name of the collection to poll.
    collection_name: &'a str,

    /// Message ID of the original request (for response correlation).
    in_response_to: &'a str,

    /// Time range filter: (exclusive_begin, inclusive_end).
    ///
    /// Only content blocks with timestamps in this range are returned.
    timeframe: (Option<DateTime<Utc>>, Option<DateTime<Utc>>),

    /// Content type filter (e.g., STIX, OpenIOC bindings).
    ///
    /// When `Some`, only matching content blocks are returned.
    content_bindings: Option<Vec<String>>,

    /// Whether to return full content (`true`) or just count (`false`).
    ///
    /// When `false`, response includes only `record_count`.
    return_content: bool,

    /// Subscription ID if polling via subscription.
    subscription_id: Option<&'a str>,

    /// Whether asynchronous polling is allowed.
    ///
    /// If `false` and results aren't ready, returns an error.
    allow_async: bool,

    /// Which part of paginated results to return (1-based).
    result_part: i32,

    /// Result set ID for retrieving subsequent pages.
    result_id: Option<&'a str>,
}

impl PollRequest11Handler {
    /// Prepare a poll response.
    ///
    /// This method handles both synchronous and asynchronous polling:
    /// - For synchronous polling, returns content blocks directly
    /// - For asynchronous polling (when ResultsNotReady is raised), returns ST_PENDING status
    async fn prepare_poll_response(
        ctx: &HandlerContext,
        params: PollParams<'_>,
    ) -> Taxii1xResult<PollResult> {
        let PollParams {
            collection_name,
            in_response_to,
            timeframe,
            content_bindings,
            return_content,
            subscription_id,
            allow_async,
            result_part,
            result_id,
        } = params;

        // Get collection
        let collection = ctx
            .persistence
            .get_collection(collection_name, Some(&ctx.service.id))
            .await?;

        let collection = collection.ok_or_else(|| {
            Taxii1xError::status_with_detail(
                StatusType::NotFound,
                "Requested collection was not found",
                Some(in_response_to.to_string()),
                collection_name,
            )
        })?;

        if !collection.available {
            return Err(Taxii1xError::failure(
                "The collection is not available",
                Some(in_response_to.to_string()),
            ));
        }

        // Get service configuration for pagination
        let count_blocks_in_poll_responses = ctx
            .service
            .get_property("count_blocks_in_poll_responses")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let max_result_size = ctx
            .service
            .get_property("max_result_size")
            .and_then(|v| v.as_i64())
            .unwrap_or(1_000_000) as usize;

        let max_result_count = ctx
            .service
            .get_property("max_result_count")
            .and_then(|v| v.as_i64())
            .unwrap_or(10_000_000);

        // Convert content bindings to entities
        let binding_entities: Option<Vec<taxii_core::ContentBindingEntity>> =
            content_bindings.clone().map(|bindings| {
                bindings
                    .iter()
                    .map(|b| taxii_core::ContentBindingEntity {
                        binding: b.clone(),
                        subtypes: Vec::new(),
                    })
                    .collect()
            });

        if return_content {
            // Calculate offset based on result_part
            let offset = ((result_part - 1) as i64) * (max_result_size as i64);

            // Try to get content blocks - may raise ResultsNotReady for async polling
            let blocks_result = ctx
                .persistence
                .get_content_blocks(
                    collection.id,
                    timeframe.0,
                    timeframe.1,
                    binding_entities.as_deref(),
                    offset,
                    Some(max_result_size as i64),
                )
                .await;

            match blocks_result {
                Ok(blocks) => {
                    // Calculate has_more and record_count
                    let (has_more, capped_count, is_partial) = if count_blocks_in_poll_responses {
                        // Count total and calculate
                        let total_count = ctx
                            .persistence
                            .get_content_blocks_count(
                                collection.id,
                                timeframe.0,
                                timeframe.1,
                                binding_entities.as_deref(),
                            )
                            .await?;

                        let has_more =
                            (total_count as f64 / max_result_size as f64) > result_part as f64;
                        let capped_count = std::cmp::min(max_result_count, total_count);
                        let is_partial = capped_count < total_count;

                        (has_more, Some(capped_count), is_partial)
                    } else {
                        // Simple check without counting
                        let has_more = blocks.len() == max_result_size;
                        (has_more, None, false)
                    };

                    // Create result set if has_more and no result_id
                    let result_id_to_use = if has_more && result_id.is_none() {
                        let result_bindings: Vec<taxii_core::ContentBindingEntity> =
                            content_bindings
                                .unwrap_or_default()
                                .into_iter()
                                .map(|b| taxii_core::ContentBindingEntity {
                                    binding: b,
                                    subtypes: Vec::new(),
                                })
                                .collect();

                        let result_set_entity = taxii_core::ResultSetEntity {
                            id: generate_id(),
                            collection_id: collection.id.unwrap_or(0),
                            content_bindings: result_bindings,
                            timeframe,
                        };

                        let result_set = ctx
                            .persistence
                            .create_result_set(&result_set_entity)
                            .await?;

                        Some(result_set.id)
                    } else {
                        result_id.map(|s| s.to_string())
                    };

                    // Build response
                    let mut response = tm11::PollResponse::new(
                        generate_id(),
                        in_response_to.to_string(),
                        collection_name.to_string(),
                    );

                    response.more = Some(has_more);
                    response.result_id = result_id_to_use;
                    response.result_part_number = Some(result_part);

                    // Set subscription ID if provided
                    if let Some(sub_id) = subscription_id {
                        response.subscription_id = Some(sub_id.to_string());
                    }

                    // Set timeframe in response
                    if let Some(start) = timeframe.0 {
                        response.exclusive_begin_timestamp_label = Some(start.to_rfc3339());
                    }
                    if let Some(end) = timeframe.1 {
                        response.inclusive_end_timestamp_label = Some(end.to_rfc3339());
                    }

                    // Set record count
                    if let Some(count) = capped_count {
                        response.record_count = Some(tm11::RecordCount {
                            partial_count: is_partial,
                            record_count: count,
                        });
                    }

                    // Add content blocks
                    response.content_blocks = blocks
                        .into_iter()
                        .map(|block| tm11::ContentBlock {
                            content_binding: tm11::ContentBinding::new(
                                block
                                    .content_binding
                                    .as_ref()
                                    .map(|cb| cb.binding.as_str())
                                    .unwrap_or(""),
                            ),
                            content: String::from_utf8_lossy(&block.content).into_owned(),
                            timestamp_label: Some(block.timestamp_label.to_rfc3339()),
                            message: block.message,
                            padding: None,
                        })
                        .collect();

                    Ok(PollResult::Response(response))
                }
                Err(DatabaseError::ResultsNotReady) => {
                    // Handle async polling - results not immediately available
                    if !allow_async {
                        return Err(Taxii1xError::failure(
                            "The content is not available now and the request has allow_asynch set to false",
                            Some(in_response_to.to_string()),
                        ));
                    }

                    // Build content binding entities for result set
                    let result_bindings: Vec<taxii_core::ContentBindingEntity> = content_bindings
                        .unwrap_or_default()
                        .into_iter()
                        .map(|b| taxii_core::ContentBindingEntity {
                            binding: b,
                            subtypes: Vec::new(),
                        })
                        .collect();

                    // Create a result set for poll fulfillment
                    let result_set_entity = taxii_core::ResultSetEntity {
                        id: generate_id(),
                        collection_id: collection.id.unwrap_or(0),
                        content_bindings: result_bindings,
                        timeframe,
                    };

                    let result_set = ctx
                        .persistence
                        .create_result_set(&result_set_entity)
                        .await?;

                    // Get service configuration for wait_time and can_push
                    let wait_time = ctx
                        .service
                        .get_property("wait_time")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(300); // Default 5 minutes

                    let can_push = ctx
                        .service
                        .get_property("can_push")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    // Build status details
                    let status_detail = HashMap::from([
                        (SD_ESTIMATED_WAIT.to_owned(), wait_time.to_string()),
                        (SD_RESULT_ID.to_owned(), result_set.id),
                        (SD_WILL_PUSH.to_owned(), can_push.to_string()),
                    ]);

                    let status = tm11::StatusMessage::new(
                        generate_id(),
                        in_response_to.to_string(),
                        ST_PENDING.to_string(),
                    )
                    .with_status_detail(status_detail);

                    Ok(PollResult::Status(status))
                }
                Err(e) => Err(e.into()),
            }
        } else {
            // COUNT_ONLY response - just count the blocks
            let count = ctx
                .persistence
                .get_content_blocks_count(
                    collection.id,
                    timeframe.0,
                    timeframe.1,
                    binding_entities.as_deref(),
                )
                .await?;

            let mut response = tm11::PollResponse::new(
                generate_id(),
                in_response_to.to_string(),
                collection_name.to_string(),
            );

            // Set subscription ID if provided
            if let Some(sub_id) = subscription_id {
                response.subscription_id = Some(sub_id.to_string());
            }

            // Set timeframe in response
            if let Some(start) = timeframe.0 {
                response.exclusive_begin_timestamp_label = Some(start.to_rfc3339());
            }
            if let Some(end) = timeframe.1 {
                response.inclusive_end_timestamp_label = Some(end.to_rfc3339());
            }

            response.record_count = Some(tm11::RecordCount {
                partial_count: false,
                record_count: count,
            });

            Ok(PollResult::Response(response))
        }
    }

    /// Handle a TAXII 1.1 Poll Request.
    pub async fn handle_11(
        &self,
        ctx: &HandlerContext,
        _headers: &TaxiiHeaders,
        message: &tm11::Taxii11Message,
    ) -> Taxii1xResult<tm11::Taxii11Message> {
        let request = match message {
            tm11::Taxii11Message::PollRequest(req) => req,
            _ => return Err(Taxii1xError::failure("Expected Poll Request message", None)),
        };

        let collection_name = &request.collection_name;

        // Check if subscription is required for this service
        let subscription_required = ctx
            .service
            .get_property("subscription_required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if subscription_required && request.subscription_id.is_none() {
            return Err(Taxii1xError::StatusMessage {
                message: "Subscription id is required".to_string(),
                in_response_to: Some(request.message_id.clone()),
                status_type: StatusType::Denied,
                status_detail: None,
            });
        }

        // Log warning if both subscription_id and poll_parameters present
        if request.subscription_id.is_some() && request.poll_parameters.is_some() {
            tracing::warn!(
                service_id = %ctx.service.id,
                subscription_id = ?request.subscription_id,
                "Both subscription ID and Poll Parameters present"
            );
        }

        // Get collection
        let collection = ctx
            .persistence
            .get_collection(collection_name, Some(&ctx.service.id))
            .await?
            .ok_or_else(|| {
                Taxii1xError::status_with_detail(
                    StatusType::NotFound,
                    "Requested collection was not found",
                    Some(request.message_id.clone()),
                    collection_name,
                )
            })?;

        if !collection.available {
            return Err(Taxii1xError::failure(
                "The collection is not available",
                Some(request.message_id.clone()),
            ));
        }

        // Parse timeframe
        let start = request
            .exclusive_begin_timestamp_label
            .as_deref()
            .and_then(parse_timestamp);
        let end = request
            .inclusive_end_timestamp_label
            .as_deref()
            .and_then(parse_timestamp);

        // Validate timeframe
        if let (Some(s), Some(e)) = (start, end) {
            if s > e {
                return Err(Taxii1xError::failure(
                    "Exclusive begin timestamp label is later than inclusive end timestamp label",
                    Some(request.message_id.clone()),
                ));
            }
        }

        // Resolve content bindings and response parameters
        let resolved = resolve_poll_bindings_11(ctx, request, &collection).await?;
        let return_content = resolved.response_type != RT_COUNT_ONLY;

        // Convert bindings to string format for prepare_poll_response
        let binding_strings: Option<Vec<String>> = if resolved.content_bindings.is_empty() {
            None
        } else {
            Some(
                resolved
                    .content_bindings
                    .iter()
                    .map(|cb| cb.binding.clone())
                    .collect(),
            )
        };

        let result = Self::prepare_poll_response(
            ctx,
            PollParams {
                collection_name,
                in_response_to: &request.message_id,
                timeframe: (start, end),
                content_bindings: binding_strings,
                return_content,
                subscription_id: request.subscription_id.as_deref(),
                allow_async: resolved.allow_async,
                result_part: 1,
                result_id: None,
            },
        )
        .await?;

        match result {
            PollResult::Response(response) => Ok(tm11::Taxii11Message::PollResponse(response)),
            PollResult::Status(status) => Ok(tm11::Taxii11Message::StatusMessage(status)),
        }
    }
}

/// TAXII 1.0 Poll Request Handler.
pub struct PollRequest10Handler;

impl PollRequest10Handler {
    /// Handle a TAXII 1.0 Poll Request.
    pub async fn handle_10(
        &self,
        ctx: &HandlerContext,
        _headers: &TaxiiHeaders,
        message: &tm10::Taxii10Message,
    ) -> Taxii1xResult<tm10::Taxii10Message> {
        let request = match message {
            tm10::Taxii10Message::PollRequest(req) => req,
            _ => return Err(Taxii1xError::failure("Expected Poll Request message", None)),
        };

        let feed_name = &request.feed_name;

        // Get collection
        let collection = ctx
            .persistence
            .get_collection(feed_name, Some(&ctx.service.id))
            .await?
            .ok_or_else(|| {
                Taxii1xError::status_with_detail(
                    StatusType::NotFound,
                    "Requested feed was not found",
                    Some(request.message_id.clone()),
                    feed_name,
                )
            })?;

        // TAXII 1.0 only supports DATA_FEED
        if collection.collection_type != CT_DATA_FEED {
            return Err(Taxii1xError::StatusMessage {
                message: "The Named Data Collection is not a Data Feed, it is a Data Set. Only Data Feeds can be polled in TAXII 1.0".to_string(),
                in_response_to: Some(request.message_id.clone()),
                status_type: StatusType::NotFound,
                status_detail: Some(feed_name.clone()),
            });
        }

        if !collection.available {
            return Err(Taxii1xError::failure(
                "The feed is not available",
                Some(request.message_id.clone()),
            ));
        }

        // Resolve content bindings
        let content_bindings = resolve_poll_bindings_10(ctx, request, &collection).await?;

        // Parse timeframe
        let start = request
            .exclusive_begin_timestamp_label
            .as_deref()
            .and_then(parse_timestamp);
        let end = request
            .inclusive_end_timestamp_label
            .as_deref()
            .and_then(parse_timestamp);
        let end_response = end.unwrap_or_else(Utc::now);

        // Build response
        let mut response = tm10::PollResponse::new(generate_id(), &request.message_id, feed_name);
        response.subscription_id = request.subscription_id.clone();
        if let Some(s) = start {
            response.inclusive_begin_timestamp_label = Some(s.to_rfc3339());
        }
        response.inclusive_end_timestamp_label = Some(end_response.to_rfc3339());

        // Convert bindings for query
        let binding_entities: Option<&[ContentBindingEntity]> = if content_bindings.is_empty() {
            None
        } else {
            Some(&content_bindings)
        };

        // Get content blocks
        let blocks = ctx
            .persistence
            .get_content_blocks(collection.id, start, end, binding_entities, 0, None)
            .await?;

        response.content_blocks = blocks
            .into_iter()
            .map(|block| tm10::ContentBlock {
                content_binding: block
                    .content_binding
                    .map(|cb| cb.binding)
                    .unwrap_or_default(),
                content: String::from_utf8_lossy(&block.content).into_owned(),
                timestamp_label: Some(block.timestamp_label.to_rfc3339()),
                padding: None,
            })
            .collect();

        Ok(tm10::Taxii10Message::PollResponse(response))
    }
}
