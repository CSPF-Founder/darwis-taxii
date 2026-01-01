//! STIX Object Versioning
//!
//! This module provides utilities for creating new versions of STIX objects
//! and revoking them.
//!
//! ## Key Features
//!
//! - Create new versions with updated timestamps
//! - Apply property changes when creating new versions
//! - Revoke objects
//! - Prevent modification of immutable properties
//!
//! ## Example
//!
//! ```rust,ignore
//! use stix2::versioning::{new_version, new_version_with_changes, VersionBuilder};
//!
//! // Simple version bump
//! let new_obj = new_version(&obj)?;
//!
//! // Version with property changes
//! let new_obj = new_version_with_changes(&obj, &changes)?;
//!
//! // Fluent builder API
//! let new_obj = VersionBuilder::new(&obj)
//!     .set("name", "Updated Name")?
//!     .set("description", "New description")?
//!     .build()?;
//! ```

use chrono::Duration;
use serde_json::{Map, Value};

use crate::core::error::{Error, Result};
use crate::core::stix_object::StixObject;
use crate::core::timestamp::Timestamp;

/// Properties that cannot be modified when creating a new version.
pub const UNMODIFIABLE_PROPERTIES: &[&str] = &["created", "created_by_ref", "id", "type"];

/// Check if an object type is versionable (SDOs and SROs are, SCOs are not).
pub fn is_versionable(obj: &StixObject) -> bool {
    matches!(
        obj,
        StixObject::AttackPattern(_)
            | StixObject::Campaign(_)
            | StixObject::CourseOfAction(_)
            | StixObject::Grouping(_)
            | StixObject::Identity(_)
            | StixObject::Incident(_)
            | StixObject::Indicator(_)
            | StixObject::Infrastructure(_)
            | StixObject::IntrusionSet(_)
            | StixObject::Location(_)
            | StixObject::Malware(_)
            | StixObject::MalwareAnalysis(_)
            | StixObject::Note(_)
            | StixObject::ObservedData(_)
            | StixObject::Opinion(_)
            | StixObject::Report(_)
            | StixObject::ThreatActor(_)
            | StixObject::Tool(_)
            | StixObject::Vulnerability(_)
            | StixObject::Relationship(_)
            | StixObject::Sighting(_)
            | StixObject::LanguageContent(_)
    )
}

/// Check if an object is revoked.
pub fn is_revoked(obj: &StixObject) -> bool {
    match obj {
        StixObject::AttackPattern(o) => o.common.revoked,
        StixObject::Campaign(o) => o.common.revoked,
        StixObject::CourseOfAction(o) => o.common.revoked,
        StixObject::Grouping(o) => o.common.revoked,
        StixObject::Identity(o) => o.common.revoked,
        StixObject::Incident(o) => o.common.revoked,
        StixObject::Indicator(o) => o.common.revoked,
        StixObject::Infrastructure(o) => o.common.revoked,
        StixObject::IntrusionSet(o) => o.common.revoked,
        StixObject::Location(o) => o.common.revoked,
        StixObject::Malware(o) => o.common.revoked,
        StixObject::MalwareAnalysis(o) => o.common.revoked,
        StixObject::Note(o) => o.common.revoked,
        StixObject::ObservedData(o) => o.common.revoked,
        StixObject::Opinion(o) => o.common.revoked,
        StixObject::Report(o) => o.common.revoked,
        StixObject::ThreatActor(o) => o.common.revoked,
        StixObject::Tool(o) => o.common.revoked,
        StixObject::Vulnerability(o) => o.common.revoked,
        StixObject::Relationship(o) => o.common.revoked,
        StixObject::Sighting(o) => o.common.revoked,
        StixObject::LanguageContent(o) => o.common.revoked,
        _ => false,
    }
}

