//! TAXII 2.x validation.
//!
//! Uses stix2-rust for full STIX 2.1 validation with type-safe objects.

use axum::http::{HeaderMap, header};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

use crate::error::{Taxii2Error, Taxii2Result};
use crate::http::{VALID_ACCEPT_MIMETYPES, VALID_CONTENT_TYPES};
use taxii_db::{PaginationCursor, parse_next_param as db_parse_next_param};

/// TAXII 2.x datetime format.
pub const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.6fZ";

/// Version filter value: return only the first version.
pub const VERSION_FIRST: &str = "first";
/// Version filter value: return only the last (most recent) version.
pub const VERSION_LAST: &str = "last";
/// Version filter value: return all versions.
pub const VERSION_ALL: &str = "all";

/// Result of STIX bundle validation.
///
/// Contains the parsed type-safe bundle and raw JSON data for storage.
#[derive(Debug)]
pub struct ValidatedBundle {
    /// The parsed type-safe bundle.
    pub bundle: stix2::Bundle,

    /// The raw JSON data for storage.
    pub json_data: serde_json::Value,

    /// List of parsed STIX objects.
    pub objects: Vec<stix2::StixObject>,
}

// =============================================================================
// Raw Query Parameter Structs (for serde deserialization)
// =============================================================================

/// Raw query parameters for list endpoints (objects, manifest).
///
/// These are deserialized directly from the query string using Axum's Query extractor.
#[derive(Debug, Default, Deserialize)]
pub struct ListQueryParams {
    pub limit: Option<String>,
    pub added_after: Option<String>,
    pub next: Option<String>,
    #[serde(rename = "match[id]")]
    pub match_id: Option<String>,
    #[serde(rename = "match[type]")]
    pub match_type: Option<String>,
    #[serde(rename = "match[version]")]
    pub match_version: Option<String>,
    #[serde(rename = "match[spec_version]")]
    pub match_spec_version: Option<String>,
}

/// Raw query parameters for single object endpoints.
#[derive(Debug, Default, Deserialize)]
pub struct ObjectQueryParams {
    pub limit: Option<String>,
    pub added_after: Option<String>,
    pub next: Option<String>,
    #[serde(rename = "match[version]")]
    pub match_version: Option<String>,
    #[serde(rename = "match[spec_version]")]
    pub match_spec_version: Option<String>,
}

/// Raw query parameters for versions endpoint.
#[derive(Debug, Default, Deserialize)]
pub struct VersionsQueryParams {
    pub limit: Option<String>,
    pub added_after: Option<String>,
    pub next: Option<String>,
    #[serde(rename = "match[spec_version]")]
    pub match_spec_version: Option<String>,
}

/// Raw query parameters for delete endpoint.
#[derive(Debug, Default, Deserialize)]
pub struct DeleteQueryParams {
    #[serde(rename = "match[version]")]
    pub match_version: Option<String>,
    #[serde(rename = "match[spec_version]")]
    pub match_spec_version: Option<String>,
}

// =============================================================================
// Validated Filter Parameter Structs
// =============================================================================

/// Validated filter parameters for list endpoints.
#[derive(Debug, Default)]
pub struct ListFilterParams {
    pub limit: Option<i64>,
    pub added_after: Option<DateTime<Utc>>,
    pub next_cursor: Option<PaginationCursor>,
    pub match_id: Option<Vec<String>>,
    pub match_type: Option<Vec<String>>,
    pub match_version: Option<Vec<String>>,
    pub match_spec_version: Option<Vec<String>>,
}

/// Validated filter parameters for object endpoints.
#[derive(Debug, Default)]
pub struct ObjectFilterParams {
    pub limit: Option<i64>,
    pub added_after: Option<DateTime<Utc>>,
    pub next_cursor: Option<PaginationCursor>,
    pub match_version: Option<Vec<String>>,
    pub match_spec_version: Option<Vec<String>>,
}

/// Validated filter parameters for versions endpoint.
#[derive(Debug, Default)]
pub struct VersionFilterParams {
    pub limit: Option<i64>,
    pub added_after: Option<DateTime<Utc>>,
    pub next_cursor: Option<PaginationCursor>,
    pub match_spec_version: Option<Vec<String>>,
}

/// Validated filter parameters for delete endpoint.
#[derive(Debug, Default)]
pub struct DeleteFilterParams {
    pub match_version: Option<Vec<String>>,
    pub match_spec_version: Option<Vec<String>>,
}

// =============================================================================
// Parsing Helper Functions
// =============================================================================

/// Parse comma-separated filter value.
#[inline]
fn parse_filter(value: &str) -> Vec<String> {
    value.split(',').map(str::to_string).collect()
}

/// Parse version filter (handles "first", "last", "all", and datetime values).
#[inline]
fn parse_version_filter(value: &str) -> Vec<String> {
    // All values are kept as-is; datetime validation happens at query time
    value.split(',').map(str::to_string).collect()
}

