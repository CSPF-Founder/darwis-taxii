//! Collection and manifest handlers.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;

use crate::error::{Taxii2Error, Taxii2Result};
use crate::http::Taxii2Response;
use crate::responses::{
    CollectionInfo, CollectionsResponse, ManifestEntry, ManifestResponse, VersionsResponse,
};
use crate::state::{Taxii2State, enforce_pagination_limit};
use crate::validation::{
    ListQueryParams, VersionsQueryParams, validate_accept_header, validate_list_params,
    validate_versions_params,
};
use taxii_core::{Account, taxii2_datetimeformat};
use taxii_db::{PaginatedResult, Taxii2QueryParams, Taxii2Repository};

/// Collections handler.
///
/// GET /taxii2/{api_root_id}/collections/
pub async fn collections_handler(
    State(state): State<Arc<Taxii2State>>,
    Path(api_root_id): Path<String>,
    headers: HeaderMap,
    account: Option<Extension<Account>>,
) -> Taxii2Result<impl IntoResponse> {
    validate_accept_header(&headers)?;

    let account = account.map(|e| e.0);
    let api_root = state
        .persistence
        .get_api_root(&api_root_id)
        .await?
        .ok_or_else(|| {
            if account.is_none() {
                Taxii2Error::Unauthorized
            } else {
                Taxii2Error::NotFound("API root not found".to_string())
            }
        })?;

    if account.is_none() && !api_root.is_public {
        return Err(Taxii2Error::Unauthorized);
    }

    let collections = state.persistence.get_collections(&api_root_id).await?;

    let collection_infos: Vec<CollectionInfo> = collections
        .iter()
        .map(|c| CollectionInfo {
            id: c.id.clone(),
            title: c.title.clone(),
            description: c.description.clone(),
            alias: c.alias.clone(),
            can_read: c.can_read(account.as_ref()),
            can_write: c.can_write(account.as_ref()),
            media_types: vec!["application/stix+json;version=2.1".to_string()],
        })
        .collect();

    let response = if collection_infos.is_empty() {
        CollectionsResponse { collections: None }
    } else {
        CollectionsResponse {
            collections: Some(collection_infos),
        }
    };

    Ok(Taxii2Response::new(response))
}

/// Single collection handler.
///
/// GET /taxii2/{api_root_id}/collections/{collection_id}/
pub async fn collection_handler(
    State(state): State<Arc<Taxii2State>>,
    Path((api_root_id, collection_id_or_alias)): Path<(String, String)>,
    headers: HeaderMap,
    account: Option<Extension<Account>>,
) -> Taxii2Result<impl IntoResponse> {
    validate_accept_header(&headers)?;

    let account = account.map(|e| e.0);
    let collection = state
        .persistence
        .get_collection(&api_root_id, &collection_id_or_alias)
        .await?
        .ok_or_else(|| {
            if account.is_none() {
                Taxii2Error::Unauthorized
            } else {
                Taxii2Error::NotFound("Collection not found".to_string())
            }
        })?;

    // Check access
    if account.is_none()
        && !(collection.can_read(account.as_ref()) || collection.can_write(account.as_ref()))
    {
        return Err(Taxii2Error::Unauthorized);
    }

    let response = CollectionInfo {
        id: collection.id.clone(),
        title: collection.title.clone(),
        description: collection.description.clone(),
        alias: collection.alias.clone(),
        can_read: collection.can_read(account.as_ref()),
        can_write: collection.can_write(account.as_ref()),
        media_types: vec!["application/stix+json;version=2.1".to_string()],
    };

    Ok(Taxii2Response::new(response))
}

