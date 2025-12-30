//! STIX Object Semantic Equivalence
//!
//! This module provides utilities for determining semantic equivalence and
//! similarity between STIX objects.

use std::collections::HashSet;

use crate::core::stix_object::StixObject;

/// The default threshold for object equivalence (0-100).
pub const DEFAULT_THRESHOLD: f64 = 70.0;

/// Determines if two STIX objects are semantically equivalent.
///
/// Two objects are considered equivalent if their similarity score
/// is greater than or equal to the threshold.
///
/// # Arguments
/// * `obj1` - First STIX object
/// * `obj2` - Second STIX object
/// * `threshold` - Similarity threshold (0-100), default is 70
///
/// # Returns
/// `true` if the objects are equivalent, `false` otherwise
pub fn object_equivalence(obj1: &StixObject, obj2: &StixObject, threshold: Option<f64>) -> bool {
    let threshold = threshold.unwrap_or(DEFAULT_THRESHOLD);
    let similarity = object_similarity(obj1, obj2);
    similarity >= threshold
}

/// Calculates the similarity score between two STIX objects.
///
/// Returns a value between 0.0 and 100.0 indicating how similar the objects are.
///
/// # Arguments
/// * `obj1` - First STIX object
/// * `obj2` - Second STIX object
///
/// # Returns
/// Similarity score between 0.0 and 100.0
pub fn object_similarity(obj1: &StixObject, obj2: &StixObject) -> f64 {
    // Objects of different types have 0 similarity
    if std::mem::discriminant(obj1) != std::mem::discriminant(obj2) {
        return 0.0;
    }

    match (obj1, obj2) {
        (StixObject::AttackPattern(a), StixObject::AttackPattern(b)) => {
            let mut score = 0.0;
            let mut weight = 0.0;

            // Name comparison (30% weight)
            score += 30.0 * partial_string_match(&a.name, &b.name);
            weight += 30.0;

            // External references (70% weight)
            score += 70.0 * partial_external_references_match(&a.common, &b.common);
            weight += 70.0;

            if weight > 0.0 {
                (score / weight) * 100.0
            } else {
                0.0
            }
        }
        (StixObject::Campaign(a), StixObject::Campaign(b)) => {
            let mut score = 0.0;
            let mut weight = 0.0;

            // Name comparison (60% weight)
            score += 60.0 * partial_string_match(&a.name, &b.name);
            weight += 60.0;

            // Aliases (40% weight)
            score += 40.0 * partial_list_match(&a.aliases, &b.aliases);
            weight += 40.0;

            if weight > 0.0 {
                (score / weight) * 100.0
            } else {
                0.0
            }
        }
        (StixObject::Identity(a), StixObject::Identity(b)) => {
            let mut score = 0.0;
            let mut weight = 0.0;

            // Name comparison (60% weight)
            score += 60.0 * partial_string_match(&a.name, &b.name);
            weight += 60.0;

            // Identity class (20% weight)
            if let (Some(class1), Some(class2)) = (&a.identity_class, &b.identity_class) {
                score += 20.0 * exact_match(&class1.as_str(), &class2.as_str());
                weight += 20.0;
            }

            // Sectors (20% weight)
            let sectors1: Vec<String> = a.sectors.iter().map(|s| s.as_str().to_string()).collect();
            let sectors2: Vec<String> = b.sectors.iter().map(|s| s.as_str().to_string()).collect();
            score += 20.0 * partial_list_match(&sectors1, &sectors2);
            weight += 20.0;

            if weight > 0.0 {
                (score / weight) * 100.0
            } else {
                0.0
            }
        }
        (StixObject::Indicator(a), StixObject::Indicator(b)) => {
            let mut score = 0.0;
            let mut weight = 0.0;

            // Indicator types (15% weight)
            let types1: Vec<String> = a
                .indicator_types
                .iter()
                .map(|t| t.as_str().to_string())
                .collect();
            let types2: Vec<String> = b
                .indicator_types
                .iter()
                .map(|t| t.as_str().to_string())
                .collect();
            score += 15.0 * partial_list_match(&types1, &types2);
            weight += 15.0;

            // Pattern (80% weight)
            score += 80.0 * exact_match(&a.pattern, &b.pattern);
            weight += 80.0;

            // Valid from (5% weight) - simplified to exact match
            score += 5.0 * exact_match(&a.valid_from.to_string(), &b.valid_from.to_string());
            weight += 5.0;

            if weight > 0.0 {
                (score / weight) * 100.0
            } else {
                0.0
            }
        }
        (StixObject::Malware(a), StixObject::Malware(b)) => {
            let mut score = 0.0;
            let mut weight = 0.0;

            // Malware types (20% weight)
            let types1: Vec<String> = a
                .malware_types
                .iter()
                .map(|t| t.as_str().to_string())
                .collect();
            let types2: Vec<String> = b
                .malware_types
                .iter()
                .map(|t| t.as_str().to_string())
                .collect();
            score += 20.0 * partial_list_match(&types1, &types2);
            weight += 20.0;

            // Name (80% weight)
            if let (Some(name1), Some(name2)) = (&a.name, &b.name) {
                score += 80.0 * partial_string_match(name1, name2);
                weight += 80.0;
            }

            if weight > 0.0 {
                (score / weight) * 100.0
            } else {
                0.0
            }
        }
        (StixObject::ThreatActor(a), StixObject::ThreatActor(b)) => {
            let mut score = 0.0;
            let mut weight = 0.0;

            // Name (60% weight)
            score += 60.0 * partial_string_match(&a.name, &b.name);
            weight += 60.0;

            // Threat actor types (20% weight)
            let types1: Vec<String> = a
                .threat_actor_types
                .iter()
                .map(|t| t.as_str().to_string())
                .collect();
            let types2: Vec<String> = b
                .threat_actor_types
                .iter()
                .map(|t| t.as_str().to_string())
                .collect();
            score += 20.0 * partial_list_match(&types1, &types2);
            weight += 20.0;

            // Aliases (20% weight)
            score += 20.0 * partial_list_match(&a.aliases, &b.aliases);
            weight += 20.0;

            if weight > 0.0 {
                (score / weight) * 100.0
            } else {
                0.0
            }
        }
        (StixObject::Tool(a), StixObject::Tool(b)) => {
            let mut score = 0.0;
            let mut weight = 0.0;

            // Tool types (20% weight)
            let types1: Vec<String> = a
                .tool_types
                .iter()
                .map(|t| t.as_str().to_string())
                .collect();
            let types2: Vec<String> = b
                .tool_types
                .iter()
                .map(|t| t.as_str().to_string())
                .collect();
            score += 20.0 * partial_list_match(&types1, &types2);
            weight += 20.0;

            // Name (80% weight)
            score += 80.0 * partial_string_match(&a.name, &b.name);
            weight += 80.0;

            if weight > 0.0 {
                (score / weight) * 100.0
            } else {
                0.0
            }
        }
        (StixObject::Vulnerability(a), StixObject::Vulnerability(b)) => {
            let mut score = 0.0;
            let mut weight = 0.0;

            // Name (30% weight)
            score += 30.0 * partial_string_match(&a.name, &b.name);
            weight += 30.0;

            // External references (70% weight)
            score += 70.0 * partial_external_references_match(&a.common, &b.common);
            weight += 70.0;

            if weight > 0.0 {
                (score / weight) * 100.0
            } else {
                0.0
            }
        }
        (StixObject::Relationship(a), StixObject::Relationship(b)) => {
            let mut score = 0.0;
            let mut weight = 0.0;

            // Relationship type (20% weight)
            score += 20.0 * exact_match(&a.relationship_type, &b.relationship_type);
            weight += 20.0;

            // Source ref (40% weight)
            score += 40.0 * exact_match(&a.source_ref.to_string(), &b.source_ref.to_string());
            weight += 40.0;

            // Target ref (40% weight)
            score += 40.0 * exact_match(&a.target_ref.to_string(), &b.target_ref.to_string());
            weight += 40.0;

            if weight > 0.0 {
                (score / weight) * 100.0
            } else {
                0.0
            }
        }
        // For objects without specific similarity logic, use ID-based comparison
        _ => {
            let id1 = get_id(obj1);
            let id2 = get_id(obj2);
            if id1 == id2 { 100.0 } else { 0.0 }
        }
    }
}

