//! Property Validation Types
//!
//! This module provides property validation for STIX objects.
//! Each property type has a `clean()` method that returns `CleanResult<T>` with
//! the validated value and a `has_custom` flag.

use crate::core::error::{Error, Result};
use crate::registry::SpecVersion;
use base64::Engine;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Result of cleaning/validating a property value.
#[derive(Debug, Clone)]
pub struct CleanResult<T> {
    /// The cleaned/validated value
    pub value: T,
    /// Whether the value contains custom content
    pub has_custom: bool,
}

impl<T> CleanResult<T> {
    pub fn new(value: T, has_custom: bool) -> Self {
        Self { value, has_custom }
    }

    pub fn ok(value: T) -> Self {
        Self::new(value, false)
    }

    pub fn custom(value: T) -> Self {
        Self::new(value, true)
    }
}

// =============================================================================
// Regex patterns (compiled once)
// =============================================================================

/// UUID format regex for interoperability mode
#[expect(clippy::expect_used, reason = "infallible: valid regex literal")]
static ID_REGEX_INTEROPERABILITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
        .expect("Invalid regex")
});

/// STIX 2.0 type name pattern
#[expect(clippy::expect_used, reason = "infallible: valid regex literal")]
static TYPE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-?[a-z0-9]+(-[a-z0-9]+)*-?$").expect("Invalid regex"));

/// STIX 2.1 type name pattern
#[expect(clippy::expect_used, reason = "infallible: valid regex literal")]
static TYPE_21_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-z][a-z0-9]*)+([a-z0-9-]+)*-?$").expect("Invalid regex"));

/// Dictionary key pattern
#[expect(clippy::expect_used, reason = "infallible: valid regex literal")]
static DICT_KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_-]+$").expect("Invalid regex"));

/// Hex string pattern (even number of hex chars)
#[expect(clippy::expect_used, reason = "infallible: valid regex literal")]
static HEX_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-fA-F0-9]{2})+$").expect("Invalid regex"));

/// Granular marking selector pattern
#[expect(clippy::expect_used, reason = "infallible: valid regex literal")]
static SELECTOR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([a-z0-9_-]{3,250}(\.(\[\d+\]|[a-z0-9_-]{1,250}))*|id)$").expect("Invalid regex")
});

// =============================================================================
// ID/Type Validation
// =============================================================================

/// Check whether the given UUID string is valid with respect to the given STIX
/// spec version. STIX 2.0 requires UUIDv4; 2.1 only requires the RFC 4122 variant.
pub fn check_uuid(uuid_str: &str, spec_version: SpecVersion, interoperability: bool) -> bool {
    if interoperability {
        return ID_REGEX_INTEROPERABILITY.is_match(uuid_str);
    }

    // Parse the UUID
    let uuid_obj = match uuid::Uuid::parse_str(uuid_str) {
        Ok(u) => u,
        Err(_) => return false,
    };

    // Check RFC 4122 variant
    let is_rfc4122 = matches!(uuid_obj.get_variant(), uuid::Variant::RFC4122);
    if !is_rfc4122 {
        return false;
    }

    // STIX 2.0 requires UUIDv4
    if spec_version == SpecVersion::V20 {
        return uuid_obj.get_version_num() == 4;
    }

    true
}

/// Validate a STIX identifier.
pub fn validate_id(
    id: &str,
    spec_version: SpecVersion,
    required_prefix: Option<&str>,
    interoperability: bool,
) -> Result<()> {
    // Check prefix if required
    if let Some(prefix) = required_prefix
        && !id.starts_with(prefix)
    {
        return Err(Error::InvalidId(format!("must start with '{prefix}'")));
    }

    // Extract UUID part
    let uuid_part = if let Some(prefix) = required_prefix {
        &id[prefix.len()..]
    } else {
        match id.find("--") {
            Some(idx) => &id[idx + 2..],
            None => {
                return Err(Error::InvalidId(format!(
                    "not a valid STIX identifier, must match <object-type>--<UUID>: {id}"
                )));
            }
        }
    };

    if !check_uuid(uuid_part, spec_version, interoperability) {
        return Err(Error::InvalidId(format!(
            "not a valid STIX identifier, must match <object-type>--<UUID>: {id}"
        )));
    }

    Ok(())
}

