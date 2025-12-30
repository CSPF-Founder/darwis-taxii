//! TAXII 1.x route handlers.

use std::sync::{Arc, LazyLock};

use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderName, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use tracing::error;

use taxii_1x::{
    HTTP_X_FORWARDED_PROTO, HTTP_X_FORWARDED_SSL, HTTP_X_TAXII_ACCEPT, HTTP_X_TAXII_CONTENT_TYPE,
    HTTP_X_TAXII_PROTOCOL, HTTP_X_TAXII_SERVICES, HandlerContext, HandlerRegistry, ServiceInfo,
    TaxiiHeaders, TaxiiMessage, VID_TAXII_HTTP_10, VID_TAXII_HTTPS_10, VID_TAXII_XML_10,
    VID_TAXII_XML_11, get_message_from_xml, messages::messages_10 as tm10,
    messages::messages_11 as tm11,
};
use taxii_core::Account;
use taxii_db::{DbTaxii1Repository, Taxii1Repository};

/// Pre-parsed TAXII header names for response building.
#[expect(clippy::expect_used, reason = "infallible: valid header literal")]
static HEADER_X_TAXII_CONTENT_TYPE: LazyLock<HeaderName> = LazyLock::new(|| {
    HTTP_X_TAXII_CONTENT_TYPE
        .parse()
        .expect("valid header name")
});

#[expect(clippy::expect_used, reason = "infallible: valid header literal")]
static HEADER_X_TAXII_PROTOCOL: LazyLock<HeaderName> =
    LazyLock::new(|| HTTP_X_TAXII_PROTOCOL.parse().expect("valid header name"));

#[expect(clippy::expect_used, reason = "infallible: valid header literal")]
static HEADER_X_TAXII_SERVICES: LazyLock<HeaderName> =
    LazyLock::new(|| HTTP_X_TAXII_SERVICES.parse().expect("valid header name"));

/// State for TAXII 1.x routes.
#[derive(Clone)]
pub struct Taxii1xState {
    pub persistence: Arc<DbTaxii1Repository>,
    pub handler_registry: Arc<HandlerRegistry>,
    pub hooks: Option<taxii_core::SharedHookRegistry>,
}

/// Detect if the request is secure (HTTPS).
///
/// Checks X-Forwarded-Proto header (for reverse proxies) or URI scheme.
fn is_request_secure(headers: &HeaderMap, uri: &Uri) -> bool {
    // Check X-Forwarded-Proto header (common for reverse proxies like nginx, AWS ALB)
    if let Some(proto) = headers
        .get(HTTP_X_FORWARDED_PROTO)
        .and_then(|v| v.to_str().ok())
    {
        return proto.eq_ignore_ascii_case("https");
    }

    // Check X-Forwarded-Ssl header (alternative)
    if let Some(ssl) = headers
        .get(HTTP_X_FORWARDED_SSL)
        .and_then(|v| v.to_str().ok())
    {
        return ssl.eq_ignore_ascii_case("on");
    }

    // Check the URI scheme directly
    if let Some(scheme) = uri.scheme_str() {
        return scheme.eq_ignore_ascii_case("https");
    }

    false
}

/// Get the appropriate X-TAXII-Protocol value based on secure status.
fn get_protocol_binding(is_secure: bool) -> &'static str {
    if is_secure {
        VID_TAXII_HTTPS_10
    } else {
        VID_TAXII_HTTP_10
    }
}

