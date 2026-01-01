//! TAXII 1.x HTTP handling.

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::constants::{
    VID_TAXII_HTTP_10, VID_TAXII_HTTPS_10, VID_TAXII_SERVICES_10, VID_TAXII_SERVICES_11,
    VID_TAXII_XML_10, VID_TAXII_XML_11,
};
use crate::error::{Taxii1xError, Taxii1xResult};

// HTTP Headers
pub const HTTP_CONTENT_TYPE: &str = "Content-Type";
pub const HTTP_ACCEPT: &str = "Accept";
pub const HTTP_AUTHORIZATION: &str = "Authorization";
pub const HTTP_ALLOW: &str = "Allow";

// Proxy headers for HTTPS detection
pub const HTTP_X_FORWARDED_PROTO: &str = "X-Forwarded-Proto";
pub const HTTP_X_FORWARDED_SSL: &str = "X-Forwarded-Ssl";

// TAXII-specific headers
pub const HTTP_X_TAXII_CONTENT_TYPE: &str = "X-TAXII-Content-Type";
pub const HTTP_X_TAXII_PROTOCOL: &str = "X-TAXII-Protocol";
pub const HTTP_X_TAXII_ACCEPT: &str = "X-TAXII-Accept";
pub const HTTP_X_TAXII_SERVICES: &str = "X-TAXII-Services";

// Custom TAXII header
pub const HTTP_X_TAXII_CONTENT_TYPES: &str = "X-TAXII-Content-Types";

// Content types
pub const HTTP_CONTENT_XML: &str = "application/xml";

/// Basic request headers (minimum required for parsing).
pub const BASIC_REQUEST_HEADERS: &[&str] = &[HTTP_CONTENT_TYPE, HTTP_X_TAXII_CONTENT_TYPE];

/// Required request headers (for full validation).
pub const REQUIRED_REQUEST_HEADERS: &[&str] = &[
    HTTP_CONTENT_TYPE,
    HTTP_X_TAXII_CONTENT_TYPE,
    HTTP_X_TAXII_SERVICES,
];

/// Required response headers.
pub const REQUIRED_RESPONSE_HEADERS: &[&str] = &[
    HTTP_CONTENT_TYPE,
    HTTP_X_TAXII_CONTENT_TYPE,
    HTTP_X_TAXII_PROTOCOL,
    HTTP_X_TAXII_SERVICES,
];

/// Supported message bindings.
pub const SUPPORTED_MESSAGE_BINDINGS: &[&str] = &[VID_TAXII_XML_10, VID_TAXII_XML_11];

/// Service bindings.
pub const SERVICE_BINDINGS: &[&str] = &[VID_TAXII_SERVICES_10, VID_TAXII_SERVICES_11];

/// Protocol bindings.
pub const PROTOCOL_BINDINGS: &[&str] = &[VID_TAXII_HTTP_10, VID_TAXII_HTTPS_10];

/// Static TAXII 1.1 HTTPS headers (lazily initialized).
static TAXII_11_HTTPS_HEADERS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        HashMap::from([
            (HTTP_CONTENT_TYPE, HTTP_CONTENT_XML),
            (HTTP_X_TAXII_CONTENT_TYPE, VID_TAXII_XML_11),
            (HTTP_X_TAXII_PROTOCOL, VID_TAXII_HTTPS_10),
            (HTTP_X_TAXII_SERVICES, VID_TAXII_SERVICES_11),
        ])
    });

/// Static TAXII 1.1 HTTP headers (lazily initialized).
static TAXII_11_HTTP_HEADERS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (HTTP_CONTENT_TYPE, HTTP_CONTENT_XML),
        (HTTP_X_TAXII_CONTENT_TYPE, VID_TAXII_XML_11),
        (HTTP_X_TAXII_PROTOCOL, VID_TAXII_HTTP_10),
        (HTTP_X_TAXII_SERVICES, VID_TAXII_SERVICES_11),
    ])
});

/// Static TAXII 1.0 HTTPS headers (lazily initialized).
static TAXII_10_HTTPS_HEADERS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        HashMap::from([
            (HTTP_CONTENT_TYPE, HTTP_CONTENT_XML),
            (HTTP_X_TAXII_CONTENT_TYPE, VID_TAXII_XML_10),
            (HTTP_X_TAXII_PROTOCOL, VID_TAXII_HTTPS_10),
            (HTTP_X_TAXII_SERVICES, VID_TAXII_SERVICES_10),
        ])
    });

/// Static TAXII 1.0 HTTP headers (lazily initialized).
static TAXII_10_HTTP_HEADERS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (HTTP_CONTENT_TYPE, HTTP_CONTENT_XML),
        (HTTP_X_TAXII_CONTENT_TYPE, VID_TAXII_XML_10),
        (HTTP_X_TAXII_PROTOCOL, VID_TAXII_HTTP_10),
        (HTTP_X_TAXII_SERVICES, VID_TAXII_SERVICES_10),
    ])
});

/// Get TAXII 1.1 HTTPS headers as static reference.
pub fn taxii_11_https_headers() -> &'static HashMap<&'static str, &'static str> {
    &TAXII_11_HTTPS_HEADERS
}

