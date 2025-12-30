//! TAXII 1.x XML message types.

pub mod common;
pub mod messages_10;
pub mod messages_11;

pub use common::*;
pub use messages_10 as tm10;
pub use messages_11 as tm11;

use quick_xml::de::from_str;

use crate::constants::{VID_TAXII_XML_10, VID_TAXII_XML_11};
use crate::error::{Taxii1xError, Taxii1xResult};

/// Parse a TAXII message from XML, detecting the version automatically.
pub fn get_message_from_xml(xml_string: &str) -> Taxii1xResult<TaxiiMessage> {
    // Detect version by namespace
    if xml_string.contains(common::NS_TAXII_11) {
        let msg: messages_11::Taxii11Message =
            from_str(xml_string).map_err(Taxii1xError::xml_parse)?;
        Ok(TaxiiMessage::V11(msg))
    } else if xml_string.contains(common::NS_TAXII_10) {
        let msg: messages_10::Taxii10Message =
            from_str(xml_string).map_err(Taxii1xError::xml_parse)?;
        Ok(TaxiiMessage::V10(msg))
    } else {
        Err(Taxii1xError::UnsupportedVersion(
            "Unknown TAXII namespace".to_string(),
        ))
    }
}

/// Enum wrapper for TAXII messages of different versions.
#[derive(Debug, Clone)]
pub enum TaxiiMessage {
    V10(messages_10::Taxii10Message),
    V11(messages_11::Taxii11Message),
}

impl TaxiiMessage {
    /// Get the message ID.
    pub fn message_id(&self) -> &str {
        match self {
            TaxiiMessage::V10(msg) => msg.message_id(),
            TaxiiMessage::V11(msg) => msg.message_id(),
        }
    }

    /// Get the version string.
    pub fn version(&self) -> &str {
        match self {
            TaxiiMessage::V10(_) => VID_TAXII_XML_10,
            TaxiiMessage::V11(_) => VID_TAXII_XML_11,
        }
    }

    /// Get the message type string.
    pub fn message_type(&self) -> &str {
        match self {
            TaxiiMessage::V10(msg) => msg.message_type(),
            TaxiiMessage::V11(msg) => msg.message_type(),
        }
    }

    /// Serialize to XML.
    pub fn to_xml(&self) -> Taxii1xResult<String> {
        match self {
            TaxiiMessage::V10(msg) => msg.to_xml(),
            TaxiiMessage::V11(msg) => msg.to_xml(),
        }
    }
}
