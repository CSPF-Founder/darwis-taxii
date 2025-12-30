//! Core types and configuration for TAXII.

pub mod config;
pub mod entities;
pub mod error;
pub mod signals;

pub use config::ServerConfig;
pub use entities::{Account, PermissionValue};
pub use error::TaxiiError;
pub use signals::{
    ContentBlockCreatedEvent, HookRegistry, InboxMessageCreatedEvent, SharedHookRegistry,
    SignalEvent, SubscriptionCreatedEvent,
};

// Re-export TAXII 1.x entities
pub use entities::taxii1::{
    CollectionEntity, ContentBindingEntity, ContentBlockEntity, InboxMessageEntity,
    PollRequestParametersEntity, ResultSetEntity, ServiceEntity, SubscriptionEntity,
    SubscriptionParameters, collection_type, response_type, subscription_status,
};

// Re-export TAXII 2.x entities
pub use entities::taxii2::{
    ApiRoot, Collection, DATETIME_FORMAT, Job, JobDetail, JobDetails, ManifestRecord, STIXObject,
    VersionRecord, taxii2_datetimeformat,
};
