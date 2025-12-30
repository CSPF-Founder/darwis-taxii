//! Poll fulfillment request handlers.
//!
//! Note: Poll fulfillment is only available in TAXII 1.1.

use crate::constants::StatusType;
use crate::error::{Taxii1xError, Taxii1xResult};
use crate::messages::tm11;
use taxii_db::Taxii1Repository;

use super::base::{HandlerContext, TaxiiHeaders, generate_id};

/// Default max result size
const DEFAULT_MAX_RESULT_SIZE: i64 = 1_000_000;

/// TAXII 1.1 Poll Fulfillment Request Handler.
pub struct PollFulfillmentRequest11Handler;

impl PollFulfillmentRequest11Handler {
    /// Handle a TAXII 1.1 Poll Fulfillment Request.
    pub async fn handle_11(
        &self,
        ctx: &HandlerContext,
        _headers: &TaxiiHeaders,
        message: &tm11::Taxii11Message,
    ) -> Taxii1xResult<tm11::Taxii11Message> {
        let request = match message {
            tm11::Taxii11Message::PollFulfillmentRequest(req) => req,
            _ => {
                return Err(Taxii1xError::failure(
                    "Expected Poll Fulfillment Request message",
                    None,
                ));
            }
        };

        let collection_name = &request.collection_name;
        let result_id = &request.result_id;
        let result_part = request.result_part_number.unwrap_or(1);

        // Get result set
        let result_set = ctx.persistence.get_result_set(result_id).await?;

        let result_set = result_set.ok_or_else(|| {
            Taxii1xError::status_with_detail(
                StatusType::NotFound,
                "Requested result set was not found",
                Some(request.message_id.clone()),
                result_id,
            )
        })?;

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

        // Verify result set belongs to this collection
        if let Some(collection_id) = collection.id {
            if result_set.collection_id != collection_id {
                return Err(Taxii1xError::StatusMessage {
                    message: "Result set does not belong to this collection".to_string(),
                    in_response_to: Some(request.message_id.clone()),
                    status_type: StatusType::NotFound,
                    status_detail: Some(result_id.clone()),
                });
            }
        }

        let mut response =
            tm11::PollResponse::new(generate_id(), &request.message_id, collection_name);

        response.result_id = Some(result_id.clone());
        response.result_part_number = Some(result_part);

        // Extract fields from result set (consuming it)
        let (timeframe, content_bindings) = (result_set.timeframe, result_set.content_bindings);
        let (start, end) = timeframe;

        if let Some(s) = start {
            response.exclusive_begin_timestamp_label = Some(s.to_rfc3339());
        }
        if let Some(e) = end {
            response.inclusive_end_timestamp_label = Some(e.to_rfc3339());
        }

        // Get content bindings from result set as entities
        let binding_entities: Option<Vec<taxii_core::ContentBindingEntity>> =
            if content_bindings.is_empty() {
                None
            } else {
                Some(content_bindings)
            };

        // Get max_result_size from service properties
        let max_result_size = ctx
            .service
            .get_property("max_result_size")
            .and_then(|v| v.as_i64())
            .unwrap_or(DEFAULT_MAX_RESULT_SIZE);

        // Calculate offset based on result_part
        // offset = (part_number - 1) * max_result_size
        let offset = ((result_part - 1) as i64) * max_result_size;

        // Get total count for pagination
        let total_count = ctx
            .persistence
            .get_content_blocks_count(collection.id, start, end, binding_entities.as_deref())
            .await?;

        // Get content blocks with proper pagination
        let blocks = ctx
            .persistence
            .get_content_blocks(
                collection.id,
                start,
                end,
                binding_entities.as_deref(),
                offset,
                Some(max_result_size),
            )
            .await?;

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

        // Check if more parts available
        let has_more = (total_count as f64 / max_result_size as f64) > result_part as f64;
        response.more = Some(has_more);

        response.record_count = Some(tm11::RecordCount {
            partial_count: has_more,
            record_count: total_count,
        });

        Ok(tm11::Taxii11Message::PollResponse(response))
    }
}
