//! Constraint Validation Functions
//!
//! This module provides constraint checking functions for STIX object validation.

use crate::core::error::{Error, Result};
use crate::core::timestamp::Timestamp;

/// Check that at most one (or exactly one) of the specified properties is set.
///
/// # Arguments
///
/// * `present_props` - List of property names that are currently present/set
/// * `exclusive_props` - List of property names that are mutually exclusive
/// * `at_least_one` - If true, exactly one must be present (XOR). If false, at most one (optional XOR).
///
/// # Errors
///
/// Returns `Error::MutuallyExclusiveProperties` if more than one is present.
/// Returns `Error::AtLeastOneRequired` if `at_least_one=true` and none are present.
///
/// # Example
///
/// ```rust,ignore
/// let present = vec!["lang"];
/// check_mutually_exclusive(&present, &["lang", "marking_ref"], true)?;
/// ```
pub fn check_mutually_exclusive(
    present_props: &[&str],
    exclusive_props: &[&str],
    at_least_one: bool,
) -> Result<()> {
    // Count how many of the exclusive properties are present
    let present_count = exclusive_props
        .iter()
        .filter(|p| present_props.contains(p))
        .count();

    if present_count > 1 {
        return Err(Error::MutuallyExclusiveProperties(
            exclusive_props.iter().map(|s| s.to_string()).collect(),
        ));
    }

    if at_least_one && present_count == 0 {
        return Err(Error::AtLeastOneRequired(
            exclusive_props.iter().map(|s| s.to_string()).collect(),
        ));
    }

    Ok(())
}

/// Check that at least one of the specified properties is set.
///
/// # Arguments
///
/// * `present_props` - List of property names that are currently present/set
/// * `required_props` - List of property names where at least one must be present
///
/// # Errors
///
/// Returns `Error::AtLeastOneRequired` if none of the required properties are present.
///
/// # Example
///
/// ```rust,ignore
/// let present = vec!["name"];
/// check_at_least_one(&present, &["hashes", "name"])?;
/// ```
pub fn check_at_least_one(present_props: &[&str], required_props: &[&str]) -> Result<()> {
    let has_any = required_props.iter().any(|p| present_props.contains(p));

    if !has_any {
        return Err(Error::AtLeastOneRequired(
            required_props.iter().map(|s| s.to_string()).collect(),
        ));
    }

    Ok(())
}

/// Check that dependent properties are present when base properties are set.
///
/// The dependency logic is:
/// - If a base property is NOT present but a dependent property IS present: ERROR
/// - If a base property is present but None/False and dependent is present: ERROR
///
/// # Arguments
///
/// * `base_props` - List of base property names
/// * `dependent_props` - List of dependent property names that require base properties
/// * `is_present` - Function that returns true if property is present and has valid value
///
/// # Errors
///
/// Returns `Error::PropertyDependency` if dependency is violated.
///
/// # Example
///
/// ```rust,ignore
/// // If 'url' is present, 'hashes' must also be present
/// check_properties_dependency(
///     &["hashes"],
///     &["url"],
///     |prop| match prop {
///         "hashes" => !self.hashes.is_empty(),
///         "url" => self.url.is_some(),
///         _ => false,
///     }
/// )?;
/// ```
pub fn check_properties_dependency<F>(
    base_props: &[&str],
    dependent_props: &[&str],
    is_present: F,
) -> Result<()>
where
    F: Fn(&str) -> bool,
{
    // Check if any base property is present
    let base_present = base_props.iter().any(|p| is_present(p));

    // For each dependent property
    for dep in dependent_props {
        let dep_present = is_present(dep);

        // If dependent is present but base is not: error
        if dep_present && !base_present {
            return Err(Error::PropertyDependency {
                dependent: dep.to_string(),
                dependency: base_props.join(" or "),
            });
        }
    }

    Ok(())
}

/// Check that the second timestamp is greater than or equal to the first.
///
/// # Arguments
///
/// * `first` - The earlier timestamp (e.g., first_seen, valid_from)
/// * `second` - The later timestamp (e.g., last_seen, valid_until)
/// * `first_name` - Name of the first property for error messages
/// * `second_name` - Name of the second property for error messages
///
/// # Errors
///
/// Returns `Error::InvalidPropertyValue` if second < first.
pub fn check_timestamp_order(
    first: Option<&Timestamp>,
    second: Option<&Timestamp>,
    first_name: &str,
    second_name: &str,
) -> Result<()> {
    if let (Some(f), Some(s)) = (first, second)
        && s.datetime() < f.datetime()
    {
        return Err(Error::InvalidPropertyValue {
            property: second_name.to_string(),
            message: format!("{second_name} must be greater than or equal to {first_name}"),
        });
    }
    Ok(())
}

