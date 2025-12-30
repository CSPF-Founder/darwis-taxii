//! STIX Identifier handling.
//!
//! STIX identifiers follow the format: `<type>--<uuid>`
//! For example: `indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f`

use crate::core::error::{Error, Result};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;
use uuid::Uuid;

/// Compiled regex for validating STIX type names.
/// STIX type must be lowercase alphanumeric with hyphens.
///
/// # Safety
/// The regex pattern is a compile-time constant that is known to be valid.
/// The `expect` is acceptable here as it will never fail in practice.
#[allow(clippy::expect_used)]
static TYPE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$").expect("TYPE_REGEX pattern is valid")
});

/// A STIX identifier consisting of a type prefix and UUID.
///
/// STIX identifiers follow the format `<type>--<uuid>`, where:
/// - `<type>` is the STIX object type (e.g., "indicator", "malware")
/// - `<uuid>` is a valid UUID (version 4 for STIX 2.0, any RFC 4122 for STIX 2.1)
///
/// # Example
///
/// ```rust,no_run
/// use stix2::Identifier;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let id = Identifier::new("indicator")?;
///     assert!(id.to_string().starts_with("indicator--"));
///
///     let parsed: Identifier = "malware--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f".parse()?;
///     assert_eq!(parsed.object_type(), "malware");
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    object_type: String,
    uuid: Uuid,
}

impl Identifier {
    /// Create a new identifier with a random UUID v4.
    ///
    /// # Arguments
    ///
    /// * `object_type` - The STIX object type (e.g., "indicator", "malware")
    ///
    /// # Errors
    ///
    /// Returns an error if the object type is invalid.
    pub fn new(object_type: &str) -> Result<Self> {
        Self::validate_type(object_type)?;
        Ok(Self {
            object_type: object_type.to_lowercase(),
            uuid: Uuid::new_v4(),
        })
    }

    /// Create an identifier with a specific UUID.
    ///
    /// # Arguments
    ///
    /// * `object_type` - The STIX object type
    /// * `uuid` - The UUID to use
    ///
    /// # Errors
    ///
    /// Returns an error if the object type is invalid.
    pub fn with_uuid(object_type: &str, uuid: Uuid) -> Result<Self> {
        Self::validate_type(object_type)?;
        Ok(Self {
            object_type: object_type.to_lowercase(),
            uuid,
        })
    }

    /// Create a deterministic identifier using UUID v5.
    ///
    /// This is primarily used for STIX Cyber Observable objects in STIX 2.1,
    /// where identifiers are derived from the object's properties.
    ///
    /// # Arguments
    ///
    /// * `object_type` - The STIX object type
    /// * `namespace` - The UUID namespace
    /// * `name` - The name to hash
    pub fn deterministic(object_type: &str, namespace: Uuid, name: &str) -> Result<Self> {
        Self::validate_type(object_type)?;
        let uuid = Uuid::new_v5(&namespace, name.as_bytes());
        Ok(Self {
            object_type: object_type.to_lowercase(),
            uuid,
        })
    }

    /// Get the STIX namespace UUID for SCO deterministic IDs.
    #[must_use]
    pub fn stix_namespace() -> Uuid {
        // STIX 2.1 namespace: "00abedb4-aa42-466c-9c01-fed23315a9b7"
        const STIX_NAMESPACE: Uuid = uuid::uuid!("00abedb4-aa42-466c-9c01-fed23315a9b7");
        STIX_NAMESPACE
    }

    // =========================================================================
    // Dedicated constructors for known STIX types (infallible)
    // These bypass validation since the type names are compile-time constants.
    // =========================================================================

    /// Create a bundle identifier with a specific UUID.
    #[must_use]
    pub fn bundle(uuid: Uuid) -> Self {
        Self {
            object_type: "bundle".to_string(),
            uuid,
        }
    }

    /// Create a bundle identifier with a random UUID.
    #[must_use]
    pub fn new_bundle() -> Self {
        Self::bundle(Uuid::new_v4())
    }

