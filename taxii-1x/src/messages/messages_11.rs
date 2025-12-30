//! TAXII 1.1 message types.

use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};

use super::common::NS_TAXII_11;
use crate::constants::{
    MSG_COLLECTION_INFORMATION_REQUEST, MSG_COLLECTION_INFORMATION_RESPONSE, MSG_DISCOVERY_REQUEST,
    MSG_DISCOVERY_RESPONSE, MSG_INBOX_MESSAGE, MSG_MANAGE_COLLECTION_SUBSCRIPTION_REQUEST,
    MSG_MANAGE_COLLECTION_SUBSCRIPTION_RESPONSE, MSG_POLL_FULFILLMENT_REQUEST, MSG_POLL_REQUEST,
    MSG_POLL_RESPONSE, MSG_STATUS_MESSAGE, ST_FAILURE, ST_SUCCESS,
};
use crate::error::{Taxii1xError, Taxii1xResult};

// Re-export common types
pub use super::common::{
    ContentBinding, ExtendedHeader, PushParameters, RecordCount, StatusDetail,
    SubscriptionInformation, SubscriptionParameters,
};

/// Wrapper enum for all TAXII 1.1 message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Taxii11Message {
    #[serde(rename = "Discovery_Request")]
    DiscoveryRequest(DiscoveryRequest),
    #[serde(rename = "Discovery_Response")]
    DiscoveryResponse(DiscoveryResponse),
    #[serde(rename = "Collection_Information_Request")]
    CollectionInformationRequest(CollectionInformationRequest),
    #[serde(rename = "Collection_Information_Response")]
    CollectionInformationResponse(CollectionInformationResponse),
    #[serde(rename = "Poll_Request")]
    PollRequest(PollRequest),
    #[serde(rename = "Poll_Response")]
    PollResponse(PollResponse),
    #[serde(rename = "Inbox_Message")]
    InboxMessage(InboxMessage),
    #[serde(rename = "Status_Message")]
    StatusMessage(StatusMessage),
    #[serde(rename = "Subscription_Management_Request")]
    ManageCollectionSubscriptionRequest(ManageCollectionSubscriptionRequest),
    #[serde(rename = "Subscription_Management_Response")]
    ManageCollectionSubscriptionResponse(ManageCollectionSubscriptionResponse),
    #[serde(rename = "Poll_Fulfillment")]
    PollFulfillmentRequest(PollFulfillmentRequest),
}

impl Taxii11Message {
    /// Get message ID.
    pub fn message_id(&self) -> &str {
        match self {
            Self::DiscoveryRequest(m) => &m.message_id,
            Self::DiscoveryResponse(m) => &m.message_id,
            Self::CollectionInformationRequest(m) => &m.message_id,
            Self::CollectionInformationResponse(m) => &m.message_id,
            Self::PollRequest(m) => &m.message_id,
            Self::PollResponse(m) => &m.message_id,
            Self::InboxMessage(m) => &m.message_id,
            Self::StatusMessage(m) => &m.message_id,
            Self::ManageCollectionSubscriptionRequest(m) => &m.message_id,
            Self::ManageCollectionSubscriptionResponse(m) => &m.message_id,
            Self::PollFulfillmentRequest(m) => &m.message_id,
        }
    }

    /// Get message type.
    pub fn message_type(&self) -> &str {
        match self {
            Self::DiscoveryRequest(_) => MSG_DISCOVERY_REQUEST,
            Self::DiscoveryResponse(_) => MSG_DISCOVERY_RESPONSE,
            Self::CollectionInformationRequest(_) => MSG_COLLECTION_INFORMATION_REQUEST,
            Self::CollectionInformationResponse(_) => MSG_COLLECTION_INFORMATION_RESPONSE,
            Self::PollRequest(_) => MSG_POLL_REQUEST,
            Self::PollResponse(_) => MSG_POLL_RESPONSE,
            Self::InboxMessage(_) => MSG_INBOX_MESSAGE,
            Self::StatusMessage(_) => MSG_STATUS_MESSAGE,
            Self::ManageCollectionSubscriptionRequest(_) => {
                MSG_MANAGE_COLLECTION_SUBSCRIPTION_REQUEST
            }
            Self::ManageCollectionSubscriptionResponse(_) => {
                MSG_MANAGE_COLLECTION_SUBSCRIPTION_RESPONSE
            }
            Self::PollFulfillmentRequest(_) => MSG_POLL_FULFILLMENT_REQUEST,
        }
    }

