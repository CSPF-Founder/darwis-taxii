//! Marking Operations
//!
//! This module provides functions for manipulating object-level and granular markings
//! on STIX objects.

use super::GranularMarking;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::HashSet;

// ============================================================================
// Object Marking Operations
// ============================================================================

/// Get all object-level marking refs from a STIX object.
///
/// Returns the list of marking definition identifiers applied at the object level.
pub fn get_object_markings(object_marking_refs: &[Identifier]) -> Vec<&Identifier> {
    object_marking_refs.iter().collect()
}

/// Add a marking to object-level markings (returns new list).
///
/// Deduplicates the marking if already present.
pub fn add_object_marking(
    object_marking_refs: &[Identifier],
    marking: Identifier,
) -> Vec<Identifier> {
    let mut result: Vec<Identifier> = object_marking_refs.to_vec();
    if !result.contains(&marking) {
        result.push(marking);
    }
    result
}

/// Add multiple markings to object-level markings (returns new list).
pub fn add_object_markings(
    object_marking_refs: &[Identifier],
    markings: &[Identifier],
) -> Vec<Identifier> {
    let mut result: Vec<Identifier> = object_marking_refs.to_vec();
    for marking in markings {
        if !result.contains(marking) {
            result.push(marking.clone());
        }
    }
    result
}

/// Remove a marking from object-level markings (returns new list).
pub fn remove_object_marking(
    object_marking_refs: &[Identifier],
    marking: &Identifier,
) -> Vec<Identifier> {
    object_marking_refs
        .iter()
        .filter(|m| *m != marking)
        .cloned()
        .collect()
}

/// Remove multiple markings from object-level markings (returns new list).
pub fn remove_object_markings(
    object_marking_refs: &[Identifier],
    markings: &[Identifier],
) -> Vec<Identifier> {
    let to_remove: HashSet<_> = markings.iter().collect();
    object_marking_refs
        .iter()
        .filter(|m| !to_remove.contains(m))
        .cloned()
        .collect()
}

/// Clear all object-level markings (returns empty list).
pub fn clear_object_markings() -> Vec<Identifier> {
    Vec::new()
}

/// Set object-level markings (replace all with new markings).
pub fn set_object_markings(markings: &[Identifier]) -> Vec<Identifier> {
    // Deduplicate
    let mut seen = HashSet::new();
    markings
        .iter()
        .filter(|m| seen.insert(m.to_string()))
        .cloned()
        .collect()
}

/// Check if an object has a specific marking (or any marking if None).
pub fn is_object_marked(object_marking_refs: &[Identifier], marking: Option<&Identifier>) -> bool {
    match marking {
        Some(m) => object_marking_refs.contains(m),
        None => !object_marking_refs.is_empty(),
    }
}

// ============================================================================
// Granular Marking Operations
// ============================================================================

/// Get granular markings for specific selectors.
///
/// # Arguments
///
/// * `granular_markings` - The list of granular markings on the object
/// * `selectors` - The property paths to check (e.g., ["description", "name"])
/// * `inherited` - If true, include markings from parent properties
/// * `descendants` - If true, include markings from child properties
///
/// # Returns
///
/// List of marking definition identifiers that apply to the specified selectors.
pub fn get_granular_markings<'a>(
    granular_markings: &'a [GranularMarking],
    selectors: &[&str],
    inherited: bool,
    descendants: bool,
) -> Vec<&'a Identifier> {
    let mut result = Vec::new();

    for gm in granular_markings {
        if let Some(ref marking_ref) = gm.marking_ref {
            for selector in selectors {
                for gm_selector in &gm.selectors {
                    // Exact match
                    if gm_selector == *selector {
                        if !result.contains(&marking_ref) {
                            result.push(marking_ref);
                        }
                        continue;
                    }

                    // Inherited: parent selector matches (e.g., "description" matches "description.text")
                    if inherited
                        && selector.starts_with(&format!("{gm_selector}."))
                        && !result.contains(&marking_ref)
                    {
                        result.push(marking_ref);
                        continue;
                    }

                    // Descendants: child selector matches (e.g., "description.text" when checking "description")
                    if descendants
                        && gm_selector.starts_with(&format!("{selector}."))
                        && !result.contains(&marking_ref)
                    {
                        result.push(marking_ref);
                    }
                }
            }
        }
    }

    result
}

