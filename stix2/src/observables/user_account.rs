//! User Account SCO

use crate::core::error::Result;
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use crate::vocab::AccountType;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// User Account STIX Cyber Observable Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserAccount {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_type: Option<AccountType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_service_account: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_privileged: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub can_escalate_privs: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_disabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_created: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_expires: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_last_changed: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_first_login: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_last_login: Option<Timestamp>,
    /// References to marking definitions that apply to this object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_marking_refs: Vec<Identifier>,
    /// Granular markings for specific properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granular_markings: Vec<GranularMarking>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,
}

fn default_spec_version() -> String {
    "2.1".to_string()
}

impl UserAccount {
    pub const TYPE: &'static str = "user-account";

    pub fn new() -> Result<Self> {
        Ok(Self {
            type_: Self::TYPE.to_string(),
            id: Identifier::new(Self::TYPE)?,
            spec_version: default_spec_version(),
            defanged: false,
            user_id: None,
            credential: None,
            account_login: None,
            account_type: None,
            display_name: None,
            is_service_account: false,
            is_privileged: false,
            can_escalate_privs: false,
            is_disabled: false,
            account_created: None,
            account_expires: None,
            credential_last_changed: None,
            account_first_login: None,
            account_last_login: None,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
        })
    }
}

impl_sco_traits!(UserAccount, "user-account");

impl crate::observables::IdContributing for UserAccount {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] =
        &["account_type", "user_id", "account_login"];
}