/// Performs an exact match comparison.
fn exact_match<T: PartialEq>(val1: &T, val2: &T) -> f64 {
    if val1 == val2 { 1.0 } else { 0.0 }
}

/// Performs a partial string match using basic similarity.
///
/// Uses a simplified approach: if strings are equal, score is 1.0.
/// If they share common words, score is proportional to overlap.
fn partial_string_match(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }

    let words1: HashSet<&str> = s1.split_whitespace().collect();
    let words2: HashSet<&str> = s2.split_whitespace().collect();

    if words1.is_empty() || words2.is_empty() {
        return 0.0;
    }

    let intersection = words1.intersection(&words2).count() as f64;
    let max_len = words1.len().max(words2.len()) as f64;

    intersection / max_len
}

/// Performs a partial list match based on set intersection.
fn partial_list_match(list1: &[String], list2: &[String]) -> f64 {
    if list1.is_empty() && list2.is_empty() {
        return 1.0;
    }

    if list1.is_empty() || list2.is_empty() {
        return 0.0;
    }

    let set1: HashSet<&str> = list1.iter().map(|s| s.as_str()).collect();
    let set2: HashSet<&str> = list2.iter().map(|s| s.as_str()).collect();

    let intersection = set1.intersection(&set2).count() as f64;
    let max_len = set1.len().max(set2.len()) as f64;

    intersection / max_len
}