/// Get the modified timestamp from an object.
pub fn get_modified(obj: &StixObject) -> Option<&Timestamp> {
    match obj {
        StixObject::AttackPattern(o) => Some(&o.common.modified),
        StixObject::Campaign(o) => Some(&o.common.modified),
        StixObject::CourseOfAction(o) => Some(&o.common.modified),
        StixObject::Grouping(o) => Some(&o.common.modified),
        StixObject::Identity(o) => Some(&o.common.modified),
        StixObject::Incident(o) => Some(&o.common.modified),
        StixObject::Indicator(o) => Some(&o.common.modified),
        StixObject::Infrastructure(o) => Some(&o.common.modified),
        StixObject::IntrusionSet(o) => Some(&o.common.modified),
        StixObject::Location(o) => Some(&o.common.modified),
        StixObject::Malware(o) => Some(&o.common.modified),
        StixObject::MalwareAnalysis(o) => Some(&o.common.modified),
        StixObject::Note(o) => Some(&o.common.modified),
        StixObject::ObservedData(o) => Some(&o.common.modified),
        StixObject::Opinion(o) => Some(&o.common.modified),
        StixObject::Report(o) => Some(&o.common.modified),
        StixObject::ThreatActor(o) => Some(&o.common.modified),
        StixObject::Tool(o) => Some(&o.common.modified),
        StixObject::Vulnerability(o) => Some(&o.common.modified),
        StixObject::Relationship(o) => Some(&o.common.modified),
        StixObject::Sighting(o) => Some(&o.common.modified),
        StixObject::LanguageContent(o) => Some(&o.common.modified),
        _ => None,
    }
}

/// Ensure the new modified timestamp is newer than the old one.
fn fudge_modified(old_modified: &Timestamp, new_modified: Timestamp) -> Timestamp {
    let old_dt = old_modified.datetime();
    let new_dt = new_modified.datetime();

    if new_dt <= old_dt {
        // Push new_modified to be at least 1 microsecond after old
        Timestamp::new(old_dt + Duration::microseconds(1))
    } else {
        new_modified
    }
}

/// Create a new version of a STIX object with an updated modified timestamp.
///
/// This function clones the object and updates its `modified` timestamp.
/// The new timestamp is guaranteed to be later than the current one.
///
/// # Errors
///
/// Returns an error if:
/// - The object is not versionable (e.g., SCOs)
/// - The object is already revoked
///
/// # Example
///
/// ```rust,ignore
/// use stix2::versioning::new_version;
///
/// let indicator = Indicator::builder()
///     .name("Evil IP")
///     .pattern("[ipv4-addr:value = '10.0.0.1']")
///     .pattern_type(PatternType::Stix)
///     .valid_from_now()
///     .build()?;
///
/// let new_indicator = new_version(&StixObject::Indicator(indicator))?;
/// ```
pub fn new_version(obj: &StixObject) -> Result<StixObject> {
    if !is_versionable(obj) {
        return Err(Error::validation(
            "Object type is not versionable (SCOs cannot be versioned)",
        ));
    }

    if is_revoked(obj) {
        return Err(Error::validation(
            "Cannot create new version of a revoked object",
        ));
    }

    let old_modified = get_modified(obj)
        .ok_or_else(|| Error::validation("Object does not have a modified timestamp"))?;

    let new_modified = fudge_modified(old_modified, Timestamp::now());

    // Clone and update the modified timestamp
    let mut new_obj = obj.clone();
    set_modified(&mut new_obj, new_modified);

    Ok(new_obj)
}

/// Create a new version of a STIX object with a custom modified timestamp.
pub fn new_version_with_timestamp(obj: &StixObject, modified: Timestamp) -> Result<StixObject> {
    if !is_versionable(obj) {
        return Err(Error::validation(
            "Object type is not versionable (SCOs cannot be versioned)",
        ));
    }

    if is_revoked(obj) {
        return Err(Error::validation(
            "Cannot create new version of a revoked object",
        ));
    }

    let old_modified = get_modified(obj)
        .ok_or_else(|| Error::validation("Object does not have a modified timestamp"))?;

    if modified.datetime() <= old_modified.datetime() {
        return Err(Error::validation(
            "New modified timestamp must be later than the current one",
        ));
    }

    let mut new_obj = obj.clone();
    set_modified(&mut new_obj, modified);

    Ok(new_obj)
}

