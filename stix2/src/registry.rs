//! STIX Object Type Registry
//!
//! This module provides a registry for STIX object types, enabling
//! dynamic lookup and custom type registration.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::RwLock;

use once_cell::sync::Lazy;

use crate::core::error::{Error, Result};
use crate::core::stix_object::StixObject;

/// STIX specification version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecVersion {
    /// STIX 2.0
    V20,
    /// STIX 2.1
    V21,
}

impl FromStr for SpecVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "2.0" => Ok(SpecVersion::V20),
            "2.1" => Ok(SpecVersion::V21),
            _ => Err(Error::InvalidType(format!("Unknown STIX version: {}", s))),
        }
    }
}

impl SpecVersion {
    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            SpecVersion::V20 => "2.0",
            SpecVersion::V21 => "2.1",
        }
    }
}

/// Object category in the registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectCategory {
    /// STIX Domain Objects (SDO)
    DomainObject,
    /// STIX Relationship Objects (SRO)
    RelationshipObject,
    /// STIX Cyber Observable Objects (SCO)
    Observable,
    /// Marking definitions
    Marking,
    /// Extensions
    Extension,
}

/// Parser function type for custom objects
pub type ObjectParser = fn(&str) -> Result<StixObject>;

/// Validator function type for custom objects
pub type ObjectValidator = fn(&serde_json::Value) -> Result<()>;

/// Type information for a registered STIX type
#[derive(Clone)]
pub struct TypeInfo {
    /// The STIX type name
    pub type_name: String,
    /// Object category
    pub category: ObjectCategory,
    /// Spec versions this type is available in
    pub spec_versions: Vec<SpecVersion>,
    /// Custom parser (if any)
    pub parser: Option<ObjectParser>,
    /// Whether this is a custom type
    pub is_custom: bool,
    /// ID-contributing properties for SCOs (STIX 2.1+)
    pub id_contributing_props: Option<Vec<String>>,
    /// Custom validator function
    pub validator: Option<ObjectValidator>,
}

/// Global type registry
static REGISTRY: Lazy<RwLock<TypeRegistry>> = Lazy::new(|| {
    let mut registry = TypeRegistry::new();
    registry.register_builtin_types();
    RwLock::new(registry)
});

/// Type registry for STIX objects
pub struct TypeRegistry {
    /// Types by name and version
    types: HashMap<(String, SpecVersion), TypeInfo>,
    /// Custom type parsers
    custom_parsers: HashMap<String, ObjectParser>,
}

