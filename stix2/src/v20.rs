//! STIX 2.0 Compatibility Layer
//!
//! This module provides compatibility with STIX 2.0 objects, allowing
//! parsing and conversion between STIX 2.0 and 2.1 formats.
//!
//! ## Key Differences between STIX 2.0 and 2.1
//!
//! - **spec_version**: 2.0 objects don't have this field; 2.1 objects do
//! - **New object types in 2.1**: Grouping, Incident, Infrastructure, Location,
//!   Malware Analysis, Note, Opinion, Language Content
//! - **SCO changes**: 2.0 SCOs don't have IDs; 2.1 SCOs do
//! - **Relationship changes**: 2.1 has more relationship types
//! - **Confidence**: Optional in 2.1, not present in 2.0 core spec
//!
//! ## Usage
//!
//! ```rust,ignore
//! use stix2::v20::{parse_v20, upgrade_to_v21};
//!
//! // Parse STIX 2.0 JSON
//! let v20_obj = parse_v20(json_str)?;
//!
//! // Upgrade to 2.1
//! let v21_obj = upgrade_to_v21(&v20_obj)?;
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::core::error::{Error, Result};
use crate::core::stix_object::StixObject;

/// STIX 2.0 specification version string.
pub const SPEC_VERSION_20: &str = "2.0";

/// STIX 2.1 specification version string.
pub const SPEC_VERSION_21: &str = "2.1";

/// Object types available in STIX 2.0.
pub const STIX_20_TYPES: &[&str] = &[
    // SDOs
    "attack-pattern",
    "campaign",
    "course-of-action",
    "identity",
    "indicator",
    "intrusion-set",
    "malware",
    "observed-data",
    "report",
    "threat-actor",
    "tool",
    "vulnerability",
    // SROs
    "relationship",
    "sighting",
    // Marking
    "marking-definition",
];

/// Object types added in STIX 2.1.
pub const STIX_21_ONLY_TYPES: &[&str] = &[
    "grouping",
    "incident",
    "infrastructure",
    "location",
    "malware-analysis",
    "note",
    "opinion",
    "language-content",
];

/// Detect STIX spec version from JSON string.
///
/// Returns "2.0", "2.1", or "2.1" (default) for unknown versions.
///
/// # Example
///
/// ```rust,ignore
/// let version = detect_spec_version(r#"{"type": "indicator", "spec_version": "2.1"}"#)?;
/// assert_eq!(version, "2.1");
/// ```
pub fn detect_spec_version(json: &str) -> Result<&'static str> {
    let value: Value =
        serde_json::from_str(json).map_err(|e| Error::Custom(format!("JSON parse error: {e}")))?;

    Ok(match detect_version(&value) {
        StixVersion::V20 => "2.0",
        StixVersion::V21 => "2.1",
        StixVersion::Unknown => "2.1", // Default to latest
    })
}

/// Detect the STIX version of a JSON value.
pub fn detect_version(value: &Value) -> StixVersion {
    // Check for spec_version field
    if let Some(spec) = value.get("spec_version").and_then(|v| v.as_str()) {
        if spec.starts_with("2.1") {
            return StixVersion::V21;
        } else if spec.starts_with("2.0") {
            return StixVersion::V20;
        }
    }

    // Check object type
    if let Some(type_str) = value.get("type").and_then(|v| v.as_str())
        && STIX_21_ONLY_TYPES.contains(&type_str)
    {
        return StixVersion::V21;
    }

    // Check for 2.1-specific features
    if value.get("confidence").is_some() {
        return StixVersion::V21;
    }

    // Check if it's a bundle
    if value.get("type").and_then(|v| v.as_str()) == Some("bundle")
        && let Some(objects) = value.get("objects").and_then(|v| v.as_array())
    {
        for obj in objects {
            let version = detect_version(obj);
            if version != StixVersion::Unknown {
                return version;
            }
        }
    }

    // For SCOs, check for id field (2.1 has IDs, 2.0 doesn't)
    if is_observable_type(value.get("type").and_then(|v| v.as_str()).unwrap_or("")) {
        if value.get("id").is_some() {
            return StixVersion::V21;
        } else {
            return StixVersion::V20;
        }
    }

    StixVersion::Unknown
}

