//! Utility Functions
//!
//! This module provides various utility functions for working with STIX objects.

use crate::core::id::Identifier;
use crate::core::stix_object::StixObject;
use std::collections::HashMap;

/// Get the STIX type from an identifier.
pub fn get_type_from_id(id: &Identifier) -> &str {
    id.object_type()
}

/// Check if a string is a valid STIX type.
pub fn is_valid_stix_type(type_str: &str) -> bool {
    matches!(
        type_str,
        "attack-pattern"
            | "campaign"
            | "course-of-action"
            | "grouping"
            | "identity"
            | "incident"
            | "indicator"
            | "infrastructure"
            | "intrusion-set"
            | "location"
            | "malware"
            | "malware-analysis"
            | "note"
            | "observed-data"
            | "opinion"
            | "report"
            | "threat-actor"
            | "tool"
            | "vulnerability"
            | "relationship"
            | "sighting"
            | "artifact"
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
            | "marking-definition"
            | "language-content"
            | "bundle"
    )
}

/// Check if a type is a STIX Domain Object (SDO).
pub fn is_sdo_type(type_str: &str) -> bool {
    matches!(
        type_str,
        "attack-pattern"
            | "campaign"
            | "course-of-action"
            | "grouping"
            | "identity"
            | "incident"
            | "indicator"
            | "infrastructure"
            | "intrusion-set"
            | "location"
            | "malware"
            | "malware-analysis"
            | "note"
            | "observed-data"
            | "opinion"
            | "report"
            | "threat-actor"
            | "tool"
            | "vulnerability"
    )
}

/// Check if a type is a STIX Relationship Object (SRO).
pub fn is_sro_type(type_str: &str) -> bool {
    matches!(type_str, "relationship" | "sighting")
}

/// Check if a type is a STIX Cyber Observable Object (SCO).
pub fn is_sco_type(type_str: &str) -> bool {
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

/// Deduplicate a list of STIX objects, keeping the most recent version.
pub fn deduplicate(objects: Vec<StixObject>) -> Vec<StixObject> {
    let mut seen: HashMap<String, StixObject> = HashMap::new();

    for obj in objects {
        let id = obj.id().to_string();
        let modified = obj.modified();

        match seen.get(&id) {
            Some(existing) => {
                let existing_modified = existing.modified();
                match (modified, existing_modified) {
                    (Some(new_mod), Some(old_mod)) if new_mod > old_mod => {
                        seen.insert(id, obj);
                    }
                    (Some(_), None) => {
                        seen.insert(id, obj);
                    }
                    _ => {}
                }
            }
            None => {
                seen.insert(id, obj);
            }
        }
    }

    seen.into_values().collect()
}

/// Confidence scale conversion utilities.
pub mod confidence {
    /// Convert from None/Low/Med/High scale to 0-100.
    pub fn from_nlmh(value: &str) -> u8 {
        match value.to_lowercase().as_str() {
            "none" => 0,
            "low" => 15,
            "med" | "medium" => 50,
            "high" => 85,
            _ => 0,
        }
    }

    /// Convert from 0-100 to None/Low/Med/High scale.
    pub fn to_nlmh(value: u8) -> &'static str {
        match value {
            0..=10 => "None",
            11..=30 => "Low",
            31..=70 => "Medium",
            71..=100 => "High",
            _ => "None",
        }
    }

    /// Convert from Admiralty Credibility scale (A-F) to 0-100.
    pub fn from_admiralty(value: char) -> u8 {
        match value.to_ascii_uppercase() {
            'A' => 100,
            'B' => 80,
            'C' => 60,
            'D' => 40,
            'E' => 20,
            'F' => 0,
            _ => 0,
        }
    }

    /// Convert from 0-100 to Admiralty Credibility scale.
    pub fn to_admiralty(value: u8) -> char {
        match value {
            90..=100 => 'A',
            70..=89 => 'B',
            50..=69 => 'C',
            30..=49 => 'D',
            10..=29 => 'E',
            _ => 'F',
        }
    }

    /// Convert from WEP (Words of Estimative Probability) to 0-100.
    pub fn from_wep(value: &str) -> u8 {
        match value.to_lowercase().as_str() {
            "certain" => 100,
            "almost certain" | "almost certainly" => 95,
            "highly likely" | "probable" | "very likely" => 85,
            "likely" => 70,
            "roughly even chance" | "even chance" | "chances about even" => 50,
            "unlikely" => 30,
            "highly unlikely" | "improbable" => 15,
            "almost no chance" | "almost certainly not" | "remote" => 5,
            "impossible" => 0,
            _ => 50,
        }
    }

    /// Convert from DNI (Director of National Intelligence) scale to 0-100.
    pub fn from_dni(value: &str) -> u8 {
        match value.to_lowercase().as_str() {
            "almost no chance" | "remote" => 5,
            "very unlikely" | "highly improbable" => 15,
            "unlikely" | "improbable" => 30,
            "roughly even chance" | "roughly even" => 50,
            "likely" | "probable" => 70,
            "very likely" | "highly probable" => 85,
            "almost certain" | "nearly certain" => 95,
            _ => 50,
        }
    }

    /// Convert from 0-100 to WEP (Words of Estimative Probability) scale.
    pub fn to_wep(value: u8) -> &'static str {
        match value {
            0 => "Impossible",
            1..=9 => "Highly Unlikely",
            10..=29 => "Unlikely",
            30..=69 => "Even Chance",
            70..=89 => "Likely",
            90..=99 => "Highly Likely",
            100 => "Certain",
            _ => "Even Chance",
        }
    }

    /// Convert from 0-100 to DNI (Director of National Intelligence) scale.
    pub fn to_dni(value: u8) -> &'static str {
        match value {
            0..=4 => "Almost No Chance",
            5..=24 => "Very Unlikely",
            25..=39 => "Unlikely",
            40..=59 => "Roughly Even Chance",
            60..=79 => "Likely",
            80..=94 => "Very Likely",
            95..=100 => "Almost Certain",
            _ => "Roughly Even Chance",
        }
    }

    /// Convert from 0-10 scale to 0-100.
    pub fn from_zero_ten(value: &str) -> u8 {
        match value.trim() {
            "0" => 0,
            "1" => 10,
            "2" => 20,
            "3" => 30,
            "4" => 40,
            "5" => 50,
            "6" => 60,
            "7" => 70,
            "8" => 80,
            "9" => 90,
            "10" => 100,
            _ => 50,
        }
    }

    /// Convert from 0-100 to 0-10 scale.
    pub fn to_zero_ten(value: u8) -> &'static str {
        match value {
            0..=4 => "0",
            5..=14 => "1",
            15..=24 => "2",
            25..=34 => "3",
            35..=44 => "4",
            45..=54 => "5",
            55..=64 => "6",
            65..=74 => "7",
            75..=84 => "8",
            85..=94 => "9",
            95..=100 => "10",
            _ => "5",
        }
    }
}

