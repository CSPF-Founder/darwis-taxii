//! STIX Pattern Language
//!
//! This module provides parsing and manipulation of STIX patterns.
//! STIX patterns are used in Indicators to describe observable patterns
//! that might be seen in cyber threat activity.

mod parser;
mod types;

pub use parser::{PatternParser, parse_pattern};
pub use types::*;

use crate::core::error::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A STIX pattern expression.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternExpression {
    /// A simple comparison expression.
    Comparison(ComparisonExpression),
    /// AND of two expressions.
    And(Box<PatternExpression>, Box<PatternExpression>),
    /// OR of two expressions.
    Or(Box<PatternExpression>, Box<PatternExpression>),
    /// FOLLOWEDBY temporal operator.
    FollowedBy(Box<PatternExpression>, Box<PatternExpression>),
    /// Observation expression with qualifier.
    Qualified(Box<PatternExpression>, Qualifier),
}

impl PatternExpression {
    /// Create an AND expression.
    pub fn and(self, other: PatternExpression) -> Self {
        PatternExpression::And(Box::new(self), Box::new(other))
    }

    /// Create an OR expression.
    pub fn or(self, other: PatternExpression) -> Self {
        PatternExpression::Or(Box::new(self), Box::new(other))
    }

    /// Add a WITHIN qualifier.
    pub fn within(self, seconds: u64) -> Self {
        PatternExpression::Qualified(Box::new(self), Qualifier::Within(seconds))
    }

    /// Add a REPEATS qualifier.
    pub fn repeats(self, count: u64) -> Self {
        PatternExpression::Qualified(Box::new(self), Qualifier::Repeats(count))
    }
}

impl fmt::Display for PatternExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternExpression::Comparison(c) => write!(f, "[{}]", c),
            PatternExpression::And(a, b) => write!(f, "{} AND {}", a, b),
            PatternExpression::Or(a, b) => write!(f, "{} OR {}", a, b),
            PatternExpression::FollowedBy(a, b) => write!(f, "{} FOLLOWEDBY {}", a, b),
            PatternExpression::Qualified(expr, qual) => write!(f, "{} {}", expr, qual),
        }
    }
}

/// A comparison expression within a pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct ComparisonExpression {
    /// The object type (e.g., "file", "ipv4-addr").
    pub object_type: String,
    /// The object path (e.g., "hashes.'SHA-256'").
    pub object_path: String,
    /// The comparison operator.
    pub operator: ComparisonOperator,
    /// The value to compare against.
    pub value: PatternValue,
    /// Whether this comparison is negated.
    pub negated: bool,
}

impl ComparisonExpression {
    /// Create a new comparison expression.
    pub fn new(
        object_type: impl Into<String>,
        object_path: impl Into<String>,
        operator: ComparisonOperator,
        value: PatternValue,
    ) -> Self {
        Self {
            object_type: object_type.into(),
            object_path: object_path.into(),
            operator,
            value,
            negated: false,
        }
    }

    /// Negate this comparison.
    pub fn negate(mut self) -> Self {
        self.negated = !self.negated;
        self
    }
}

impl fmt::Display for ComparisonExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let neg = if self.negated { "NOT " } else { "" };
        write!(
            f,
            "{}{}:{} {} {}",
            neg, self.object_type, self.object_path, self.operator, self.value
        )
    }
}

/// Comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    /// Equality (=).
    Equal,
    /// Inequality (!=).
    NotEqual,
    /// Less than (<).
    LessThan,
    /// Less than or equal (<=).
    LessThanOrEqual,
    /// Greater than (>).
    GreaterThan,
    /// Greater than or equal (>=).
    GreaterThanOrEqual,
    /// Pattern match (MATCHES).
    Matches,
    /// Like pattern (LIKE).
    Like,
    /// In set (IN).
    In,
    /// Is subset (ISSUBSET).
    IsSubset,
    /// Is superset (ISSUPERSET).
    IsSuperset,
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ComparisonOperator::Equal => "=",
            ComparisonOperator::NotEqual => "!=",
            ComparisonOperator::LessThan => "<",
            ComparisonOperator::LessThanOrEqual => "<=",
            ComparisonOperator::GreaterThan => ">",
            ComparisonOperator::GreaterThanOrEqual => ">=",
            ComparisonOperator::Matches => "MATCHES",
            ComparisonOperator::Like => "LIKE",
            ComparisonOperator::In => "IN",
            ComparisonOperator::IsSubset => "ISSUBSET",
            ComparisonOperator::IsSuperset => "ISSUPERSET",
        };
        write!(f, "{}", s)
    }
}