/// Performs external reference matching.
fn partial_external_references_match(
    common1: &crate::core::common::CommonProperties,
    common2: &crate::core::common::CommonProperties,
) -> f64 {
    if common1.external_references.is_empty() && common2.external_references.is_empty() {
        return 1.0;
    }

    if common1.external_references.is_empty() || common2.external_references.is_empty() {
        return 0.0;
    }

    let mut matches = 0;

    // Known STIX external reference sources
    let stix_sources: HashSet<&str> = ["veris", "cve", "capec", "mitre-attack"]
        .iter()
        .copied()
        .collect();

    for ref1 in &common1.external_references {
        for ref2 in &common2.external_references {
            let sn_match = ref1.source_name == ref2.source_name;
            let ei_match = ref1.external_id.is_some()
                && ref2.external_id.is_some()
                && ref1.external_id == ref2.external_id;
            let url_match = ref1.url.is_some() && ref2.url.is_some() && ref1.url == ref2.url;

            // Perfect match for STIX-defined sources
            if sn_match
                && (ei_match || url_match)
                && stix_sources.contains(ref1.source_name.as_str())
            {
                return 1.0;
            }

            if sn_match || ei_match || url_match {
                matches += 1;
                break;
            }
        }
    }

    let max_refs = common1
        .external_references
        .len()
        .max(common2.external_references.len()) as f64;
    matches as f64 / max_refs
}