/// Get TAXII 1.1 HTTP headers as static reference.
pub fn taxii_11_http_headers() -> &'static HashMap<&'static str, &'static str> {
    &TAXII_11_HTTP_HEADERS
}

/// Get TAXII 1.0 HTTPS headers as static reference.
pub fn taxii_10_https_headers() -> &'static HashMap<&'static str, &'static str> {
    &TAXII_10_HTTPS_HEADERS
}

/// Get TAXII 1.0 HTTP headers as static reference.
pub fn taxii_10_http_headers() -> &'static HashMap<&'static str, &'static str> {
    &TAXII_10_HTTP_HEADERS
}

/// Get content type from headers.
pub fn get_content_type(headers: &HashMap<String, String>) -> Option<&String> {
    headers.get(HTTP_X_TAXII_CONTENT_TYPE)
}

/// Get HTTP headers for a version (returns static reference).
pub fn get_http_headers(
    version: &str,
    is_secure: bool,
) -> Taxii1xResult<&'static HashMap<&'static str, &'static str>> {
    let taxii_11 = [VID_TAXII_XML_11, VID_TAXII_SERVICES_11];
    if taxii_11.contains(&version) {
        return Ok(if is_secure {
            taxii_11_https_headers()
        } else {
            taxii_11_http_headers()
        });
    }

    let taxii_10 = [VID_TAXII_XML_10, VID_TAXII_SERVICES_10];
    if taxii_10.contains(&version) {
        return Ok(if is_secure {
            taxii_10_https_headers()
        } else {
            taxii_10_http_headers()
        });
    }

    Err(Taxii1xError::InvalidRequest(format!(
        "Unknown combination: version={version}, is_secure={is_secure}"
    )))
}

/// Validate request headers (post-parse).
pub fn validate_request_headers_post_parse(
    headers: &HashMap<String, String>,
    supported_message_bindings: &[&str],
    service_bindings: &[&str],
    protocol_bindings: &[&str],
) -> Taxii1xResult<()> {
    for h in REQUIRED_REQUEST_HEADERS {
        if !headers.contains_key(*h) {
            return Err(Taxii1xError::failure(
                format!("Header {h} was not specified"),
                None,
            ));
        }
    }

    let taxii_services = headers
        .get(HTTP_X_TAXII_SERVICES)
        .ok_or(Taxii1xError::MissingHeader(HTTP_X_TAXII_SERVICES))?;

    let taxii_protocol = headers.get(HTTP_X_TAXII_PROTOCOL);
    let taxii_accept = headers.get(HTTP_X_TAXII_ACCEPT);

    // Validate the X-TAXII-Services header
    if !service_bindings.contains(&taxii_services.as_str()) {
        return Err(Taxii1xError::failure(
            format!("The value of {HTTP_X_TAXII_SERVICES} was not recognized"),
            None,
        ));
    }

    // Validate the X-TAXII-Protocol header
    if let Some(protocol) = taxii_protocol {
        if !protocol_bindings.contains(&protocol.as_str()) {
            return Err(Taxii1xError::failure(
                "The specified value of X-TAXII-Protocol is not supported",
                None,
            ));
        }
    }

    // Validate the X-TAXII-Accept header
    if let Some(accept) = taxii_accept {
        if !supported_message_bindings.contains(&accept.as_str()) {
            return Err(Taxii1xError::failure(
                "The specified value of X-TAXII-Accept is not recognized",
                None,
            ));
        }
    }

    Ok(())
}

/// Validate basic request headers.
pub fn validate_request_headers(
    headers: &HashMap<String, String>,
    supported_message_bindings: &[&str],
) -> Taxii1xResult<()> {
    for h in BASIC_REQUEST_HEADERS {
        if !headers.contains_key(*h) {
            return Err(Taxii1xError::failure(
                format!("Header {h} was not specified"),
                None,
            ));
        }
    }

    let content_type = headers
        .get(HTTP_X_TAXII_CONTENT_TYPE)
        .ok_or(Taxii1xError::MissingHeader(HTTP_X_TAXII_CONTENT_TYPE))?;

    if !supported_message_bindings.contains(&content_type.as_str()) {
        return Err(Taxii1xError::failure(
            format!("TAXII Content Type \"{content_type}\" is not supported"),
            None,
        ));
    }

    let http_content_type = headers
        .get(HTTP_CONTENT_TYPE)
        .ok_or(Taxii1xError::MissingHeader(HTTP_CONTENT_TYPE))?;

    if !http_content_type.contains("application/xml") {
        return Err(Taxii1xError::failure(
            "The specified value of Content-Type is not supported",
            None,
        ));
    }

    Ok(())
}

/// Validate response headers.
pub fn validate_response_headers(headers: &HashMap<String, String>) -> Taxii1xResult<()> {
    for h in REQUIRED_RESPONSE_HEADERS {
        if !headers.contains_key(*h) {
            return Err(Taxii1xError::InvalidRequest(format!(
                "Required response header not specified: {h}"
            )));
        }
    }
    Ok(())
}

/// Generate a unique message ID.
pub fn generate_message_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
