//! File SCO
//!
//! The File Object represents the properties of a file.

use crate::core::common::Hashes;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sco_traits;
use crate::markings::GranularMarking;
use crate::validation::Constrained;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// File STIX Cyber Observable Object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct File {
    #[serde(rename = "type")]
    pub type_: String,
    pub id: Identifier,
    #[serde(default = "default_spec_version")]
    pub spec_version: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub defanged: bool,
    #[serde(default, skip_serializing_if = "Hashes::is_empty")]
    pub hashes: Hashes,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_enc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magic_number_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ctime: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtime: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atime: Option<Timestamp>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_directory_ref: Option<Identifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contains_refs: Vec<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_ref: Option<Identifier>,
    /// References to marking definitions that apply to this object.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_marking_refs: Vec<Identifier>,
    /// Granular markings for specific properties.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub granular_markings: Vec<GranularMarking>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub extensions: IndexMap<String, Value>,
}

fn default_spec_version() -> String {
    "2.1".to_string()
}

impl File {
    pub const TYPE: &'static str = "file";

    pub fn builder() -> FileBuilder {
        FileBuilder::new()
    }

    pub fn new() -> Result<Self> {
        Self::builder().build()
    }
}

impl_sco_traits!(File, "file");

impl crate::observables::IdContributing for File {
    const ID_CONTRIBUTING_PROPERTIES: &'static [&'static str] =
        &["hashes", "name", "parent_directory_ref", "extensions"];
}

impl Constrained for File {
    /// Validate File constraints.
    ///
    /// - At least one of `hashes` or `name` must be present
    fn validate_constraints(&self) -> Result<()> {
        use crate::validation::check_optional_ref_type;

        // At least one of hashes or name must be present
        if self.hashes.is_empty() && self.name.is_none() {
            return Err(Error::AtLeastOneRequired(vec![
                "hashes".to_string(),
                "name".to_string(),
            ]));
        }

        // Validate reference types
        check_optional_ref_type(
            self.parent_directory_ref.as_ref(),
            "parent_directory_ref",
            &["directory"],
        )?;
        check_optional_ref_type(self.content_ref.as_ref(), "content_ref", &["artifact"])?;
        // contains_refs can be any SCO type per the spec (embedded files)

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct FileBuilder {
    hashes: Hashes,
    size: Option<u64>,
    name: Option<String>,
    mime_type: Option<String>,
    ctime: Option<Timestamp>,
    mtime: Option<Timestamp>,
    atime: Option<Timestamp>,
    parent_directory_ref: Option<Identifier>,
    contains_refs: Vec<Identifier>,
    content_ref: Option<Identifier>,
    defanged: bool,
}

impl FileBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hash(mut self, algorithm: impl Into<String>, value: impl Into<String>) -> Self {
        self.hashes.insert(algorithm.into(), value.into());
        self
    }

    pub fn md5(self, value: impl Into<String>) -> Self {
        self.hash("MD5", value)
    }

    pub fn sha1(self, value: impl Into<String>) -> Self {
        self.hash("SHA-1", value)
    }

    pub fn sha256(self, value: impl Into<String>) -> Self {
        self.hash("SHA-256", value)
    }

    pub fn sha512(self, value: impl Into<String>) -> Self {
        self.hash("SHA-512", value)
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    pub fn ctime(mut self, ctime: Timestamp) -> Self {
        self.ctime = Some(ctime);
        self
    }

    pub fn mtime(mut self, mtime: Timestamp) -> Self {
        self.mtime = Some(mtime);
        self
    }

    pub fn atime(mut self, atime: Timestamp) -> Self {
        self.atime = Some(atime);
        self
    }

    pub fn parent_directory_ref(mut self, ref_: Identifier) -> Self {
        self.parent_directory_ref = Some(ref_);
        self
    }

    pub fn contains_ref(mut self, ref_: Identifier) -> Self {
        self.contains_refs.push(ref_);
        self
    }

    pub fn content_ref(mut self, ref_: Identifier) -> Self {
        self.content_ref = Some(ref_);
        self
    }

    pub fn defanged(mut self, defanged: bool) -> Self {
        self.defanged = defanged;
        self
    }

    pub fn build(self) -> Result<File> {
        Ok(File {
            type_: File::TYPE.to_string(),
            id: Identifier::new(File::TYPE)?,
            spec_version: default_spec_version(),
            defanged: self.defanged,
            hashes: self.hashes,
            size: self.size,
            name: self.name,
            name_enc: None,
            magic_number_hex: None,
            mime_type: self.mime_type,
            ctime: self.ctime,
            mtime: self.mtime,
            atime: self.atime,
            parent_directory_ref: self.parent_directory_ref,
            contains_refs: self.contains_refs,
            content_ref: self.content_ref,
            object_marking_refs: Vec::new(),
            granular_markings: Vec::new(),
            extensions: IndexMap::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_file() {
        let file = File::builder()
            .name("malware.exe")
            .sha256("abc123def456")
            .size(1024)
            .build()
            .unwrap();

        assert_eq!(file.type_, "file");
        assert_eq!(file.name, Some("malware.exe".to_string()));
    }

    #[test]
    fn test_serialization() {
        let file = File::builder().name("test.txt").build().unwrap();

        let json = serde_json::to_string(&file).unwrap();
        let parsed: File = serde_json::from_str(&json).unwrap();
        assert_eq!(file.name, parsed.name);
    }
}
