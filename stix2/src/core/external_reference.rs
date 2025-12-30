//! External reference type for linking to external resources.
//!
//! External references provide links to related information outside
//! of the STIX content, such as CVE entries, external reports, etc.

use crate::core::common::Hashes;
use crate::core::error::Result;
use crate::validation::{Constrained, check_at_least_one, check_hash_algorithms};
use serde::{Deserialize, Serialize};

/// An external reference to additional information.
///
/// External references are used to describe pointers to information
/// represented outside of STIX. Examples include CVE references,
/// links to reports, or references to threat intelligence platforms.
///
/// # Example
///
/// ```rust
/// use stix2::core::ExternalReference;
///
/// let cve_ref = ExternalReference::new("cve")
///     .with_external_id("CVE-2021-44228")
///     .with_url("https://nvd.nist.gov/vuln/detail/CVE-2021-44228");
///
/// let mitre_ref = ExternalReference::new("mitre-attack")
///     .with_external_id("T1566.001")
///     .with_url("https://attack.mitre.org/techniques/T1566/001");
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalReference {
    /// The name of the source (e.g., "cve", "mitre-attack").
    pub source_name: String,

    /// A human-readable description of the reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The URL reference to an external resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Hash values for the external reference content.
    #[serde(default, skip_serializing_if = "Hashes::is_empty")]
    pub hashes: Hashes,

    /// An identifier for the external reference (e.g., CVE ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
}

impl ExternalReference {
    /// Create a new external reference with the given source name.
    ///
    /// # Arguments
    ///
    /// * `source_name` - The name of the external source
    pub fn new(source_name: impl Into<String>) -> Self {
        Self {
            source_name: source_name.into(),
            description: None,
            url: None,
            hashes: Hashes::new(),
            external_id: None,
        }
    }

    /// Create a CVE reference.
    ///
    /// # Arguments
    ///
    /// * `cve_id` - The CVE identifier (e.g., "CVE-2021-44228")
    pub fn cve(cve_id: impl Into<String>) -> Self {
        let cve_id = cve_id.into();
        Self::new("cve")
            .with_external_id(&cve_id)
            .with_url(format!("https://nvd.nist.gov/vuln/detail/{}", cve_id))
    }

    /// Create a MITRE ATT&CK reference.
    ///
    /// # Arguments
    ///
    /// * `technique_id` - The technique ID (e.g., "T1566.001")
    pub fn mitre_attack(technique_id: impl Into<String>) -> Self {
        let technique_id = technique_id.into();
        let url = format!(
            "https://attack.mitre.org/techniques/{}",
            technique_id.replace('.', "/")
        );
        Self::new("mitre-attack")
            .with_external_id(&technique_id)
            .with_url(url)
    }

    /// Create a CAPEC reference.
    ///
    /// # Arguments
    ///
    /// * `capec_id` - The CAPEC ID (e.g., "CAPEC-1")
    pub fn capec(capec_id: impl Into<String>) -> Self {
        let capec_id = capec_id.into();
        let id_num = capec_id.strip_prefix("CAPEC-").unwrap_or(&capec_id);
        Self::new("capec")
            .with_external_id(&capec_id)
            .with_url(format!(
                "https://capec.mitre.org/data/definitions/{}.html",
                id_num
            ))
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set the external ID.
    pub fn with_external_id(mut self, external_id: impl Into<String>) -> Self {
        self.external_id = Some(external_id.into());
        self
    }

    /// Add a hash value.
    pub fn with_hash(mut self, algorithm: impl Into<String>, value: impl Into<String>) -> Self {
        self.hashes.insert(algorithm.into(), value.into());
        self
    }

    /// Check if this is a CVE reference.
    pub fn is_cve(&self) -> bool {
        self.source_name.eq_ignore_ascii_case("cve")
    }

    /// Check if this is a MITRE ATT&CK reference.
    pub fn is_mitre_attack(&self) -> bool {
        self.source_name.eq_ignore_ascii_case("mitre-attack")
    }

    /// Get the CVE ID if this is a CVE reference.
    pub fn cve_id(&self) -> Option<&str> {
        if self.is_cve() {
            self.external_id.as_deref()
        } else {
            None
        }
    }

    /// Validate this external reference.
    ///
    /// Called automatically on deserialization when using strict mode.
    pub fn validate(&self) -> Result<()> {
        self.validate_constraints()
    }
}

impl Constrained for ExternalReference {
    /// Validate ExternalReference constraints.
    ///
    /// - At least one of `description`, `external_id`, or `url` must be present
    /// - Hash algorithms must be from the standard list
    fn validate_constraints(&self) -> Result<()> {
        // Build list of present properties
        let mut present = Vec::new();
        if self.description.is_some() {
            present.push("description");
        }
        if self.external_id.is_some() {
            present.push("external_id");
        }
        if self.url.is_some() {
            present.push("url");
        }

        // At least one must be present
        check_at_least_one(&present, &["description", "external_id", "url"])?;

        // Validate hash algorithms
        if !self.hashes.is_empty() {
            let algorithms: Vec<&str> = self.hashes.keys().map(|s| s.as_str()).collect();
            check_hash_algorithms(&algorithms)?;
        }

        Ok(())
    }
}

/// Well-known source names for external references.
pub mod sources {
    /// CVE - Common Vulnerabilities and Exposures
    pub const CVE: &str = "cve";
    /// MITRE ATT&CK
    pub const MITRE_ATTACK: &str = "mitre-attack";
    /// CAPEC - Common Attack Pattern Enumeration and Classification
    pub const CAPEC: &str = "capec";
    /// CWE - Common Weakness Enumeration
    pub const CWE: &str = "cwe";
    /// NIST NVD
    pub const NIST_NVD: &str = "nist-nvd";
    /// Virus Total
    pub const VIRUSTOTAL: &str = "virustotal";
    /// AbuseIPDB
    pub const ABUSEIPDB: &str = "abuseipdb";
    /// Shodan
    pub const SHODAN: &str = "shodan";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_external_reference() {
        let ref_ = ExternalReference::new("cve")
            .with_external_id("CVE-2021-44228")
            .with_description("Log4Shell vulnerability");

        assert_eq!(ref_.source_name, "cve");
        assert_eq!(ref_.external_id.as_deref(), Some("CVE-2021-44228"));
        assert!(ref_.is_cve());
    }

    #[test]
    fn test_cve_reference() {
        let ref_ = ExternalReference::cve("CVE-2021-44228");
        assert!(ref_.is_cve());
        assert_eq!(ref_.cve_id(), Some("CVE-2021-44228"));
        assert!(ref_.url.as_ref().unwrap().contains("nvd.nist.gov"));
    }

    #[test]
    fn test_mitre_attack_reference() {
        let ref_ = ExternalReference::mitre_attack("T1566.001");
        assert!(ref_.is_mitre_attack());
        assert!(ref_.url.as_ref().unwrap().contains("attack.mitre.org"));
    }

    #[test]
    fn test_serialization() {
        let ref_ = ExternalReference::new("test-source")
            .with_external_id("TEST-123")
            .with_url("https://example.com");

        let json = serde_json::to_string(&ref_).unwrap();
        let parsed: ExternalReference = serde_json::from_str(&json).unwrap();
        assert_eq!(ref_, parsed);
    }

    #[test]
    fn test_with_hash() {
        let ref_ = ExternalReference::new("file-report").with_hash("SHA-256", "abc123");

        assert_eq!(ref_.hashes.get("SHA-256"), Some(&"abc123".to_string()));
    }
}