    /// Serialize to XML.
    pub fn to_xml(&self) -> Taxii1xResult<String> {
        to_string(self).map_err(Taxii1xError::xml_serialize)
    }
}

// ============================================================================
// Discovery Messages
// ============================================================================

/// TAXII 1.1 Discovery Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Discovery_Request")]
pub struct DiscoveryRequest {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_11")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,
}

impl DiscoveryRequest {
    /// Create a new discovery request.
    pub fn new(message_id: impl Into<String>) -> Self {
        Self {
            xmlns: NS_TAXII_11.to_string(),
            message_id: message_id.into(),
            extended_headers: None,
        }
    }
}

/// TAXII 1.1 Discovery Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Discovery_Response")]
pub struct DiscoveryResponse {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_11")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@in_response_to", skip_serializing_if = "Option::is_none")]
    pub in_response_to: Option<String>,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Service_Instance", default)]
    pub service_instances: Vec<ServiceInstance>,
}

impl DiscoveryResponse {
    /// Create a new discovery response.
    pub fn new(message_id: impl Into<String>, in_response_to: impl Into<String>) -> Self {
        Self {
            xmlns: NS_TAXII_11.to_string(),
            message_id: message_id.into(),
            in_response_to: Some(in_response_to.into()),
            extended_headers: None,
            service_instances: Vec::new(),
        }
    }
}

// ============================================================================
// Collection Information Messages
// ============================================================================

/// TAXII 1.1 Collection Information Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Collection_Information_Request")]
pub struct CollectionInformationRequest {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_11")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,
}

impl CollectionInformationRequest {
    /// Create a new collection information request.
    pub fn new(message_id: impl Into<String>) -> Self {
        Self {
            xmlns: NS_TAXII_11.to_string(),
            message_id: message_id.into(),
            extended_headers: None,
        }
    }
}

/// TAXII 1.1 Collection Information Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Collection_Information_Response")]
pub struct CollectionInformationResponse {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_11")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@in_response_to", skip_serializing_if = "Option::is_none")]
    pub in_response_to: Option<String>,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Collection", default)]
    pub collections: Vec<CollectionInformation>,
}

impl CollectionInformationResponse {
    /// Create a new collection information response.
    pub fn new(message_id: impl Into<String>, in_response_to: impl Into<String>) -> Self {
        Self {
            xmlns: NS_TAXII_11.to_string(),
            message_id: message_id.into(),
            in_response_to: Some(in_response_to.into()),
            extended_headers: None,
            collections: Vec::new(),
        }
    }
}

// ============================================================================
// Poll Messages
// ============================================================================

/// TAXII 1.1 Poll Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Poll_Request")]
pub struct PollRequest {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_11")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@collection_name")]
    pub collection_name: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(
        rename = "Exclusive_Begin_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub exclusive_begin_timestamp_label: Option<String>,

    #[serde(
        rename = "Inclusive_End_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub inclusive_end_timestamp_label: Option<String>,

    #[serde(rename = "Subscription_ID", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,

    #[serde(rename = "Poll_Parameters", skip_serializing_if = "Option::is_none")]
    pub poll_parameters: Option<PollParameters>,
}

/// Poll parameters for poll requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollParameters {
    #[serde(rename = "@allow_asynch", skip_serializing_if = "Option::is_none")]
    pub allow_asynch: Option<bool>,

    #[serde(rename = "Response_Type", skip_serializing_if = "Option::is_none")]
    pub response_type: Option<String>,

    #[serde(rename = "Content_Binding", default)]
    pub content_bindings: Vec<ContentBinding>,
    // Query not implemented yet
}

