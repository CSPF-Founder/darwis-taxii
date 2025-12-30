//! Grouping SDO (STIX 2.1)
//!
//! A Grouping object explicitly asserts that the referenced STIX Objects have
//! a shared context.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::impl_sdo_traits;
use crate::vocab::GroupingContext;
use serde::{Deserialize, Serialize};

/// Grouping STIX Domain Object.
///
/// A Grouping object explicitly asserts that the referenced STIX Objects
/// have a shared context, unlike a STIX Bundle which does not convey any
/// meaning about the contents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Grouping {
    /// The type property.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property.
    pub id: Identifier,

    /// Common properties.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// A short descriptor for this Grouping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// A description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The context of this grouping.
    pub context: GroupingContext,

    /// The STIX Objects that are included in this grouping.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_refs: Vec<Identifier>,
}

impl Grouping {
    pub const TYPE: &'static str = "grouping";

    pub fn builder() -> GroupingBuilder {
        GroupingBuilder::new()
    }
}

impl_sdo_traits!(Grouping, "grouping");

#[derive(Debug, Default)]
pub struct GroupingBuilder {
    name: Option<String>,
    description: Option<String>,
    context: Option<GroupingContext>,
    object_refs: Vec<Identifier>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(GroupingBuilder);

impl GroupingBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn context(mut self, context: GroupingContext) -> Self {
        self.context = Some(context);
        self
    }

    pub fn object_ref(mut self, object_ref: Identifier) -> Self {
        self.object_refs.push(object_ref);
        self
    }

    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    pub fn build(self) -> Result<Grouping> {
        let context = self
            .context
            .ok_or_else(|| Error::missing_property("context"))?;

        // Per STIX 2.1 spec, object_refs is required and must not be empty
        if self.object_refs.is_empty() {
            return Err(Error::missing_property("object_refs"));
        }

        Ok(Grouping {
            type_: Grouping::TYPE.to_string(),
            id: Identifier::new(Grouping::TYPE)?,
            common: self.common,
            name: self.name,
            description: self.description,
            context,
            object_refs: self.object_refs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_grouping() {
        let indicator_id = Identifier::new("indicator").unwrap();
        let grouping = Grouping::builder()
            .name("Suspicious Activity Group")
            .context(GroupingContext::SuspiciousActivity)
            .object_ref(indicator_id)
            .build()
            .unwrap();

        assert_eq!(grouping.type_, "grouping");
        assert!(!grouping.object_refs.is_empty());
    }

    #[test]
    fn test_grouping_requires_object_refs() {
        let result = Grouping::builder()
            .name("Empty Group")
            .context(GroupingContext::SuspiciousActivity)
            .build();

        assert!(result.is_err());
    }
}