/// Validate a STIX type name.
pub fn validate_type(type_name: &str, spec_version: SpecVersion) -> Result<()> {
    let valid_pattern = if spec_version == SpecVersion::V20 {
        if !TYPE_REGEX.is_match(type_name) {
            return Err(Error::InvalidType(format!(
                "Invalid type name '{type_name}': must only contain the characters a-z (lowercase ASCII), 0-9, and hyphen (-)."
            )));
        }
        true
    } else {
        if !TYPE_21_REGEX.is_match(type_name) {
            return Err(Error::InvalidType(format!(
                "Invalid type name '{type_name}': must only contain the characters a-z (lowercase ASCII), 0-9, and hyphen (-) and must begin with an a-z character"
            )));
        }
        true
    };

    if valid_pattern && (type_name.len() < 3 || type_name.len() > 250) {
        return Err(Error::InvalidType(format!(
            "Invalid type name '{type_name}': must be between 3 and 250 characters."
        )));
    }

    Ok(())
}

// =============================================================================
// Property Trait
// =============================================================================

/// Trait for property validators.
pub trait PropertyValidator<T> {
    /// Clean/validate a value.
    ///
    /// Returns `CleanResult` with the validated value and `has_custom` flag.
    fn clean(&self, value: T, allow_custom: bool, interoperability: bool)
    -> Result<CleanResult<T>>;
}

// =============================================================================
// String Property
// =============================================================================

/// String property validator - converts values to strings.
#[derive(Debug, Clone, Default)]
pub struct StringProperty;

impl StringProperty {
    pub fn new() -> Self {
        Self
    }

    /// Clean a string value.
    pub fn clean(&self, value: &str) -> CleanResult<String> {
        CleanResult::ok(value.to_string())
    }

    /// Clean any value that can be converted to string.
    pub fn clean_any<T: ToString>(&self, value: T) -> CleanResult<String> {
        CleanResult::ok(value.to_string())
    }
}

// =============================================================================
// Integer Property
// =============================================================================

/// Integer property validator with optional min/max bounds.
#[derive(Debug, Clone, Default)]
pub struct IntegerProperty {
    pub min: Option<i64>,
    pub max: Option<i64>,
}

impl IntegerProperty {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bounds(min: Option<i64>, max: Option<i64>) -> Self {
        Self { min, max }
    }

    pub fn min(mut self, min: i64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: i64) -> Self {
        self.max = Some(max);
        self
    }

    /// Clean an integer value with min/max validation.
    pub fn clean(&self, value: i64) -> Result<CleanResult<i64>> {
        if let Some(min) = self.min
            && value < min
        {
            return Err(Error::InvalidPropertyValue {
                property: "integer".to_string(),
                message: format!("minimum value is {min}. received {value}"),
            });
        }

        if let Some(max) = self.max
            && value > max
        {
            return Err(Error::InvalidPropertyValue {
                property: "integer".to_string(),
                message: format!("maximum value is {max}. received {value}"),
            });
        }

        Ok(CleanResult::ok(value))
    }

    /// Try to clean a string value by parsing it as integer.
    pub fn clean_str(&self, value: &str) -> Result<CleanResult<i64>> {
        let parsed: i64 = value.parse().map_err(|_| Error::InvalidPropertyValue {
            property: "integer".to_string(),
            message: "must be an integer.".to_string(),
        })?;
        self.clean(parsed)
    }
}

// =============================================================================
// Float Property
// =============================================================================

/// Float property validator with optional min/max bounds.
#[derive(Debug, Clone, Default)]
pub struct FloatProperty {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl FloatProperty {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bounds(min: Option<f64>, max: Option<f64>) -> Self {
        Self { min, max }
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    /// Clean a float value with min/max validation.
    pub fn clean(&self, value: f64) -> Result<CleanResult<f64>> {
        if let Some(min) = self.min
            && value < min
        {
            return Err(Error::InvalidPropertyValue {
                property: "float".to_string(),
                message: format!("minimum value is {min}. received {value}"),
            });
        }

        if let Some(max) = self.max
            && value > max
        {
            return Err(Error::InvalidPropertyValue {
                property: "float".to_string(),
                message: format!("maximum value is {max}. received {value}"),
            });
        }

        Ok(CleanResult::ok(value))
    }
}

// =============================================================================
// Boolean Property
// =============================================================================

/// Boolean property validator with truthy/falsy string conversion.
///
/// - Truthy values: 'true', 't', '1', 1, True
/// - Falsy values: 'false', 'f', '0', 0, False
#[derive(Debug, Clone, Default)]
pub struct BooleanProperty;

impl BooleanProperty {
    pub fn new() -> Self {
        Self
    }