/// Add a granular marking to specific selectors.
///
/// Returns a new list of granular markings with the new marking added.
pub fn add_granular_marking(
    granular_markings: &[GranularMarking],
    marking: Identifier,
    selectors: Vec<String>,
) -> Vec<GranularMarking> {
    let mut result = granular_markings.to_vec();

    // Check if we can merge with an existing marking
    for gm in &mut result {
        if gm.marking_ref.as_ref() == Some(&marking) {
            // Add selectors to existing marking
            for selector in &selectors {
                if !gm.selectors.contains(selector) {
                    gm.selectors.push(selector.clone());
                }
            }
            return result;
        }
    }

    // Add new granular marking
    result.push(GranularMarking::new(marking, selectors));
    result
}

/// Remove a granular marking from specific selectors.
///
/// If `selectors` is empty, removes the marking from all selectors.
pub fn remove_granular_marking(
    granular_markings: &[GranularMarking],
    marking: &Identifier,
    selectors: &[&str],
) -> Vec<GranularMarking> {
    let mut result = Vec::new();

    for gm in granular_markings {
        if gm.marking_ref.as_ref() == Some(marking) {
            if selectors.is_empty() {
                // Remove entirely
                continue;
            }

            // Remove specific selectors
            let remaining_selectors: Vec<String> = gm
                .selectors
                .iter()
                .filter(|s| !selectors.contains(&s.as_str()))
                .cloned()
                .collect();

            if !remaining_selectors.is_empty() {
                result.push(GranularMarking {
                    lang: gm.lang.clone(),
                    marking_ref: gm.marking_ref.clone(),
                    selectors: remaining_selectors,
                });
            }
        } else {
            result.push(gm.clone());
        }
    }

    result
}

/// Clear granular markings from specific selectors.
///
/// If `selectors` is empty, clears all granular markings.
pub fn clear_granular_markings(
    granular_markings: &[GranularMarking],
    selectors: &[&str],
) -> Vec<GranularMarking> {
    if selectors.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();

    for gm in granular_markings {
        let remaining_selectors: Vec<String> = gm
            .selectors
            .iter()
            .filter(|s| !selectors.contains(&s.as_str()))
            .cloned()
            .collect();

        if !remaining_selectors.is_empty() {
            result.push(GranularMarking {
                lang: gm.lang.clone(),
                marking_ref: gm.marking_ref.clone(),
                selectors: remaining_selectors,
            });
        }
    }

    result
}

/// Set granular markings for selectors (replace existing).
pub fn set_granular_markings(
    granular_markings: &[GranularMarking],
    marking: Identifier,
    selectors: Vec<String>,
) -> Vec<GranularMarking> {
    // First clear existing markings for these selectors
    let selector_refs: Vec<&str> = selectors.iter().map(|s| s.as_str()).collect();
    let mut result = clear_granular_markings(granular_markings, &selector_refs);

    // Then add the new marking
    result.push(GranularMarking::new(marking, selectors));
    result
}

