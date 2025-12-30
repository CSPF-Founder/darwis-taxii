//! Note SDO (STIX 2.1)
//!
//! A Note is a comment or note containing informative text.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::impl_sdo_traits;
use serde::{Deserialize, Serialize};

/// Note STIX Domain Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(flatten)]
    pub common: CommonProperties,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abstract_: Option<String>,
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_refs: Vec<Identifier>,
}

impl Note {
    pub const TYPE: &'static str = "note";

    pub fn builder() -> NoteBuilder {
        NoteBuilder::new()
    }
}

impl_sdo_traits!(Note, "note");

#[derive(Debug, Default)]
pub struct NoteBuilder {
    abstract_: Option<String>,
    content: Option<String>,
    authors: Vec<String>,
    object_refs: Vec<Identifier>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(NoteBuilder);

impl NoteBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn abstract_(mut self, abstract_: impl Into<String>) -> Self {
        self.abstract_ = Some(abstract_.into());
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.authors.push(author.into());
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

    pub fn build(self) -> Result<Note> {
        let content = self
            .content
            .ok_or_else(|| Error::missing_property("content"))?;

        // Per STIX 2.1 spec, object_refs is required and must not be empty
        if self.object_refs.is_empty() {
            return Err(Error::missing_property("object_refs"));
        }

        Ok(Note {
            type_: Note::TYPE.to_string(),
            id: Identifier::new(Note::TYPE)?,
            common: self.common,
            abstract_: self.abstract_,
            content,
            authors: self.authors,
            object_refs: self.object_refs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_note() {
        let indicator_id = Identifier::new("indicator").unwrap();
        let note = Note::builder()
            .content("This is an important note about the attack.")
            .author("Security Analyst")
            .object_ref(indicator_id)
            .build()
            .unwrap();

        assert_eq!(note.type_, "note");
        assert!(!note.object_refs.is_empty());
    }

    #[test]
    fn test_note_requires_object_refs() {
        let result = Note::builder().content("Note without object refs").build();

        assert!(result.is_err());
    }
}
