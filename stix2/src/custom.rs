//! Custom STIX Object System
//!
//! This module provides macros for defining custom STIX object types.
//!
//! # Overview
//!
//! STIX allows for extending the standard with custom object types.
//! Custom types must have names that begin with `x-` (e.g., `x-my-custom-object`).
//!
//! # Examples
//!
//! ## Custom SDO (Domain Object)
//!
//! ```rust,ignore
//! use stix2::define_custom_object;
//!
//! define_custom_object!(MyCustomObject, "x-my-custom-object", {
//!     required {
//!         name: String,
//!     }
//!     optional {
//!         description: Option<String>,
//!         score: Option<i64>,
//!     }
//! });
//! ```
//!
//! ## Custom SCO (Observable)
//!
//! ```rust,ignore
//! use stix2::define_custom_observable;
//!
//! define_custom_observable!(MyCustomObservable, "x-my-observable", {
//!     id_contributing: [value],
//!     required {
//!         value: String,
//!     }
//!     optional {
//!         metadata: Option<String>,
//!     }
//! });
//! ```

use crate::core::error::{Error, Result};
use crate::registry::{
    CustomTypeOptions, ObjectCategory, SpecVersion, class_for_type, register_custom_type,
};

/// Validates that a custom type name follows STIX conventions.
///
/// Custom type names must:
/// - Start with `x-` for custom types
/// - OR be a registered extension definition ID (`extension-definition--<UUID>`)
/// - Contain only lowercase letters, numbers, and hyphens
///
/// # Examples
///
/// ```rust
/// use stix2::custom::validate_custom_type_name;
///
/// assert!(validate_custom_type_name("x-my-custom-type").is_ok());
/// assert!(validate_custom_type_name("x-acme-threat-score").is_ok());
/// assert!(validate_custom_type_name("invalid-type").is_err());
/// ```
pub fn validate_custom_type_name(type_name: &str) -> Result<()> {
    // Must start with x- for custom types, or be an extension definition
    if !type_name.starts_with("x-") && !type_name.starts_with("extension-definition--") {
        return Err(Error::InvalidType(format!(
            "Custom type name '{type_name}' must start with 'x-' or 'extension-definition--'"
        )));
    }

    // Validate characters (lowercase, numbers, hyphens only)
    let valid_chars = type_name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');

    if !valid_chars {
        return Err(Error::InvalidType(format!(
            "Custom type name '{type_name}' contains invalid characters. \
             Use only lowercase letters, numbers, and hyphens."
        )));
    }

    Ok(())
}

/// Validates that a custom extension type name follows STIX 2.1 conventions.
///
/// Extension type names must:
/// - End with `-ext` for standard extensions
/// - OR start with `extension-definition--<UUID>` for extension definitions
///
/// # Examples
///
/// ```rust
/// use stix2::custom::validate_extension_type_name;
///
/// assert!(validate_extension_type_name("x-acme-ext").is_ok());
/// assert!(validate_extension_type_name("extension-definition--a1a2a3a4-b1b2-c1c2-d1d2-e1e2e3e4e5e6").is_ok());
/// assert!(validate_extension_type_name("invalid-extension").is_err());
/// ```
pub fn validate_extension_type_name(type_name: &str) -> Result<()> {
    if !type_name.ends_with("-ext") && !type_name.starts_with("extension-definition--") {
        return Err(Error::InvalidType(format!(
            "Extension type name '{type_name}' must end with '-ext' or start with 'extension-definition--'"
        )));
    }

    Ok(())
}

/// Check if a type is already registered.
///
/// Returns an error if the type is already registered to prevent duplicates.
pub fn check_not_registered(type_name: &str, version: SpecVersion) -> Result<()> {
    if class_for_type(type_name, version).is_some() {
        return Err(Error::DuplicateType(format!(
            "Type '{}' is already registered for STIX {}",
            type_name,
            version.as_str()
        )));
    }
    Ok(())
}

/// Register a custom SDO type.
///
/// This is the lower-level API for registering custom domain objects.
/// Most users should prefer the `define_custom_object!` macro.
///
/// # Arguments
///
/// * `type_name` - The STIX type name (must start with `x-`)
/// * `versions` - Which STIX versions to register for
/// * `validator` - Optional validation function
///
/// # Example
///
/// ```rust,ignore
/// use stix2::custom::register_custom_sdo;
/// use stix2::registry::SpecVersion;
///
/// register_custom_sdo("x-my-type", vec![SpecVersion::V21], None)?;
/// ```
pub fn register_custom_sdo(
    type_name: &str,
    versions: Vec<SpecVersion>,
    validator: Option<fn(&serde_json::Value) -> Result<()>>,
) -> Result<()> {
    validate_custom_type_name(type_name)?;

    for version in &versions {
        check_not_registered(type_name, *version)?;
    }

    register_custom_type(
        type_name,
        ObjectCategory::DomainObject,
        versions,
        Some(CustomTypeOptions {
            parser: None,
            id_contributing_props: None,
            validator,
        }),
    )
}

