//! Discovery and API root handlers.

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;

use crate::error::{Taxii2Error, Taxii2Result};
use crate::http::Taxii2Response;
use crate::responses::{ApiRootResponse, DiscoveryResponse};
use crate::state::Taxii2State;
use crate::validation::validate_accept_header;
use taxii_core::Account;
use taxii_db::Taxii2Repository;

/// Discovery handler.
///
/// GET /taxii2/
pub async fn discovery_handler(
    State(state): State<Arc<Taxii2State>>,
    headers: HeaderMap,
    account: Option<Extension<Account>>,
) -> Taxii2Result<impl IntoResponse> {
    validate_accept_header(&headers)?;

    let account = account.map(|e| e.0);
    // Check public discovery
    if account.is_none() && !state.config.public_discovery {
        return Err(Taxii2Error::Unauthorized);
    }

    let api_roots = state.persistence.get_api_roots().await?;

    let mut default_api_root: Option<String> = None;
    let mut root_urls = Vec::new();

    for root in &api_roots {
        if root.default {
            default_api_root = Some(format!("/taxii2/{}/", root.id));
        }
        root_urls.push(format!("/taxii2/{}/", root.id));
    }

    let response = DiscoveryResponse {
        title: state.config.title.clone(),
        description: state.config.description.clone(),
        contact: state.config.contact.clone(),
        default: default_api_root,
        api_roots: root_urls,
    };

    Ok(Taxii2Response::new(response))
}

/// API Root handler.
///
/// GET /taxii2/{api_root_id}/
pub async fn api_root_handler(
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

    let response = ApiRootResponse {
        title: api_root.title,
        description: api_root.description,
        versions: vec!["application/taxii+json;version=2.1".to_string()],
        max_content_length: state.config.max_content_length,
    };

    Ok(Taxii2Response::new(response))
}

/// Job status handler.
///
/// GET /taxii2/{api_root_id}/status/{job_id}/
pub async fn job_handler(
    State(state): State<Arc<Taxii2State>>,
    Path((api_root_id, job_id)): Path<(String, String)>,
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

    let job = state
        .persistence
        .get_job_and_details(&api_root_id, &job_id)
        .await?
        .ok_or_else(|| Taxii2Error::NotFound("Job not found".to_string()))?;

    Ok(Taxii2Response::new(job.as_taxii2_dict()))
}