fn get_id(obj: &StixObject) -> String {
    match obj {
        StixObject::AttackPattern(o) => o.id.to_string(),
        StixObject::Campaign(o) => o.id.to_string(),
        StixObject::CourseOfAction(o) => o.id.to_string(),
        StixObject::Grouping(o) => o.id.to_string(),
        StixObject::Identity(o) => o.id.to_string(),
        StixObject::Incident(o) => o.id.to_string(),
        StixObject::Indicator(o) => o.id.to_string(),
        StixObject::Infrastructure(o) => o.id.to_string(),
        StixObject::IntrusionSet(o) => o.id.to_string(),
        StixObject::Location(o) => o.id.to_string(),
        StixObject::Malware(o) => o.id.to_string(),
        StixObject::MalwareAnalysis(o) => o.id.to_string(),
        StixObject::Note(o) => o.id.to_string(),
        StixObject::ObservedData(o) => o.id.to_string(),
        StixObject::Opinion(o) => o.id.to_string(),
        StixObject::Report(o) => o.id.to_string(),
        StixObject::ThreatActor(o) => o.id.to_string(),
        StixObject::Tool(o) => o.id.to_string(),
        StixObject::Vulnerability(o) => o.id.to_string(),
        StixObject::Relationship(o) => o.id.to_string(),
        StixObject::Sighting(o) => o.id.to_string(),
        StixObject::MarkingDefinition(o) => o.id.to_string(),
        StixObject::LanguageContent(o) => o.id.to_string(),
        StixObject::Artifact(o) => o.id.to_string(),
        StixObject::AutonomousSystem(o) => o.id.to_string(),
        StixObject::Directory(o) => o.id.to_string(),
        StixObject::DomainName(o) => o.id.to_string(),
        StixObject::EmailAddress(o) => o.id.to_string(),
        StixObject::EmailMessage(o) => o.id.to_string(),
        StixObject::File(o) => o.id.to_string(),
        StixObject::IPv4Address(o) => o.id.to_string(),
        StixObject::IPv6Address(o) => o.id.to_string(),
        StixObject::MacAddress(o) => o.id.to_string(),
        StixObject::Mutex(o) => o.id.to_string(),
        StixObject::NetworkTraffic(o) => o.id.to_string(),
        StixObject::Process(o) => o.id.to_string(),
        StixObject::Software(o) => o.id.to_string(),
        StixObject::Url(o) => o.id.to_string(),
        StixObject::UserAccount(o) => o.id.to_string(),
        StixObject::WindowsRegistryKey(o) => o.id.to_string(),
        StixObject::X509Certificate(o) => o.id.to_string(),
        StixObject::Custom(o) => o.id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::{Indicator, Malware};
    use crate::vocab::{MalwareType, PatternType};

    #[test]
    fn test_exact_match() {
        assert_eq!(exact_match(&"hello", &"hello"), 1.0);
        assert_eq!(exact_match(&"hello", &"world"), 0.0);
    }

    #[test]
    fn test_partial_string_match() {
        assert_eq!(partial_string_match("hello world", "hello world"), 1.0);
        assert!(partial_string_match("hello world", "hello there") > 0.0);
        assert!(partial_string_match("hello world", "goodbye moon") < 0.5);
    }

    #[test]
    fn test_partial_list_match() {
        let list1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let list2 = vec!["b".to_string(), "c".to_string(), "d".to_string()];
        assert!(partial_list_match(&list1, &list2) > 0.5);
    }

    #[test]
    fn test_indicator_similarity() {
        let ind1 = Indicator::builder()
            .name("Malicious IP")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let ind2 = Indicator::builder()
            .name("Malicious IP")
            .pattern("[ipv4-addr:value = '10.0.0.1']")
            .pattern_type(PatternType::Stix)
            .valid_from_now()
            .build()
            .unwrap();

        let similarity =
            object_similarity(&StixObject::Indicator(ind1), &StixObject::Indicator(ind2));
        assert!(similarity > 80.0);
    }

    #[test]
    fn test_malware_equivalence() {
        let mal1 = Malware::builder()
            .name("Evil Malware")
            .is_family(false)
            .malware_type(MalwareType::Ransomware)
            .build()
            .unwrap();

        let mal2 = Malware::builder()
            .name("Evil Malware")
            .is_family(false)
            .malware_type(MalwareType::Ransomware)
            .build()
            .unwrap();

        assert!(object_equivalence(
            &StixObject::Malware(mal1),
            &StixObject::Malware(mal2),
            None
        ));
    }
}