/// Manifest handler.
///
/// GET /taxii2/{api_root_id}/collections/{collection_id}/manifest/
pub async fn manifest_handler(
    State(state): State<Arc<Taxii2State>>,
    Path((api_root_id, collection_id_or_alias)): Path<(String, String)>,
    headers: HeaderMap,
    Query(params): Query<ListQueryParams>,
    account: Option<Extension<Account>>,
) -> Taxii2Result<impl IntoResponse> {
    validate_accept_header(&headers)?;

    let account = account.map(|e| e.0);
    let filter = validate_list_params(&params)?;

    // Get collection first to check access
    let collection = state
        .persistence
        .get_collection(&api_root_id, &collection_id_or_alias)
        .await?
        .ok_or_else(|| {
            if account.is_none() {
                Taxii2Error::Unauthorized
            } else {
                Taxii2Error::NotFound("Collection not found".to_string())
            }
        })?;

    if !collection.can_read(account.as_ref()) {
        return Err(if account.is_none() {
            Taxii2Error::Unauthorized
        } else {
            Taxii2Error::NotFound("Collection not found".to_string())
        });
    }

    // Enforce pagination limits
    let effective_limit = enforce_pagination_limit(
        filter.limit,
        state.config.default_pagination_limit,
        state.config.max_pagination_limit,
    );

    let params = Taxii2QueryParams {
        limit: Some(effective_limit),
        added_after: filter.added_after,
        next: filter.next_cursor.as_ref(),
        match_id: filter.match_id.as_deref(),
        match_type: filter.match_type.as_deref(),
        match_version: filter.match_version.as_deref(),
        match_spec_version: filter.match_spec_version.as_deref(),
    };
    let PaginatedResult {
        items: manifest,
        more,
        next: next_param,
    } = state
        .persistence
        .get_manifest(&collection.id, &params)
        .await?;

    if manifest.is_empty() {
        return Ok(Taxii2Response::new(ManifestResponse {
            more: None,
            next: None,
            objects: None,
        }));
    }

    let entries: Vec<ManifestEntry> = manifest
        .iter()
        .map(|m| ManifestEntry {
            id: m.id.clone(),
            date_added: taxii2_datetimeformat(&m.date_added),
            version: taxii2_datetimeformat(&m.version),
            media_type: format!("application/stix+json;version={}", m.spec_version),
        })
        .collect();

    let headers = build_date_headers(&entries, |e| e.date_added.clone());

    let response = ManifestResponse {
        more: Some(more),
        next: next_param,
        objects: Some(entries),
    };

    Ok(Taxii2Response::new(response).with_headers(headers))
}

/// Build X-TAXII-Date-Added-First and X-TAXII-Date-Added-Last headers.
fn build_date_headers<T, F>(items: &[T], date_fn: F) -> Vec<(String, String)>
where
    F: Fn(&T) -> String,
{
    if items.is_empty() {
        return Vec::new();
    }

    let dates: Vec<String> = items.iter().map(&date_fn).collect();
    let first = dates.iter().min();
    let last = dates.iter().max();

    [
        first.map(|d| ("X-TAXII-Date-Added-First".to_string(), d.clone())),
        last.map(|d| ("X-TAXII-Date-Added-Last".to_string(), d.clone())),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Versions handler.
///
/// GET /taxii2/{api_root_id}/collections/{collection_id}/objects/{object_id}/versions/
pub async fn versions_handler(
    State(state): State<Arc<Taxii2State>>,
    Path((api_root_id, collection_id_or_alias, object_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    Query(params): Query<VersionsQueryParams>,
    account: Option<Extension<Account>>,
) -> Taxii2Result<impl IntoResponse> {
    validate_accept_header(&headers)?;

    let account = account.map(|e| e.0);
    let filter = validate_versions_params(&params)?;

    let collection = state
        .persistence
        .get_collection(&api_root_id, &collection_id_or_alias)
        .await?
        .ok_or_else(|| {
            if account.is_none() {
                Taxii2Error::Unauthorized
            } else {
                Taxii2Error::NotFound("Collection not found".to_string())
            }
        })?;

    if !collection.can_read(account.as_ref()) {
        return Err(if account.is_none() {
            Taxii2Error::Unauthorized
        } else {
            Taxii2Error::NotFound("Collection not found".to_string())
        });
    }

    // Enforce pagination limits
    let effective_limit = enforce_pagination_limit(
        filter.limit,
        state.config.default_pagination_limit,
        state.config.max_pagination_limit,
    );

    let PaginatedResult {
        items: versions,
        more,
        next: next_param,
    } = state
        .persistence
        .get_versions(
            &collection.id,
            &object_id,
            Some(effective_limit),
            filter.added_after,
            filter.next_cursor,
            filter.match_spec_version.as_deref(),
        )
        .await?;

    if versions.is_empty() {
        return Ok(Taxii2Response::new(VersionsResponse {
            more: None,
            next: None,
            versions: None,
        }));
    }

    let version_strings: Vec<String> = versions
        .iter()
        .map(|v| taxii2_datetimeformat(&v.version))
        .collect();

    let headers = build_date_headers(&versions, |v| taxii2_datetimeformat(&v.date_added));

    let response = VersionsResponse {
        more: Some(more),
        next: next_param,
        versions: Some(version_strings),
    };

    Ok(Taxii2Response::new(response).with_headers(headers))
}
