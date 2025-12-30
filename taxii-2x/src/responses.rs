//! TAXII 2.x response types.
//!
//! This module contains all response structs used by TAXII 2.1 handlers.
//! These types are serialized to JSON and returned to clients.

use serde::Serialize;
use serde_json::Value;

/// Discovery response.
///
/// Returned by `GET /taxii2/`
#[derive(Debug, Serialize)]
pub struct DiscoveryResponse {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    pub api_roots: Vec<String>,
}

/// API Root response.
///
/// Returned by `GET /taxii2/{api_root_id}/`
#[derive(Debug, Serialize)]
pub struct ApiRootResponse {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub versions: Vec<String>,
    pub max_content_length: usize,
}

/// Collections response.
///
/// Returned by `GET /taxii2/{api_root_id}/collections/`
#[derive(Debug, Serialize)]
pub struct CollectionsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collections: Option<Vec<CollectionInfo>>,
}

/// Collection information.
///
/// Used in both collections list and single collection responses.
#[derive(Debug, Serialize)]
pub struct CollectionInfo {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    pub can_read: bool,
    pub can_write: bool,
    pub media_types: Vec<String>,
}

/// Manifest response.
///
/// Returned by `GET /taxii2/{api_root_id}/collections/{collection_id}/manifest/`
#[derive(Debug, Serialize)]
pub struct ManifestResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objects: Option<Vec<ManifestEntry>>,
}

/// Single manifest entry.
#[derive(Debug, Serialize)]
pub struct ManifestEntry {
    pub id: String,
    pub date_added: String,
    pub version: String,
    pub media_type: String,
}

/// Objects response.
///
/// Returned by:
/// - `GET /taxii2/{api_root_id}/collections/{collection_id}/objects/`
/// - `GET /taxii2/{api_root_id}/collections/{collection_id}/objects/{object_id}/`
#[derive(Debug, Serialize)]
pub struct ObjectsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objects: Option<Vec<Value>>,
}

/// Versions response.
///
/// Returned by `GET /taxii2/{api_root_id}/collections/{collection_id}/objects/{object_id}/versions/`
#[derive(Debug, Serialize)]
pub struct VersionsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub versions: Option<Vec<String>>,
}
