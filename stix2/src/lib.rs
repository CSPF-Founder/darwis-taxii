//! # STIX 2.1 Rust Library
//!
//! A comprehensive Rust implementation of the STIX 2.1 (Structured Threat Information Expression)
//! specification for representing and exchanging cyber threat intelligence.
//!
//! ## Overview
//!
//! STIX (Structured Threat Information Expression) is a language and serialization format
//! used to exchange cyber threat intelligence (CTI). This library provides:
//!
//! - All STIX Domain Objects (SDOs): Attack Pattern, Campaign, Course of Action, etc.
//! - STIX Relationship Objects (SROs): Relationship and Sighting
//! - STIX Cyber Observable Objects (SCOs): File, IP Address, URL, etc.
//! - Data Markings: TLP, Statement markings
//! - Pattern Language: Parser for STIX indicator patterns
//! - DataStore Abstractions: Memory, FileSystem stores
//! - Validation: Complete property validation
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use stix2::prelude::*;
//!
//! fn main() -> stix2::Result<()> {
//!     // Create an indicator
//!     let indicator = Indicator::builder()
//!         .name("Malicious File Hash")
//!         .pattern("[file:hashes.'SHA-256' = 'abc123']")
//!         .pattern_type(PatternType::Stix)
//!         .valid_from_now()
//!         .build()?;
//!
//!     // Serialize to JSON
//!     let json = stix2::serialize_pretty(&indicator)?;
//!
//!     // Parse from JSON
//!     let parsed: StixObject = stix2::parse(&json)?;
//!     Ok(())
//! }
//! ```
//!
//! ## Modules
//!
//! - [`core`]: Core types, traits, and error handling
//! - [`objects`]: STIX Domain Objects (SDOs)
//! - [`relationship`]: STIX Relationship Objects (SROs)
//! - [`observables`]: STIX Cyber Observable Objects (SCOs)
//! - [`extensions`]: Observable and object extensions
//! - [`markings`]: Data marking definitions
//! - [`patterns`]: STIX pattern language parser
//! - [`datastore`]: DataStore abstractions
//! - [`vocab`]: STIX vocabularies
//! - [`utils`]: Utility functions
//! - [`versioning`]: Object versioning utilities
//! - [`equivalence`]: Semantic equivalence checking

// Struct fields are defined by the STIX 2.1 specification and are self-documenting.
// Struct-level and module-level documentation is provided.
#![warn(clippy::all)]
#![deny(unsafe_code)]
// Allow unwrap/expect/panic in tests only
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
// Allow excessive nesting in complex algorithms (graph traversal, file operations)
#![allow(clippy::excessive_nesting)]

pub mod canonicalization;
pub mod core;
pub mod custom;
pub mod datastore;
pub mod environment;
pub mod equivalence;
pub mod extensions;
pub mod graph;
pub mod markings;
pub mod objects;
pub mod observables;
pub mod pattern_equivalence;
pub mod patterns;
pub mod registry;
pub mod relationship;
pub mod utils;
pub mod v20;
pub mod validation;
pub mod versioning;
pub mod vocab;
pub mod workbench;

// Re-export commonly used types
pub use crate::core::bundle::Bundle;
pub use crate::core::error::{Error, Result};
pub use crate::core::id::Identifier;
pub use crate::core::stix_object::StixObject;
pub use crate::core::timestamp::Timestamp;

// Re-export all SDOs
pub use crate::objects::{
    AttackPattern, Campaign, CourseOfAction, Grouping, Identity, Incident, Indicator,
    Infrastructure, IntrusionSet, Location, Malware, MalwareAnalysis, Note, ObservedData, Opinion,
    Report, ThreatActor, Tool, Vulnerability,
};

// Re-export SROs
pub use crate::relationship::{Relationship, Sighting};

// Re-export SCOs
pub use crate::observables::{
    Artifact, AutonomousSystem, Directory, DomainName, EmailAddress, EmailMessage, File,
    IPv4Address, IPv6Address, MacAddress, Mutex, NetworkTraffic, Process, Software, Url,
    UserAccount, WindowsRegistryKey, X509Certificate,
};

// Re-export markings
pub use crate::markings::{
    GranularMarking, MarkingDefinition, StatementMarking, TlpLevel, TlpMarking,
};

// Re-export patterns
pub use crate::patterns::{Pattern, PatternExpression};

// Re-export datastore
pub use crate::datastore::{
    CompositeDataSource, DataSink, DataSource, DataStore, FileSystemStore, MemoryStore,
};

// Re-export versioning
pub use crate::versioning::{
    UNMODIFIABLE_PROPERTIES, VersionBuilder, is_versionable, new_version, new_version_with_changes,
    remove_custom_properties, revoke,
};

// Re-export equivalence
pub use crate::equivalence::{object_equivalence, object_similarity};