/// Register a custom SCO type.
///
/// This is the lower-level API for registering custom observable objects.
/// Most users should prefer the `define_custom_observable!` macro.
///
/// # Arguments
///
/// * `type_name` - The STIX type name (must start with `x-`)
/// * `versions` - Which STIX versions to register for
/// * `id_contributing_props` - Properties that contribute to the deterministic ID
/// * `validator` - Optional validation function
///
/// # Example
///
/// ```rust,ignore
/// use stix2::custom::register_custom_sco;
/// use stix2::registry::SpecVersion;
///
/// register_custom_sco(
///     "x-my-observable",
///     vec![SpecVersion::V21],
///     Some(vec!["value".to_string()]),
///     None,
/// )?;
/// ```
pub fn register_custom_sco(
    type_name: &str,
    versions: Vec<SpecVersion>,
    id_contributing_props: Option<Vec<String>>,
    validator: Option<fn(&serde_json::Value) -> Result<()>>,
) -> Result<()> {
    validate_custom_type_name(type_name)?;

    for version in &versions {
        check_not_registered(type_name, *version)?;
    }

    register_custom_type(
        type_name,
        ObjectCategory::Observable,
        versions,
        Some(CustomTypeOptions {
            parser: None,
            id_contributing_props,
            validator,
        }),
    )
}

/// Register a custom extension type.
///
/// # Arguments
///
/// * `type_name` - The extension type name (must end with `-ext` or start with `extension-definition--`)
/// * `versions` - Which STIX versions to register for
/// * `validator` - Optional validation function
pub fn register_custom_extension(
    type_name: &str,
    versions: Vec<SpecVersion>,
    validator: Option<fn(&serde_json::Value) -> Result<()>>,
) -> Result<()> {
    validate_extension_type_name(type_name)?;

    for version in &versions {
        check_not_registered(type_name, *version)?;
    }

    register_custom_type(
        type_name,
        ObjectCategory::Extension,
        versions,
        Some(CustomTypeOptions {
            parser: None,
            id_contributing_props: None,
            validator,
        }),
    )
}

/// Register a custom marking definition type.
///
/// # Arguments
///
/// * `type_name` - The marking type name (must start with `x-`)
/// * `versions` - Which STIX versions to register for
/// * `validator` - Optional validation function
pub fn register_custom_marking(
    type_name: &str,
    versions: Vec<SpecVersion>,
    validator: Option<fn(&serde_json::Value) -> Result<()>>,
) -> Result<()> {
    validate_custom_type_name(type_name)?;

    for version in &versions {
        check_not_registered(type_name, *version)?;
    }

    register_custom_type(
        type_name,
        ObjectCategory::Marking,
        versions,
        Some(CustomTypeOptions {
            parser: None,
            id_contributing_props: None,
            validator,
        }),
    )
}