/// Hash algorithm utilities.
pub mod hashes {
    /// Infer hash algorithm from name (case-insensitive).
    pub fn infer_algorithm(name: &str) -> String {
        match name.to_uppercase().as_str() {
            "MD5" => "MD5".to_string(),
            "SHA1" | "SHA-1" => "SHA-1".to_string(),
            "SHA256" | "SHA-256" => "SHA-256".to_string(),
            "SHA512" | "SHA-512" => "SHA-512".to_string(),
            "SHA3-256" | "SHA3256" => "SHA3-256".to_string(),
            "SHA3-512" | "SHA3512" => "SHA3-512".to_string(),
            "SSDEEP" => "SSDEEP".to_string(),
            "TLSH" => "TLSH".to_string(),
            _ => name.to_string(),
        }
    }

    /// Get expected length for a hash algorithm.
    pub fn expected_length(algorithm: &str) -> Option<usize> {
        match algorithm.to_uppercase().as_str() {
            "MD5" => Some(32),
            "SHA-1" | "SHA1" => Some(40),
            "SHA-256" | "SHA256" => Some(64),
            "SHA-512" | "SHA512" => Some(128),
            "SHA3-256" => Some(64),
            "SHA3-512" => Some(128),
            _ => None,
        }
    }

    /// Validate a hash value for the given algorithm.
    pub fn validate(algorithm: &str, value: &str) -> bool {
        if let Some(expected) = expected_length(algorithm) {
            value.len() == expected && value.chars().all(|c| c.is_ascii_hexdigit())
        } else {
            true // Unknown algorithm, accept any value
        }
    }
}

/// Defang/refang utilities for IOCs.
pub mod defang {
    /// Defang a URL.
    pub fn defang_url(url: &str) -> String {
        url.replace("http", "hxxp").replace(".", "[.]")
    }

    /// Refang a defanged URL.
    pub fn refang_url(url: &str) -> String {
        url.replace("hxxp", "http")
            .replace("[.]", ".")
            .replace("(dot)", ".")
    }

    /// Defang an IP address.
    pub fn defang_ip(ip: &str) -> String {
        ip.replace(".", "[.]")
    }

    /// Refang a defanged IP address.
    pub fn refang_ip(ip: &str) -> String {
        ip.replace("[.]", ".").replace("(dot)", ".")
    }

