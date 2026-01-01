//! Router setup.

use std::net::IpAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{StatusCode, header::USER_AGENT};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::catch_panic::CatchPanicLayer;
use tracing::error;

use taxii_1x::HandlerRegistry;
use taxii_2x::{Taxii2Config, Taxii2State};
use taxii_auth::{AuthAPI, ClientInfo};
use taxii_core::{HookRegistry, SharedHookRegistry};
use taxii_db::{DbTaxii1Repository, DbTaxii2Repository};

use crate::AuthLayer;
use crate::config::ServerConfig;
use crate::taxii1x_routes::{Taxii1xState, taxii1x_options_handler, taxii1x_service_handler};

/// Health check response.
#[derive(Serialize)]
struct HealthResponse {
    alive: bool,
}

/// Health check handler.
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { alive: true })
}

/// Auth request body.
#[derive(Deserialize)]
struct AuthRequest {
    username: String,
    password: String,
}

/// Auth response.
#[derive(Serialize)]
struct AuthResponse {
    token: String,
}

/// State for management routes that need auth.
struct ManagementState {
    auth: Arc<AuthAPI>,
}

/// Extract client IP from headers or connection.
fn extract_client_ip(headers: &axum::http::HeaderMap) -> Option<IpAddr> {
    // Try X-Forwarded-For first (for reverse proxies)
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(xff_str) = xff.to_str() {
            // Take the first IP in the chain (original client)
            if let Some(first_ip) = xff_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse() {
                    return Some(ip);
                }
            }
        }
    }

    // Try X-Real-IP
    if let Some(xri) = headers.get("x-real-ip") {
        if let Ok(xri_str) = xri.to_str() {
            if let Ok(ip) = xri_str.trim().parse() {
                return Some(ip);
            }
        }
    }

    None
}

