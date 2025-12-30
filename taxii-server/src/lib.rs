//! HTTP server for DARWIS TAXII.

pub mod auth_middleware;
pub mod config;
pub mod error;
pub mod router;
pub mod taxii1x_routes;

pub use auth_middleware::AuthLayer;
pub use config::{ConfigError, ServerConfig};
pub use error::{ServerError, ServerResult};
pub use router::{RouterWithHooks, create_router, create_router_with_hooks};
pub use taxii1x_routes::Taxii1xState;

// Re-export signal types for hook subscribers
pub use taxii_core::{
    ContentBlockCreatedEvent, HookRegistry, InboxMessageCreatedEvent, SharedHookRegistry,
    SignalEvent, SubscriptionCreatedEvent,
};