/// STIX specification version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StixVersion {
    /// STIX 2.0
    V20,
    /// STIX 2.1
    V21,
    /// Unknown version
    Unknown,
}

/// Check if a type is an observable (SCO) type.
fn is_observable_type(type_str: &str) -> bool {
    matches!(
        type_str,
        "artifact"
            | "autonomous-system"
            | "directory"
            | "domain-name"
            | "email-addr"
            | "email-message"
            | "file"
            | "ipv4-addr"
            | "ipv6-addr"
            | "mac-addr"
            | "mutex"
            | "network-traffic"
            | "process"
            | "software"
            | "url"
            | "user-account"
            | "windows-registry-key"
            | "x509-certificate"
    )
}

/// Parse STIX 2.0 JSON and return a version-aware wrapper.
pub fn parse_v20(json: &str) -> Result<Stix20Object> {
    let value: Value =
        serde_json::from_str(json).map_err(|e| Error::Custom(format!("JSON parse error: {e}")))?;

    let version = detect_version(&value);
    if version == StixVersion::V21 {
        return Err(Error::Custom(
            "Object appears to be STIX 2.1, not 2.0".to_string(),
        ));
    }

    Ok(Stix20Object { value })
}

/// Parse any STIX version (2.0 or 2.1) and return a 2.1 object.
pub fn parse_any_version(json: &str) -> Result<StixObject> {
    let value: Value =
        serde_json::from_str(json).map_err(|e| Error::Custom(format!("JSON parse error: {e}")))?;

    let version = detect_version(&value);

    match version {
        StixVersion::V21 | StixVersion::Unknown => {
            // Parse directly as 2.1
            crate::parse(json)
        }
        StixVersion::V20 => {
            // Upgrade to 2.1 first
            let v20 = Stix20Object { value };
            upgrade_to_v21(&v20)
        }
    }
}

/// STIX 2.0 object wrapper.
#[derive(Debug, Clone)]
pub struct Stix20Object {
    /// The raw JSON value.
    pub value: Value,
}

impl Stix20Object {
    /// Get the object type.
    pub fn type_name(&self) -> Option<&str> {
        self.value.get("type").and_then(|v| v.as_str())
    }

    /// Get the object ID (if present).
    pub fn id(&self) -> Option<&str> {
        self.value.get("id").and_then(|v| v.as_str())
    }

    /// Get the raw JSON value.
    pub fn as_value(&self) -> &Value {
        &self.value
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(&self.value)
            .map_err(|e| Error::Custom(format!("Serialization error: {e}")))
    }

    /// Serialize to pretty JSON string.
    pub fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.value)
            .map_err(|e| Error::Custom(format!("Serialization error: {e}")))
    }
}

/// Upgrade a STIX 2.0 object to STIX 2.1.
pub fn upgrade_to_v21(v20: &Stix20Object) -> Result<StixObject> {
    let mut value = v20.value.clone();

    if let Value::Object(ref mut map) = value {
        // Add spec_version if not present
        if !map.contains_key("spec_version") {
            map.insert(
                "spec_version".to_string(),
                Value::String(SPEC_VERSION_21.to_string()),
            );
        }

        // Handle SCOs - add ID if not present
        if let Some(type_str) = map.get("type").and_then(|v| v.as_str())
            && is_observable_type(type_str)
            && !map.contains_key("id")
        {
            // Generate a deterministic ID based on content
            let id = generate_sco_id(type_str, map)?;
            map.insert("id".to_string(), Value::String(id));
        }

        // Handle specific type migrations
        migrate_object_properties(map)?;
    }

    // Parse as STIX 2.1
    let json = serde_json::to_string(&value)
        .map_err(|e| Error::Custom(format!("Serialization error: {e}")))?;

    crate::parse(&json)
}