/// Check that the second timestamp is strictly greater than the first.
///
/// Used for Relationship's stop_time which must be strictly after start_time.
pub fn check_timestamp_order_strict(
    first: Option<&Timestamp>,
    second: Option<&Timestamp>,
    first_name: &str,
    second_name: &str,
) -> Result<()> {
    if let (Some(f), Some(s)) = (first, second)
        && s.datetime() <= f.datetime()
    {
        return Err(Error::InvalidPropertyValue {
            property: second_name.to_string(),
            message: format!("{second_name} must be later than {first_name}"),
        });
    }
    Ok(())
}

/// Check that a property is present when a condition is true.
pub fn check_conditional_required(
    condition: bool,
    condition_desc: &str,
    property_name: &str,
    is_present: bool,
) -> Result<()> {
    if condition && !is_present {
        return Err(Error::InvalidPropertyValue {
            property: property_name.to_string(),
            message: format!("'{property_name}' is a required property when {condition_desc}"),
        });
    }
    Ok(())
}

/// Check that a property is NOT present when a condition is true.
pub fn check_conditional_excluded(
    condition: bool,
    condition_desc: &str,
    property_name: &str,
    is_present: bool,
) -> Result<()> {
    if condition && is_present {
        return Err(Error::InvalidPropertyValue {
            property: property_name.to_string(),
            message: format!("'{property_name}' must not be present when {condition_desc}"),
        });
    }
    Ok(())
}

/// Validate hash algorithm names against the STIX 2.1 hash-algorithm-ov.
pub fn check_hash_algorithms(algorithms: &[&str]) -> Result<()> {
    const LEGAL_HASHES: &[&str] = &[
        "MD5", "SHA-1", "SHA-256", "SHA-512", "SHA3-256", "SHA3-512", "SSDEEP", "TLSH",
    ];

    for alg in algorithms {
        if !LEGAL_HASHES.contains(alg) {
            return Err(Error::InvalidPropertyValue {
                property: "hashes".to_string(),
                message: "Hash algorithm names must be members of hash-algorithm-ov".to_string(),
            });
        }
    }

    Ok(())
}

/// Check socket extension options keys.
///
/// Keys must start with one of: SO_, ICMP_, ICMP6_, IP_, IPV6_, MCAST_, TCP_, or IRLMP_.
pub fn check_socket_options_keys(keys: &[&str]) -> Result<()> {
    const ACCEPTABLE_PREFIXES: &[&str] = &[
        "SO_", "ICMP_", "ICMP6_", "IP_", "IPV6_", "MCAST_", "TCP_", "IRLMP_",
    ];

    for key in keys {
        let prefix = key.find('_').map(|i| &key[..=i]).unwrap_or("");

        if !ACCEPTABLE_PREFIXES.contains(&prefix) {
            return Err(Error::InvalidPropertyValue {
                property: "options".to_string(),
                message: format!("Incorrect options key: {key}"),
            });
        }
    }

    Ok(())
}

/// Check socket extension options values are integers.
pub fn check_socket_options_values(values: &[&serde_json::Value]) -> Result<()> {
    for value in values {
        if !value.is_i64() && !value.is_u64() {
            return Err(Error::InvalidPropertyValue {
                property: "options".to_string(),
                message: "Socket option values must be integers".to_string(),
            });
        }
    }
    Ok(())
}

/// Validate a reference points to one of the expected types.
///
/// # Arguments
///
/// * `identifier` - The identifier to validate
/// * `property_name` - The property name for error messages
/// * `valid_types` - List of valid object types
///
/// # Example
///
/// ```rust,ignore
/// // Validate creator_user_ref must be a user-account
/// check_ref_type(&creator_user_ref, "creator_user_ref", &["user-account"])?;
/// ```
pub fn check_ref_type(
    identifier: &crate::core::id::Identifier,
    property_name: &str,
    valid_types: &[&str],
) -> Result<()> {
    let obj_type = identifier.object_type();
    if !valid_types.contains(&obj_type) {
        return Err(Error::InvalidPropertyValue {
            property: property_name.to_string(),
            message: format!(
                "'{}' must reference one of: {}; got '{}'",
                property_name,
                valid_types.join(", "),
                obj_type
            ),
        });
    }
    Ok(())
}

