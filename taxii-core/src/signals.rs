//! Signal hooks for TAXII events.
//!
//! This module provides an event system for TAXII operations,
//! allowing external code to react to specific events like
//! content block creation, inbox message arrival, and subscription creation.
//!
//! # Example
//!
//! ```ignore
//! use taxii_core::signals::{HookRegistry, SignalEvent};
//!
//! let hooks = HookRegistry::new();
//!
//! // Register a hook
//! hooks.on_content_block_created(|event| {
//!     println!("Content block created: {:?}", event);
//! });
//!
//! // Emit an event (called internally by handlers)
//! hooks.emit_content_block_created(event_data);
//! ```

use std::sync::Arc;
use tokio::sync::broadcast;

use crate::{ContentBlockEntity, InboxMessageEntity, SubscriptionEntity};

/// Channel capacity for signal broadcasts.
const CHANNEL_CAPACITY: usize = 100;

/// Events that can be emitted by the TAXII server.
#[derive(Debug, Clone)]
pub enum SignalEvent {
    /// A content block was created via inbox.
    ContentBlockCreated(ContentBlockCreatedEvent),

    /// An inbox message was received.
    InboxMessageCreated(InboxMessageCreatedEvent),

    /// A subscription was created.
    SubscriptionCreated(SubscriptionCreatedEvent),
}

/// Event data for content block creation.
#[derive(Debug, Clone)]
pub struct ContentBlockCreatedEvent {
    /// The created content block.
    pub content_block: ContentBlockEntity,

    /// Collection IDs the block was added to.
    pub collection_ids: Vec<i32>,

    /// Service ID that received the block.
    pub service_id: Option<String>,
}

/// Event data for inbox message creation.
#[derive(Debug, Clone)]
pub struct InboxMessageCreatedEvent {
    /// The created inbox message.
    pub inbox_message: InboxMessageEntity,

    /// Service ID that received the message.
    pub service_id: Option<String>,
}

/// Event data for subscription creation.
#[derive(Debug, Clone)]
pub struct SubscriptionCreatedEvent {
    /// The created subscription.
    pub subscription: SubscriptionEntity,

    /// Collection name for the subscription.
    pub collection_name: String,
}

/// Registry for signal hooks.
///
/// Uses tokio broadcast channels for async event dispatch.
/// Multiple receivers can subscribe to each signal type.
#[derive(Clone)]
pub struct HookRegistry {
    /// Sender for all events.
    sender: broadcast::Sender<SignalEvent>,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HookRegistry {
    /// Create a new hook registry.
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self { sender }
    }

    /// Get a receiver for all events.
    ///
    /// The receiver will receive all events emitted after subscription.
    /// Multiple receivers can be created to handle events independently.
    pub fn subscribe(&self) -> broadcast::Receiver<SignalEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active receivers.
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }

    // ========================================================================
    // Emit methods (called by handlers)
    // ========================================================================

    /// Emit a content block created event.
    pub fn emit_content_block_created(&self, event: ContentBlockCreatedEvent) {
        // Ignore send errors (no receivers)
        let _ = self.sender.send(SignalEvent::ContentBlockCreated(event));
    }

    /// Emit an inbox message created event.
    pub fn emit_inbox_message_created(&self, event: InboxMessageCreatedEvent) {
        let _ = self.sender.send(SignalEvent::InboxMessageCreated(event));
    }

    /// Emit a subscription created event.
    pub fn emit_subscription_created(&self, event: SubscriptionCreatedEvent) {
        let _ = self.sender.send(SignalEvent::SubscriptionCreated(event));
    }
}

/// Wrapper for shared hook registry.
pub type SharedHookRegistry = Arc<HookRegistry>;

/// Create a new shared hook registry.
pub fn create_hook_registry() -> SharedHookRegistry {
    Arc::new(HookRegistry::new())
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "tests are allowed to use panic for assertions"
)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_content_block_created_signal() {
        let registry = HookRegistry::new();
        let mut receiver = registry.subscribe();

        let event = ContentBlockCreatedEvent {
            content_block: ContentBlockEntity {
                id: Some(1),
                content: b"test content".to_vec(),
                timestamp_label: Utc::now(),
                content_binding: None,
                message: None,
                inbox_message_id: None,
            },
            collection_ids: vec![1, 2],
            service_id: Some("inbox-1".to_string()),
        };

        registry.emit_content_block_created(event.clone());

        let received = receiver.recv().await;
        assert!(received.is_ok());

        if let Ok(SignalEvent::ContentBlockCreated(e)) = received {
            assert_eq!(e.collection_ids, vec![1, 2]);
            assert_eq!(e.service_id, Some("inbox-1".to_string()));
        } else {
            panic!("Expected ContentBlockCreated event");
        }
    }

    #[tokio::test]
    async fn test_multiple_receivers() {
        let registry = HookRegistry::new();
        let mut receiver1 = registry.subscribe();
        let mut receiver2 = registry.subscribe();

        assert_eq!(registry.receiver_count(), 2);

        let event = ContentBlockCreatedEvent {
            content_block: ContentBlockEntity {
                id: Some(1),
                content: b"test".to_vec(),
                timestamp_label: Utc::now(),
                content_binding: None,
                message: None,
                inbox_message_id: None,
            },
            collection_ids: vec![1],
            service_id: None,
        };

        registry.emit_content_block_created(event);

        // Both receivers should get the event
        assert!(receiver1.recv().await.is_ok());
        assert!(receiver2.recv().await.is_ok());
    }
}