/// Downgrade a STIX 2.1 object to STIX 2.0 format.
///
/// Note: This may lose information for 2.1-only features.
pub fn downgrade_to_v20(v21: &StixObject) -> Result<Stix20Object> {
    let mut value = serde_json::to_value(v21)
        .map_err(|e| Error::Custom(format!("Serialization error: {e}")))?;

    // Check if the object type exists in 2.0
    if let Some(type_str) = value.get("type").and_then(|v| v.as_str())
        && STIX_21_ONLY_TYPES.contains(&type_str)
    {
        return Err(Error::Custom(format!(
            "Object type '{type_str}' does not exist in STIX 2.0"
        )));
    }

    if let Value::Object(ref mut map) = value {
        // Remove spec_version
        map.remove("spec_version");

        // Remove 2.1-only common properties
        map.remove("confidence");
        map.remove("lang");

        // Handle SCOs - remove ID for 2.0 format
        // (Actually, 2.0 used object indices in observed-data, but we'll keep IDs for usability)

        // Remove 2.1-only properties from specific types
        remove_v21_properties(map);
    }

    Ok(Stix20Object { value })
}

/// Generate a deterministic SCO ID based on its content.
fn generate_sco_id(type_name: &str, properties: &Map<String, Value>) -> Result<String> {
    use sha2::{Digest, Sha256};

    // Build a canonical string from key properties
    let mut canonical = String::new();
    canonical.push_str(type_name);

    // Use type-specific key properties
    match type_name {
        "file" => {
            if let Some(hashes) = properties.get("hashes").and_then(|v| v.as_object()) {
                for (k, v) in hashes {
                    canonical.push_str(k);
                    canonical.push_str(v.as_str().unwrap_or(""));
                }
            }
            if let Some(name) = properties.get("name").and_then(|v| v.as_str()) {
                canonical.push_str(name);
            }
        }
        "ipv4-addr" | "ipv6-addr" | "domain-name" | "url" | "email-addr" => {
            if let Some(value) = properties.get("value").and_then(|v| v.as_str()) {
                canonical.push_str(value);
            }
        }
        "process" => {
            if let Some(pid) = properties.get("pid").and_then(|v| v.as_i64()) {
                canonical.push_str(&pid.to_string());
            }
            if let Some(name) = properties.get("name").and_then(|v| v.as_str()) {
                canonical.push_str(name);
            }
        }
        _ => {
            // For other types, use all properties
            if let Ok(json) = serde_json::to_string(properties) {
                canonical.push_str(&json);
            }
        }
    }

    // Generate UUID v5 from the canonical string
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let hash = hasher.finalize();

    // Convert to UUID format (simplified - using first 16 bytes)
    let uuid = format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([hash[0], hash[1], hash[2], hash[3]]),
        u16::from_be_bytes([hash[4], hash[5]]),
        u16::from_be_bytes([hash[6], hash[7]]),
        u16::from_be_bytes([hash[8], hash[9]]),
        u64::from_be_bytes([
            0, 0, hash[10], hash[11], hash[12], hash[13], hash[14], hash[15]
        ])
    );

    Ok(format!("{type_name}--{uuid}"))
}