/// Revoke a STIX object.
///
/// Returns a new version of the object with `revoked` set to `true`.
///
/// # Errors
///
/// Returns an error if:
/// - The object is not versionable
/// - The object is already revoked
pub fn revoke(obj: &StixObject) -> Result<StixObject> {
    if !is_versionable(obj) {
        return Err(Error::validation(
            "Object type is not versionable (SCOs cannot be revoked)",
        ));
    }

    if is_revoked(obj) {
        return Err(Error::validation("Object is already revoked"));
    }

    let old_modified = get_modified(obj)
        .ok_or_else(|| Error::validation("Object does not have a modified timestamp"))?;

    let new_modified = fudge_modified(old_modified, Timestamp::now());

    let mut new_obj = obj.clone();
    set_modified(&mut new_obj, new_modified);
    set_revoked(&mut new_obj, true);

    Ok(new_obj)
}

fn set_modified(obj: &mut StixObject, modified: Timestamp) {
    match obj {
        StixObject::AttackPattern(o) => o.common.modified = modified,
        StixObject::Campaign(o) => o.common.modified = modified,
        StixObject::CourseOfAction(o) => o.common.modified = modified,
        StixObject::Grouping(o) => o.common.modified = modified,
        StixObject::Identity(o) => o.common.modified = modified,
        StixObject::Incident(o) => o.common.modified = modified,
        StixObject::Indicator(o) => o.common.modified = modified,
        StixObject::Infrastructure(o) => o.common.modified = modified,
        StixObject::IntrusionSet(o) => o.common.modified = modified,
        StixObject::Location(o) => o.common.modified = modified,
        StixObject::Malware(o) => o.common.modified = modified,
        StixObject::MalwareAnalysis(o) => o.common.modified = modified,
        StixObject::Note(o) => o.common.modified = modified,
        StixObject::ObservedData(o) => o.common.modified = modified,
        StixObject::Opinion(o) => o.common.modified = modified,
        StixObject::Report(o) => o.common.modified = modified,
        StixObject::ThreatActor(o) => o.common.modified = modified,
        StixObject::Tool(o) => o.common.modified = modified,
        StixObject::Vulnerability(o) => o.common.modified = modified,
        StixObject::Relationship(o) => o.common.modified = modified,
        StixObject::Sighting(o) => o.common.modified = modified,
        StixObject::LanguageContent(o) => o.common.modified = modified,
        _ => {}
    }
}

fn set_revoked(obj: &mut StixObject, revoked: bool) {
    match obj {
        StixObject::AttackPattern(o) => o.common.revoked = revoked,
        StixObject::Campaign(o) => o.common.revoked = revoked,
        StixObject::CourseOfAction(o) => o.common.revoked = revoked,
        StixObject::Grouping(o) => o.common.revoked = revoked,
        StixObject::Identity(o) => o.common.revoked = revoked,
        StixObject::Incident(o) => o.common.revoked = revoked,
        StixObject::Indicator(o) => o.common.revoked = revoked,
        StixObject::Infrastructure(o) => o.common.revoked = revoked,
        StixObject::IntrusionSet(o) => o.common.revoked = revoked,
        StixObject::Location(o) => o.common.revoked = revoked,
        StixObject::Malware(o) => o.common.revoked = revoked,
        StixObject::MalwareAnalysis(o) => o.common.revoked = revoked,
        StixObject::Note(o) => o.common.revoked = revoked,
        StixObject::ObservedData(o) => o.common.revoked = revoked,
        StixObject::Opinion(o) => o.common.revoked = revoked,
        StixObject::Report(o) => o.common.revoked = revoked,
        StixObject::ThreatActor(o) => o.common.revoked = revoked,
        StixObject::Tool(o) => o.common.revoked = revoked,
        StixObject::Vulnerability(o) => o.common.revoked = revoked,
        StixObject::Relationship(o) => o.common.revoked = revoked,
        StixObject::Sighting(o) => o.common.revoked = revoked,
        StixObject::LanguageContent(o) => o.common.revoked = revoked,
        _ => {}
    }
}