/// Parse next parameter into a pagination cursor.
#[inline]
fn parse_next_param(value: &str) -> Option<PaginationCursor> {
    db_parse_next_param(value)
}

/// Parse limit parameter.
#[inline]
fn parse_limit(value: Option<&str>) -> Taxii2Result<Option<i64>> {
    value
        .map(|s| {
            s.parse()
                .map_err(|_| Taxii2Error::Validation("Invalid limit".to_string()))
        })
        .transpose()
}

/// Parse added_after datetime parameter.
#[inline]
fn parse_added_after(value: Option<&str>) -> Taxii2Result<Option<DateTime<Utc>>> {
    value
        .map(|s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| Taxii2Error::Validation("Invalid added_after datetime".to_string()))
        })
        .transpose()
}

/// Validate envelope (STIX bundle) using stix2-rust.
///
/// Handles both:
/// - Full STIX bundles: `{"type": "bundle", "id": "bundle--...", "objects": [...]}`
/// - TAXII envelopes: `{"objects": [...]}` (lenient mode)
///
/// # Arguments
///
/// * `json_data` - Raw JSON bytes of the STIX bundle
/// * `_allow_custom` - Whether to allow custom STIX types and properties (reserved for future use)
///
/// # Returns
///
/// A `ValidatedBundle` containing the parsed objects and raw JSON data.
pub fn validate_envelope(json_data: &[u8], _allow_custom: bool) -> Taxii2Result<ValidatedBundle> {
    let json_str = std::str::from_utf8(json_data)
        .map_err(|e| Taxii2Error::Validation(format!("Invalid UTF-8: {e}")))?;

    // Parse JSON first to check structure
    let json_value: serde_json::Value = serde_json::from_str(json_str)?;

    // Check if this is a full bundle or just an envelope with objects
    let is_full_bundle = json_value.get("type").and_then(|v| v.as_str()) == Some("bundle")
        && json_value.get("id").is_some();

    if is_full_bundle {
        // Parse as full Bundle using stix2-rust
        let bundle: stix2::Bundle = stix2::parse_bundle(json_str)?;
        let objects: Vec<stix2::StixObject> = bundle.objects.clone();
        let json_data = serde_json::to_value(&bundle)?;

        Ok(ValidatedBundle {
            bundle,
            json_data,
            objects,
        })
    } else {
        // Lenient parsing: just require "objects" array
        let objects_array = json_value
            .get("objects")
            .and_then(|v| v.as_array())
            .ok_or_else(|| Taxii2Error::Validation("No objects array in envelope".to_string()))?;

        // Parse each object individually with stix2
        let mut objects = Vec::with_capacity(objects_array.len());
        for (idx, obj_value) in objects_array.iter().enumerate() {
            let obj: stix2::StixObject =
                serde_json::from_value(obj_value.clone()).map_err(|e| {
                    Taxii2Error::Validation(format!(
                        "Invalid STIX object at index {}: {}; object: {}",
                        idx,
                        e,
                        serde_json::to_string(obj_value).unwrap_or_default()
                    ))
                })?;
            objects.push(obj);
        }

        // Create a synthetic bundle for storage
        let bundle = stix2::Bundle::from_objects(objects.clone());
        let json_data = serde_json::to_value(&bundle)?;

        Ok(ValidatedBundle {
            bundle,
            json_data,
            objects,
        })
    }
}

/// Validate and parse list filter parameters from typed query params.
pub fn validate_list_params(params: &ListQueryParams) -> Taxii2Result<ListFilterParams> {
    Ok(ListFilterParams {
        limit: parse_limit(params.limit.as_deref())?,
        added_after: parse_added_after(params.added_after.as_deref())?,
        next_cursor: params.next.as_deref().and_then(parse_next_param),
        match_id: params.match_id.as_deref().map(parse_filter),
        match_type: params.match_type.as_deref().map(parse_filter),
        match_version: params.match_version.as_deref().map(parse_version_filter),
        match_spec_version: params.match_spec_version.as_deref().map(parse_filter),
    })
}

/// Validate and parse list filter parameters from HashMap (legacy compatibility).
#[deprecated(note = "Use validate_list_params with ListQueryParams instead")]
pub fn validate_list_filter_params(
    params: &HashMap<String, String>,
) -> Taxii2Result<ListFilterParams> {
    Ok(ListFilterParams {
        limit: parse_limit(params.get("limit").map(String::as_str))?,
        added_after: parse_added_after(params.get("added_after").map(String::as_str))?,
        next_cursor: params.get("next").and_then(|s| parse_next_param(s)),
        match_id: params.get("match[id]").map(|s| parse_filter(s)),
        match_type: params.get("match[type]").map(|s| parse_filter(s)),
        match_version: params
            .get("match[version]")
            .map(|s| parse_version_filter(s)),
        match_spec_version: params.get("match[spec_version]").map(|s| parse_filter(s)),
    })
}

