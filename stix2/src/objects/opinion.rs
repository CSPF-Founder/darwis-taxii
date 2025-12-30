//! Opinion SDO (STIX 2.1)
//!
//! An Opinion is an assessment of information as correct, incorrect, etc.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::impl_sdo_traits;
use crate::vocab::OpinionValue;
use serde::{Deserialize, Serialize};

/// Opinion STIX Domain Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Opinion {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(flatten)]
    pub common: CommonProperties,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    pub opinion: OpinionValue,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_refs: Vec<Identifier>,
}

impl Opinion {
    pub const TYPE: &'static str = "opinion";

    pub fn builder() -> OpinionBuilder {
        OpinionBuilder::new()
    }
}

impl_sdo_traits!(Opinion, "opinion");

#[derive(Debug, Default)]
pub struct OpinionBuilder {
    explanation: Option<String>,
    authors: Vec<String>,
    opinion: Option<OpinionValue>,
    object_refs: Vec<Identifier>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(OpinionBuilder);

impl OpinionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = Some(explanation.into());
        self
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.authors.push(author.into());
        self
    }

    pub fn opinion(mut self, opinion: OpinionValue) -> Self {
        self.opinion = Some(opinion);
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

    pub fn build(self) -> Result<Opinion> {
        let opinion = self
            .opinion
            .ok_or_else(|| Error::missing_property("opinion"))?;

        // Per STIX 2.1 spec, object_refs is required and must not be empty
        if self.object_refs.is_empty() {
            return Err(Error::missing_property("object_refs"));
        }

        Ok(Opinion {
            type_: Opinion::TYPE.to_string(),
            id: Identifier::new(Opinion::TYPE)?,
            common: self.common,
            explanation: self.explanation,
            authors: self.authors,
            opinion,
            object_refs: self.object_refs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_opinion() {
        let indicator_id = Identifier::new("indicator").unwrap();
        let opinion = Opinion::builder()
            .opinion(OpinionValue::Agree)
            .explanation("This assessment is accurate based on my research.")
            .object_ref(indicator_id)
            .build()
            .unwrap();

        assert_eq!(opinion.type_, "opinion");
        assert!(!opinion.object_refs.is_empty());
    }

    #[test]
    fn test_opinion_requires_object_refs() {
        let result = Opinion::builder().opinion(OpinionValue::Agree).build();

        assert!(result.is_err());
    }
}
