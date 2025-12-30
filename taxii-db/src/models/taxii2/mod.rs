//! TAXII 2.x database models.
//!
//! Tables:
//! - opentaxii_api_root
//! - opentaxii_collection
//! - opentaxii_stixobject
//! - opentaxii_job
//! - opentaxii_job_detail

pub mod api_root;
pub mod collection;
pub mod job;
pub mod query;
pub mod stix_object;

pub use api_root::ApiRoot;
pub use collection::Collection;
pub use job::{Job, JobDetail, NewJob, job_detail_status, job_status};
pub use query::{
    PaginatedResult, PaginationCursor, Taxii2QueryParams, get_next_param, parse_next_param,
};
pub use stix_object::{FilteredResult, NewSTIXObject, STIXObject, VersionInfo, VersionsResult};