    /// Create a marking-definition identifier with a specific UUID.
    #[must_use]
    pub fn marking_definition(uuid: Uuid) -> Self {
        Self {
            object_type: "marking-definition".to_string(),
            uuid,
        }
    }

    /// Create a marking-definition identifier with a random UUID.
    #[must_use]
    pub fn new_marking_definition() -> Self {
        Self::marking_definition(Uuid::new_v4())
    }

    /// Get the object type.
    #[must_use]
    pub fn object_type(&self) -> &str {
        &self.object_type
    }

    /// Get the UUID.
    #[must_use]
    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Validate the object type format.
    fn validate_type(object_type: &str) -> Result<()> {
        if !TYPE_REGEX.is_match(object_type) {
            return Err(Error::InvalidType(format!(
                "'{}' is not a valid STIX type. Types must be lowercase alphanumeric with hyphens.",
                object_type
            )));
        }

        Ok(())
    }

    /// Check if this identifier references the specified type.
    #[must_use]
    pub fn is_type(&self, type_name: &str) -> bool {
        self.object_type.eq_ignore_ascii_case(type_name)
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}--{}", self.object_type, self.uuid)
    }
}

impl FromStr for Identifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.splitn(2, "--").collect();
        if parts.len() != 2 {
            return Err(Error::InvalidId(format!(
                "'{}' does not match the STIX identifier format '<type>--<uuid>'",
                s
            )));
        }

        let object_type = parts[0];
        let uuid_str = parts[1];

        Self::validate_type(object_type)?;

        let uuid = Uuid::parse_str(uuid_str)
            .map_err(|e| Error::InvalidId(format!("Invalid UUID in identifier '{}': {}", s, e)))?;

        Ok(Self {
            object_type: object_type.to_lowercase(),
            uuid,
        })
    }
}

impl Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

/// Trait for types that have a STIX identifier.
pub trait HasId {
    /// Get the object's identifier.
    fn id(&self) -> &Identifier;

    /// Get the object type from the identifier.
    fn object_type(&self) -> &str {
        self.id().object_type()
    }
}

/// Macro to create an identifier literal at compile time.
/// Note: This macro validates format at runtime on first use.
#[macro_export]
macro_rules! stix_id {
    ($s:expr) => {{
        static ID: std::sync::OnceLock<$crate::Identifier> = std::sync::OnceLock::new();
        ID.get_or_init(|| $s.parse().expect(concat!("Invalid STIX identifier: ", $s)))
            .clone()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_identifier() {
        let id = Identifier::new("indicator").unwrap();
        assert_eq!(id.object_type(), "indicator");
        assert!(id.to_string().starts_with("indicator--"));
    }

    #[test]
    fn test_parse_identifier() {
        let id: Identifier = "malware--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
            .parse()
            .unwrap();
        assert_eq!(id.object_type(), "malware");
        assert_eq!(
            id.uuid().to_string(),
            "8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
        );
    }

    #[test]
    fn test_identifier_display() {
        let uuid = Uuid::parse_str("8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f").unwrap();
        let id = Identifier::with_uuid("indicator", uuid).unwrap();
        assert_eq!(
            id.to_string(),
            "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
        );
    }

    #[test]
    fn test_invalid_identifier_format() {
        let result: Result<Identifier> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_type() {
        let result = Identifier::new("INVALID_TYPE");
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_id() {
        let id1 = Identifier::deterministic("file", Identifier::stix_namespace(), "test-content")
            .unwrap();
        let id2 = Identifier::deterministic("file", Identifier::stix_namespace(), "test-content")
            .unwrap();
        assert_eq!(id1.uuid(), id2.uuid());
    }

    #[test]
    fn test_serde() {
        let id: Identifier = "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
            .parse()
            .unwrap();
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f\"");

        let parsed: Identifier = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_is_type() {
        let id: Identifier = "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
            .parse()
            .unwrap();
        assert!(id.is_type("indicator"));
        assert!(id.is_type("INDICATOR"));
        assert!(!id.is_type("malware"));
    }
}