/// Check if any unmodifiable properties are being changed.
///
/// Returns an error if any of the properties in `changes` are unmodifiable.
fn check_unmodifiable_properties(changes: &Map<String, Value>) -> Result<()> {
    let mut unmodifiable = Vec::new();

    for prop in UNMODIFIABLE_PROPERTIES {
        if changes.contains_key(*prop) {
            unmodifiable.push(prop.to_string());
        }
    }

    if !unmodifiable.is_empty() {
        return Err(Error::ImmutableProperty(format!(
            "Cannot modify properties: {}",
            unmodifiable.join(", ")
        )));
    }

    Ok(())
}

/// Create a new version of a STIX object with property changes.
///
/// This function creates a new version of the object by:
/// 1. Cloning the object
/// 2. Applying the property changes
/// 3. Updating the `modified` timestamp
///
/// # Arguments
///
/// * `obj` - The object to version
/// * `changes` - A map of property names to new values
///
/// # Errors
///
/// Returns an error if:
/// - The object is not versionable (e.g., SCOs)
/// - The object is already revoked
/// - Any unmodifiable property is being changed (created, created_by_ref, id, type)
///
/// # Example
///
/// ```rust,ignore
/// use stix2::versioning::new_version_with_changes;
/// use serde_json::json;
///
/// let changes = serde_json::from_value(json!({
///     "name": "Updated Indicator Name",
///     "description": "New description"
/// })).unwrap();
///
/// let new_indicator = new_version_with_changes(&indicator, &changes)?;
/// ```
pub fn new_version_with_changes(
    obj: &StixObject,
    changes: &Map<String, Value>,
) -> Result<StixObject> {
    if !is_versionable(obj) {
        return Err(Error::validation(
            "Object type is not versionable (SCOs cannot be versioned)",
        ));
    }

    if is_revoked(obj) {
        return Err(Error::validation(
            "Cannot create new version of a revoked object",
        ));
    }

    // Check that no unmodifiable properties are being changed
    check_unmodifiable_properties(changes)?;

    // Serialize the object to JSON
    let mut obj_value = serde_json::to_value(obj.clone())
        .map_err(|e| Error::custom(format!("Failed to serialize object: {e}")))?;

    // Apply the changes
    if let Value::Object(ref mut obj_map) = obj_value {
        for (key, value) in changes {
            // Handle null values as property removal
            if value.is_null() {
                obj_map.remove(key);
            } else {
                obj_map.insert(key.clone(), value.clone());
            }
        }

        // Update the modified timestamp
        let old_modified = get_modified(obj)
            .ok_or_else(|| Error::validation("Object does not have a modified timestamp"))?;

        // Check if a new modified timestamp was provided in changes
        let new_modified = if let Some(Value::String(mod_str)) = changes.get("modified") {
            let provided_modified: Timestamp = mod_str
                .parse()
                .map_err(|_| Error::InvalidTimestamp(mod_str.clone()))?;

            if provided_modified.datetime() <= old_modified.datetime() {
                return Err(Error::validation(
                    "New modified timestamp must be later than the current one",
                ));
            }
            provided_modified
        } else {
            fudge_modified(old_modified, Timestamp::now())
        };

        obj_map.insert(
            "modified".to_string(),
            Value::String(new_modified.to_string()),
        );
    }

    // Deserialize back to StixObject
    serde_json::from_value(obj_value)
        .map_err(|e| Error::custom(format!("Failed to deserialize updated object: {e}")))
}

/// Builder for creating new object versions with a fluent API.
///
/// This provides a convenient way to apply multiple property changes
/// when creating a new version.
///
/// # Example
///
/// ```rust,ignore
/// use stix2::versioning::VersionBuilder;
///
/// let new_indicator = VersionBuilder::new(&indicator)
///     .set("name", "Updated Name")?
///     .set("description", "New description")?
///     .remove("external_references")?
///     .build()?;
/// ```
#[derive(Debug)]
pub struct VersionBuilder<'a> {
    obj: &'a StixObject,
    changes: Map<String, Value>,
}