// Re-export graph
pub use crate::graph::{StixGraph, graph_equivalence, graph_similarity, graphs_equivalent};

// Re-export canonicalization
pub use crate::canonicalization::{canonical_hash, canonicalize};

// Re-export v20 compatibility
pub use crate::v20::{StixVersion, detect_version, parse_any_version};

// Re-export pattern equivalence
pub use crate::pattern_equivalence::{
    equivalent_patterns, find_equivalent_patterns, pattern_similarity,
};

// Re-export registry
pub use crate::registry::{
    CustomTypeOptions, ObjectCategory, SpecVersion, class_for_type, get_sco_types, get_sdo_types,
    get_sro_types, is_registered_type, register_custom_type,
};

// Re-export custom object functions
pub use crate::custom::{
    register_custom_extension, register_custom_marking, register_custom_sco, register_custom_sdo,
    validate_custom_type_name, validate_extension_type_name,
};

// Re-export environment
pub use crate::environment::{Environment, ObjectFactory};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::core::bundle::Bundle;
    pub use crate::core::common::*;
    pub use crate::core::error::{Error, Result};
    pub use crate::core::id::Identifier;
    pub use crate::core::stix_object::StixObject;
    pub use crate::core::timestamp::Timestamp;
    pub use crate::core::traits::*;

    pub use crate::markings::{
        GranularMarking, MarkingDefinition, StatementMarking, TlpLevel, TlpMarking,
    };

    pub use crate::objects::{
        AttackPattern, Campaign, CourseOfAction, Grouping, Identity, Incident, Indicator,
        Infrastructure, IntrusionSet, Location, Malware, MalwareAnalysis, Note, ObservedData,
        Opinion, Report, ThreatActor, Tool, Vulnerability,
    };

    pub use crate::observables::{
        Artifact, AutonomousSystem, Directory, DomainName, EmailAddress, EmailMessage, File,
        IPv4Address, IPv6Address, MacAddress, Mutex, NetworkTraffic, Process, Software, Url,
        UserAccount, WindowsRegistryKey, X509Certificate,
    };

    pub use crate::relationship::{Relationship, Sighting};

    pub use crate::patterns::{Pattern, PatternExpression};

    pub use crate::vocab::*;

    pub use crate::datastore::{
        CompositeDataSource, DataSink, DataSource, DataStore, FileSystemStore, Filter, MemoryStore,
    };

    pub use crate::equivalence::{object_equivalence, object_similarity};
    pub use crate::versioning::{
        VersionBuilder, is_versionable, new_version, new_version_with_changes, revoke,
    };

    // Graph analysis
    pub use crate::graph::{StixGraph, graph_equivalence, graph_similarity};

    // Canonicalization
    pub use crate::canonicalization::{canonical_hash, canonicalize};

    // Version compatibility
    pub use crate::v20::{StixVersion, detect_version, parse_any_version};

    // Pattern equivalence
    pub use crate::pattern_equivalence::{equivalent_patterns, pattern_similarity};

    // Registry
    pub use crate::registry::{SpecVersion, is_registered_type};

    // Environment
    pub use crate::environment::{Environment, ObjectFactory};

    pub use chrono::{DateTime, Utc};
    pub use uuid::Uuid;

    pub use crate::{parse, parse_bundle};
}

/// Parse a STIX JSON string into a StixObject
///
/// # Arguments
///
/// * `json` - A JSON string representing a STIX object
///
/// # Returns
///
/// A `Result` containing the parsed `StixObject` or an error
///
/// # Example
///
/// ```rust,ignore
/// use stix2::parse;
///
/// let json = r#"{"type": "indicator", "id": "indicator--..."}"#;
/// let obj = parse(json)?;
/// ```
pub fn parse(json: &str) -> Result<StixObject> {
    serde_json::from_str(json).map_err(Error::from)
}

/// Parse a STIX Bundle JSON string
///
/// # Arguments
///
/// * `json` - A JSON string representing a STIX Bundle
///
/// # Returns
///
/// A `Result` containing the parsed `Bundle` or an error
pub fn parse_bundle(json: &str) -> Result<Bundle> {
    serde_json::from_str(json).map_err(Error::from)
}

/// Serialize a STIX object to JSON string
///
/// # Arguments
///
/// * `obj` - Any type that implements Serialize
///
/// # Returns
///
/// A `Result` containing the JSON string or an error
pub fn serialize<T: serde::Serialize>(obj: &T) -> Result<String> {
    serde_json::to_string(obj).map_err(Error::from)
}

/// Serialize a STIX object to pretty-printed JSON string
pub fn serialize_pretty<T: serde::Serialize>(obj: &T) -> Result<String> {
    serde_json::to_string_pretty(obj).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports() {
        // Verify key types are exported
        let _: fn() -> Bundle = Bundle::new;
        let _: fn(&str) -> Result<Identifier> = Identifier::new;
    }
}