/// Handle a TAXII 1.x service request.
pub async fn taxii1x_service_handler(
    State(state): State<Arc<Taxii1xState>>,
    Path(service_id): Path<String>,
    uri: Uri,
    headers: HeaderMap,
    account: Option<axum::Extension<Account>>,
    body: String,
) -> impl IntoResponse {
    // Detect if request is over HTTPS (for proper X-TAXII-Protocol header)
    let is_secure = is_request_secure(&headers, &uri);
    // Extract TAXII headers
    let content_type = headers
        .get(HTTP_X_TAXII_CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let services = headers
        .get(HTTP_X_TAXII_SERVICES)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let accept = headers
        .get(HTTP_X_TAXII_ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    if content_type.is_empty() || services.is_empty() {
        let version = get_version_from_headers(&headers);
        return taxii_error_response(
            "Missing required TAXII headers",
            None,
            StatusCode::BAD_REQUEST,
            version,
            is_secure,
        );
    }

    let taxii_headers = TaxiiHeaders {
        content_type: content_type.to_string(),
        services: services.to_string(),
        accept,
    };

    // Parse the message
    let message = match get_message_from_xml(&body) {
        Ok(msg) => msg,
        Err(e) => {
            let version = get_version_from_headers(&headers);
            return taxii_error_response(
                &format!("Failed to parse TAXII message: {}", e),
                None,
                StatusCode::BAD_REQUEST,
                version,
                is_secure,
            );
        }
    };

    // Get message version for error responses
    let msg_version = message.version();

    // Get service info from database
    let service = match state.persistence.get_service(&service_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return taxii_error_response(
                "Service not found",
                Some(message.message_id()),
                StatusCode::NOT_FOUND,
                msg_version,
                is_secure,
            );
        }
        Err(e) => {
            error!(
                "TAXII 1.x database error fetching service '{}': {:?}",
                service_id, e
            );
            return taxii_error_response(
                "Database error occurred",
                Some(message.message_id()),
                StatusCode::INTERNAL_SERVER_ERROR,
                msg_version,
                is_secure,
            );
        }
    };

    // Create handler context
    let ctx = HandlerContext {
        account: account.map(|a| a.0),
        persistence: state.persistence.clone(),
        service: ServiceInfo {
            id: service.id.clone().unwrap_or_default(),
            service_type: service.service_type.clone(),
            address: format!("/services/{}/", service_id),
            description: service
                .properties
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            protocol_bindings: vec![taxii_1x::VID_TAXII_HTTP_10.to_string()],
            message_bindings: vec![VID_TAXII_XML_10.to_string(), VID_TAXII_XML_11.to_string()],
            available: true,
            authentication_required: false,
            properties: service.properties,
        },
        hooks: state.hooks.clone(),
    };

    // Get handler and process message
    let version = message.version();
    let message_type = message.message_type();

    let handler = match state.handler_registry.get(version, message_type) {
        Some(h) => h,
        None => {
            return taxii_error_response(
                &format!(
                    "No handler for message type: {} (version {})",
                    message_type, version
                ),
                Some(message.message_id()),
                StatusCode::BAD_REQUEST,
                version,
                is_secure,
            );
        }
    };

    // Process based on version
    let response_xml = match message {
        TaxiiMessage::V10(ref msg) => {
            // Validate headers for 1.0
            if let Err(e) = taxii_headers.validate_10(Some(msg.message_id())) {
                return taxii_error_response(
                    &e.to_string(),
                    Some(msg.message_id()),
                    StatusCode::BAD_REQUEST,
                    VID_TAXII_XML_10,
                    is_secure,
                );
            }

            match handler.handle_10(&ctx, &taxii_headers, msg).await {
                Ok(response) => response.to_xml(),
                Err(e) => {
                    error!("TAXII 1.0 handler error: {:?}", e);
                    return taxii_error_response(
                        "Processing error occurred",
                        Some(msg.message_id()),
                        StatusCode::INTERNAL_SERVER_ERROR,
                        VID_TAXII_XML_10,
                        is_secure,
                    );
                }
            }
        }
        TaxiiMessage::V11(ref msg) => {
            // Validate headers for 1.1
            if let Err(e) = taxii_headers.validate_11(Some(msg.message_id())) {
                return taxii_error_response(
                    &e.to_string(),
                    Some(msg.message_id()),
                    StatusCode::BAD_REQUEST,
                    VID_TAXII_XML_11,
                    is_secure,
                );
            }

            match handler.handle_11(&ctx, &taxii_headers, msg).await {
                Ok(response) => response.to_xml(),
                Err(e) => {
                    error!("TAXII 1.1 handler error: {:?}", e);
                    return taxii_error_response(
                        "Processing error occurred",
                        Some(msg.message_id()),
                        StatusCode::INTERNAL_SERVER_ERROR,
                        VID_TAXII_XML_11,
                        is_secure,
                    );
                }
            }
        }
    };

    // Serialize response
    match response_xml {
        Ok(xml) => {
            // Build response headers including required X-TAXII-Protocol
            // Protocol is determined by whether the request was over HTTPS
            let services_value = if version == VID_TAXII_XML_10 {
                taxii_1x::VID_TAXII_SERVICES_10
            } else {
                taxii_1x::VID_TAXII_SERVICES_11
            };

            let protocol_binding = get_protocol_binding(is_secure);

            let response_headers = [
                (header::CONTENT_TYPE, "application/xml"),
                (HEADER_X_TAXII_CONTENT_TYPE.clone(), version),
                (HEADER_X_TAXII_PROTOCOL.clone(), protocol_binding),
                (HEADER_X_TAXII_SERVICES.clone(), services_value),
            ];

            (StatusCode::OK, response_headers, xml).into_response()
        }
        Err(e) => {
            error!("TAXII 1.x response serialization failed: {:?}", e);
            taxii_error_response(
                "Response serialization failed",
                Some(message.message_id()),
                StatusCode::INTERNAL_SERVER_ERROR,
                version,
                is_secure,
            )
        }
    }
}

/// Handle OPTIONS request for TAXII 1.x services.
pub async fn taxii1x_options_handler(
    State(state): State<Arc<Taxii1xState>>,
    Path(service_id): Path<String>,
) -> impl IntoResponse {
    // Get service info from database to determine supported content types
    let service = match state.persistence.get_service(&service_id).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, "Service not found").into_response();
        }
        Err(e) => {
            error!(
                "TAXII 1.x OPTIONS database error for service '{}': {:?}",
                service_id, e
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred").into_response();
        }
    };

    // Determine supported content bindings based on service configuration
    let content_types = format!("{}, {}", VID_TAXII_XML_10, VID_TAXII_XML_11);

    let headers = [
        (header::ALLOW, "POST, OPTIONS"),
        (
            HTTP_X_TAXII_CONTENT_TYPE
                .parse()
                .unwrap_or(header::CONTENT_TYPE),
            &content_types,
        ),
    ];

    // Include service address in response if available
    let service_address = service.id.unwrap_or_default();

    (StatusCode::OK, headers, service_address).into_response()
}