/// Extract user agent from headers.
fn extract_user_agent(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// Auth handler - authenticate user and return JWT token.
async fn auth_handler(
    State(state): State<Arc<ManagementState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<AuthRequest>,
) -> impl IntoResponse {
    if req.username.is_empty() || req.password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "Both username and password are required",
        )
            .into_response();
    }

    let client_info = ClientInfo::new(extract_client_ip(&headers), extract_user_agent(&headers));

    match state
        .auth
        .authenticate_with_logging(&req.username, &req.password, client_info)
        .await
    {
        Ok(Some(token)) => Json(AuthResponse { token }).into_response(),
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Err(e) => {
            error!("Authentication error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// Result of creating a router, includes the hook registry for event subscription.
pub struct RouterWithHooks {
    /// The Axum router.
    pub router: Router,
    /// Hook registry for subscribing to TAXII events.
    ///
    /// Use `hooks.subscribe()` to get a receiver for events like:
    /// - `SignalEvent::ContentBlockCreated` - when content is added via inbox
    /// - `SignalEvent::InboxMessageCreated` - when an inbox message is received
    /// - `SignalEvent::SubscriptionCreated` - when a subscription is created
    pub hooks: SharedHookRegistry,
}

/// Create the Axum router with hook registry.
///
/// Returns both the router and the hook registry. The hook registry can be used
/// to subscribe to TAXII events (content block created, inbox message, subscription).
///
/// # Example
///
/// ```ignore
/// let RouterWithHooks { router, hooks } = create_router_with_hooks(
///     taxii1_persistence,
///     taxii2_persistence,
///     auth,
///     &config,
/// );
///
/// // Subscribe to events
/// let mut receiver = hooks.subscribe();
/// tokio::spawn(async move {
///     while let Ok(event) = receiver.recv().await {
///         match event {
///             SignalEvent::ContentBlockCreated(e) => {
///                 println!("Content block created: {:?}", e.content_block.id);
///             }
///             SignalEvent::InboxMessageCreated(e) => {
///                 println!("Inbox message: {}", e.inbox_message.message_id);
///             }
///             SignalEvent::SubscriptionCreated(e) => {
///                 println!("Subscription: {:?}", e.subscription.subscription_id);
///             }
///         }
///     }
/// });
///
/// // Start server
/// axum::serve(listener, router).await?;
/// ```
pub fn create_router_with_hooks(
    taxii1_persistence: DbTaxii1Repository,
    taxii2_persistence: DbTaxii2Repository,
    auth: AuthAPI,
    config: &ServerConfig,
) -> RouterWithHooks {
    let hooks = Arc::new(HookRegistry::new());
    let router = create_router_internal(
        taxii1_persistence,
        taxii2_persistence,
        auth,
        config,
        Some(hooks.clone()),
    );
    RouterWithHooks { router, hooks }
}

/// Create the Axum router without hook support.
///
/// Use `create_router_with_hooks` if you need to subscribe to TAXII events.
pub fn create_router(
    taxii1_persistence: DbTaxii1Repository,
    taxii2_persistence: DbTaxii2Repository,
    auth: AuthAPI,
    config: &ServerConfig,
) -> Router {
    create_router_internal(taxii1_persistence, taxii2_persistence, auth, config, None)
}

/// Internal router creation with optional hooks.
fn create_router_internal(
    taxii1_persistence: DbTaxii1Repository,
    taxii2_persistence: DbTaxii2Repository,
    auth: AuthAPI,
    config: &ServerConfig,
    hooks: Option<SharedHookRegistry>,
) -> Router {
    let auth = Arc::new(auth);

    // TAXII 2.x state
    let taxii2_config = Taxii2Config {
        title: config.title.clone(),
        description: config.description.clone(),
        contact: config.contact.clone(),
        max_content_length: config.max_content_length,
        public_discovery: config.public_discovery,
        allow_custom_properties: config.allow_custom_properties,
        default_pagination_limit: config.default_pagination_limit,
        max_pagination_limit: config.max_pagination_limit,
    };

    let taxii2_state = Arc::new(Taxii2State {
        persistence: taxii2_persistence,
        config: taxii2_config,
    });

    // TAXII 2.x routes
    // Note: Using :param syntax for Axum path parameters
    let taxii2_routes = Router::new()
        // Discovery
        .route("/taxii2/", get(taxii_2x::discovery_handler))
        // API Root
        .route("/taxii2/{api_root_id}/", get(taxii_2x::api_root_handler))
        // Job status
        .route(
            "/taxii2/{api_root_id}/status/{job_id}/",
            get(taxii_2x::job_handler),
        )
        // Collections
        .route(
            "/taxii2/{api_root_id}/collections/",
            get(taxii_2x::collections_handler),
        )
        // Single collection
        .route(
            "/taxii2/{api_root_id}/collections/{collection_id}/",
            get(taxii_2x::collection_handler),
        )
        // Manifest
        .route(
            "/taxii2/{api_root_id}/collections/{collection_id}/manifest/",
            get(taxii_2x::manifest_handler),
        )
        // Objects (GET/POST)
        .route(
            "/taxii2/{api_root_id}/collections/{collection_id}/objects/",
            get(taxii_2x::objects_get_handler).post(taxii_2x::objects_post_handler),
        )
        // Single object (GET/DELETE)
        .route(
            "/taxii2/{api_root_id}/collections/{collection_id}/objects/{object_id}/",
            get(taxii_2x::object_get_handler).delete(taxii_2x::object_delete_handler),
        )
        // Versions
        .route(
            "/taxii2/{api_root_id}/collections/{collection_id}/objects/{object_id}/versions/",
            get(taxii_2x::versions_handler),
        )
        .with_state(taxii2_state);

    // TAXII 1.x state
    let taxii1x_state = Arc::new(Taxii1xState {
        persistence: Arc::new(taxii1_persistence),
        handler_registry: Arc::new(HandlerRegistry::new()),
        hooks,
    });

    // TAXII 1.x routes
    // Single POST endpoint per service, routing based on message type
    // OPTIONS endpoint for content type negotiation
    let taxii1x_routes = Router::new()
        .route(
            "/services/{service_id}/",
            post(taxii1x_service_handler).options(taxii1x_options_handler),
        )
        .with_state(taxii1x_state);

    // Management routes (no auth required)
    // Note: /management/auth needs AuthAPI access but doesn't require authentication itself
    let management_state = Arc::new(ManagementState { auth: auth.clone() });

    let management_routes = Router::new()
        .route("/management/health", get(health_handler))
        .route(
            "/management/auth",
            post(auth_handler).with_state(management_state),
        );

    // Combine routes with auth middleware
    // CatchPanicLayer is the outermost layer as a safety net for unhandled panics
    Router::new()
        .merge(management_routes) // Health endpoint before auth
        .merge(taxii2_routes)
        .merge(taxii1x_routes)
        .layer(AuthLayer::new(auth, config.support_basic_auth))
        .layer(CatchPanicLayer::custom(|panic_info| {
            // Log the panic with full details for debugging
            error!("Handler panicked: {:?}", panic_info);
            // Return a generic error response - no internal details exposed
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }))
}