/// Validate an optional reference points to one of the expected types.
pub fn check_optional_ref_type(
    identifier: Option<&crate::core::id::Identifier>,
    property_name: &str,
    valid_types: &[&str],
) -> Result<()> {
    if let Some(id) = identifier {
        check_ref_type(id, property_name, valid_types)?;
    }
    Ok(())
}

/// Validate a list of references all point to expected types.
pub fn check_refs_type(
    identifiers: &[crate::core::id::Identifier],
    property_name: &str,
    valid_types: &[&str],
) -> Result<()> {
    for id in identifiers {
        check_ref_type(id, property_name, valid_types)?;
    }
    Ok(())
}

/// Check that an integer is non-negative.
///
/// Used for validating properties like Process.pid which must be >= 0.
pub fn check_non_negative(value: i64, property_name: &str) -> Result<()> {
    if value < 0 {
        return Err(Error::InvalidPropertyValue {
            property: property_name.to_string(),
            message: format!("'{property_name}' must be a non-negative integer"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutually_exclusive_one_present() {
        let present = vec!["lang"];
        assert!(check_mutually_exclusive(&present, &["lang", "marking_ref"], true).is_ok());
    }

    #[test]
    fn test_mutually_exclusive_both_present() {
        let present = vec!["lang", "marking_ref"];
        let result = check_mutually_exclusive(&present, &["lang", "marking_ref"], true);
        assert!(matches!(result, Err(Error::MutuallyExclusiveProperties(_))));
    }

    #[test]
    fn test_mutually_exclusive_none_present_at_least_one() {
        let present: Vec<&str> = vec![];
        let result = check_mutually_exclusive(&present, &["lang", "marking_ref"], true);
        assert!(matches!(result, Err(Error::AtLeastOneRequired(_))));
    }

    #[test]
    fn test_mutually_exclusive_none_present_optional() {
        let present: Vec<&str> = vec![];
        assert!(check_mutually_exclusive(&present, &["lang", "marking_ref"], false).is_ok());
    }

    #[test]
    fn test_at_least_one_present() {
        let present = vec!["name"];
        assert!(check_at_least_one(&present, &["hashes", "name"]).is_ok());
    }

    #[test]
    fn test_at_least_one_none_present() {
        let present: Vec<&str> = vec![];
        let result = check_at_least_one(&present, &["hashes", "name"]);
        assert!(matches!(result, Err(Error::AtLeastOneRequired(_))));
    }

    #[test]
    fn test_properties_dependency_satisfied() {
        // url requires hashes
        let result = check_properties_dependency(&["hashes"], &["url"], |prop| {
            matches!(prop, "hashes" | "url")
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_properties_dependency_violated() {
        // url present but hashes missing
        let result = check_properties_dependency(&["hashes"], &["url"], |prop| prop == "url");
        assert!(matches!(result, Err(Error::PropertyDependency { .. })));
    }

    #[test]
    fn test_timestamp_order_valid() {
        let first = Timestamp::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let second = Timestamp::now();

        assert!(check_timestamp_order(Some(&first), Some(&second), "first", "second").is_ok());
    }

    #[test]
    fn test_timestamp_order_invalid() {
        let second = Timestamp::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let first = Timestamp::now();

        let result = check_timestamp_order(Some(&first), Some(&second), "first", "second");
        assert!(matches!(result, Err(Error::InvalidPropertyValue { .. })));
    }

    #[test]
    fn test_timestamp_order_none() {
        // Should pass if either is None
        let ts = Timestamp::now();
        assert!(check_timestamp_order(Some(&ts), None, "first", "second").is_ok());
        assert!(check_timestamp_order(None, Some(&ts), "first", "second").is_ok());
        assert!(check_timestamp_order(None, None, "first", "second").is_ok());
    }

    #[test]
    fn test_conditional_required() {
        // is_family requires name
        assert!(check_conditional_required(true, "is_family is true", "name", true).is_ok());
        assert!(check_conditional_required(true, "is_family is true", "name", false).is_err());
        assert!(check_conditional_required(false, "is_family is true", "name", false).is_ok());
    }

    #[test]
    fn test_check_hash_algorithms() {
        assert!(check_hash_algorithms(&["MD5", "SHA-256"]).is_ok());
        assert!(check_hash_algorithms(&["invalid-algo"]).is_err());
    }

    #[test]
    fn test_socket_options_keys() {
        assert!(check_socket_options_keys(&["SO_KEEPALIVE", "TCP_NODELAY"]).is_ok());
        assert!(check_socket_options_keys(&["INVALID_KEY"]).is_err());
    }
}