/// Validate and parse object filter parameters from typed query params.
pub fn validate_object_params(params: &ObjectQueryParams) -> Taxii2Result<ObjectFilterParams> {
    Ok(ObjectFilterParams {
        limit: parse_limit(params.limit.as_deref())?,
        added_after: parse_added_after(params.added_after.as_deref())?,
        next_cursor: params.next.as_deref().and_then(parse_next_param),
        match_version: params.match_version.as_deref().map(parse_version_filter),
        match_spec_version: params.match_spec_version.as_deref().map(parse_filter),
    })
}

/// Validate and parse object filter parameters from HashMap (legacy compatibility).
#[deprecated(note = "Use validate_object_params with ObjectQueryParams instead")]
pub fn validate_object_filter_params(
    params: &HashMap<String, String>,
) -> Taxii2Result<ObjectFilterParams> {
    Ok(ObjectFilterParams {
        limit: parse_limit(params.get("limit").map(String::as_str))?,
        added_after: parse_added_after(params.get("added_after").map(String::as_str))?,
        next_cursor: params.get("next").and_then(|s| parse_next_param(s)),
        match_version: params
            .get("match[version]")
            .map(|s| parse_version_filter(s)),
        match_spec_version: params.get("match[spec_version]").map(|s| parse_filter(s)),
    })
}

/// Validate and parse version filter parameters from typed query params.
pub fn validate_versions_params(params: &VersionsQueryParams) -> Taxii2Result<VersionFilterParams> {
    Ok(VersionFilterParams {
        limit: parse_limit(params.limit.as_deref())?,
        added_after: parse_added_after(params.added_after.as_deref())?,
        next_cursor: params.next.as_deref().and_then(parse_next_param),
        match_spec_version: params.match_spec_version.as_deref().map(parse_filter),
    })
}

/// Validate and parse version filter parameters from HashMap (legacy compatibility).
#[deprecated(note = "Use validate_versions_params with VersionsQueryParams instead")]
pub fn validate_versions_filter_params(
    params: &HashMap<String, String>,
) -> Taxii2Result<VersionFilterParams> {
    Ok(VersionFilterParams {
        limit: parse_limit(params.get("limit").map(String::as_str))?,
        added_after: parse_added_after(params.get("added_after").map(String::as_str))?,
        next_cursor: params.get("next").and_then(|s| parse_next_param(s)),
        match_spec_version: params.get("match[spec_version]").map(|s| parse_filter(s)),
    })
}

/// Validate and parse delete filter parameters from typed query params.
pub fn validate_delete_params(params: &DeleteQueryParams) -> Taxii2Result<DeleteFilterParams> {
    Ok(DeleteFilterParams {
        match_version: params.match_version.as_deref().map(parse_version_filter),
        match_spec_version: params.match_spec_version.as_deref().map(parse_filter),
    })
}

/// Validate and parse delete filter parameters from HashMap (legacy compatibility).
#[deprecated(note = "Use validate_delete_params with DeleteQueryParams instead")]
pub fn validate_delete_filter_params(
    params: &HashMap<String, String>,
) -> Taxii2Result<DeleteFilterParams> {
    Ok(DeleteFilterParams {
        match_version: params
            .get("match[version]")
            .map(|s| parse_version_filter(s)),
        match_spec_version: params.get("match[spec_version]").map(|s| parse_filter(s)),
    })
}

// =============================================================================
// HTTP Header Validation
// =============================================================================

/// Validate Accept header for TAXII 2.x requests.
///
/// Checks that the Accept header contains a valid TAXII 2.x media type.
/// Accepts `*/*` as a wildcard.
pub fn validate_accept_header(headers: &HeaderMap) -> Taxii2Result<()> {
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("*/*");

    let is_valid = VALID_ACCEPT_MIMETYPES
        .iter()
        .any(|valid| accept.contains(valid) || accept == "*/*");

    if !is_valid {
        return Err(Taxii2Error::NotAcceptable);
    }
    Ok(())
}

/// Validate Content-Type header for POST requests.
///
/// Ensures the Content-Type matches one of the valid TAXII 2.x content types.
pub fn validate_content_type(headers: &HeaderMap) -> Taxii2Result<()> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !VALID_CONTENT_TYPES.contains(&content_type) {
        return Err(Taxii2Error::UnsupportedMediaType);
    }
    Ok(())
}

/// Validate content length against maximum allowed size.
///
/// Checks both the Content-Length header (if present) and the actual body length.
pub fn validate_content_length(
    headers: &HeaderMap,
    body_len: usize,
    max_len: usize,
) -> Taxii2Result<()> {
    // Check Content-Length header if present
    if let Some(content_length) = headers
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
    {
        if content_length > max_len {
            return Err(Taxii2Error::RequestEntityTooLarge);
        }
    }

    // Also check actual body length
    if body_len > max_len {
        return Err(Taxii2Error::RequestEntityTooLarge);
    }

    Ok(())
}
