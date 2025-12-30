//! Observed Data SDO
//!
//! Observed Data conveys information about cyber security related entities.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sdo_traits;
use crate::validation::{Constrained, check_timestamp_order};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Maximum value for number_observed per STIX 2.1 spec.
const NUMBER_OBSERVED_MAX: u64 = 999_999_999;

/// Observed Data STIX Domain Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedData {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(flatten)]
    pub common: CommonProperties,
    pub first_observed: Timestamp,
    pub last_observed: Timestamp,
    pub number_observed: u64,
    /// DEPRECATED: The objects property is deprecated in STIX 2.1.
    /// Use object_refs instead. This is kept for backwards compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objects: Option<IndexMap<String, Value>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_refs: Vec<Identifier>,
}

impl ObservedData {
    pub const TYPE: &'static str = "observed-data";

    pub fn builder() -> ObservedDataBuilder {
        ObservedDataBuilder::new()
    }
}

impl_sdo_traits!(ObservedData, "observed-data");

impl Constrained for ObservedData {
    /// Validate ObservedData constraints.
    ///
    /// - `last_observed` must be >= `first_observed`
    /// - `number_observed` must be between 1 and 999999999
    /// - `objects` and `object_refs` are mutually exclusive
    fn validate_constraints(&self) -> Result<()> {
        check_timestamp_order(
            Some(&self.first_observed),
            Some(&self.last_observed),
            "first_observed",
            "last_observed",
        )?;

        // Validate number_observed range (1-999999999)
        if self.number_observed < 1 || self.number_observed > NUMBER_OBSERVED_MAX {
            return Err(Error::InvalidPropertyValue {
                property: "number_observed".to_string(),
                message: format!(
                    "number_observed must be between 1 and {}",
                    NUMBER_OBSERVED_MAX
                ),
            });
        }

        // objects and object_refs are mutually exclusive
        if self.objects.is_some() && !self.object_refs.is_empty() {
            return Err(Error::MutuallyExclusiveProperties(vec![
                "objects".to_string(),
                "object_refs".to_string(),
            ]));
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ObservedDataBuilder {
    first_observed: Option<Timestamp>,
    last_observed: Option<Timestamp>,
    number_observed: Option<u64>,
    objects: Option<IndexMap<String, Value>>,
    object_refs: Vec<Identifier>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(ObservedDataBuilder);

impl ObservedDataBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn first_observed(mut self, first_observed: Timestamp) -> Self {
        self.first_observed = Some(first_observed);
        self
    }

    pub fn last_observed(mut self, last_observed: Timestamp) -> Self {
        self.last_observed = Some(last_observed);
        self
    }

    pub fn number_observed(mut self, count: u64) -> Self {
        self.number_observed = Some(count);
        self
    }

    pub fn object_ref(mut self, object_ref: Identifier) -> Self {
        self.object_refs.push(object_ref);
        self
    }

    /// Set objects (DEPRECATED - use object_refs instead).
    #[deprecated(note = "Use object_refs instead. This is kept for backwards compatibility.")]
    pub fn objects(mut self, objects: IndexMap<String, Value>) -> Self {
        self.objects = Some(objects);
        self
    }

    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    pub fn build(self) -> Result<ObservedData> {
        let first_observed = self
            .first_observed
            .ok_or_else(|| Error::missing_property("first_observed"))?;
        let last_observed = self
            .last_observed
            .ok_or_else(|| Error::missing_property("last_observed"))?;
        let number_observed = self
            .number_observed
            .ok_or_else(|| Error::missing_property("number_observed"))?;

        let observed_data = ObservedData {
            type_: ObservedData::TYPE.to_string(),
            id: Identifier::new(ObservedData::TYPE)?,
            common: self.common,
            first_observed,
            last_observed,
            number_observed,
            objects: self.objects,
            object_refs: self.object_refs,
        };

        // Validate constraints
        observed_data.validate_constraints()?;

        Ok(observed_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_observed_data() {
        let now = Timestamp::now();
        let od = ObservedData::builder()
            .first_observed(now)
            .last_observed(now)
            .number_observed(5)
            .build()
            .unwrap();

        assert_eq!(od.type_, "observed-data");
        assert_eq!(od.number_observed, 5);
    }
}
