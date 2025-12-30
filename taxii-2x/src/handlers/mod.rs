//! TAXII 2.x route handlers.
//!
//! This module implements the TAXII 2.1 RESTful API endpoints for exchanging
//! cyber threat intelligence (CTI) over HTTPS.
//!
//! # TAXII 2.x Overview
//!
//! TAXII 2.x is a significant redesign from 1.x, using a RESTful JSON API
//! instead of XML over HTTP. Key concepts:
//!
//! - **API Roots**: Logical groupings of collections, each a separate TAXII API instance
//! - **Collections**: Repositories of CTI objects that clients can read from or write to
//! - **Objects**: STIX 2.1 cyber threat intelligence objects (indicators, malware, etc.)
//!
//! # Endpoints
//!
//! | Endpoint | Description |
//! |----------|-------------|
//! | `GET /taxii2/` | Server discovery - lists available API roots |
//! | `GET /taxii2/{api_root}/` | API root information |
//! | `GET /taxii2/{api_root}/collections/` | List collections in an API root |
//! | `GET /taxii2/{api_root}/collections/{id}/` | Get collection details |
//! | `GET /taxii2/{api_root}/collections/{id}/objects/` | Get objects from collection |
//! | `POST /taxii2/{api_root}/collections/{id}/objects/` | Add objects to collection |
//! | `GET /taxii2/{api_root}/collections/{id}/manifest/` | List object metadata |
//! | `DELETE /taxii2/{api_root}/collections/{id}/objects/{id}/` | Delete an object |
//!
//! # Content Types
//!
//! TAXII 2.1 uses `application/taxii+json;version=2.1` for TAXII responses
//! and `application/stix+json;version=2.1` for STIX content.

mod collections;
mod discovery;
mod objects;

pub use collections::{
    collection_handler, collections_handler, manifest_handler, versions_handler,
};
pub use discovery::{api_root_handler, discovery_handler, job_handler};
pub use objects::{
    object_delete_handler, object_get_handler, objects_get_handler, objects_post_handler,
};
