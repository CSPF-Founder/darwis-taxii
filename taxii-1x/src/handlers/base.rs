//! Base handler trait and utilities.
//!
//! This module provides the foundational types for TAXII 1.x message handling:
//!
//! - [`TaxiiHeaders`] - TAXII-specific HTTP headers required by the protocol
//! - [`HandlerContext`] - Request context passed to all message handlers
//! - [`ServiceInfo`] - Configuration and metadata for a TAXII service endpoint
//!
//! # TAXII 1.x Protocol Overview
//!
//! TAXII (Trusted Automated eXchange of Intelligence Information) 1.x uses
//! XML messages over HTTP/HTTPS with custom headers to negotiate message
//! format and protocol version. Each request includes:
//!
//! - `X-TAXII-Content-Type`: The message binding (e.g., `urn:taxii.mitre.org:message:xml:1.1`)
//! - `X-TAXII-Services`: The services binding (e.g., `urn:taxii.mitre.org:services:1.1`)
//! - `X-TAXII-Accept`: (Optional) Preferred response format

use std::sync::Arc;

use crate::constants::{
    VID_TAXII_SERVICES_10, VID_TAXII_SERVICES_11, VID_TAXII_XML_10, VID_TAXII_XML_11,
};
use crate::error::{Taxii1xError, Taxii1xResult};
use crate::messages::common::generate_message_id;

/// TAXII-specific HTTP headers extracted from a request.
///
/// TAXII 1.x requires custom HTTP headers to negotiate the message format
/// and protocol version between client and server. These headers are distinct
/// from standard HTTP `Content-Type` and `Accept` headers.
///
/// # Protocol Bindings
///
/// The `services` header indicates which TAXII services specification is in use:
/// - TAXII 1.0: `urn:taxii.mitre.org:services:1.0`
/// - TAXII 1.1: `urn:taxii.mitre.org:services:1.1`
///
/// The `content_type` header indicates the XML message binding:
/// - TAXII 1.0: `urn:taxii.mitre.org:message:xml:1.0`
/// - TAXII 1.1: `urn:taxii.mitre.org:message:xml:1.1`
///
/// # Example
///
/// ```ignore
/// let headers = TaxiiHeaders {
///     content_type: "urn:taxii.mitre.org:message:xml:1.1".to_string(),
///     services: "urn:taxii.mitre.org:services:1.1".to_string(),
///     accept: Some("urn:taxii.mitre.org:message:xml:1.1".to_string()),
/// };
/// headers.validate_11(Some("msg-123"))?;
/// ```
#[derive(Debug, Clone)]
pub struct TaxiiHeaders {
    /// The `X-TAXII-Content-Type` header value indicating the message binding.
    ///
    /// This specifies the XML schema version of the TAXII message being sent.
    pub content_type: String,

    /// The `X-TAXII-Services` header value indicating the services binding.
    ///
    /// This specifies which version of the TAXII services specification is in use.
    pub services: String,

    /// The optional `X-TAXII-Accept` header value for content negotiation.
    ///
    /// When present, indicates the client's preferred response format.
    /// If absent, the server typically responds using the same binding as the request.
    pub accept: Option<String>,
}

impl TaxiiHeaders {
    /// Validate headers for TAXII 1.0.
    pub fn validate_10(&self, in_response_to: Option<&str>) -> Taxii1xResult<()> {
        if self.services != VID_TAXII_SERVICES_10 {
            return Err(Taxii1xError::failure(
                format!(
                    "The specified value of X-TAXII-Services is not supported: {}",
                    self.services
                ),
                in_response_to.map(String::from),
            ));
        }

        if self.content_type != VID_TAXII_XML_10 {
            return Err(Taxii1xError::failure(
                "The specified value of X-TAXII-Content-Type is not supported",
                in_response_to.map(String::from),
            ));
        }

        if let Some(ref accept) = self.accept {
            if accept != VID_TAXII_XML_10 {
                return Err(Taxii1xError::failure(
                    "The specified value of X-TAXII-Accept is not supported",
                    in_response_to.map(String::from),
                ));
            }
        }

        Ok(())
    }

    /// Validate headers for TAXII 1.1.
    pub fn validate_11(&self, in_response_to: Option<&str>) -> Taxii1xResult<()> {
        if self.services != VID_TAXII_SERVICES_11 {
            return Err(Taxii1xError::failure(
                format!(
                    "The specified value of X-TAXII-Services is not supported: {}",
                    self.services
                ),
                in_response_to.map(String::from),
            ));
        }

        if self.content_type != VID_TAXII_XML_11 {
            return Err(Taxii1xError::failure(
                "The specified value of X-TAXII-Content-Type is not supported",
                in_response_to.map(String::from),
            ));
        }

        if let Some(ref accept) = self.accept {
            if accept != VID_TAXII_XML_11 {
                return Err(Taxii1xError::failure(
                    "The specified value of X-TAXII-Accept is not supported",
                    in_response_to.map(String::from),
                ));
            }
        }

        Ok(())
    }
}

