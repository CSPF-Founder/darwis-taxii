//! STIX Relationship Objects (SROs)
//!
//! This module contains the two STIX Relationship Objects:
//! - Relationship: Links two STIX objects with a relationship type
//! - Sighting: Records the sighting of an indicator or other object

mod core;
mod sighting;

pub use core::{Relationship, RelationshipBuilder};
pub use sighting::{Sighting, SightingBuilder};

/// Standard relationship types defined in STIX 2.1
pub mod relationship_types {
    /// Attack Pattern delivers Malware
    pub const DELIVERS: &str = "delivers";
    /// Attack Pattern targets Identity/Location/Vulnerability
    pub const TARGETS: &str = "targets";
    /// Attack Pattern uses Tool/Malware
    pub const USES: &str = "uses";
    /// Campaign attributed-to Threat Actor/Intrusion Set
    pub const ATTRIBUTED_TO: &str = "attributed-to";
    /// Campaign compromises Infrastructure
    pub const COMPROMISES: &str = "compromises";
    /// Campaign originates-from Location
    pub const ORIGINATES_FROM: &str = "originates-from";
    /// Course of Action investigates/mitigates/remediates
    pub const INVESTIGATES: &str = "investigates";
    pub const MITIGATES: &str = "mitigates";
    pub const REMEDIATES: &str = "remediates";
    /// Identity located-at Location
    pub const LOCATED_AT: &str = "located-at";
    /// Indicator indicates Malware/Attack Pattern/Campaign/etc.
    pub const INDICATES: &str = "indicates";
    /// Indicator based-on Observable
    pub const BASED_ON: &str = "based-on";
    /// Infrastructure communicates-with/consists-of/controls/hosts/uses
    pub const COMMUNICATES_WITH: &str = "communicates-with";
    pub const CONSISTS_OF: &str = "consists-of";
    pub const CONTROLS: &str = "controls";
    pub const HOSTS: &str = "hosts";
    /// Intrusion Set authored-by Threat Actor
    pub const AUTHORED_BY: &str = "authored-by";
    /// Malware authored-by/beacons-to/communicates-with/drops/downloads/etc.
    pub const BEACONS_TO: &str = "beacons-to";
    pub const DOWNLOADS: &str = "downloads";
    pub const DROPS: &str = "drops";
    pub const EXFILTRATES_TO: &str = "exfiltrates-to";
    /// Malware variant-of another Malware
    pub const VARIANT_OF: &str = "variant-of";
    /// Objects derived-from other objects
    pub const DERIVED_FROM: &str = "derived-from";
    /// Objects duplicate-of other objects
    pub const DUPLICATE_OF: &str = "duplicate-of";
    /// Objects related-to other objects (generic)
    pub const RELATED_TO: &str = "related-to";
    /// Threat Actor impersonates Identity
    pub const IMPERSONATES: &str = "impersonates";
    /// Vulnerability has CVE reference
    pub const HAS: &str = "has";
}
