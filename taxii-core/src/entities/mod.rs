//! Common entities.

pub mod taxii1;
pub mod taxii2;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Permission value that supports both TAXII 1.x and 2.x formats.
///
/// - TAXII 1.x: `"read"` or `"modify"` (string)
/// - TAXII 2.x: `["read"]`, `["write"]`, or `["read", "write"]` (list)
///
/// This enum handles both formats with full backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PermissionValue {
    /// TAXII 1.x format: "read" or "modify" (string value)
    Taxii1(String),
    /// TAXII 2.x format: ["read"], ["write"], or ["read", "write"] (list)
    Taxii2(Vec<String>),
}

impl PermissionValue {
    /// Check if permission allows read access.
    ///
    /// Works for both formats:
    /// - TAXII 1.x: "read" or "modify" grants read
    /// - TAXII 2.x: ["read"] or ["read", "write"] grants read
    pub fn can_read(&self) -> bool {
        match self {
            PermissionValue::Taxii1(s) => s == "read" || s == "modify",
            PermissionValue::Taxii2(list) => list.iter().any(|p| p == "read"),
        }
    }

    /// Check if permission allows write access.
    ///
    /// Works for both formats:
    /// - TAXII 1.x: "modify" grants write
    /// - TAXII 2.x: ["write"] or ["read", "write"] grants write
    pub fn can_write(&self) -> bool {
        match self {
            PermissionValue::Taxii1(s) => s == "modify",
            PermissionValue::Taxii2(list) => list.iter().any(|p| p == "write"),
        }
    }
}

/// Account entity.
///
/// Permission checks are handled by Collection entities (TAXII 1.x CollectionEntity
/// and TAXII 2.x Collection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account ID.
    pub id: i32,

    /// Username.
    pub username: String,

    /// Whether account is admin.
    pub is_admin: bool,

    /// Permissions as collection_identifier -> permission mapping.
    ///
    /// - For TAXII 1.x: key is collection name (string), value is "read" or "modify"
    /// - For TAXII 2.x: key is collection UUID (stringified), value is ["read"], ["write"], or both
    pub permissions: HashMap<String, PermissionValue>,

    /// Additional details.
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
}
