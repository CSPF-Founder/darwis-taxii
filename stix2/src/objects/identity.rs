//! Identity SDO
//!
//! Identities represent actual individuals, organizations, or groups,
//! as well as classes of individuals, organizations, systems, or groups.

use crate::core::common::CommonProperties;
use crate::core::error::{Error, Result};
use crate::core::id::Identifier;
use crate::impl_sdo_traits;
use crate::vocab::{IdentityClass, IndustrySector};
use serde::{Deserialize, Serialize};

/// Identity STIX Domain Object.
///
/// Identities can represent actual individuals, organizations, or groups
/// (e.g., ACME, Inc.) as well as classes of individuals, organizations,
/// systems or groups (e.g., the finance sector).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Identity {
    /// The type property identifies the type of STIX Object.
    #[serde(rename = "type")]
    pub type_: String,

    /// The id property uniquely identifies this object.
    pub id: Identifier,

    /// Common properties shared by all SDOs.
    #[serde(flatten)]
    pub common: CommonProperties,

    /// The name of this Identity.
    pub name: String,

    /// A description that provides more details about the Identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The list of roles that this Identity performs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,

    /// The type of entity that this Identity describes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_class: Option<IdentityClass>,

    /// The list of industry sectors that this Identity belongs to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sectors: Vec<IndustrySector>,

    /// The contact information for this Identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_information: Option<String>,
}

impl Identity {
    /// The STIX type identifier for Identity.
    pub const TYPE: &'static str = "identity";

    /// Create a new IdentityBuilder.
    pub fn builder() -> IdentityBuilder {
        IdentityBuilder::new()
    }

    /// Create a new Identity with the given name.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        Self::builder().name(name).build()
    }

    /// Create an individual identity.
    pub fn individual(name: impl Into<String>) -> Result<Self> {
        Self::builder()
            .name(name)
            .identity_class(IdentityClass::Individual)
            .build()
    }

    /// Create an organization identity.
    pub fn organization(name: impl Into<String>) -> Result<Self> {
        Self::builder()
            .name(name)
            .identity_class(IdentityClass::Organization)
            .build()
    }

    /// Create a group identity.
    pub fn group(name: impl Into<String>) -> Result<Self> {
        Self::builder()
            .name(name)
            .identity_class(IdentityClass::Group)
            .build()
    }
}

impl_sdo_traits!(Identity, "identity");

/// Builder for creating Identity objects.
#[derive(Debug, Default)]
pub struct IdentityBuilder {
    name: Option<String>,
    description: Option<String>,
    roles: Vec<String>,
    identity_class: Option<IdentityClass>,
    sectors: Vec<IndustrySector>,
    contact_information: Option<String>,
    common: CommonProperties,
}

// Implement common builder methods
crate::impl_common_builder_methods!(IdentityBuilder);

impl IdentityBuilder {
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

    /// Add a role.
    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }

    /// Set the identity class.
    pub fn identity_class(mut self, class: IdentityClass) -> Self {
        self.identity_class = Some(class);
        self
    }

    /// Add an industry sector.
    pub fn sector(mut self, sector: IndustrySector) -> Self {
        self.sectors.push(sector);
        self
    }

    /// Set the contact information.
    pub fn contact_information(mut self, contact: impl Into<String>) -> Self {
        self.contact_information = Some(contact.into());
        self
    }

    /// Set the created_by_ref.
    pub fn created_by_ref(mut self, identity_ref: Identifier) -> Self {
        self.common.created_by_ref = Some(identity_ref);
        self
    }

    /// Add a label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.common.labels.push(label.into());
        self
    }

    /// Build the Identity.
    pub fn build(self) -> Result<Identity> {
        let name = self.name.ok_or_else(|| Error::missing_property("name"))?;

        Ok(Identity {
            type_: Identity::TYPE.to_string(),
            id: Identifier::new(Identity::TYPE)?,
            common: self.common,
            name,
            description: self.description,
            roles: self.roles,
            identity_class: self.identity_class,
            sectors: self.sectors,
            contact_information: self.contact_information,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_identity() {
        let identity = Identity::builder()
            .name("ACME Corporation")
            .identity_class(IdentityClass::Organization)
            .sector(IndustrySector::Technology)
            .build()
            .unwrap();

        assert_eq!(identity.name, "ACME Corporation");
        assert_eq!(identity.type_, "identity");
    }

    #[test]
    fn test_individual_identity() {
        let identity = Identity::individual("John Doe").unwrap();
        assert_eq!(identity.identity_class, Some(IdentityClass::Individual));
    }

    #[test]
    fn test_organization_identity() {
        let identity = Identity::organization("ACME Inc").unwrap();
        assert_eq!(identity.identity_class, Some(IdentityClass::Organization));
    }

    #[test]
    fn test_serialization() {
        let identity = Identity::builder().name("Test Identity").build().unwrap();

        let json = serde_json::to_string(&identity).unwrap();
        let parsed: Identity = serde_json::from_str(&json).unwrap();
        assert_eq!(identity.name, parsed.name);
    }
}