impl<'a> VersionBuilder<'a> {
    /// Create a new VersionBuilder for the given object.
    pub fn new(obj: &'a StixObject) -> Self {
        Self {
            obj,
            changes: Map::new(),
        }
    }

    /// Set a property to a new value.
    ///
    /// # Arguments
    ///
    /// * `key` - The property name
    /// * `value` - The new value (must be serializable)
    ///
    /// # Errors
    ///
    /// Returns an error if the property is unmodifiable.
    pub fn set<V: serde::Serialize>(mut self, key: &str, value: V) -> Result<Self> {
        // Check if this is an unmodifiable property
        if UNMODIFIABLE_PROPERTIES.contains(&key) {
            return Err(Error::ImmutableProperty(format!(
                "Cannot modify property: {key}"
            )));
        }

        let json_value = serde_json::to_value(value)
            .map_err(|e| Error::custom(format!("Failed to serialize value: {e}")))?;

        self.changes.insert(key.to_string(), json_value);
        Ok(self)
    }

    /// Remove a property from the object.
    ///
    /// The property will be set to null, which causes it to be removed
    /// during versioning.
    ///
    /// # Errors
    ///
    /// Returns an error if the property is unmodifiable.
    pub fn remove(mut self, key: &str) -> Result<Self> {
        // Check if this is an unmodifiable property
        if UNMODIFIABLE_PROPERTIES.contains(&key) {
            return Err(Error::ImmutableProperty(format!(
                "Cannot remove property: {key}"
            )));
        }

        self.changes.insert(key.to_string(), Value::Null);
        Ok(self)
    }

    /// Set a custom modified timestamp for the new version.
    ///
    /// # Errors
    ///
    /// Returns an error if the timestamp is not later than the current one.
    pub fn modified(mut self, timestamp: Timestamp) -> Self {
        self.changes
            .insert("modified".to_string(), Value::String(timestamp.to_string()));
        self
    }

    /// Build the new version with all the accumulated changes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The object is not versionable
    /// - The object is revoked
    /// - The modified timestamp is invalid
    pub fn build(self) -> Result<StixObject> {
        new_version_with_changes(self.obj, &self.changes)
    }
}

