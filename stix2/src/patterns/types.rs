//! Additional pattern types and helpers.

use super::Pattern;
use crate::core::error::{Error, Result};

/// Builder for creating STIX patterns programmatically.
#[derive(Debug, Default)]
pub struct PatternBuilder {
    expressions: Vec<String>,
}

impl PatternBuilder {
    /// Create a new pattern builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an IPv4 address comparison.
    pub fn ipv4_addr(mut self, value: &str) -> Self {
        self.expressions
            .push(format!("[ipv4-addr:value = '{value}']"));
        self
    }

    /// Add an IPv6 address comparison.
    pub fn ipv6_addr(mut self, value: &str) -> Self {
        self.expressions
            .push(format!("[ipv6-addr:value = '{value}']"));
        self
    }

    /// Add a domain name comparison.
    pub fn domain_name(mut self, value: &str) -> Self {
        self.expressions
            .push(format!("[domain-name:value = '{value}']"));
        self
    }

    /// Add a URL comparison.
    pub fn url(mut self, value: &str) -> Self {
        self.expressions.push(format!("[url:value = '{value}']"));
        self
    }

    /// Add a file hash comparison.
    pub fn file_hash(mut self, algorithm: &str, value: &str) -> Self {
        self.expressions
            .push(format!("[file:hashes.'{algorithm}' = '{value}']"));
        self
    }

    /// Add a file name comparison.
    pub fn file_name(mut self, value: &str) -> Self {
        self.expressions.push(format!("[file:name = '{value}']"));
        self
    }

    /// Add a custom comparison.
    pub fn custom(
        mut self,
        object_type: &str,
        object_path: &str,
        operator: &str,
        value: &str,
    ) -> Self {
        self.expressions.push(format!(
            "[{object_type}:{object_path} {operator} '{value}']"
        ));
        self
    }

    /// Build the pattern with AND logic.
    ///
    /// Returns an error if no expressions were added to the builder.
    pub fn build_and(self) -> Result<Pattern> {
        if self.expressions.is_empty() {
            return Err(Error::builder("PatternBuilder has no expressions"));
        }
        Ok(Pattern::new(self.expressions.join(" AND ")))
    }

    /// Build the pattern with OR logic.
    ///
    /// Returns an error if no expressions were added to the builder.
    pub fn build_or(self) -> Result<Pattern> {
        if self.expressions.is_empty() {
            return Err(Error::builder("PatternBuilder has no expressions"));
        }
        Ok(Pattern::new(self.expressions.join(" OR ")))
    }

    /// Build a single pattern (takes the first expression).
    ///
    /// Returns an error if no expressions were added to the builder.
    pub fn build(self) -> Result<Pattern> {
        self.expressions
            .into_iter()
            .next()
            .map(Pattern::new)
            .ok_or_else(|| Error::builder("PatternBuilder has no expressions"))
    }
}

/// Helper for creating common patterns.
pub mod patterns {
    use super::Pattern;

    /// Create a pattern matching any of the given IP addresses.
    pub fn ip_addresses(ips: &[&str]) -> Pattern {
        let exprs: Vec<String> = ips
            .iter()
            .map(|ip| format!("[ipv4-addr:value = '{ip}']"))
            .collect();
        Pattern::new(exprs.join(" OR "))
    }

    /// Create a pattern matching any of the given domains.
    pub fn domains(domains: &[&str]) -> Pattern {
        let exprs: Vec<String> = domains
            .iter()
            .map(|d| format!("[domain-name:value = '{d}']"))
            .collect();
        Pattern::new(exprs.join(" OR "))
    }

    /// Create a pattern matching any of the given file hashes.
    pub fn file_hashes(algorithm: &str, hashes: &[&str]) -> Pattern {
        let exprs: Vec<String> = hashes
            .iter()
            .map(|h| format!("[file:hashes.'{algorithm}' = '{h}']"))
            .collect();
        Pattern::new(exprs.join(" OR "))
    }

    /// Create a pattern for network traffic to/from an IP.
    pub fn network_traffic_ip(ip: &str) -> Pattern {
        Pattern::new(format!(
            "[network-traffic:src_ref.type = 'ipv4-addr' AND network-traffic:src_ref.value = '{ip}'] OR \
             [network-traffic:dst_ref.type = 'ipv4-addr' AND network-traffic:dst_ref.value = '{ip}']"
        ))
    }

    /// Create a pattern for a process with command line matching.
    pub fn process_command_line(pattern: &str) -> Pattern {
        Pattern::new(format!("[process:command_line MATCHES '{pattern}']"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_builder() {
        let pattern = PatternBuilder::new()
            .ipv4_addr("10.0.0.1")
            .ipv4_addr("10.0.0.2")
            .build_or()
            .unwrap();

        assert!(pattern.as_str().contains("10.0.0.1"));
        assert!(pattern.as_str().contains("10.0.0.2"));
        assert!(pattern.as_str().contains(" OR "));
    }

    #[test]
    fn test_pattern_builder_empty_error() {
        let result = PatternBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_patterns_helpers() {
        let pattern = patterns::ip_addresses(&["10.0.0.1", "10.0.0.2"]);
        assert!(pattern.as_str().contains("OR"));
    }
}
