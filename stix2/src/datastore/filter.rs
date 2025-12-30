//! Filter support for DataStore queries.

use crate::core::id::Identifier;
use serde::{Deserialize, Serialize};

/// Filter operator for queries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
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
    /// Value is in a set.
    In,
    /// Value contains substring.
    Contains,
}

/// A filter for querying STIX objects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    /// The property to filter on.
    pub property: String,
    /// The operator to use.
    pub operator: FilterOperator,
    /// The value to compare against.
    pub value: FilterValue,
}

/// Value types that can be used in filters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    /// String value.
    String(String),
    /// Integer value.
    Integer(i64),
    /// Float value.
    Float(f64),
    /// Boolean value.
    Boolean(bool),
    /// List of values.
    List(Vec<String>),
}

impl Filter {
    /// Create a new filter.
    pub fn new(
        property: impl Into<String>,
        operator: FilterOperator,
        value: impl Into<FilterValue>,
    ) -> Self {
        Self {
            property: property.into(),
            operator,
            value: value.into(),
        }
    }

    /// Create an equality filter.
    pub fn eq(property: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        Self::new(property, FilterOperator::Equal, value)
    }

    /// Create a not-equal filter.
    pub fn neq(property: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        Self::new(property, FilterOperator::NotEqual, value)
    }

    /// Create a type filter.
    pub fn by_type(type_name: impl Into<String>) -> Self {
        Self::eq("type", type_name.into())
    }

    /// Create a created_by filter.
    pub fn created_by(identity_id: impl Into<String>) -> Self {
        Self::eq("created_by_ref", identity_id.into())
    }

    /// Check if an object matches this filter.
    pub fn matches(&self, value: &serde_json::Value) -> bool {
        let obj_value = match value.get(&self.property) {
            Some(v) => v,
            None => return false,
        };

        match (&self.operator, &self.value) {
            (FilterOperator::Equal, FilterValue::String(s)) => {
                obj_value.as_str() == Some(s.as_str())
            }
            (FilterOperator::NotEqual, FilterValue::String(s)) => {
                obj_value.as_str() != Some(s.as_str())
            }
            (FilterOperator::Equal, FilterValue::Integer(i)) => obj_value.as_i64() == Some(*i),
            (FilterOperator::Equal, FilterValue::Boolean(b)) => obj_value.as_bool() == Some(*b),
            (FilterOperator::In, FilterValue::List(items)) => {
                if let Some(s) = obj_value.as_str() {
                    items.iter().any(|item| item == s)
                } else {
                    false
                }
            }
            (FilterOperator::Contains, FilterValue::String(s)) => {
                if let Some(obj_str) = obj_value.as_str() {
                    obj_str.contains(s.as_str())
                } else {
                    false
                }
            }
            (FilterOperator::LessThan, FilterValue::Integer(i)) => {
                obj_value.as_i64().map(|v| v < *i).unwrap_or(false)
            }
            (FilterOperator::LessThanOrEqual, FilterValue::Integer(i)) => {
                obj_value.as_i64().map(|v| v <= *i).unwrap_or(false)
            }
            (FilterOperator::GreaterThan, FilterValue::Integer(i)) => {
                obj_value.as_i64().map(|v| v > *i).unwrap_or(false)
            }
            (FilterOperator::GreaterThanOrEqual, FilterValue::Integer(i)) => {
                obj_value.as_i64().map(|v| v >= *i).unwrap_or(false)
            }
            _ => false,
        }
    }
}

impl From<String> for FilterValue {
    fn from(s: String) -> Self {
        FilterValue::String(s)
    }
}

impl From<&str> for FilterValue {
    fn from(s: &str) -> Self {
        FilterValue::String(s.to_string())
    }
}

impl From<i64> for FilterValue {
    fn from(i: i64) -> Self {
        FilterValue::Integer(i)
    }
}

impl From<bool> for FilterValue {
    fn from(b: bool) -> Self {
        FilterValue::Boolean(b)
    }
}

impl From<Vec<String>> for FilterValue {
    fn from(v: Vec<String>) -> Self {
        FilterValue::List(v)
    }
}

impl From<&Identifier> for FilterValue {
    fn from(id: &Identifier) -> Self {
        FilterValue::String(id.to_string())
    }
}

impl From<Identifier> for FilterValue {
    fn from(id: Identifier) -> Self {
        FilterValue::String(id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_creation() {
        let filter = Filter::eq("type", "indicator");
        assert_eq!(filter.property, "type");
    }

    #[test]
    fn test_filter_matches() {
        let filter = Filter::eq("type", "indicator");
        let obj = serde_json::json!({"type": "indicator"});
        assert!(filter.matches(&obj));
    }

    #[test]
    fn test_filter_not_matches() {
        let filter = Filter::eq("type", "indicator");
        let obj = serde_json::json!({"type": "malware"});
        assert!(!filter.matches(&obj));
    }
}
