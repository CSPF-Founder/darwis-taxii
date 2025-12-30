//! TAXII 1.x database models.
//!
//! Tables:
//! - services
//! - data_collections
//! - content_blocks
//! - inbox_messages
//! - result_sets
//! - subscriptions
//!
//! Junction tables:
//! - collection_to_content_block
//! - service_to_collection

pub mod collection;
pub mod content_block;
pub mod inbox_message;
pub mod result_set;
pub mod service;
pub mod subscription;

pub use collection::{DataCollection, UpdateDataCollection};
pub use content_block::{ContentBindingFilter, ContentBlock, ContentBlockFilter};
pub use inbox_message::{InboxMessage, NewInboxMessage};
pub use result_set::ResultSet;
pub use service::Service;
pub use subscription::{Subscription, status as subscription_status};
