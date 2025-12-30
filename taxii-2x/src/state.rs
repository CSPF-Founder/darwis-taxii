//! TAXII 2.x server state and configuration.

use taxii_db::DbTaxii2Repository;

/// Configuration for a TAXII 2.1 server instance.
///
/// Controls server-wide behavior including discovery information,
/// content limits, and pagination settings.
///
/// # Discovery Response
///
/// The `title`, `description`, and `contact` fields are returned in the
/// server discovery response (`GET /taxii2/`), helping clients identify
/// the server and its operator.
///
/// # Security
///
/// - `public_discovery`: When `false`, the discovery endpoint requires
///   authentication. Set to `true` for public servers.
///
/// # Content Limits
///
/// - `max_content_length`: Maximum size of POST bodies (STIX bundles).
///   Protects against resource exhaustion attacks.
/// - `allow_custom_properties`: Whether to accept STIX objects with
///   custom properties beyond the specification.
///
/// # Pagination
///
/// TAXII 2.1 requires pagination for large result sets. The server
/// enforces limits to prevent excessive memory usage:
/// - `default_pagination_limit`: Used when client doesn't specify `limit`
/// - `max_pagination_limit`: Hard cap regardless of client request
#[derive(Debug, Clone)]
pub struct Taxii2Config {
    /// Server title shown in discovery response.
    pub title: String,

    /// Optional server description for discovery response.
    pub description: Option<String>,

    /// Optional contact information (email, URL) for server operator.
    pub contact: Option<String>,

    /// Maximum allowed content length for POST requests in bytes.
    ///
    /// Requests exceeding this limit receive HTTP 413 (Request Entity Too Large).
    pub max_content_length: usize,

    /// Whether to allow unauthenticated access to the discovery endpoint.
    ///
    /// When `false`, clients must authenticate to discover API roots.
    pub public_discovery: bool,

    /// Whether to accept STIX objects with custom properties.
    ///
    /// The STIX specification allows custom properties prefixed with `x_`.
    /// Set to `false` for strict validation.
    pub allow_custom_properties: bool,

    /// Default pagination limit when client omits the `limit` parameter.
    ///
    /// Applied to objects, manifest, and versions endpoints.
    pub default_pagination_limit: i64,

    /// Maximum pagination limit (hard cap).
    ///
    /// Client-requested limits exceeding this value are reduced.
    pub max_pagination_limit: i64,
}

impl Default for Taxii2Config {
    fn default() -> Self {
        Self {
            title: "DARWIS TAXII".to_string(),
            description: None,
            contact: None,
            max_content_length: 10 * 1024 * 1024, // 10MB
            public_discovery: false,
            allow_custom_properties: true,
            default_pagination_limit: 1000,
            max_pagination_limit: 1000,
        }
    }
}

/// Shared application state for TAXII 2.1 route handlers.
///
/// This struct is wrapped in `Arc` and passed to all Axum handlers via
/// the `State` extractor. It provides:
///
/// - Database access for API roots, collections, and STIX objects
/// - Server configuration for limits and behavior
///
/// # Example
///
/// ```ignore
/// let state = Arc::new(Taxii2State {
///     persistence: DbTaxii2Repository::new(pool),
///     config: Taxii2Config::default(),
/// });
///
/// let app = Router::new()
///     .route("/taxii2/", get(discovery_handler))
///     .with_state(state);
/// ```
pub struct Taxii2State {
    /// Database access layer for TAXII 2.1 entities.
    ///
    /// Provides methods for querying API roots, collections, objects,
    /// manifests, and versions.
    pub persistence: DbTaxii2Repository,

    /// Server configuration controlling limits and behavior.
    pub config: Taxii2Config,
}

/// Enforce pagination limits on a requested limit value.
///
/// Returns the effective limit to use:
/// - If no limit requested, use default_limit
/// - If limit requested, cap at max_limit
#[inline]
pub fn enforce_pagination_limit(requested: Option<i64>, default_limit: i64, max_limit: i64) -> i64 {
    let limit = requested.unwrap_or(default_limit);
    limit.min(max_limit)
}