/// TAXII 1.1 Poll Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Poll_Response")]
pub struct PollResponse {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_11")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@in_response_to", skip_serializing_if = "Option::is_none")]
    pub in_response_to: Option<String>,

    #[serde(rename = "@collection_name")]
    pub collection_name: String,

    #[serde(rename = "@more", skip_serializing_if = "Option::is_none")]
    pub more: Option<bool>,

    #[serde(rename = "@result_id", skip_serializing_if = "Option::is_none")]
    pub result_id: Option<String>,

    #[serde(
        rename = "@result_part_number",
        skip_serializing_if = "Option::is_none"
    )]
    pub result_part_number: Option<i32>,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(
        rename = "Exclusive_Begin_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub exclusive_begin_timestamp_label: Option<String>,

    #[serde(
        rename = "Inclusive_End_Timestamp_Label",
        skip_serializing_if = "Option::is_none"
    )]
    pub inclusive_end_timestamp_label: Option<String>,

    #[serde(rename = "Subscription_ID", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,

    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(rename = "Record_Count", skip_serializing_if = "Option::is_none")]
    pub record_count: Option<RecordCount>,

    #[serde(rename = "Content_Block", default)]
    pub content_blocks: Vec<ContentBlock>,
}

impl PollResponse {
    /// Create a new poll response.
    pub fn new(
        message_id: impl Into<String>,
        in_response_to: impl Into<String>,
        collection_name: impl Into<String>,
    ) -> Self {
        Self {
            xmlns: NS_TAXII_11.to_string(),
            message_id: message_id.into(),
            in_response_to: Some(in_response_to.into()),
            collection_name: collection_name.into(),
            more: None,
            result_id: None,
            result_part_number: None,
            extended_headers: None,
            exclusive_begin_timestamp_label: None,
            inclusive_end_timestamp_label: None,
            subscription_id: None,
            message: None,
            record_count: None,
            content_blocks: Vec::new(),
        }
    }
}

/// Poll Fulfillment Request (TAXII 1.1 only).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Poll_Fulfillment")]
pub struct PollFulfillmentRequest {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_11")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@collection_name")]
    pub collection_name: String,

    #[serde(rename = "@result_id")]
    pub result_id: String,

    #[serde(
        rename = "@result_part_number",
        skip_serializing_if = "Option::is_none"
    )]
    pub result_part_number: Option<i32>,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,
}

// ============================================================================
// Inbox Messages
// ============================================================================

/// TAXII 1.1 Inbox Message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Inbox_Message")]
pub struct InboxMessage {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_11")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@result_id", skip_serializing_if = "Option::is_none")]
    pub result_id: Option<String>,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Destination_Collection_Name", default)]
    pub destination_collection_names: Vec<String>,

    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(
        rename = "Source_Subscription",
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_information: Option<SubscriptionInformation>,

    #[serde(rename = "Record_Count", skip_serializing_if = "Option::is_none")]
    pub record_count: Option<RecordCount>,

    #[serde(rename = "Content_Block", default)]
    pub content_blocks: Vec<ContentBlock>,
}

// ============================================================================
// Status Messages
// ============================================================================

/// TAXII 1.1 Status Message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Status_Message")]
pub struct StatusMessage {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_11")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@in_response_to", skip_serializing_if = "Option::is_none")]
    pub in_response_to: Option<String>,

    #[serde(rename = "@status_type")]
    pub status_type: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(rename = "Status_Detail", default)]
    pub status_details: Vec<StatusDetail>,
}

impl StatusMessage {
    /// Create a new status message.
    pub fn new(
        message_id: impl Into<String>,
        in_response_to: impl Into<String>,
        status_type: impl Into<String>,
    ) -> Self {
        Self {
            xmlns: NS_TAXII_11.to_string(),
            message_id: message_id.into(),
            in_response_to: Some(in_response_to.into()),
            status_type: status_type.into(),
            extended_headers: None,
            message: None,
            status_details: Vec::new(),
        }
    }

