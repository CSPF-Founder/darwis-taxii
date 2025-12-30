//! Email Message SCO

use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::extensions::EmailMimeComponent;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use crate::validation::Constrained;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Email Message STIX Cyber Observable Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailMessage {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,
    pub is_multipart: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_ref: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_ref: Option<Identifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to_refs: Vec<Identifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc_refs: Vec<Identifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bcc_refs: Vec<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub received_lines: Vec<String>,
    /// Specifies any other header fields (except for date, received_lines,
    /// content_type, from_ref, sender_ref, to_refs, cc_refs, bcc_refs, and subject)
    /// found in the email message, as a dictionary.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub additional_header_fields: IndexMap<String, String>,
    /// Specifies the body of the email message (for non-multipart messages).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Specifies a list of MIME parts for multipart email messages.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body_multipart: Vec<EmailMimeComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_email_ref: Option<Identifier>,
    /// References to marking definitions that apply to this object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_marking_refs: Vec<Identifier>,
    /// Granular markings for specific properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granular_markings: Vec<GranularMarking>,
    /// Extensions for this object.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,
}

fn default_spec_version() -> String {
    "2.1".to_string()
}

impl EmailMessage {
    pub const TYPE: &'static str = "email-message";

    pub fn new(is_multipart: bool) -> Result<Self> {
        Ok(Self {
            type_: Self::TYPE.to_string(),
            id: Identifier::new(Self::TYPE)?,
            spec_version: default_spec_version(),
            defanged: false,
            is_multipart,
            date: None,
            content_type: None,
            from_ref: None,
            sender_ref: None,
            to_refs: Vec::new(),
            cc_refs: Vec::new(),
            bcc_refs: Vec::new(),
            message_id: None,
            subject: None,
            received_lines: Vec::new(),
            additional_header_fields: IndexMap::new(),
            body: None,
            body_multipart: Vec::new(),
            raw_email_ref: None,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
        })
    }
}

impl_sco_traits!(EmailMessage, "email-message");

impl crate::observables::IdContributing for EmailMessage {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &["from_ref", "subject", "body"];
}

impl Constrained for EmailMessage {
    /// Validate EmailMessage constraints.
    ///
    /// - If `is_multipart` is true, `body_multipart` must be present
    /// - If `is_multipart` is true, `body` must NOT be present
    fn validate_constraints(&self) -> Result<()> {
        use crate::validation::{check_optional_ref_type, check_refs_type};

        if self.is_multipart && self.body_multipart.is_empty() {
            return Err(Error::PropertyDependency {
                dependent: "is_multipart".to_string(),
                dependency: "body_multipart".to_string(),
            });
        }

        // If is_multipart is true, body must not be present
        if self.is_multipart && self.body.is_some() {
            return Err(Error::InvalidPropertyValue {
                property: "body".to_string(),
                message: "'body' cannot be present when 'is_multipart' is true".to_string(),
            });
        }

        // Validate reference types
        check_optional_ref_type(self.from_ref.as_ref(), "from_ref", &["email-addr"])?;
        check_optional_ref_type(self.sender_ref.as_ref(), "sender_ref", &["email-addr"])?;
        check_refs_type(&self.to_refs, "to_refs", &["email-addr"])?;
        check_refs_type(&self.cc_refs, "cc_refs", &["email-addr"])?;
        check_refs_type(&self.bcc_refs, "bcc_refs", &["email-addr"])?;
        check_optional_ref_type(self.raw_email_ref.as_ref(), "raw_email_ref", &["artifact"])?;

        Ok(())
    }
}
