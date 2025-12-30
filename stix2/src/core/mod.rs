//! Core types, traits, and fundamental STIX structures.
//!
//! This module contains the foundational types used throughout the library:
//!
//! - [`error`]: Error types and Result alias
//! - [`id`]: STIX identifier handling
//! - [`timestamp`]: STIX timestamp with precision
//! - [`traits`]: Core traits for STIX objects
//! - [`common`]: Common properties shared by all STIX objects
//! - [`bundle`]: STIX Bundle container
//! - [`stix_object`]: Unified STIX object enum

pub mod bundle;
pub mod common;
pub mod error;
pub mod external_reference;
pub mod id;
pub mod kill_chain_phase;
pub mod stix_object;
pub mod timestamp;
pub mod traits;

pub use bundle::Bundle;
pub use common::*;
pub use error::{Error, Result};
pub use external_reference::ExternalReference;
pub use id::Identifier;
pub use kill_chain_phase::KillChainPhase;
pub use stix_object::StixObject;
pub use timestamp::Timestamp;
pub use traits::*;