/// Create a TAXII error response with proper XML StatusMessage.
fn taxii_error_response(
    message: &str,
    in_response_to: Option<&str>,
    status: StatusCode,
    version: &str,
    is_secure: bool,
) -> Response {
    let message_id = taxii_1x::http::generate_message_id();
    let protocol_binding = get_protocol_binding(is_secure);

    // Determine if this is TAXII 1.0 or 1.1 based on version
    let is_10 = version == VID_TAXII_XML_10 || version == taxii_1x::VID_TAXII_SERVICES_10;

    let (xml_result, services_value, content_type_value) = if is_10 {
        // TAXII 1.0 StatusMessage
        let status_msg = tm10::StatusMessage::failure(
            message_id,
            in_response_to.map(String::from),
            Some(message.to_string()),
        );
        let xml = tm10::Taxii10Message::StatusMessage(status_msg).to_xml();
        (xml, taxii_1x::VID_TAXII_SERVICES_10, VID_TAXII_XML_10)
    } else {
        // TAXII 1.1 StatusMessage (default)
        let status_msg = tm11::StatusMessage::failure(
            message_id,
            in_response_to.map(String::from),
            Some(message.to_string()),
        );
        let xml = tm11::Taxii11Message::StatusMessage(status_msg).to_xml();
        (xml, taxii_1x::VID_TAXII_SERVICES_11, VID_TAXII_XML_11)
    };

    match xml_result {
        Ok(xml) => {
            let response_headers = [
                (header::CONTENT_TYPE, "application/xml"),
                (HEADER_X_TAXII_CONTENT_TYPE.clone(), content_type_value),
                (HEADER_X_TAXII_PROTOCOL.clone(), protocol_binding),
                (HEADER_X_TAXII_SERVICES.clone(), services_value),
            ];
            (status, response_headers, xml).into_response()
        }
        Err(e) => {
            // Fallback to plain text if XML serialization fails
            error!("Failed to serialize TAXII error response: {:?}", e);
            (status, message.to_string()).into_response()
        }
    }
}

/// Get version from request headers for error responses.
///
/// Checks X-TAXII-Accept first, then X-TAXII-Content-Type. Defaults to TAXII 1.1.
fn get_version_from_headers(headers: &HeaderMap) -> &'static str {
    // First try X-TAXII-Accept
    if let Some(accept) = headers
        .get(HTTP_X_TAXII_ACCEPT)
        .and_then(|v| v.to_str().ok())
    {
        if accept == VID_TAXII_XML_10 {
            return VID_TAXII_XML_10;
        }
    }
    // Then try X-TAXII-Content-Type
    if let Some(content_type) = headers
        .get(HTTP_X_TAXII_CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
    {
        if content_type == VID_TAXII_XML_10 {
            return VID_TAXII_XML_10;
        }
    }
    // Default to 1.1
    VID_TAXII_XML_11
}