    /// Create a success status message.
    pub fn success(message_id: impl Into<String>, in_response_to: impl Into<String>) -> Self {
        Self::new(message_id, in_response_to, ST_SUCCESS)
    }

    /// Create a failure status message.
    pub fn failure(
        message_id: impl Into<String>,
        in_response_to: Option<String>,
        message: Option<String>,
    ) -> Self {
        Self {
            xmlns: NS_TAXII_11.to_string(),
            message_id: message_id.into(),
            in_response_to,
            status_type: ST_FAILURE.to_string(),
            extended_headers: None,
            message,
            status_details: Vec::new(),
        }
    }

    /// Add status details from a HashMap.
    pub fn with_status_detail(
        mut self,
        details: std::collections::HashMap<String, String>,
    ) -> Self {
        for (name, value) in details {
            self.status_details.push(super::common::StatusDetail {
                name,
                value: Some(value),
            });
        }
        self
    }
}

// ============================================================================
// Subscription Messages
// ============================================================================

/// TAXII 1.1 Manage Collection Subscription Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Subscription_Management_Request")]
pub struct ManageCollectionSubscriptionRequest {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_11")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@action")]
    pub action: String,

    #[serde(rename = "@collection_name")]
    pub collection_name: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Subscription_ID", skip_serializing_if = "Option::is_none")]
    pub subscription_id: Option<String>,

    #[serde(
        rename = "Subscription_Parameters",
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_parameters: Option<SubscriptionParameters>,

    #[serde(rename = "Push_Parameters", skip_serializing_if = "Option::is_none")]
    pub push_parameters: Option<PushParameters>,
}

/// TAXII 1.1 Manage Collection Subscription Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Subscription_Management_Response")]
pub struct ManageCollectionSubscriptionResponse {
    #[serde(rename = "@xmlns")]
    #[serde(default = "default_ns_11")]
    pub xmlns: String,

    #[serde(rename = "@message_id")]
    pub message_id: String,

    #[serde(rename = "@in_response_to", skip_serializing_if = "Option::is_none")]
    pub in_response_to: Option<String>,

    #[serde(rename = "@collection_name")]
    pub collection_name: String,

    #[serde(rename = "Extended_Headers", skip_serializing_if = "Option::is_none")]
    pub extended_headers: Option<ExtendedHeaders>,

    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(rename = "Subscription", default)]
    pub subscription_instances: Vec<SubscriptionInstance>,
}

impl ManageCollectionSubscriptionResponse {
    /// Create a new subscription response.
    pub fn new(
        message_id: impl Into<String>,
        in_response_to: impl Into<String>,
        collection_name: impl Into<String>,
    ) -> Self {
        Self {
            xmlns: NS_TAXII_11.to_string(),
            message_id: message_id.into(),
            in_response_to: Some(in_response_to.into()),
            collection_name: collection_name.into(),
            extended_headers: None,
            message: None,
            subscription_instances: Vec::new(),
        }
    }
}

// ============================================================================
// Supporting Types
// ============================================================================

/// Extended headers container.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtendedHeaders {
    #[serde(rename = "Extended_Header", default)]
    pub headers: Vec<ExtendedHeader>,
}

/// Service instance for discovery responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    #[serde(rename = "@service_type")]
    pub service_type: String,

    #[serde(rename = "@service_version")]
    pub services_version: String,

    #[serde(rename = "@available", skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,

    #[serde(rename = "Protocol_Binding")]
    pub protocol_binding: String,

    #[serde(rename = "Address")]
    pub service_address: String,

    #[serde(rename = "Message_Binding", default)]
    pub message_bindings: Vec<String>,

    #[serde(rename = "Content_Binding", default)]
    pub inbox_service_accepted_content: Vec<ContentBinding>,

    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    // SupportedQuery not implemented yet
}

