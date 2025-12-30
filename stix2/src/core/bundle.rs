//! STIX Bundle container.
//!
//! A Bundle is a container for STIX objects that allows multiple objects
//! to be transmitted or stored together.

use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::stix_object::StixObject;
use serde::{Deserialize, Serialize};

/// A STIX Bundle containing multiple STIX objects.
///
/// Bundles are the primary mechanism for packaging and transmitting
/// STIX content. They are not considered STIX Objects themselves and
/// do not have properties like `created` or `modified`.
///
/// # Example
///
/// ```rust,no_run
/// use stix2::prelude::*;
///
/// fn main() -> stix2::Result<()> {
///     let mut bundle = Bundle::new();
///
///     let indicator = Indicator::builder()
///         .name("Malicious IP")
///         .pattern("[ipv4-addr:value = '10.0.0.1']")
///         .pattern_type(PatternType::Stix)
///         .valid_from_now()
///         .build()?;
///
///     bundle.add_object(indicator);
///
///     let json = bundle.to_json_pretty()?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bundle {
    /// The type field, always "bundle".
    #[serde(rename = "type")]
    pub type_: String,

    /// The bundle's unique identifier.
    pub id: Identifier,

    /// The STIX objects contained in this bundle.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub objects: Vec<StixObject>,
}

impl Bundle {
    /// Create a new empty bundle with a random ID.
    pub fn new() -> Self {
        Self {
            type_: "bundle".to_string(),
            id: Identifier::new_bundle(),
            objects: Vec::new(),
        }
    }

    /// Create a bundle with a specific ID.
    pub fn with_id(id: Identifier) -> Result<Self> {
        if id.object_type() != "bundle" {
            return Err(Error::InvalidType(format!(
                "Expected bundle identifier, got: {}",
                id.object_type()
            )));
        }
        Ok(Self {
            type_: "bundle".to_string(),
            id,
            objects: Vec::new(),
        })
    }

    /// Create a bundle from a list of objects.
    pub fn from_objects(objects: Vec<StixObject>) -> Self {
        let mut bundle = Self::new();
        bundle.objects = objects;
        bundle
    }

    /// Add a STIX object to the bundle.
    pub fn add_object<T: Into<StixObject>>(&mut self, object: T) {
        self.objects.push(object.into());
    }

    /// Add multiple STIX objects to the bundle.
    pub fn add_objects<I, T>(&mut self, objects: I)
    where
        I: IntoIterator<Item = T>,
        T: Into<StixObject>,
    {
        for obj in objects {
            self.objects.push(obj.into());
        }
    }

    /// Get the number of objects in the bundle.
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Check if the bundle is empty.
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    /// Get an iterator over the objects.
    pub fn iter(&self) -> impl Iterator<Item = &StixObject> {
        self.objects.iter()
    }

    /// Get a mutable iterator over the objects.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut StixObject> {
        self.objects.iter_mut()
    }

    /// Find objects by type.
    pub fn find_by_type(&self, type_name: &str) -> Vec<&StixObject> {
        self.objects
            .iter()
            .filter(|obj| obj.type_name() == type_name)
            .collect()
    }

    /// Find an object by ID.
    pub fn find_by_id(&self, id: &Identifier) -> Option<&StixObject> {
        self.objects.iter().find(|obj| obj.id() == id)
    }

    /// Remove an object by ID.
    pub fn remove_by_id(&mut self, id: &Identifier) -> Option<StixObject> {
        if let Some(pos) = self.objects.iter().position(|obj| obj.id() == id) {
            Some(self.objects.remove(pos))
        } else {
            None
        }
    }

    /// Get all object IDs in the bundle.
    pub fn object_ids(&self) -> Vec<&Identifier> {
        self.objects.iter().map(|obj| obj.id()).collect()
    }

    /// Merge another bundle into this one.
    pub fn merge(&mut self, other: Bundle) {
        self.objects.extend(other.objects);
    }

    /// Deduplicate objects by ID and modified timestamp.
    ///
    /// When multiple versions of the same object exist, only the
    /// most recently modified version is kept.
    pub fn deduplicate(&mut self) {
        use std::collections::HashMap;

        let mut seen: HashMap<String, (usize, Option<chrono::DateTime<chrono::Utc>>)> =
            HashMap::new();
        let mut to_remove = Vec::new();

        for (idx, obj) in self.objects.iter().enumerate() {
            let id_str = obj.id().to_string();
            let modified = obj.modified();

            match seen.get(&id_str) {
                Some((existing_idx, existing_modified)) => {
                    // Compare modified timestamps
                    match (modified, existing_modified) {
                        (Some(new_mod), Some(old_mod)) if new_mod > *old_mod => {
                            // New object is newer, remove old one
                            to_remove.push(*existing_idx);
                            seen.insert(id_str, (idx, Some(new_mod)));
                        }
                        _ => {
                            // Keep existing, remove new
                            to_remove.push(idx);
                        }
                    }
                }
                None => {
                    seen.insert(id_str, (idx, modified));
                }
            }
        }

        // Remove duplicates in reverse order to preserve indices
        to_remove.sort_unstable();
        to_remove.reverse();
        for idx in to_remove {
            self.objects.remove(idx);
        }
    }

    /// Serialize the bundle to JSON.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Error::from)
    }

    /// Serialize the bundle to pretty-printed JSON.
    pub fn to_json_pretty(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Error::from)
    }

    /// Parse a bundle from JSON.
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Error::from)
    }
}

impl Default for Bundle {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Bundle {
    type Item = StixObject;
    type IntoIter = std::vec::IntoIter<StixObject>;

    fn into_iter(self) -> Self::IntoIter {
        self.objects.into_iter()
    }
}

impl<'a> IntoIterator for &'a Bundle {
    type Item = &'a StixObject;
    type IntoIter = std::slice::Iter<'a, StixObject>;

    fn into_iter(self) -> Self::IntoIter {
        self.objects.iter()
    }
}

impl Extend<StixObject> for Bundle {
    fn extend<T: IntoIterator<Item = StixObject>>(&mut self, iter: T) {
        self.objects.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bundle() {
        let bundle = Bundle::new();
        assert_eq!(bundle.type_, "bundle");
        assert!(bundle.id.to_string().starts_with("bundle--"));
        assert!(bundle.is_empty());
    }

    #[test]
    fn test_bundle_serialization() {
        let bundle = Bundle::new();
        let json = bundle.to_json().unwrap();
        assert!(json.contains("\"type\":\"bundle\""));

        let parsed = Bundle::from_json(&json).unwrap();
        assert_eq!(bundle.id, parsed.id);
    }

    #[test]
    fn test_find_by_type() {
        let bundle = Bundle::new();
        // Empty bundle should return empty results
        let results = bundle.find_by_type("indicator");
        assert!(results.is_empty());
    }
}