    /// Clean a boolean value from a string.
    pub fn clean_str(&self, value: &str) -> Result<CleanResult<bool>> {
        let lower = value.to_lowercase();
        match lower.as_str() {
            "true" | "t" | "1" => Ok(CleanResult::ok(true)),
            "false" | "f" | "0" => Ok(CleanResult::ok(false)),
            _ => Err(Error::InvalidPropertyValue {
                property: "boolean".to_string(),
                message: "must be a boolean value.".to_string(),
            }),
        }
    }

    /// Clean an integer value (0 or 1).
    pub fn clean_int(&self, value: i64) -> Result<CleanResult<bool>> {
        match value {
            1 => Ok(CleanResult::ok(true)),
            0 => Ok(CleanResult::ok(false)),
            _ => Err(Error::InvalidPropertyValue {
                property: "boolean".to_string(),
                message: "must be a boolean value.".to_string(),
            }),
        }
    }

    /// Clean an already-boolean value.
    pub fn clean_bool(&self, value: bool) -> CleanResult<bool> {
        CleanResult::ok(value)
    }
}

// =============================================================================
// Dictionary Property
// =============================================================================

/// Dictionary property validator with key length and character validation.
#[derive(Debug, Clone)]
pub struct DictionaryProperty {
    pub spec_version: SpecVersion,
}

impl Default for DictionaryProperty {
    fn default() -> Self {
        Self {
            spec_version: SpecVersion::V21,
        }
    }
}

impl DictionaryProperty {
    pub fn new(spec_version: SpecVersion) -> Self {
        Self { spec_version }
    }

    /// Clean a dictionary value.
    ///
    /// Validates all keys per STIX spec:
    /// - STIX 2.0: keys must be 3-256 characters
    /// - STIX 2.1: keys must be 1-250 characters
    /// - All versions: keys must match `^[a-zA-Z0-9_-]+$`
    pub fn clean<V: Clone>(
        &self,
        value: &HashMap<String, V>,
    ) -> Result<CleanResult<HashMap<String, V>>> {
        if value.is_empty() {
            return Err(Error::InvalidPropertyValue {
                property: "dictionary".to_string(),
                message: "must not be empty.".to_string(),
            });
        }

        for key in value.keys() {
            self.validate_key(key)?;
        }

        Ok(CleanResult::ok(value.clone()))
    }

    /// Validate a single dictionary key.
    pub fn validate_key(&self, key: &str) -> Result<()> {
        match self.spec_version {
            SpecVersion::V20 => {
                if key.len() < 3 {
                    return Err(Error::DictionaryKeyError {
                        key: key.to_string(),
                        reason: "shorter than 3 characters".to_string(),
                    });
                }
                if key.len() > 256 {
                    return Err(Error::DictionaryKeyError {
                        key: key.to_string(),
                        reason: "longer than 256 characters".to_string(),
                    });
                }
            }
            SpecVersion::V21 => {
                if key.len() > 250 {
                    return Err(Error::DictionaryKeyError {
                        key: key.to_string(),
                        reason: "longer than 250 characters".to_string(),
                    });
                }
            }
        }

        if !DICT_KEY_REGEX.is_match(key) {
            return Err(Error::DictionaryKeyError {
                key: key.to_string(),
                reason: "contains characters other than lowercase a-z, uppercase A-Z, numerals 0-9, hyphen (-), or underscore (_)".to_string(),
            });
        }

        Ok(())
    }
}

// =============================================================================
// Binary Property
// =============================================================================

/// Binary property validator for base64-encoded data.
#[derive(Debug, Clone, Default)]
pub struct BinaryProperty;

impl BinaryProperty {
    pub fn new() -> Self {
        Self
    }

