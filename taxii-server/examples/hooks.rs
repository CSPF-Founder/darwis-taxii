//! Example: Using signal hooks to react to TAXII events.
//!
//! This example shows how to subscribe to TAXII server events using the
//! signal/hook system.
//!
//! # Signal Types
//!
//! Three event types are emitted by the TAXII 1.x handlers:
//!
//! - `ContentBlockCreated` - When a content block is added via inbox
//! - `InboxMessageCreated` - When an inbox message is received
//! - `SubscriptionCreated` - When a new subscription is created
//!
//! # Use Cases
//!
//! - Logging and monitoring
//! - Integration with external systems (webhooks, message queues)
//! - Analytics and metrics collection
//! - Data enrichment or validation after object creation

use taxii_server::{
    ContentBlockCreatedEvent, InboxMessageCreatedEvent, SignalEvent, SubscriptionCreatedEvent,
};
use tokio::sync::broadcast;

/// Example hook handler that logs all events.
#[allow(dead_code)] // This is example code showing the pattern
async fn log_events(mut receiver: broadcast::Receiver<SignalEvent>) {
    println!("Hook subscriber started, waiting for events...");

    while let Ok(event) = receiver.recv().await {
        match event {
            SignalEvent::ContentBlockCreated(ContentBlockCreatedEvent {
                content_block,
                collection_ids,
                service_id,
            }) => {
                println!(
                    "[HOOK] Content block created: id={:?}, collections={:?}, service={:?}",
                    content_block.id, collection_ids, service_id
                );
            }

            SignalEvent::InboxMessageCreated(InboxMessageCreatedEvent {
                inbox_message,
                service_id,
            }) => {
                println!(
                    "[HOOK] Inbox message received: id={:?}, message_id={}, service={:?}",
                    inbox_message.id, inbox_message.message_id, service_id
                );
            }

            SignalEvent::SubscriptionCreated(SubscriptionCreatedEvent {
                subscription,
                collection_name,
            }) => {
                println!(
                    "[HOOK] Subscription created: id={:?}, collection={}",
                    subscription.subscription_id, collection_name
                );
            }
        }
    }

    println!("Hook subscriber ended");
}

/// Example showing how to integrate hooks with the server.
///
/// This is a conceptual example - in production you would integrate this
/// with your actual server initialization code.
fn main() {
    println!("Signal Hooks Example");
    println!("====================");
    println!();
    println!("To use hooks in your server, do the following:");
    println!();
    println!("1. Use create_router_with_hooks instead of create_router:");
    println!();
    println!("   let RouterWithHooks {{ router, hooks }} = create_router_with_hooks(");
    println!("       taxii1_persistence,");
    println!("       taxii2_persistence,");
    println!("       auth,");
    println!("       &config,");
    println!("   );");
    println!();
    println!("2. Subscribe to events and spawn a handler task:");
    println!();
    println!("   let receiver = hooks.subscribe();");
    println!("   tokio::spawn(async move {{");
    println!("       log_events(receiver).await;");
    println!("   }});");
    println!();
    println!("3. Events will be emitted when:");
    println!("   - Content blocks are added via TAXII 1.x inbox");
    println!("   - Inbox messages are received");
    println!("   - Subscriptions are created");
    println!();
    println!("See the log_events function in this example for handling code.");
}

// The following would be used in actual integration:
//
// ```rust
// use taxii_server::{create_router_with_hooks, RouterWithHooks, SignalEvent};
//
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     // ... setup persistence, auth, config ...
//
//     let RouterWithHooks { router, hooks } = create_router_with_hooks(
//         taxii1_persistence,
//         taxii2_persistence,
//         auth,
//         &config,
//     );
//
//     // Subscribe and spawn event handler
//     let receiver = hooks.subscribe();
//     tokio::spawn(log_events(receiver));
//
//     // Start server
//     let listener = tokio::net::TcpListener::bind("0.0.0.0:9000").await?;
//     axum::serve(listener, router).await?;
//
//     Ok(())
// }
// ```