impl ServiceInstance {
    /// Create a new service instance.
    pub fn new(
        service_type: impl Into<String>,
        services_version: impl Into<String>,
        protocol_binding: impl Into<String>,
        service_address: impl Into<String>,
        message_bindings: Vec<String>,
    ) -> Self {
        Self {
            service_type: service_type.into(),
            services_version: services_version.into(),
            protocol_binding: protocol_binding.into(),
            service_address: service_address.into(),
            message_bindings,
            available: None,
            inbox_service_accepted_content: Vec::new(),
            message: None,
        }
    }
}

/// Collection information for collection information responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionInformation {
    #[serde(rename = "@collection_name")]
    pub collection_name: String,

    #[serde(rename = "@collection_type", skip_serializing_if = "Option::is_none")]
    pub collection_type: Option<String>,

    #[serde(rename = "@available", skip_serializing_if = "Option::is_none")]
    pub available: Option<bool>,

    #[serde(rename = "Description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(rename = "Collection_Volume", skip_serializing_if = "Option::is_none")]
    pub collection_volume: Option<i32>,

    #[serde(rename = "Content_Binding", default)]
    pub supported_contents: Vec<ContentBinding>,

    #[serde(rename = "Polling_Service", default)]
    pub polling_service_instances: Vec<PollingServiceInstance>,

    #[serde(rename = "Subscription_Service", default)]
    pub subscription_methods: Vec<SubscriptionMethod>,

    #[serde(rename = "Receiving_Inbox_Service", default)]
    pub receiving_inbox_services: Vec<ReceivingInboxService>,
    // Push_Method not implemented yet
}

/// Polling service instance for collection information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollingServiceInstance {
    #[serde(rename = "Protocol_Binding")]
    pub poll_protocol: String,

    #[serde(rename = "Address")]
    pub poll_address: String,

    #[serde(rename = "Message_Binding", default)]
    pub poll_message_bindings: Vec<String>,
}

/// Subscription method for collection information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionMethod {
    #[serde(rename = "Protocol_Binding")]
    pub subscription_protocol: String,

    #[serde(rename = "Address")]
    pub subscription_address: String,

    #[serde(rename = "Message_Binding", default)]
    pub subscription_message_bindings: Vec<String>,
}

/// Receiving inbox service for collection information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivingInboxService {
    #[serde(rename = "Protocol_Binding")]
    pub inbox_protocol: String,

    #[serde(rename = "Address")]
    pub inbox_address: String,

    #[serde(rename = "Message_Binding", default)]
    pub inbox_message_bindings: Vec<String>,

    #[serde(rename = "Content_Binding", default)]
    pub supported_contents: Vec<ContentBinding>,
}

/// Content block in poll/inbox messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "Content_Binding")]
    pub content_binding: ContentBinding,

    #[serde(rename = "Content")]
    pub content: String,

    #[serde(rename = "Timestamp_Label", skip_serializing_if = "Option::is_none")]
    pub timestamp_label: Option<String>,

    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    #[serde(rename = "Padding", skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
}

/// Poll instance for subscription responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollInstance {
    #[serde(rename = "Protocol_Binding")]
    pub poll_protocol: String,

    #[serde(rename = "Address")]
    pub poll_address: String,

    #[serde(rename = "Message_Binding", default)]
    pub poll_message_bindings: Vec<String>,
}

/// Subscription instance for subscription responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionInstance {
    #[serde(rename = "@subscription_id")]
    pub subscription_id: String,

    #[serde(rename = "@status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    #[serde(
        rename = "Subscription_Parameters",
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription_parameters: Option<SubscriptionParameters>,

    #[serde(rename = "Push_Parameters", skip_serializing_if = "Option::is_none")]
    pub push_parameters: Option<PushParameters>,

    #[serde(rename = "Poll_Instance", default)]
    pub poll_instances: Vec<PollInstance>,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn default_ns_11() -> String {
    NS_TAXII_11.to_string()
}