    /// Clean a base64-encoded value.
    ///
    /// Validates that the string is valid base64.
    pub fn clean(&self, value: &str) -> Result<CleanResult<String>> {
        base64::engine::general_purpose::STANDARD
            .decode(value)
            .map_err(|_| Error::InvalidPropertyValue {
                property: "binary".to_string(),
                message: "must contain a base64 encoded string".to_string(),
            })?;

        Ok(CleanResult::ok(value.to_string()))
    }
}

// =============================================================================
// Hex Property
// =============================================================================

/// Hex property validator for hexadecimal strings.
#[derive(Debug, Clone, Default)]
pub struct HexProperty;

impl HexProperty {
    pub fn new() -> Self {
        Self
    }

    /// Clean a hexadecimal value.
    ///
    /// Validates that the string contains an even number of hex characters.
    pub fn clean(&self, value: &str) -> Result<CleanResult<String>> {
        if !HEX_REGEX.is_match(value) {
            return Err(Error::InvalidPropertyValue {
                property: "hex".to_string(),
                message: "must contain an even number of hexadecimal characters".to_string(),
            });
        }
        Ok(CleanResult::ok(value.to_string()))
    }
}

// =============================================================================
// Selector Property
// =============================================================================

/// Selector property validator for granular marking selectors.
#[derive(Debug, Clone, Default)]
pub struct SelectorProperty;

impl SelectorProperty {
    pub fn new() -> Self {
        Self
    }

    /// Clean a selector value.
    ///
    /// Validates that the string matches the selector syntax.
    pub fn clean(&self, value: &str) -> Result<CleanResult<String>> {
        if !SELECTOR_REGEX.is_match(value) {
            return Err(Error::InvalidPropertyValue {
                property: "selector".to_string(),
                message: "must adhere to selector syntax.".to_string(),
            });
        }
        Ok(CleanResult::ok(value.to_string()))
    }
}

// =============================================================================
// Enum Property
// =============================================================================

/// Enum property validator for closed vocabularies (no customization allowed).
#[derive(Debug, Clone)]
pub struct EnumProperty {
    pub allowed: Vec<String>,
}

impl EnumProperty {
    pub fn new(allowed: Vec<String>) -> Self {
        Self { allowed }
    }