/// Define a custom STIX Domain Object (SDO).
///
/// This macro generates a struct with the appropriate STIX properties
/// and registers it with the type registry.
///
/// # Examples
///
/// ```rust,ignore
/// use stix2::define_custom_object;
///
/// define_custom_object! {
///     /// A custom threat score object.
///     pub struct ThreatScore("x-threat-score") {
///         /// The name of the threat.
///         pub name: String,
///         /// Description of the threat.
///         #[serde(skip_serializing_if = "Option::is_none")]
///         pub description: Option<String>,
///         /// Numeric score from 0-100.
///         pub score: u8,
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_custom_object {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ($type_str:literal) {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $field_ty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        $vis struct $name {
            /// The STIX type identifier.
            #[serde(rename = "type")]
            pub type_: String,

            /// Unique identifier for this object.
            pub id: $crate::core::id::Identifier,

            /// Common properties shared by all SDOs.
            #[serde(flatten)]
            pub common: $crate::core::common::CommonProperties,

            $(
                $(#[$field_meta])*
                $field_vis $field: $field_ty,
            )*
        }

        impl $name {
            /// The STIX type identifier for this custom object.
            pub const TYPE: &'static str = $type_str;

            /// Create a new instance with required fields.
            pub fn new() -> $crate::core::error::Result<Self> {
                Ok(Self {
                    type_: $type_str.to_string(),
                    id: $crate::core::id::Identifier::new($type_str)?,
                    common: $crate::core::common::CommonProperties::default(),
                    $(
                        $field: Default::default(),
                    )*
                })
            }

            /// Register this custom type with the global registry.
            pub fn register() -> $crate::core::error::Result<()> {
                $crate::custom::register_custom_sdo(
                    $type_str,
                    vec![$crate::registry::SpecVersion::V21],
                    None,
                )
            }
        }

        impl $crate::validation::Constrained for $name {
            fn validate_constraints(&self) -> $crate::core::error::Result<()> {
                Ok(())
            }
        }
    };
}

/// Define a custom STIX Cyber Observable (SCO).
///
/// This macro generates a struct with the appropriate STIX observable properties
/// and registers it with the type registry, including ID-contributing properties.
///
/// # Examples
///
/// ```rust,ignore
/// use stix2::define_custom_observable;
///
/// define_custom_observable! {
///     /// A custom network sensor reading.
///     pub struct NetworkSensor("x-network-sensor") {
///         id_contributing: [sensor_id, reading_type],
///
///         /// The unique sensor identifier.
///         pub sensor_id: String,
///         /// Type of reading (e.g., "temperature", "bandwidth").
///         pub reading_type: String,
///         /// The sensor reading value.
///         pub value: f64,
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_custom_observable {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ($type_str:literal) {
            id_contributing: [$($id_prop:ident),* $(,)?],

            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $field_ty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        $vis struct $name {
            /// The STIX type identifier.
            #[serde(rename = "type")]
            pub type_: String,

            /// Unique identifier for this observable.
            pub id: $crate::core::id::Identifier,

            /// The STIX specification version.
            #[serde(default = "default_spec_version")]
            pub spec_version: String,

            /// Object marking references.
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            pub object_marking_refs: Vec<$crate::core::id::Identifier>,

            /// Granular markings.
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            pub granular_markings: Vec<$crate::markings::GranularMarking>,

            /// Indicates the object has been revoked.
            #[serde(default, skip_serializing_if = "std::ops::Not::not")]
            pub defanged: bool,

            $(
                $(#[$field_meta])*
                $field_vis $field: $field_ty,
            )*
        }

        fn default_spec_version() -> String {
            "2.1".to_string()
        }

        impl $name {
            /// The STIX type identifier for this custom observable.
            pub const TYPE: &'static str = $type_str;

            /// Properties that contribute to the deterministic ID.
            pub const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] = &[
                $(stringify!($id_prop)),*
            ];

            /// Register this custom type with the global registry.
            pub fn register() -> $crate::core::error::Result<()> {
                $crate::custom::register_custom_sco(
                    $type_str,
                    vec![$crate::registry::SpecVersion::V21],
                    Some(vec![$(stringify!($id_prop).to_string()),*]),
                    None,
                )
            }
        }

        impl $crate::validation::Constrained for $name {
            fn validate_constraints(&self) -> $crate::core::error::Result<()> {
                Ok(())
            }
        }
    };
}

/// Define a custom STIX Extension.
///
/// # Examples
///
/// ```rust,ignore
/// use stix2::define_custom_extension;
///
/// define_custom_extension! {
///     /// Custom extension for threat intelligence scores.
///     pub struct ThreatScoreExt("x-threat-score-ext") {
///         extension_type: "property-extension",
///
///         /// The threat score (0-100).
///         pub score: u8,
///         /// Confidence in the score.
///         #[serde(skip_serializing_if = "Option::is_none")]
///         pub confidence: Option<u8>,
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_custom_extension {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident ($type_str:literal) {
            extension_type: $ext_type:literal,

            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $field_ty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
        $vis struct $name {
            /// The extension type.
            pub extension_type: String,

            $(
                $(#[$field_meta])*
                $field_vis $field: $field_ty,
            )*
        }

        impl $name {
            /// The STIX type identifier for this extension.
            pub const TYPE: &'static str = $type_str;

            /// The extension type (e.g., "property-extension", "new-sdo").
            pub const EXTENSION_TYPE: &'static str = $ext_type;

            /// Create a new extension instance.
            pub fn new() -> Self {
                Self {
                    extension_type: $ext_type.to_string(),
                    $(
                        $field: Default::default(),
                    )*
                }
            }

            /// Register this extension with the global registry.
            pub fn register() -> $crate::core::error::Result<()> {
                $crate::custom::register_custom_extension(
                    $type_str,
                    vec![$crate::registry::SpecVersion::V21],
                    None,
                )
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_custom_type_name() {
        // Valid custom type names
        assert!(validate_custom_type_name("x-custom-type").is_ok());
        assert!(validate_custom_type_name("x-my-org-threat-score").is_ok());
        assert!(validate_custom_type_name("x-acme-widget-123").is_ok());
        assert!(
            validate_custom_type_name("extension-definition--a1a2a3a4-b1b2-c1c2-d1d2-e1e2e3e4e5e6")
                .is_ok()
        );

        // Invalid custom type names
        assert!(validate_custom_type_name("custom-type").is_err());
        assert!(validate_custom_type_name("indicator").is_err());
        assert!(validate_custom_type_name("x-UPPERCASE").is_err());
        assert!(validate_custom_type_name("x-invalid_underscore").is_err());
    }

    #[test]
    fn test_validate_extension_type_name() {
        // Valid extension names
        assert!(validate_extension_type_name("x-acme-ext").is_ok());
        assert!(validate_extension_type_name("new-sdo-ext").is_ok());
        assert!(
            validate_extension_type_name(
                "extension-definition--a1a2a3a4-b1b2-c1c2-d1d2-e1e2e3e4e5e6"
            )
            .is_ok()
        );

        // Invalid extension names
        assert!(validate_extension_type_name("x-acme-extension").is_err());
        assert!(validate_extension_type_name("invalid").is_err());
    }

    #[test]
    fn test_register_custom_sdo() {
        // Register a test custom type
        let result = register_custom_sdo("x-test-custom-sdo-001", vec![SpecVersion::V21], None);
        assert!(result.is_ok());

        // Should fail on duplicate registration
        let result = register_custom_sdo("x-test-custom-sdo-001", vec![SpecVersion::V21], None);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_custom_sco() {
        let result = register_custom_sco(
            "x-test-custom-sco-001",
            vec![SpecVersion::V21],
            Some(vec!["value".to_string()]),
            None,
        );
        assert!(result.is_ok());

        // Check it was registered
        let info = class_for_type("x-test-custom-sco-001", SpecVersion::V21);
        assert!(info.is_some());
        let info = info.unwrap();
        assert!(info.is_custom);
        assert_eq!(info.id_contributing_props, Some(vec!["value".to_string()]));
    }

    #[test]
    fn test_register_custom_extension() {
        let result = register_custom_extension("x-test-custom-ext", vec![SpecVersion::V21], None);
        assert!(result.is_ok());

        // Invalid extension name should fail
        let result = register_custom_extension("x-invalid-extension", vec![SpecVersion::V21], None);
        assert!(result.is_err());
    }

    // Test the macros
    define_custom_object! {
        /// Test custom object for unit tests.
        pub struct TestCustomObject("x-test-macro-object") {
            /// Test name field.
            pub name: String,
            /// Optional description.
            #[serde(skip_serializing_if = "Option::is_none")]
            pub description: Option<String>,
        }
    }

    #[test]
    fn test_define_custom_object_macro() {
        // Test the TYPE constant
        assert_eq!(TestCustomObject::TYPE, "x-test-macro-object");

        // Test registration works
        assert!(TestCustomObject::register().is_ok());

        let obj = TestCustomObject::new().unwrap();
        assert_eq!(obj.type_, TestCustomObject::TYPE);
        assert!(obj.id.to_string().starts_with("x-test-macro-object--"));

        // Test serialization
        let json = serde_json::to_string(&obj).unwrap();
        assert!(json.contains("\"type\":\"x-test-macro-object\""));
    }

    define_custom_observable! {
        /// Test custom observable for unit tests.
        pub struct TestCustomObservable("x-test-macro-observable") {
            id_contributing: [sensor_id],

            /// The sensor identifier.
            pub sensor_id: String,
            /// The reading value.
            pub value: f64,
        }
    }

    #[test]
    fn test_define_custom_observable_macro() {
        assert_eq!(TestCustomObservable::TYPE, "x-test-macro-observable");
        assert_eq!(
            TestCustomObservable::ID_CONTRIBUTING_PROPERTIES,
            &["sensor_id"]
        );

        // Test registration works
        assert!(TestCustomObservable::register().is_ok());
    }

    define_custom_extension! {
        /// Test custom extension for unit tests.
        pub struct TestCustomExtension("x-test-ext") {
            extension_type: "property-extension",

            /// Test score field.
            pub score: u8,
        }
    }

    #[test]
    fn test_define_custom_extension_macro() {
        // Test the constants
        assert_eq!(TestCustomExtension::TYPE, "x-test-ext");
        assert_eq!(TestCustomExtension::EXTENSION_TYPE, "property-extension");

        // Test registration works
        assert!(TestCustomExtension::register().is_ok());

        let ext = TestCustomExtension::new();
        assert_eq!(ext.extension_type, TestCustomExtension::EXTENSION_TYPE);
    }
}