    /// Defang an email address.
    pub fn defang_email(email: &str) -> String {
        email.replace("@", "[@]").replace(".", "[.]")
    }

    /// Refang a defanged email address.
    pub fn refang_email(email: &str) -> String {
        email
            .replace("[@]", "@")
            .replace("[.]", ".")
            .replace("(at)", "@")
            .replace("(dot)", ".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_stix_type() {
        assert!(is_valid_stix_type("indicator"));
        assert!(is_valid_stix_type("malware"));
        assert!(!is_valid_stix_type("invalid-type"));
    }

    #[test]
    fn test_confidence_conversion() {
        assert_eq!(confidence::from_nlmh("high"), 85);
        assert_eq!(confidence::to_nlmh(85), "High");
    }

    #[test]
    fn test_confidence_wep() {
        // Test from_wep
        assert_eq!(confidence::from_wep("certain"), 100);
        assert_eq!(confidence::from_wep("almost certain"), 95);
        assert_eq!(confidence::from_wep("highly likely"), 85);
        assert_eq!(confidence::from_wep("likely"), 70);
        assert_eq!(confidence::from_wep("even chance"), 50);
        assert_eq!(confidence::from_wep("unlikely"), 30);
        assert_eq!(confidence::from_wep("highly unlikely"), 15);
        assert_eq!(confidence::from_wep("almost no chance"), 5);
        assert_eq!(confidence::from_wep("impossible"), 0);

        // Test to_wep
        assert_eq!(confidence::to_wep(0), "Impossible");
        assert_eq!(confidence::to_wep(5), "Highly Unlikely");
        assert_eq!(confidence::to_wep(20), "Unlikely");
        assert_eq!(confidence::to_wep(50), "Even Chance");
        assert_eq!(confidence::to_wep(75), "Likely");
        assert_eq!(confidence::to_wep(95), "Highly Likely");
        assert_eq!(confidence::to_wep(100), "Certain");
    }

    #[test]
    fn test_confidence_dni() {
        // Test from_dni
        assert_eq!(confidence::from_dni("almost no chance"), 5);
        assert_eq!(confidence::from_dni("very unlikely"), 15);
        assert_eq!(confidence::from_dni("unlikely"), 30);
        assert_eq!(confidence::from_dni("roughly even chance"), 50);
        assert_eq!(confidence::from_dni("likely"), 70);
        assert_eq!(confidence::from_dni("very likely"), 85);
        assert_eq!(confidence::from_dni("almost certain"), 95);

        // Test to_dni
        assert_eq!(confidence::to_dni(0), "Almost No Chance");
        assert_eq!(confidence::to_dni(10), "Very Unlikely");
        assert_eq!(confidence::to_dni(30), "Unlikely");
        assert_eq!(confidence::to_dni(50), "Roughly Even Chance");
        assert_eq!(confidence::to_dni(70), "Likely");
        assert_eq!(confidence::to_dni(90), "Very Likely");
        assert_eq!(confidence::to_dni(100), "Almost Certain");
    }

    #[test]
    fn test_confidence_zero_ten() {
        // Test from_zero_ten
        assert_eq!(confidence::from_zero_ten("0"), 0);
        assert_eq!(confidence::from_zero_ten("5"), 50);
        assert_eq!(confidence::from_zero_ten("10"), 100);

        // Test to_zero_ten
        assert_eq!(confidence::to_zero_ten(0), "0");
        assert_eq!(confidence::to_zero_ten(50), "5");
        assert_eq!(confidence::to_zero_ten(100), "10");

        // Test boundary cases
        assert_eq!(confidence::to_zero_ten(4), "0");
        assert_eq!(confidence::to_zero_ten(5), "1");
        assert_eq!(confidence::to_zero_ten(94), "9");
        assert_eq!(confidence::to_zero_ten(95), "10");
    }

    #[test]
    fn test_hash_validation() {
        assert!(hashes::validate("MD5", "d41d8cd98f00b204e9800998ecf8427e"));
        assert!(!hashes::validate("MD5", "invalid"));
    }

    #[test]
    fn test_defang_url() {
        let url = "https://example.com/malware";
        let defanged = defang::defang_url(url);
        assert!(defanged.contains("hxxps"));
        assert!(defanged.contains("[.]"));

        let refanged = defang::refang_url(&defanged);
        assert_eq!(refanged, url);
    }

    #[test]
    fn test_defang_ip() {
        let ip = "10.0.0.1";
        let defanged = defang::defang_ip(ip);
        assert_eq!(defanged, "10[.]0[.]0[.]1");

        let refanged = defang::refang_ip(&defanged);
        assert_eq!(refanged, ip);
    }
}
