//! Language Content SDO (STIX 2.1)
//!
//! Language Content represents text content in multiple languages.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::impl_sdo_traits;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Language Content STIX Domain Object.
///
/// The Language Content object represents text content for STIX Objects
/// represented in languages other than that of the original object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageContent {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(flatten)]
    pub common: CommonProperties,
    /// The id of the object that this language content is associated with.
    pub object_ref: Identifier,
    /// The object_modified property identifies the modified time of the object
    /// that this Language Content applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_modified: Option<crate::core::timestamp::Timestamp>,
    /// The contents property contains the actual Language Content.
    /// Keys are RFC 5646 language codes, values are dicts of property to translation.
    pub contents: IndexMap<String, IndexMap<String, Value>>,
}

impl LanguageContent {
    pub const TYPE: &'static str = "language-content";

    pub fn builder() -> LanguageContentBuilder {
        LanguageContentBuilder::new()
    }
}

impl_sdo_traits!(LanguageContent, "language-content");

impl crate::validation::Constrained for LanguageContent {
    /// Validate LanguageContent constraints.
    ///
    /// - object_ref must reference an SDO, SCO, or SRO
    fn validate_constraints(&self) -> crate::core::error::Result<()> {
        // STIX objects that cannot be referenced in LanguageContent
        const INVALID_REF_TYPES: &[&str] = &["bundle", "language-content", "marking-definition"];

        let obj_type = self.object_ref.object_type();
        if INVALID_REF_TYPES.contains(&obj_type) {
            return Err(crate::core::error::Error::InvalidPropertyValue {
                property: "object_ref".to_string(),
                message: format!(
                    "object_ref must reference an SDO, SCO, or SRO; '{}' is not valid",
                    obj_type
                ),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct LanguageContentBuilder {
    object_ref: Option<Identifier>,
    object_modified: Option<crate::core::timestamp::Timestamp>,
    contents: IndexMap<String, IndexMap<String, Value>>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(LanguageContentBuilder);

impl LanguageContentBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn object_ref(mut self, object_ref: Identifier) -> Self {
        self.object_ref = Some(object_ref);
        self
    }

    pub fn object_modified(mut self, modified: crate::core::timestamp::Timestamp) -> Self {
        self.object_modified = Some(modified);
        self
    }

    /// Add a translation for a specific language and property.
    pub fn translation(
        mut self,
        lang_code: impl Into<String>,
        property: impl Into<String>,
        value: impl Into<Value>,
    ) -> Self {
        let lang = lang_code.into();
        let prop = property.into();
        self.contents
            .entry(lang)
            .or_default()
            .insert(prop, value.into());
        self
    }

    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    pub fn build(self) -> Result<LanguageContent> {
        let object_ref = self
            .object_ref
            .ok_or_else(|| Error::missing_property("object_ref"))?;

        if self.contents.is_empty() {
            return Err(Error::missing_property("contents"));
        }

        Ok(LanguageContent {
            type_: LanguageContent::TYPE.to_string(),
            id: Identifier::new(LanguageContent::TYPE)?,
            common: self.common,
            object_ref,
            object_modified: self.object_modified,
            contents: self.contents,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_language_content() {
        let object_ref: Identifier = "indicator--8e2e2d2b-17d4-4cbf-938f-98ee46b3cd3f"
            .parse()
            .unwrap();

        let lc = LanguageContent::builder()
            .object_ref(object_ref)
            .translation("de", "name", "Böser Indikator")
            .translation("de", "description", "Ein schädlicher Indikator")
            .translation("es", "name", "Indicador malo")
            .build()
            .unwrap();

        assert_eq!(lc.type_, "language-content");
        assert_eq!(lc.contents.len(), 2);
    }
}