/// Remove custom properties from a STIX object.
///
/// Returns a new version of the object with any custom properties
/// (those starting with `x_`) removed.
///
/// # Arguments
///
/// * `obj` - The object to remove custom properties from
///
/// # Returns
///
/// - `Ok(Some(StixObject))` if the object had custom properties removed
/// - `Ok(None)` if the entire object is custom (type starts with `x-`)
/// - `Ok(Some(obj))` (unchanged) if there were no custom properties
pub fn remove_custom_properties(obj: &StixObject) -> Result<Option<StixObject>> {
    // Check if the entire object is custom
    let type_name = obj.type_name();
    if type_name.starts_with("x-") {
        return Ok(None);
    }

    // Serialize to find custom properties
    let obj_value = serde_json::to_value(obj.clone())
        .map_err(|e| Error::custom(format!("Failed to serialize object: {e}")))?;

    if let Value::Object(obj_map) = &obj_value {
        // Find custom properties
        let custom_props: Vec<String> = obj_map
            .keys()
            .filter(|k| k.starts_with("x_"))
            .cloned()
            .collect();

        if custom_props.is_empty() {
            // No custom properties, return unchanged
            return Ok(Some(obj.clone()));
        }

        // Create changes map with nulls for custom properties
        let mut changes = Map::new();
        for prop in custom_props {
            changes.insert(prop, Value::Null);
        }

        // Create new version without custom properties
        let new_obj = new_version_with_changes(obj, &changes)?;
        Ok(Some(new_obj))
    } else {
        Ok(Some(obj.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::Indicator;
    use crate::vocab::PatternType;
    use std::thread::sleep;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_is_versionable() {
        let indicator = Indicator::builder()
            .name("Test")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        assert!(is_versionable(&StixObject::Indicator(indicator)));
    }

    #[test]
    fn test_new_version() {
        let indicator = Indicator::builder()
            .name("Test")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let obj = StixObject::Indicator(indicator);
        let old_modified = *get_modified(&obj).unwrap();

        // Small delay to ensure timestamp difference
        sleep(StdDuration::from_millis(10));

        let new_obj = new_version(&obj).unwrap();
        let new_modified = get_modified(&new_obj).unwrap();

        assert!(new_modified.datetime() > old_modified.datetime());
    }

    #[test]
    fn test_revoke() {
        let indicator = Indicator::builder()
            .name("Test")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let obj = StixObject::Indicator(indicator);
        assert!(!is_revoked(&obj));

        let revoked_obj = revoke(&obj).unwrap();
        assert!(is_revoked(&revoked_obj));
    }

    #[test]
    fn test_cannot_revoke_twice() {
        let indicator = Indicator::builder()
            .name("Test")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let obj = StixObject::Indicator(indicator);
        let revoked_obj = revoke(&obj).unwrap();

        // Trying to revoke again should fail
        assert!(revoke(&revoked_obj).is_err());
    }

    #[test]
    fn test_new_version_with_changes() {
        let indicator = Indicator::builder()
            .name("Test")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let obj = StixObject::Indicator(indicator);
        let old_modified = *get_modified(&obj).unwrap();

        // Small delay to ensure timestamp difference
        sleep(StdDuration::from_millis(10));

        // Create changes map
        let mut changes = Map::new();
        changes.insert(
            "name".to_string(),
            Value::String("Updated Name".to_string()),
        );
        changes.insert(
            "description".to_string(),
            Value::String("New description".to_string()),
        );

        let new_obj = new_version_with_changes(&obj, &changes).unwrap();

        // Verify the name was changed
        if let StixObject::Indicator(ind) = &new_obj {
            assert_eq!(ind.name.as_deref(), Some("Updated Name"));
            assert_eq!(ind.description.as_deref(), Some("New description"));
        } else {
            panic!("Expected Indicator");
        }

        // Verify modified timestamp was updated (fudge_modified ensures it's always later)
        let new_modified = get_modified(&new_obj).unwrap();
        assert!(new_modified.datetime() >= old_modified.datetime());
    }

    #[test]
    fn test_version_builder() {
        let indicator = Indicator::builder()
            .name("Test")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let obj = StixObject::Indicator(indicator);

        let new_obj = VersionBuilder::new(&obj)
            .set("name", "Builder Updated Name")
            .unwrap()
            .set("description", "Builder description")
            .unwrap()
            .build()
            .unwrap();

        if let StixObject::Indicator(ind) = &new_obj {
            assert_eq!(ind.name.as_deref(), Some("Builder Updated Name"));
            assert_eq!(ind.description.as_deref(), Some("Builder description"));
        } else {
            panic!("Expected Indicator");
        }
    }

    #[test]
    fn test_cannot_modify_immutable_properties() {
        let indicator = Indicator::builder()
            .name("Test")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let obj = StixObject::Indicator(indicator);

        // Try to modify 'id' - should fail
        let result = VersionBuilder::new(&obj).set("id", "indicator--new-id");
        assert!(result.is_err());

        // Try to modify 'type' - should fail
        let result = VersionBuilder::new(&obj).set("type", "malware");
        assert!(result.is_err());

        // Try to modify 'created' - should fail
        let result = VersionBuilder::new(&obj).set("created", "2024-01-01T00:00:00.000Z");
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_property() {
        let indicator = Indicator::builder()
            .name("Test")
            .description("Original description")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let obj = StixObject::Indicator(indicator);

        let new_obj = VersionBuilder::new(&obj)
            .remove("description")
            .unwrap()
            .build()
            .unwrap();

        if let StixObject::Indicator(ind) = &new_obj {
            assert!(ind.description.is_none());
        } else {
            panic!("Expected Indicator");
        }
    }
}