impl TypeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            custom_parsers: HashMap::new(),
        }
    }

    /// Register built-in STIX types
    fn register_builtin_types(&mut self) {
        // SDOs available in both 2.0 and 2.1
        let common_sdos = [
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
        ];

        for type_name in common_sdos {
            self.register_type(TypeInfo {
                type_name: type_name.to_string(),
                category: ObjectCategory::DomainObject,
                spec_versions: vec![SpecVersion::V20, SpecVersion::V21],
                parser: None,
                is_custom: false,
                id_contributing_props: None,
                validator: None,
            });
        }

        // SDOs only in 2.1
        let v21_sdos = [
            "grouping",
            "incident",
            "infrastructure",
            "location",
            "malware-analysis",
            "note",
            "opinion",
            "language-content",
        ];

        for type_name in v21_sdos {
            self.register_type(TypeInfo {
                type_name: type_name.to_string(),
                category: ObjectCategory::DomainObject,
                spec_versions: vec![SpecVersion::V21],
                parser: None,
                is_custom: false,
                id_contributing_props: None,
                validator: None,
            });
        }

        // SROs
        let sros = ["relationship", "sighting"];
        for type_name in sros {
            self.register_type(TypeInfo {
                type_name: type_name.to_string(),
                category: ObjectCategory::RelationshipObject,
                spec_versions: vec![SpecVersion::V20, SpecVersion::V21],
                parser: None,
                is_custom: false,
                id_contributing_props: None,
                validator: None,
            });
        }

        // SCOs (observables)
        let scos = [
            "artifact",
            "autonomous-system",
            "directory",
            "domain-name",
            "email-addr",
            "email-message",
            "file",
            "ipv4-addr",
            "ipv6-addr",
            "mac-addr",
            "mutex",
            "network-traffic",
            "process",
            "software",
            "url",
            "user-account",
            "windows-registry-key",
            "x509-certificate",
        ];

        for type_name in scos {
            self.register_type(TypeInfo {
                type_name: type_name.to_string(),
                category: ObjectCategory::Observable,
                spec_versions: vec![SpecVersion::V20, SpecVersion::V21],
                parser: None,
                is_custom: false,
                id_contributing_props: None,
                validator: None,
            });
        }

        // Marking definition
        self.register_type(TypeInfo {
            type_name: "marking-definition".to_string(),
            category: ObjectCategory::Marking,
            spec_versions: vec![SpecVersion::V20, SpecVersion::V21],
            parser: None,
            is_custom: false,
            id_contributing_props: None,
            validator: None,
        });
    }

    /// Register a type
    pub fn register_type(&mut self, info: TypeInfo) {
        for version in &info.spec_versions {
            self.types
                .insert((info.type_name.clone(), *version), info.clone());
        }
    }

    /// Get type info
    pub fn get_type(&self, type_name: &str, version: SpecVersion) -> Option<&TypeInfo> {
        self.types.get(&(type_name.to_string(), version))
    }

    /// Check if a type exists
    pub fn has_type(&self, type_name: &str, version: SpecVersion) -> bool {
        self.types.contains_key(&(type_name.to_string(), version))
    }

    /// Get all types for a version
    pub fn types_for_version(&self, version: SpecVersion) -> Vec<&TypeInfo> {
        self.types
            .iter()
            .filter(|((_, v), _)| *v == version)
            .map(|(_, info)| info)
            .collect()
    }

    /// Get types by category
    pub fn types_by_category(
        &self,
        category: ObjectCategory,
        version: SpecVersion,
    ) -> Vec<&TypeInfo> {
        self.types
            .iter()
            .filter(|((_, v), info)| *v == version && info.category == category)
            .map(|(_, info)| info)
            .collect()
    }

    /// Register a custom parser
    pub fn register_custom_parser(&mut self, type_name: &str, parser: ObjectParser) {
        self.custom_parsers.insert(type_name.to_string(), parser);
    }

    /// Get custom parser
    pub fn get_custom_parser(&self, type_name: &str) -> Option<&ObjectParser> {
        self.custom_parsers.get(type_name)
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Public API functions that use the global registry

/// Options for registering a custom type.
#[derive(Debug, Clone, Default)]
pub struct CustomTypeOptions {
    /// Custom parser function
    pub parser: Option<ObjectParser>,
    /// ID-contributing properties for SCOs
    pub id_contributing_props: Option<Vec<String>>,
    /// Custom validator function
    pub validator: Option<ObjectValidator>,
}

/// Register a custom STIX type.
///
/// # Example
///
/// ```rust,ignore
/// use stix2::registry::{register_custom_type, ObjectCategory, SpecVersion};
///
/// register_custom_type(
///     "x-custom-object",
///     ObjectCategory::DomainObject,
///     vec![SpecVersion::V21],
///     None,
/// );
/// ```
pub fn register_custom_type(
    type_name: &str,
    category: ObjectCategory,
    spec_versions: Vec<SpecVersion>,
    options: Option<CustomTypeOptions>,
) -> Result<()> {
    let mut registry = REGISTRY
        .write()
        .map_err(|_| Error::Custom("Failed to acquire registry lock".to_string()))?;

    let opts = options.unwrap_or_default();

    registry.register_type(TypeInfo {
        type_name: type_name.to_string(),
        category,
        spec_versions,
        parser: opts.parser,
        is_custom: true,
        id_contributing_props: opts.id_contributing_props,
        validator: opts.validator,
    });

    Ok(())
}

/// Register a custom parser for a type.
pub fn register_custom_parser(type_name: &str, parser: ObjectParser) -> Result<()> {
    let mut registry = REGISTRY
        .write()
        .map_err(|_| Error::Custom("Failed to acquire registry lock".to_string()))?;

    registry.register_custom_parser(type_name, parser);
    Ok(())
}

/// Get the class/type info for a STIX type.
pub fn class_for_type(type_name: &str, version: SpecVersion) -> Option<TypeInfo> {
    let registry = REGISTRY.read().ok()?;
    registry.get_type(type_name, version).cloned()
}

/// Check if a type is registered.
pub fn is_registered_type(type_name: &str, version: SpecVersion) -> bool {
    if let Ok(registry) = REGISTRY.read() {
        registry.has_type(type_name, version)
    } else {
        false
    }
}

/// Get all registered SDO types.
pub fn get_sdo_types(version: SpecVersion) -> Vec<String> {
    if let Ok(registry) = REGISTRY.read() {
        registry
            .types_by_category(ObjectCategory::DomainObject, version)
            .iter()
            .map(|info| info.type_name.clone())
            .collect()
    } else {
        vec![]
    }
}

/// Get all registered SRO types.
pub fn get_sro_types(version: SpecVersion) -> Vec<String> {
    if let Ok(registry) = REGISTRY.read() {
        registry
            .types_by_category(ObjectCategory::RelationshipObject, version)
            .iter()
            .map(|info| info.type_name.clone())
            .collect()
    } else {
        vec![]
    }
}

/// Get all registered SCO types.
pub fn get_sco_types(version: SpecVersion) -> Vec<String> {
    if let Ok(registry) = REGISTRY.read() {
        registry
            .types_by_category(ObjectCategory::Observable, version)
            .iter()
            .map(|info| info.type_name.clone())
            .collect()
    } else {
        vec![]
    }
}

/// Get all registered types for a version.
pub fn get_all_types(version: SpecVersion) -> Vec<String> {
    if let Ok(registry) = REGISTRY.read() {
        registry
            .types_for_version(version)
            .iter()
            .map(|info| info.type_name.clone())
            .collect()
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_version_from_str() {
        assert_eq!("2.0".parse::<SpecVersion>().unwrap(), SpecVersion::V20);
        assert_eq!("2.1".parse::<SpecVersion>().unwrap(), SpecVersion::V21);
        assert!("3.0".parse::<SpecVersion>().is_err());
    }

    #[test]
    fn test_builtin_types_registered() {
        assert!(is_registered_type("indicator", SpecVersion::V21));
        assert!(is_registered_type("malware", SpecVersion::V21));
        assert!(is_registered_type("relationship", SpecVersion::V21));
        assert!(is_registered_type("file", SpecVersion::V21));
    }

    #[test]
    fn test_v21_only_types() {
        assert!(is_registered_type("grouping", SpecVersion::V21));
        assert!(!is_registered_type("grouping", SpecVersion::V20));
        assert!(is_registered_type("infrastructure", SpecVersion::V21));
        assert!(!is_registered_type("infrastructure", SpecVersion::V20));
    }

    #[test]
    fn test_get_sdo_types() {
        let sdos = get_sdo_types(SpecVersion::V21);
        assert!(sdos.contains(&"indicator".to_string()));
        assert!(sdos.contains(&"malware".to_string()));
        assert!(sdos.contains(&"grouping".to_string()));
    }

    #[test]
    fn test_get_sco_types() {
        let scos = get_sco_types(SpecVersion::V21);
        assert!(scos.contains(&"file".to_string()));
        assert!(scos.contains(&"ipv4-addr".to_string()));
        assert!(scos.contains(&"url".to_string()));
    }

    #[test]
    fn test_class_for_type() {
        let info = class_for_type("indicator", SpecVersion::V21);
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.type_name, "indicator");
        assert_eq!(info.category, ObjectCategory::DomainObject);
    }
}