/// Execution context for TAXII 1.x message handlers.
///
/// This struct provides handlers with all the resources needed to process
/// TAXII messages, including database access, authentication state, and
/// service configuration.
///
/// # Components
///
/// - **Account**: The authenticated user making the request (if any). Used for
///   authorization checks on collections and services.
///
/// - **Persistence**: Database access layer for collections, content blocks,
///   subscriptions, and other TAXII entities.
///
/// - **Service**: Configuration for the specific TAXII service endpoint handling
///   this request (Discovery, Poll, Inbox, etc.).
///
/// - **Hooks**: Optional event hooks for integrations (e.g., notifying external
///   systems when content is received).
///
/// # Thread Safety
///
/// The context uses `Arc` for shared ownership of the persistence layer,
/// allowing concurrent request handling.
#[derive(Clone)]
pub struct HandlerContext {
    /// The authenticated account making this request, if any.
    ///
    /// When `None`, the request is anonymous. Handlers should check this
    /// for authorization decisions on collection access.
    pub account: Option<taxii_core::Account>,

    /// Database persistence API for TAXII entities.
    ///
    /// Provides access to collections, content blocks, subscriptions,
    /// inbox messages, and result sets.
    pub persistence: Arc<taxii_db::DbTaxii1Repository>,

    /// Configuration and metadata for the TAXII service handling this request.
    ///
    /// Different service types (Discovery, Poll, Inbox, Collection Management)
    /// have different configurations and capabilities.
    pub service: ServiceInfo,

    /// Optional hook registry for emitting events.
    ///
    /// Used to notify external systems when content blocks are created,
    /// inbox messages are received, etc.
    pub hooks: Option<taxii_core::SharedHookRegistry>,
}

impl std::fmt::Debug for HandlerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandlerContext")
            .field("account", &self.account)
            .field("service", &self.service)
            .finish_non_exhaustive()
    }
}

/// Configuration and metadata for a TAXII 1.x service endpoint.
///
/// A TAXII server exposes multiple services, each providing specific functionality:
///
/// | Service Type | Description |
/// |-------------|-------------|
/// | `DISCOVERY` | Lists available services and their capabilities |
/// | `POLL` | Allows clients to request (pull) content from collections |
/// | `INBOX` | Accepts content pushed by clients |
/// | `COLLECTION_MANAGEMENT` | Manages subscriptions to collections |
/// | `FEED_MANAGEMENT` | (TAXII 1.0) Manages subscriptions to data feeds |
///
/// # Protocol and Message Bindings
///
/// Each service advertises which protocols it supports:
/// - `urn:taxii.mitre.org:protocol:http:1.0` - HTTP transport
/// - `urn:taxii.mitre.org:protocol:https:1.0` - HTTPS transport
///
/// And which message formats:
/// - `urn:taxii.mitre.org:message:xml:1.0` - TAXII 1.0 XML
/// - `urn:taxii.mitre.org:message:xml:1.1` - TAXII 1.1 XML
///
/// # Custom Properties
///
/// Service-specific configuration is stored in `properties` as JSON. Common
/// properties include:
/// - `max_result_size`: Maximum content blocks per poll response
/// - `subscription_required`: Whether polling requires a subscription
/// - `destination_collection_required`: Whether inbox requires explicit destinations
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Unique identifier for this service instance.
    pub id: String,

    /// The type of TAXII service (e.g., "DISCOVERY", "POLL", "INBOX").
    ///
    /// Determines which message types this service can handle.
    pub service_type: String,

    /// The URL path where this service is accessible.
    ///
    /// Relative to the server root, e.g., `/services/poll/`.
    pub address: String,

    /// Human-readable description of this service.
    pub description: Option<String>,

    /// Protocol bindings supported by this service.
    ///
    /// Typically HTTP and/or HTTPS transport URNs.
    pub protocol_bindings: Vec<String>,

    /// Message bindings supported by this service.
    ///
    /// The XML schema versions this service can process.
    pub message_bindings: Vec<String>,

    /// Whether this service is currently accepting requests.
    ///
    /// Unavailable services should return appropriate status messages.
    pub available: bool,

    /// Whether authentication is required to use this service.
    pub authentication_required: bool,

    /// Additional service-specific configuration as JSON.
    ///
    /// Allows flexible per-service settings without schema changes.
    /// Access values using [`ServiceInfo::get_property`].
    pub properties: serde_json::Value,
}

impl ServiceInfo {
    /// Get a property value from the service configuration.
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }
}

// Note: MessageHandler trait removed - using Handler enum dispatch instead
// See handlers/mod.rs for the Handler enum implementation

/// Generate a unique message ID.
pub fn generate_id() -> String {
    generate_message_id()
}