/// Migrate object properties from 2.0 to 2.1 format.
fn migrate_object_properties(map: &mut Map<String, Value>) -> Result<()> {
    let type_name = map
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    match type_name.as_str() {
        "malware" => {
            // In 2.1, malware requires is_family property
            if !map.contains_key("is_family") {
                map.insert("is_family".to_string(), Value::Bool(false));
            }

            // labels -> malware_types in 2.1
            if !map.contains_key("malware_types")
                && let Some(labels) = map.get("labels").cloned()
            {
                map.insert("malware_types".to_string(), labels);
            }
        }
        "tool" => {
            // labels -> tool_types in 2.1
            if !map.contains_key("tool_types")
                && let Some(labels) = map.get("labels").cloned()
            {
                map.insert("tool_types".to_string(), labels);
            }
        }
        "attack-pattern" => {
            // No major changes, but ensure external_references format is correct
        }
        "indicator" => {
            // pattern_type is required in 2.1
            if !map.contains_key("pattern_type") {
                map.insert(
                    "pattern_type".to_string(),
                    Value::String("stix".to_string()),
                );
            }
        }
        "observed-data" => {
            // objects -> object_refs in 2.1
            // This is a complex migration as 2.0 embeds objects, 2.1 uses references
            // For simplicity, we'll extract embedded objects if present
            if let Some(objects) = map.remove("objects")
                && let Value::Object(embedded) = objects
            {
                let mut object_refs = Vec::new();
                for (_key, obj_value) in embedded {
                    if let Value::Object(mut obj) = obj_value {
                        // Generate ID for the embedded object
                        if let Some(type_str) = obj.get("type").and_then(|v| v.as_str()) {
                            if !obj.contains_key("id") {
                                if let Ok(id) = generate_sco_id(type_str, &obj) {
                                    obj.insert("id".to_string(), Value::String(id.clone()));
                                    object_refs.push(Value::String(id));
                                }
                            } else if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                                object_refs.push(Value::String(id.to_string()));
                            }
                        }
                    }
                }
                if !object_refs.is_empty() {
                    map.insert("object_refs".to_string(), Value::Array(object_refs));
                }
            }
        }
        "report" => {
            // object_refs is required in 2.1
            if !map.contains_key("object_refs") {
                map.insert("object_refs".to_string(), Value::Array(vec![]));
            }
        }
        _ => {}
    }

    Ok(())
}

/// Remove 2.1-only properties when downgrading to 2.0.
fn remove_v21_properties(map: &mut Map<String, Value>) {
    let type_name = map
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Remove common 2.1-only properties
    map.remove("extensions");

    match type_name.as_str() {
        "malware" => {
            // is_family -> labels in 2.0
            map.remove("is_family");
            // malware_types -> labels in 2.0
            if let Some(types) = map.remove("malware_types") {
                map.insert("labels".to_string(), types);
            }
        }
        "tool" => {
            // tool_types -> labels in 2.0
            if let Some(types) = map.remove("tool_types") {
                map.insert("labels".to_string(), types);
            }
        }
        "indicator" => {
            // pattern_type not in 2.0 (always STIX pattern)
            map.remove("pattern_type");
            map.remove("pattern_version");
        }
        "report" => {
            // report_types not in 2.0
            map.remove("report_types");
        }
        _ => {}
    }
}

/// STIX 2.0 Bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle20 {
    /// Bundle type (always "bundle").
    #[serde(rename = "type")]
    pub type_: String,
    /// Bundle ID.
    pub id: String,
    /// Spec version (may not be present in 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spec_version: Option<String>,
    /// Objects in the bundle.
    #[serde(default)]
    pub objects: Vec<Value>,
}

