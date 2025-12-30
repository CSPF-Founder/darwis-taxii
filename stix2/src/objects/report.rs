//! Report SDO
//!
//! Reports are collections of threat intelligence focused on one or more topics.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::core::timestamp::Timestamp;
use crate::impl_sdo_traits;
use crate::vocab::ReportType;
use serde::{Deserialize, Serialize};

/// Report STIX Domain Object.
///
/// Reports are collections of threat intelligence focused on one or more
/// topics, such as a description of a threat actor, malware, or attack
/// technique, including context and related details.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    /// The type property identifies the type of STIX Object.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property uniquely identifies this object.
    pub id: Identifier,

    /// Common properties shared by all SDOs.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// The name of the Report.
    pub name: String,

    /// A description that provides more details about the Report.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The types of content in this report.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub report_types: Vec<ReportType>,

    /// The date the report was published.
    pub published: Timestamp,

    /// STIX Objects that are included in the report.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub object_refs: Vec<Identifier>,
}

impl Report {
    /// The STIX type identifier.
    pub const TYPE: &'static str = "report";

    /// Create a new builder.
    pub fn builder() -> ReportBuilder {
        ReportBuilder::new()
    }
}

impl_sdo_traits!(Report, "report");

/// Builder for creating Report objects.
#[derive(Debug, Default)]
pub struct ReportBuilder {
    name: Option<String>,
    description: Option<String>,
    report_types: Vec<ReportType>,
    published: Option<Timestamp>,
    object_refs: Vec<Identifier>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(ReportBuilder);

impl ReportBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the name (required).
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a report type.
    pub fn report_type(mut self, report_type: ReportType) -> Self {
        self.report_types.push(report_type);
        self
    }

    /// Set the published timestamp (required).
    pub fn published(mut self, published: Timestamp) -> Self {
        self.published = Some(published);
        self
    }

    /// Set published to now.
    pub fn published_now(mut self) -> Self {
        self.published = Some(Timestamp::now());
        self
    }

    /// Add an object reference.
    pub fn object_ref(mut self, object_ref: Identifier) -> Self {
        self.object_refs.push(object_ref);
        self
    }

    /// Add multiple object references.
    pub fn object_refs(mut self, refs: Vec<Identifier>) -> Self {
        self.object_refs.extend(refs);
        self
    }

    /// Set the created_by_ref.
    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    /// Build the Report.
    pub fn build(self) -> Result<Report> {
        let name = self.name.ok_or_else(|| Error::missing_property("name"))?;
        let published = self
            .published
            .ok_or_else(|| Error::missing_property("published"))?;

        // Per STIX 2.1 spec, object_refs is required and must not be empty
        if self.object_refs.is_empty() {
            return Err(Error::missing_property("object_refs"));
        }

        Ok(Report {
            type_: Report::TYPE.to_string(),
            id: Identifier::new(Report::TYPE)?,
            common: self.common,
            name,
            description: self.description,
            report_types: self.report_types,
            published,
            object_refs: self.object_refs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_report() {
        let indicator_id = Identifier::new("indicator").unwrap();
        let report = Report::builder()
            .name("APT28 Campaign Analysis")
            .report_type(ReportType::ThreatReport)
            .published_now()
            .object_ref(indicator_id)
            .build()
            .unwrap();

        assert_eq!(report.name, "APT28 Campaign Analysis");
        assert_eq!(report.type_, "report");
        assert!(!report.object_refs.is_empty());
    }

    #[test]
    fn test_report_requires_object_refs() {
        let result = Report::builder()
            .name("Empty Report")
            .published_now()
            .build();

        assert!(result.is_err());
    }
}