/// Values that can appear in pattern expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum PatternValue {
    /// String literal.
    String(String),
    /// Integer literal.
    Integer(i64),
    /// Float literal.
    Float(f64),
    /// Boolean literal.
    Boolean(bool),
    /// Timestamp literal.
    Timestamp(String),
    /// Binary literal (hex encoded).
    Binary(Vec<u8>),
    /// Hex literal.
    Hex(String),
    /// List of values.
    List(Vec<PatternValue>),
}

impl fmt::Display for PatternValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternValue::String(s) => write!(f, "'{}'", s.replace('\'', "\\'")),
            PatternValue::Integer(i) => write!(f, "{}", i),
            PatternValue::Float(v) => write!(f, "{}", v),
            PatternValue::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            PatternValue::Timestamp(t) => write!(f, "t'{}'", t),
            PatternValue::Binary(b) => write!(f, "b'{}'", BASE64.encode(b)),
            PatternValue::Hex(h) => write!(f, "h'{}'", h),
            PatternValue::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
        }
    }
}

/// Pattern qualifiers.
#[derive(Debug, Clone, PartialEq)]
pub enum Qualifier {
    /// WITHIN qualifier (time window in seconds).
    Within(u64),
    /// REPEATS qualifier.
    Repeats(u64),
    /// START/STOP time window.
    StartStop { start: String, stop: String },
}

impl fmt::Display for Qualifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Qualifier::Within(secs) => write!(f, "WITHIN {} SECONDS", secs),
            Qualifier::Repeats(count) => write!(f, "REPEATS {} TIMES", count),
            Qualifier::StartStop { start, stop } => {
                write!(f, "START t'{}' STOP t'{}'", start, stop)
            }
        }
    }
}

/// A complete STIX pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    /// The raw pattern string.
    raw: String,
}

impl Pattern {
    /// Create a pattern from a string.
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            raw: pattern.into(),
        }
    }

    /// Parse and validate the pattern.
    pub fn parse(&self) -> Result<PatternExpression> {
        parse_pattern(&self.raw)
    }

    /// Get the raw pattern string.
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Validate the pattern syntax.
    pub fn validate(&self) -> Result<()> {
        self.parse()?;
        Ok(())
    }

    /// Create an IP address pattern.
    pub fn ip_address(ip: &str) -> Self {
        Self::new(format!("[ipv4-addr:value = '{}']", ip))
    }

    /// Create a domain name pattern.
    pub fn domain(domain: &str) -> Self {
        Self::new(format!("[domain-name:value = '{}']", domain))
    }

    /// Create a file hash pattern.
    pub fn file_hash(algorithm: &str, hash: &str) -> Self {
        Self::new(format!("[file:hashes.'{}' = '{}']", algorithm, hash))
    }

    /// Create a URL pattern.
    pub fn url(url: &str) -> Self {
        Self::new(format!("[url:value = '{}']", url))
    }

    /// Create an email pattern.
    pub fn email(email: &str) -> Self {
        Self::new(format!("[email-addr:value = '{}']", email))
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl From<String> for Pattern {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for Pattern {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Pattern type enumeration matching the vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatternType {
    /// STIX pattern language.
    Stix,
    /// PCRE (Perl Compatible Regular Expressions).
    Pcre,
    /// Sigma rules.
    Sigma,
    /// Snort rules.
    Snort,
    /// Suricata rules.
    Suricata,
    /// YARA rules.
    Yara,
}

impl PatternType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PatternType::Stix => "stix",
            PatternType::Pcre => "pcre",
            PatternType::Sigma => "sigma",
            PatternType::Snort => "snort",
            PatternType::Suricata => "suricata",
            PatternType::Yara => "yara",
        }
    }
}

impl fmt::Display for PatternType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_creation() {
        let pattern = Pattern::ip_address("10.0.0.1");
        assert!(pattern.as_str().contains("10.0.0.1"));
    }

    #[test]
    fn test_pattern_display() {
        let pattern = Pattern::file_hash("SHA-256", "abc123");
        assert!(pattern.to_string().contains("SHA-256"));
    }

    #[test]
    fn test_comparison_expression() {
        let expr = ComparisonExpression::new(
            "ipv4-addr",
            "value",
            ComparisonOperator::Equal,
            PatternValue::String("10.0.0.1".to_string()),
        );
        assert_eq!(expr.object_type, "ipv4-addr");
    }
}
