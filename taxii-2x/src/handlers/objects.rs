//! STIX object handlers.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde_json::{Value, json};

use crate::error::{Taxii2Error, Taxii2Result};
use crate::http::{EmptyTaxii2Response, Taxii2Response};
use crate::responses::ObjectsResponse;
use crate::state::{Taxii2State, enforce_pagination_limit};
use crate::validation::{
    DeleteQueryParams, ListQueryParams, ObjectQueryParams, validate_accept_header,
    validate_content_length, validate_content_type, validate_delete_params, validate_envelope,
    validate_list_params, validate_object_params,
};
use taxii_core::{Account, taxii2_datetimeformat};
use taxii_db::{PaginatedResult, Taxii2QueryParams, Taxii2Repository};

/// Objects GET handler.
///
/// GET /taxii2/{api_root_id}/collections/{collection_id}/objects/
pub async fn objects_get_handler(
    State(state): State<Arc<Taxii2State>>,
    Path((api_root_id, collection_id_or_alias)): Path<(String, String)>,
    headers: HeaderMap,
    Query(params): Query<ListQueryParams>,
    account: Option<Extension<Account>>,
) -> Taxii2Result<impl IntoResponse> {
    validate_accept_header(&headers)?;

    let account = account.map(|e| e.0);
    let filter = validate_list_params(&params)?;

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
        items: objects,
        more,
        next: next_param,
    } = state
        .persistence
        .get_objects(&collection.id, &params)
        .await?;

    if objects.is_empty() {
        return Ok(Taxii2Response::new(ObjectsResponse {
            more: None,
            next: None,
            objects: None,
        }));
    }

    let obj_values: Vec<Value> = objects
        .iter()
        .map(|o| {
            let mut obj = o.serialized_data.clone();
            if let Some(map) = obj.as_object_mut() {
                map.insert("id".to_string(), json!(o.id));
                map.insert("type".to_string(), json!(o.stix_type));
                map.insert("spec_version".to_string(), json!(o.spec_version));
            }
            obj
        })
        .collect();

    let headers = build_date_headers(&objects, |o| taxii2_datetimeformat(&o.date_added));

    let response = ObjectsResponse {
        more: Some(more),
        next: next_param,
        objects: Some(obj_values),
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

/// Objects POST handler.
///
/// POST /taxii2/{api_root_id}/collections/{collection_id}/objects/
pub async fn objects_post_handler(
    State(state): State<Arc<Taxii2State>>,
    Path((api_root_id, collection_id_or_alias)): Path<(String, String)>,
    headers: HeaderMap,
    account: Option<Extension<Account>>,
    body: axum::body::Bytes,
) -> Taxii2Result<impl IntoResponse> {
    validate_accept_header(&headers)?;
    validate_content_type(&headers)?;
    validate_content_length(&headers, body.len(), state.config.max_content_length)?;

    let account = account.map(|e| e.0);

    // Validate STIX bundle with stix2-rust
    let validated = validate_envelope(&body, state.config.allow_custom_properties)?;

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

    if !collection.can_write(account.as_ref()) {
        return Err(if account.is_none() {
            Taxii2Error::Unauthorized
        } else {
            Taxii2Error::NotFound("Collection not found".to_string())
        });
    }

    // Extract objects from validated bundle
    let objects = validated.json_data["objects"]
        .as_array()
        .ok_or_else(|| Taxii2Error::Validation("Objects must be an array".to_string()))?;

    let job = state
        .persistence
        .add_objects(&api_root_id, &collection.id, objects)
        .await?;

    Ok(Taxii2Response::with_status(
        job.as_taxii2_dict(),
        StatusCode::ACCEPTED,
    ))
}

/// Single object GET handler.
///
/// GET /taxii2/{api_root_id}/collections/{collection_id}/objects/{object_id}/
pub async fn object_get_handler(
    State(state): State<Arc<Taxii2State>>,
    Path((api_root_id, collection_id_or_alias, object_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    Query(params): Query<ObjectQueryParams>,
    account: Option<Extension<Account>>,
) -> Taxii2Result<impl IntoResponse> {
    validate_accept_header(&headers)?;

    let account = account.map(|e| e.0);
    let filter = validate_object_params(&params)?;

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

    // Get objects filtered by ID
    let match_ids = [object_id];
    let params = Taxii2QueryParams {
        limit: Some(effective_limit),
        added_after: filter.added_after,
        next: filter.next_cursor.as_ref(),
        match_id: Some(&match_ids),
        match_type: None,
        match_version: filter.match_version.as_deref(),
        match_spec_version: filter.match_spec_version.as_deref(),
    };
    let PaginatedResult {
        items: objects,
        more,
        next: next_param,
    } = state
        .persistence
        .get_objects(&collection.id, &params)
        .await?;

    if objects.is_empty() {
        return Ok(Taxii2Response::new(ObjectsResponse {
            more: None,
            next: None,
            objects: None,
        }));
    }

    let obj_values: Vec<Value> = objects
        .iter()
        .map(|o| {
            let mut obj = o.serialized_data.clone();
            if let Some(map) = obj.as_object_mut() {
                map.insert("id".to_string(), json!(o.id));
                map.insert("type".to_string(), json!(o.stix_type));
                map.insert("spec_version".to_string(), json!(o.spec_version));
            }
            obj
        })
        .collect();

    let headers = build_date_headers(&objects, |o| taxii2_datetimeformat(&o.date_added));

    let response = ObjectsResponse {
        more: Some(more),
        next: next_param,
        objects: Some(obj_values),
    };

    Ok(Taxii2Response::new(response).with_headers(headers))
}

/// Single object DELETE handler.
///
/// DELETE /taxii2/{api_root_id}/collections/{collection_id}/objects/{object_id}/
pub async fn object_delete_handler(
    State(state): State<Arc<Taxii2State>>,
    Path((api_root_id, collection_id_or_alias, object_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    Query(params): Query<DeleteQueryParams>,
    account: Option<Extension<Account>>,
) -> Taxii2Result<impl IntoResponse> {
    validate_accept_header(&headers)?;

    let account = account.map(|e| e.0);
    let filter = validate_delete_params(&params)?;

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

    // Need both read and write for delete
    if !collection.can_read(account.as_ref()) || !collection.can_write(account.as_ref()) {
        return Err(if account.is_none() {
            Taxii2Error::Unauthorized
        } else if !collection.can_read(account.as_ref()) && !collection.can_write(account.as_ref())
        {
            Taxii2Error::NotFound("Collection not found".to_string())
        } else {
            Taxii2Error::Forbidden
        });
    }

    state
        .persistence
        .delete_object(
            &collection.id,
            &object_id,
            filter.match_version.as_deref(),
            filter.match_spec_version.as_deref(),
        )
        .await?;

    Ok(EmptyTaxii2Response::new())
}