    pub fn from_strs(allowed: &[&str]) -> Self {
        Self {
            allowed: allowed.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Clean an enum value.
    ///
    /// Validates that the value is in the allowed list.
    pub fn clean(&self, value: &str) -> Result<CleanResult<String>> {
        if !self.allowed.contains(&value.to_string()) {
            return Err(Error::InvalidPropertyValue {
                property: "enum".to_string(),
                message: format!("value '{value}' is not valid for this enumeration."),
            });
        }
        Ok(CleanResult::ok(value.to_string()))
    }
}

// =============================================================================
// Open Vocab Property
// =============================================================================

/// Open vocabulary property validator - accepts any value but tracks custom ones.
///
/// Note: Customization detection is currently disabled (always returns has_custom=false).
#[derive(Debug, Clone)]
pub struct OpenVocabProperty {
    pub allowed: Vec<String>,
}

impl OpenVocabProperty {
    pub fn new(allowed: Vec<String>) -> Self {
        Self { allowed }
    }

    pub fn from_strs(allowed: &[&str]) -> Self {
        Self {
            allowed: allowed.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Clean an open vocab value.
    ///
    /// Customization detection is disabled as enforcing it is too strict.
    pub fn clean(&self, value: &str, _allow_custom: bool) -> Result<CleanResult<String>> {
        Ok(CleanResult::ok(value.to_string()))
    }

    /// Check if a value is in the allowed list (for informational purposes).
    pub fn is_standard(&self, value: &str) -> bool {
        self.allowed.contains(&value.to_string())
    }
}

// =============================================================================
// ID Property
// =============================================================================

/// ID property validator for STIX identifiers.
#[derive(Debug, Clone)]
pub struct IdProperty {
    pub required_prefix: String,
    pub spec_version: SpecVersion,
}

impl IdProperty {
    pub fn new(type_name: &str, spec_version: SpecVersion) -> Self {
        Self {
            required_prefix: format!("{type_name}--"),
            spec_version,
        }
    }

    /// Clean an ID value.
    pub fn clean(&self, value: &str, interoperability: bool) -> Result<CleanResult<String>> {
        validate_id(
            value,
            self.spec_version,
            Some(&self.required_prefix),
            interoperability,
        )?;
        Ok(CleanResult::ok(value.to_string()))
    }

    /// Generate a default ID.
    pub fn default(&self) -> String {
        format!("{}{}", self.required_prefix, uuid::Uuid::new_v4())
    }
}

// =============================================================================
// Type Property
// =============================================================================

/// Type property validator (fixed value).
#[derive(Debug, Clone)]
pub struct TypeProperty {
    pub fixed_value: String,
    pub spec_version: SpecVersion,
}

impl TypeProperty {
    pub fn new(type_name: &str, spec_version: SpecVersion) -> Result<Self> {
        validate_type(type_name, spec_version)?;
        Ok(Self {
            fixed_value: type_name.to_string(),
            spec_version,
        })
    }

    /// Clean a type value.
    pub fn clean(&self, value: &str) -> Result<CleanResult<String>> {
        if value != self.fixed_value {
            return Err(Error::InvalidPropertyValue {
                property: "type".to_string(),
                message: format!("must equal '{}'.", self.fixed_value),
            });
        }
        Ok(CleanResult::ok(value.to_string()))
    }
}

// =============================================================================
// Reference Property
// =============================================================================

/// Reference property validator for STIX object references with whitelist/blacklist support.
#[derive(Debug, Clone)]
pub struct ReferenceProperty {
    pub spec_version: SpecVersion,
    pub valid_types: Option<Vec<String>>,
    pub invalid_types: Option<Vec<String>>,
}

impl ReferenceProperty {
    /// Create a reference property with a whitelist of valid types.
    pub fn with_valid_types(types: Vec<String>, spec_version: SpecVersion) -> Result<Self> {
        if types.is_empty() {
            return Err(Error::Custom(
                "Impossible type constraint: empty whitelist".to_string(),
            ));
        }
        Ok(Self {
            spec_version,
            valid_types: Some(types),
            invalid_types: None,
        })
    }

    /// Create a reference property with a blacklist of invalid types.
    pub fn with_invalid_types(types: Vec<String>, spec_version: SpecVersion) -> Self {
        Self {
            spec_version,
            valid_types: None,
            invalid_types: Some(types),
        }
    }

    /// Create a reference property that accepts any type.
    pub fn any(spec_version: SpecVersion) -> Self {
        Self {
            spec_version,
            valid_types: None,
            invalid_types: Some(vec![]),
        }
    }

    /// Clean a reference value.
    pub fn clean(
        &self,
        value: &str,
        allow_custom: bool,
        interoperability: bool,
    ) -> Result<CleanResult<String>> {
        // Validate the ID format
        validate_id(value, self.spec_version, None, interoperability)?;

        // Extract the type from the ID
        let obj_type = value
            .split("--")
            .next()
            .ok_or_else(|| Error::InvalidId(format!("Invalid reference format: {value}")))?;

        // Check if it's a custom type (starts with x- or is unregistered)
        let has_custom = obj_type.starts_with("x-")
            || !crate::registry::is_registered_type(obj_type, self.spec_version);

        // Validate against whitelist/blacklist
        if let Some(valid_types) = &self.valid_types
            && !(valid_types.contains(&obj_type.to_string()) || (allow_custom && has_custom))
        {
            return Err(Error::InvalidPropertyValue {
                property: "reference".to_string(),
                message: format!(
                    "The type-specifying prefix '{}' for this property is not one of the valid types: {}",
                    obj_type,
                    valid_types.join(", ")
                ),
            });
        }

        if let Some(invalid_types) = &self.invalid_types
            && invalid_types.contains(&obj_type.to_string())
        {
            return Err(Error::InvalidPropertyValue {
                property: "reference".to_string(),
                message: format!(
                    "The type-specifying prefix '{}' for this property is one of the invalid types: {}",
                    obj_type,
                    invalid_types.join(", ")
                ),
            });
        }

        // Check custom content
        if !allow_custom && has_custom {
            return Err(Error::CustomContentError(format!(
                "reference to custom object type: {obj_type}"
            )));
        }

        Ok(CleanResult::new(value.to_string(), has_custom))
    }
}

// =============================================================================
// List Property
// =============================================================================

/// Validate a list of items using a provided validator.
///
/// # Example
///
/// ```rust,ignore
/// let result = validate_list(&items, allow_custom, |item, allow_custom| {
///     StringProperty::new().clean(item)
/// })?;
/// ```
pub fn validate_list<T, F>(
    items: &[T],
    allow_custom: bool,
    validator: F,
) -> Result<CleanResult<Vec<T>>>
where
    T: Clone,
    F: Fn(&T, bool) -> Result<CleanResult<T>>,
{
    if items.is_empty() {
        return Err(Error::InvalidPropertyValue {
            property: "list".to_string(),
            message: "must not be empty.".to_string(),
        });
    }

    let mut result = Vec::with_capacity(items.len());
    let mut has_custom = false;

    for item in items {
        let cleaned = validator(item, allow_custom)?;
        has_custom = has_custom || cleaned.has_custom;
        result.push(cleaned.value);
    }

    if !allow_custom && has_custom {
        return Err(Error::CustomContentError(
            "custom content encountered".to_string(),
        ));
    }

    Ok(CleanResult::new(result, has_custom))
}

// =============================================================================
// Hash Validation
// =============================================================================

/// Standard hash algorithm names from STIX specification.
pub const STIX_HASH_ALGORITHMS: &[&str] = &[
    "MD5", "SHA-1", "SHA-256", "SHA-512", "SHA3-256", "SHA3-512", "SSDEEP", "TLSH",
];

/// Hash value length validation per algorithm.
pub fn validate_hash_value(algorithm: &str, value: &str) -> Result<()> {
    let expected_len = match algorithm.to_uppercase().as_str() {
        "MD5" => Some(32),
        "SHA-1" => Some(40),
        "SHA-256" => Some(64),
        "SHA-512" => Some(128),
        "SHA3-256" => Some(64),
        "SHA3-512" => Some(128),
        _ => None, // SSDEEP, TLSH, and custom algorithms have variable lengths
    };

    if let Some(len) = expected_len {
        if value.len() != len {
            return Err(Error::InvalidPropertyValue {
                property: "hashes".to_string(),
                message: format!(
                    "'{}' is not a valid {} hash (expected {} characters, got {})",
                    value,
                    algorithm,
                    len,
                    value.len()
                ),
            });
        }

        // Validate hex characters
        if !value.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::InvalidPropertyValue {
                property: "hashes".to_string(),
                message: format!("'{value}' is not a valid {algorithm} hash"),
            });
        }
    }

    Ok(())
}

/// Validate and clean a hashes dictionary.
pub fn validate_hashes(
    hashes: &HashMap<String, String>,
    allow_custom: bool,
    spec_hash_names: &[&str],
) -> Result<CleanResult<HashMap<String, String>>> {
    let dict_prop = DictionaryProperty::new(SpecVersion::V21);

    // Validate as dictionary first
    for key in hashes.keys() {
        dict_prop.validate_key(key)?;
    }

    let mut result = HashMap::new();
    let mut has_custom = false;

    for (hash_key, hash_value) in hashes {
        // Check if it's a known algorithm
        let is_known = spec_hash_names.contains(&hash_key.as_str());

        if !is_known {
            has_custom = true;
            if !allow_custom {
                return Err(Error::CustomContentError(format!(
                    "custom hash algorithm: {hash_key}"
                )));
            }
        }

        // Validate hash value for known algorithms
        if is_known {
            validate_hash_value(hash_key, hash_value)?;
        }

        result.insert(hash_key.clone(), hash_value.clone());
    }

    Ok(CleanResult::new(result, has_custom))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_uuid_v4() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(check_uuid(uuid, SpecVersion::V21, false));
        assert!(check_uuid(uuid, SpecVersion::V20, false));
    }

    #[test]
    fn test_check_uuid_interoperability() {
        // Any format works in interoperability mode
        let uuid = "550e8400-e29b-11d4-a716-446655440000"; // v1 UUID
        assert!(check_uuid(uuid, SpecVersion::V21, true));
    }

    #[test]
    fn test_validate_type_2_0() {
        assert!(validate_type("indicator", SpecVersion::V20).is_ok());
        assert!(validate_type("x-custom-type", SpecVersion::V20).is_ok());
        assert!(validate_type("ab", SpecVersion::V20).is_err()); // Too short
    }

    #[test]
    fn test_validate_type_2_1() {
        assert!(validate_type("indicator", SpecVersion::V21).is_ok());
        assert!(validate_type("x-custom-type", SpecVersion::V21).is_ok());
        // 2.1 requires starting with alpha
        assert!(validate_type("1indicator", SpecVersion::V21).is_err());
    }

    #[test]
    fn test_boolean_property() {
        let prop = BooleanProperty::new();

        assert!(prop.clean_str("true").unwrap().value);
        assert!(prop.clean_str("True").unwrap().value);
        assert!(prop.clean_str("TRUE").unwrap().value);
        assert!(prop.clean_str("t").unwrap().value);
        assert!(prop.clean_str("1").unwrap().value);

        assert!(!prop.clean_str("false").unwrap().value);
        assert!(!prop.clean_str("False").unwrap().value);
        assert!(!prop.clean_str("f").unwrap().value);
        assert!(!prop.clean_str("0").unwrap().value);

        assert!(prop.clean_str("invalid").is_err());
    }

    #[test]
    fn test_integer_property() {
        let prop = IntegerProperty::new().min(0).max(100);

        assert_eq!(prop.clean(50).unwrap().value, 50);
        assert!(prop.clean(-1).is_err());
        assert!(prop.clean(101).is_err());
    }

    #[test]
    fn test_dictionary_property_key_validation() {
        let prop = DictionaryProperty::new(SpecVersion::V21);

        assert!(prop.validate_key("valid_key").is_ok());
        assert!(prop.validate_key("valid-key").is_ok());
        assert!(prop.validate_key("ValidKey123").is_ok());

        // Invalid characters
        assert!(prop.validate_key("invalid.key").is_err());
        assert!(prop.validate_key("invalid key").is_err());
    }

    #[test]
    fn test_hex_property() {
        let prop = HexProperty::new();

        assert!(prop.clean("deadbeef").is_ok());
        assert!(prop.clean("DEADBEEF").is_ok());
        assert!(prop.clean("0123456789abcdef").is_ok());

        // Odd number of characters
        assert!(prop.clean("abc").is_err());
        // Invalid characters
        assert!(prop.clean("ghij").is_err());
    }

    #[test]
    fn test_selector_property() {
        let prop = SelectorProperty::new();

        assert!(prop.clean("id").is_ok());
        assert!(prop.clean("pattern").is_ok());
        assert!(prop.clean("objects.[0]").is_ok());
        assert!(prop.clean("extensions.ext-name.property").is_ok());

        assert!(prop.clean("").is_err());
        assert!(prop.clean("invalid syntax!").is_err());
    }

    #[test]
    fn test_enum_property() {
        let prop = EnumProperty::from_strs(&["red", "green", "blue"]);

        assert!(prop.clean("red").is_ok());
        assert!(prop.clean("green").is_ok());
        assert!(prop.clean("yellow").is_err());
    }

    #[test]
    fn test_validate_list() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = validate_list(&items, false, |item, _| Ok(CleanResult::ok(item.clone())));
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap().value,
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );

        // Empty list should fail
        let empty: Vec<String> = vec![];
        let result = validate_list(&empty, false, |item, _| Ok(CleanResult::ok(item.clone())));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_hash_value() {
        // Valid MD5
        assert!(validate_hash_value("MD5", "d41d8cd98f00b204e9800998ecf8427e").is_ok());

        // Invalid MD5 (wrong length)
        assert!(validate_hash_value("MD5", "abc").is_err());

        // Valid SHA-256
        assert!(
            validate_hash_value(
                "SHA-256",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            )
            .is_ok()
        );
    }
}
