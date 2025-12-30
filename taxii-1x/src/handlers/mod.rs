//! TAXII 1.x message handlers.

pub mod base;
pub mod collection_info;
pub mod discovery;
pub mod inbox;
pub mod poll;
pub mod poll_fulfillment;
pub mod subscription;

pub use base::{HandlerContext, ServiceInfo, TaxiiHeaders, generate_id};

use std::collections::HashMap;

use crate::constants::{
    MSG_COLLECTION_INFORMATION_REQUEST, MSG_DISCOVERY_REQUEST, MSG_FEED_INFORMATION_REQUEST,
    MSG_INBOX_MESSAGE, MSG_MANAGE_COLLECTION_SUBSCRIPTION_REQUEST,
    MSG_MANAGE_FEED_SUBSCRIPTION_REQUEST, MSG_POLL_FULFILLMENT_REQUEST, MSG_POLL_REQUEST,
    VID_TAXII_XML_10, VID_TAXII_XML_11,
};
use crate::error::Taxii1xResult;
use crate::messages::{tm10, tm11};

use collection_info::{CollectionInformationRequest11Handler, FeedInformationRequest10Handler};
use discovery::{DiscoveryRequest10Handler, DiscoveryRequest11Handler};
use inbox::{InboxMessage10Handler, InboxMessage11Handler};
use poll::{PollRequest10Handler, PollRequest11Handler};
use poll_fulfillment::PollFulfillmentRequest11Handler;
use subscription::{SubscriptionRequest10Handler, SubscriptionRequest11Handler};

/// TAXII 1.x message handler.
///
/// This enum contains all supported TAXII message handlers for both 1.0 and 1.1.
/// Using an enum instead of trait objects enables native async/await support
/// without requiring the `async_trait` crate.
#[derive(Debug, Clone, Copy)]
pub enum Handler {
    /// Discovery request handler (1.0 and 1.1).
    Discovery,
    /// Feed/Collection information request handler.
    CollectionInfo,
    /// Poll request handler.
    Poll,
    /// Poll fulfillment request handler (1.1 only).
    PollFulfillment,
    /// Inbox message handler.
    Inbox,
    /// Subscription management handler.
    Subscription,
}

impl Handler {
    /// Handle a TAXII 1.0 message.
    pub async fn handle_10(
        &self,
        ctx: &HandlerContext,
        headers: &TaxiiHeaders,
        message: &tm10::Taxii10Message,
    ) -> Taxii1xResult<tm10::Taxii10Message> {
        match self {
            Self::Discovery => {
                DiscoveryRequest10Handler
                    .handle_10(ctx, headers, message)
                    .await
            }
            Self::CollectionInfo => {
                FeedInformationRequest10Handler
                    .handle_10(ctx, headers, message)
                    .await
            }
            Self::Poll => PollRequest10Handler.handle_10(ctx, headers, message).await,
            Self::Inbox => InboxMessage10Handler.handle_10(ctx, headers, message).await,
            Self::Subscription => {
                SubscriptionRequest10Handler
                    .handle_10(ctx, headers, message)
                    .await
            }
            Self::PollFulfillment => Err(crate::error::Taxii1xError::failure(
                "Poll Fulfillment is not supported in TAXII 1.0",
                None,
            )),
        }
    }

    /// Handle a TAXII 1.1 message.
    pub async fn handle_11(
        &self,
        ctx: &HandlerContext,
        headers: &TaxiiHeaders,
        message: &tm11::Taxii11Message,
    ) -> Taxii1xResult<tm11::Taxii11Message> {
        match self {
            Self::Discovery => {
                DiscoveryRequest11Handler
                    .handle_11(ctx, headers, message)
                    .await
            }
            Self::CollectionInfo => {
                CollectionInformationRequest11Handler
                    .handle_11(ctx, headers, message)
                    .await
            }
            Self::Poll => PollRequest11Handler.handle_11(ctx, headers, message).await,
            Self::PollFulfillment => {
                PollFulfillmentRequest11Handler
                    .handle_11(ctx, headers, message)
                    .await
            }
            Self::Inbox => InboxMessage11Handler.handle_11(ctx, headers, message).await,
            Self::Subscription => {
                SubscriptionRequest11Handler
                    .handle_11(ctx, headers, message)
                    .await
            }
        }
    }
}

/// Handler registry for TAXII services.
///
/// Maps message types to their appropriate handlers for both TAXII 1.0 and 1.1.
pub struct HandlerRegistry {
    handlers_10: HashMap<&'static str, Handler>,
    handlers_11: HashMap<&'static str, Handler>,
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HandlerRegistry {
    /// Create a new handler registry with all default handlers.
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            handlers_10: HashMap::new(),
            handlers_11: HashMap::new(),
        };

        // Register TAXII 1.0 handlers
        registry.register_10(MSG_DISCOVERY_REQUEST, Handler::Discovery);
        registry.register_10(MSG_FEED_INFORMATION_REQUEST, Handler::CollectionInfo);
        registry.register_10(MSG_POLL_REQUEST, Handler::Poll);
        registry.register_10(MSG_INBOX_MESSAGE, Handler::Inbox);
        registry.register_10(MSG_MANAGE_FEED_SUBSCRIPTION_REQUEST, Handler::Subscription);

        // Register TAXII 1.1 handlers
        registry.register_11(MSG_DISCOVERY_REQUEST, Handler::Discovery);
        registry.register_11(MSG_COLLECTION_INFORMATION_REQUEST, Handler::CollectionInfo);
        registry.register_11(MSG_POLL_REQUEST, Handler::Poll);
        registry.register_11(MSG_INBOX_MESSAGE, Handler::Inbox);
        registry.register_11(
            MSG_MANAGE_COLLECTION_SUBSCRIPTION_REQUEST,
            Handler::Subscription,
        );
        registry.register_11(MSG_POLL_FULFILLMENT_REQUEST, Handler::PollFulfillment);

        registry
    }

    /// Register a TAXII 1.0 handler.
    pub fn register_10(&mut self, message_type: &'static str, handler: Handler) {
        self.handlers_10.insert(message_type, handler);
    }

    /// Register a TAXII 1.1 handler.
    pub fn register_11(&mut self, message_type: &'static str, handler: Handler) {
        self.handlers_11.insert(message_type, handler);
    }

    /// Get a handler for the given version and message type.
    #[must_use]
    pub fn get(&self, version: &str, message_type: &str) -> Option<Handler> {
        if version == VID_TAXII_XML_10 {
            self.handlers_10.get(message_type).copied()
        } else if version == VID_TAXII_XML_11 {
            self.handlers_11.get(message_type).copied()
        } else {
            None
        }
    }
}