impl Bundle20 {
    /// Parse a STIX 2.0 bundle.
    pub fn parse(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| Error::Custom(format!("Bundle parse error: {e}")))
    }

    /// Upgrade all objects in the bundle to STIX 2.1.
    pub fn upgrade_to_v21(&self) -> Result<crate::core::bundle::Bundle> {
        let mut objects = Vec::new();

        for obj_value in &self.objects {
            let v20 = Stix20Object {
                value: obj_value.clone(),
            };
            match upgrade_to_v21(&v20) {
                Ok(obj) => objects.push(obj),
                Err(e) => {
                    // Log warning but continue with other objects
                    eprintln!("Warning: Failed to upgrade object: {e}");
                }
            }
        }

        Ok(crate::core::bundle::Bundle::from_objects(objects))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_spec_version() {
        let v21_json = r#"{"type": "indicator", "spec_version": "2.1", "id": "indicator--123"}"#;
        assert_eq!(detect_spec_version(v21_json).unwrap(), "2.1");

        let v21_only_type = r#"{"type": "grouping", "id": "grouping--123"}"#;
        assert_eq!(detect_spec_version(v21_only_type).unwrap(), "2.1");
    }

    #[test]
    fn test_detect_version_v20() {
        let v20_json = r#"{"type": "indicator", "id": "indicator--123", "created": "2020-01-01T00:00:00Z", "modified": "2020-01-01T00:00:00Z", "pattern": "[file:name = 'test']", "valid_from": "2020-01-01T00:00:00Z"}"#;
        let value: Value = serde_json::from_str(v20_json).unwrap();
        // Without spec_version and without 2.1-only features, version is unknown
        // because it could be either
        let version = detect_version(&value);
        assert!(version == StixVersion::Unknown || version == StixVersion::V20);
    }

    #[test]
    fn test_detect_version_v21() {
        let v21_json = r#"{"type": "indicator", "spec_version": "2.1", "id": "indicator--123"}"#;
        let value: Value = serde_json::from_str(v21_json).unwrap();
        assert_eq!(detect_version(&value), StixVersion::V21);
    }

    #[test]
    fn test_detect_version_v21_only_type() {
        let v21_json = r#"{"type": "grouping", "id": "grouping--123"}"#;
        let value: Value = serde_json::from_str(v21_json).unwrap();
        assert_eq!(detect_version(&value), StixVersion::V21);
    }

    #[test]
    fn test_upgrade_indicator() {
        let v20_json = r#"{
            "type": "indicator",
            "id": "indicator--a1b2c3d4-1234-5678-90ab-cdef12345678",
            "created": "2020-01-01T00:00:00.000Z",
            "modified": "2020-01-01T00:00:00.000Z",
            "pattern": "[file:name = 'test.exe']",
            "valid_from": "2020-01-01T00:00:00.000Z",
            "labels": ["malicious-activity"]
        }"#;

        let v20 = parse_v20(v20_json).unwrap();
        let v21 = upgrade_to_v21(&v20).unwrap();

        // Should have spec_version now
        assert_eq!(v21.type_name(), "indicator");
    }

    #[test]
    fn test_generate_sco_id() {
        let mut props = Map::new();
        props.insert(
            "value".to_string(),
            Value::String("192.168.1.1".to_string()),
        );

        let id = generate_sco_id("ipv4-addr", &props).unwrap();
        assert!(id.starts_with("ipv4-addr--"));

        // Same input should produce same ID
        let id2 = generate_sco_id("ipv4-addr", &props).unwrap();
        assert_eq!(id, id2);
    }

    #[test]
    fn test_is_stix_20_type() {
        assert!(STIX_20_TYPES.contains(&"indicator"));
        assert!(STIX_20_TYPES.contains(&"malware"));
        assert!(!STIX_20_TYPES.contains(&"grouping"));
        assert!(!STIX_20_TYPES.contains(&"infrastructure"));
    }

    #[test]
    fn test_bundle_parse() {
        let bundle_json = r#"{
            "type": "bundle",
            "id": "bundle--12345678-1234-5678-1234-567812345678",
            "objects": [
                {
                    "type": "indicator",
                    "id": "indicator--12345678-1234-5678-1234-567812345678",
                    "created": "2020-01-01T00:00:00.000Z",
                    "modified": "2020-01-01T00:00:00.000Z",
                    "pattern": "[file:name = 'test']",
                    "valid_from": "2020-01-01T00:00:00.000Z"
                }
            ]
        }"#;

        let bundle = Bundle20::parse(bundle_json).unwrap();
        assert_eq!(bundle.objects.len(), 1);
    }
}