/// Check if specific selectors are marked.
pub fn is_selector_marked(
    granular_markings: &[GranularMarking],
    selectors: &[&str],
    marking: Option<&Identifier>,
) -> bool {
    for gm in granular_markings {
        // Check if marking matches (or any if None)
        let marking_matches = match marking {
            Some(m) => gm.marking_ref.as_ref() == Some(m),
            None => gm.marking_ref.is_some(),
        };

        if marking_matches {
            for selector in selectors {
                if gm.selectors.iter().any(|s| s == *selector) {
                    return true;
                }
            }
        }
    }

    false
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Expand markings with multiple selectors into individual entries.
///
/// Each granular marking with multiple selectors becomes multiple
/// granular markings with single selectors.
pub fn expand_markings(markings: &[GranularMarking]) -> Vec<GranularMarking> {
    let mut result = Vec::new();

    for gm in markings {
        for selector in &gm.selectors {
            result.push(GranularMarking {
                lang: gm.lang.clone(),
                marking_ref: gm.marking_ref.clone(),
                selectors: vec![selector.clone()],
            });
        }
    }

    result
}

/// Compress markings with the same marking_ref into single entries.
///
/// Multiple granular markings with the same marking_ref are merged
/// into one with combined selectors.
pub fn compress_markings(markings: Vec<GranularMarking>) -> Vec<GranularMarking> {
    let mut by_marking: IndexMap<String, GranularMarking> = IndexMap::new();

    for gm in markings {
        let key = match (&gm.marking_ref, &gm.lang) {
            (Some(m), _) => format!("ref:{m}"),
            (_, Some(l)) => format!("lang:{l}"),
            _ => continue,
        };

        by_marking
            .entry(key)
            .and_modify(|existing| {
                for selector in &gm.selectors {
                    if !existing.selectors.contains(selector) {
                        existing.selectors.push(selector.clone());
                    }
                }
            })
            .or_insert(gm);
    }

    by_marking.into_values().collect()
}

/// Validate a selector against an object's JSON structure.
///
/// Checks if the selector path exists in the object.
pub fn validate_selector(obj: &Value, selector: &str) -> Result<()> {
    let parts: Vec<&str> = selector.split('.').collect();
    let mut current = obj;

    for (i, part) in parts.iter().enumerate() {
        // Handle array index notation (e.g., "labels[0]")
        if let Some(idx_start) = part.find('[') {
            let field = &part[..idx_start];
            let idx_str = &part[idx_start + 1..part.len() - 1];

            // Navigate to field
            current = current.get(field).ok_or_else(|| {
                Error::InvalidSelector(format!(
                    "Property '{}' not found at path '{}'",
                    field,
                    parts[..=i].join(".")
                ))
            })?;

            // Navigate to index
            let idx: usize = idx_str
                .parse()
                .map_err(|_| Error::InvalidSelector(format!("Invalid array index: {idx_str}")))?;

            current = current.get(idx).ok_or_else(|| {
                Error::InvalidSelector(format!("Array index {idx} out of bounds"))
            })?;
        } else {
            current = current.get(*part).ok_or_else(|| {
                Error::InvalidSelector(format!(
                    "Property '{}' not found at path '{}'",
                    part,
                    parts[..=i].join(".")
                ))
            })?;
        }
    }

    Ok(())
}

/// Walk an object tree yielding (path, value) tuples.
pub fn iter_path(obj: &Value) -> Vec<(String, &Value)> {
    let mut result = Vec::new();
    iter_path_recursive(obj, String::new(), &mut result);
    result
}

fn iter_path_recursive<'a>(obj: &'a Value, path: String, result: &mut Vec<(String, &'a Value)>) {
    match obj {
        Value::Object(map) => {
            for (key, value) in map {
                let new_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                result.push((new_path.clone(), value));
                iter_path_recursive(value, new_path, result);
            }
        }
        Value::Array(arr) => {
            for (i, value) in arr.iter().enumerate() {
                let new_path = format!("{path}[{i}]");
                result.push((new_path.clone(), value));
                iter_path_recursive(value, new_path, result);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_marking_ref(id: &str) -> Identifier {
        id.parse().unwrap()
    }

    #[test]
    fn test_add_object_marking() {
        let refs = vec![];
        let marking = make_marking_ref("marking-definition--11111111-1111-1111-1111-111111111111");

        let result = add_object_marking(&refs, marking.clone());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], marking);

        // Adding same marking should not duplicate
        let result2 = add_object_marking(&result, marking);
        assert_eq!(result2.len(), 1);
    }

    #[test]
    fn test_remove_object_marking() {
        let marking1 = make_marking_ref("marking-definition--11111111-1111-1111-1111-111111111111");
        let marking2 = make_marking_ref("marking-definition--22222222-2222-2222-2222-222222222222");
        let refs = vec![marking1.clone(), marking2.clone()];

        let result = remove_object_marking(&refs, &marking1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], marking2);
    }

    #[test]
    fn test_is_object_marked() {
        let marking = make_marking_ref("marking-definition--11111111-1111-1111-1111-111111111111");
        let refs = vec![marking.clone()];

        assert!(is_object_marked(&refs, None));
        assert!(is_object_marked(&refs, Some(&marking)));

        let other = make_marking_ref("marking-definition--22222222-2222-2222-2222-222222222222");
        assert!(!is_object_marked(&refs, Some(&other)));
    }

    #[test]
    fn test_add_granular_marking() {
        let markings = vec![];
        let marking = make_marking_ref("marking-definition--11111111-1111-1111-1111-111111111111");

        let result =
            add_granular_marking(&markings, marking.clone(), vec!["description".to_string()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].selectors, vec!["description"]);

        // Adding to same marking should merge selectors
        let result2 = add_granular_marking(&result, marking, vec!["name".to_string()]);
        assert_eq!(result2.len(), 1);
        assert!(result2[0].selectors.contains(&"description".to_string()));
        assert!(result2[0].selectors.contains(&"name".to_string()));
    }

    #[test]
    fn test_remove_granular_marking() {
        let marking = make_marking_ref("marking-definition--11111111-1111-1111-1111-111111111111");
        let gm = GranularMarking::new(
            marking.clone(),
            vec!["description".to_string(), "name".to_string()],
        );
        let markings = vec![gm];

        // Remove one selector
        let result = remove_granular_marking(&markings, &marking, &["description"]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].selectors, vec!["name"]);

        // Remove all selectors
        let result2 = remove_granular_marking(&markings, &marking, &[]);
        assert!(result2.is_empty());
    }

    #[test]
    fn test_get_granular_markings() {
        let marking = make_marking_ref("marking-definition--11111111-1111-1111-1111-111111111111");
        let gm = GranularMarking::new(marking.clone(), vec!["description".to_string()]);
        let markings = vec![gm];

        let result = get_granular_markings(&markings, &["description"], false, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], &marking);

        let result2 = get_granular_markings(&markings, &["name"], false, false);
        assert!(result2.is_empty());
    }

    #[test]
    fn test_get_granular_markings_inherited() {
        let marking = make_marking_ref("marking-definition--11111111-1111-1111-1111-111111111111");
        let gm = GranularMarking::new(marking, vec!["description".to_string()]);
        let markings = vec![gm];

        // Child property should inherit parent marking
        let result = get_granular_markings(&markings, &["description.text"], true, false);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_expand_compress_markings() {
        let marking = make_marking_ref("marking-definition--11111111-1111-1111-1111-111111111111");
        let gm = GranularMarking::new(marking, vec!["description".to_string(), "name".to_string()]);
        let markings = vec![gm];

        // Expand
        let expanded = expand_markings(&markings);
        assert_eq!(expanded.len(), 2);
        assert_eq!(expanded[0].selectors.len(), 1);
        assert_eq!(expanded[1].selectors.len(), 1);

        // Compress
        let compressed = compress_markings(expanded);
        assert_eq!(compressed.len(), 1);
        assert_eq!(compressed[0].selectors.len(), 2);
    }

    #[test]
    fn test_is_selector_marked() {
        let marking = make_marking_ref("marking-definition--11111111-1111-1111-1111-111111111111");
        let gm = GranularMarking::new(marking.clone(), vec!["description".to_string()]);
        let markings = vec![gm];

        assert!(is_selector_marked(&markings, &["description"], None));
        assert!(is_selector_marked(
            &markings,
            &["description"],
            Some(&marking)
        ));
        assert!(!is_selector_marked(&markings, &["name"], None));
    }

    #[test]
    fn test_validate_selector() {
        let obj = serde_json::json!({
            "name": "test",
            "labels": ["a", "b"],
            "external_references": [
                {"source_name": "test"}
            ]
        });

        assert!(validate_selector(&obj, "name").is_ok());
        assert!(validate_selector(&obj, "labels[0]").is_ok());
        assert!(validate_selector(&obj, "labels[2]").is_err());
        assert!(validate_selector(&obj, "nonexistent").is_err());
    }

    #[test]
    fn test_iter_path() {
        let obj = serde_json::json!({
            "name": "test",
            "labels": ["a", "b"]
        });

        let paths = iter_path(&obj);
        let path_strings: Vec<&str> = paths.iter().map(|(p, _)| p.as_str()).collect();

        assert!(path_strings.contains(&"name"));
        assert!(path_strings.contains(&"labels"));
        assert!(path_strings.contains(&"labels[0]"));
        assert!(path_strings.contains(&"labels[1]"));
    }
}
