//! Authentication middleware.

use std::sync::Arc;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::{StatusCode, header::AUTHORIZATION};
use axum::response::{IntoResponse, Response};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use futures::future::BoxFuture;
use tower::{Layer, Service};
use tracing::{error, warn};

use taxii_auth::AuthAPI;
use taxii_core::Account;

/// Authentication error that results in 401 response.
#[derive(Debug)]
struct AuthError {
    message: &'static str,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, self.message).into_response()
    }
}

/// Auth layer.
#[derive(Clone)]
pub struct AuthLayer {
    auth: Arc<AuthAPI>,
    support_basic_auth: bool,
}

impl AuthLayer {
    /// Create a new auth layer.
    pub fn new(auth: Arc<AuthAPI>, support_basic_auth: bool) -> Self {
        Self {
            auth,
            support_basic_auth,
        }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            auth: self.auth.clone(),
            support_basic_auth: self.support_basic_auth,
        }
    }
}

/// Auth middleware service.
#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    auth: Arc<AuthAPI>,
    support_basic_auth: bool,
}

impl<S> Service<Request> for AuthMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let auth = self.auth.clone();
        let support_basic_auth = self.support_basic_auth;
        let mut inner = self.inner.clone();

        // Extract auth info from headers before async
        let extract_result = extract_auth_info(&req, support_basic_auth);

        Box::pin(async move {
            match extract_result {
                ExtractResult::NoHeader | ExtractResult::Invalid => {
                    // No auth or invalid format - continue without auth (let handler decide)
                    inner.call(req).await
                }
                ExtractResult::Unauthorized(msg) => {
                    // Return 401 immediately
                    Ok(AuthError { message: msg }.into_response())
                }
                ExtractResult::Success(auth_type, token_or_creds) => {
                    // Try to authenticate
                    match authenticate(&auth, auth_type, token_or_creds).await {
                        AuthResult::Success(account) => {
                            req.extensions_mut().insert(account);
                            inner.call(req).await
                        }
                        AuthResult::Unauthorized(msg) => {
                            Ok(AuthError { message: msg }.into_response())
                        }
                    }
                }
            }
        })
    }
}

/// Auth type and token/credentials.
enum AuthInfo {
    Bearer(String),
    Basic(String, String),
}

/// Result of extracting auth info.
enum ExtractResult {
    /// No auth header present - continue without auth.
    NoHeader,
    /// Successfully extracted auth info.
    Success(String, AuthInfo),
    /// Should return 401 immediately.
    Unauthorized(&'static str),
    /// Invalid header but should continue.
    Invalid,
}

/// Extract auth info from request headers.
fn extract_auth_info(req: &Request, support_basic_auth: bool) -> ExtractResult {
    let auth_header = match req.headers().get(AUTHORIZATION) {
        Some(h) => match h.to_str() {
            Ok(s) => s,
            Err(_) => return ExtractResult::Invalid,
        },
        None => return ExtractResult::NoHeader,
    };

    let parts: Vec<&str> = auth_header.splitn(2, ' ').collect();
    if parts.len() != 2 {
        warn!(value = auth_header, "auth.header_invalid");
        return ExtractResult::Invalid;
    }

    let auth_type = parts[0].to_lowercase();
    let raw_token = parts[1];

    if auth_type == "basic" {
        if !support_basic_auth {
            return ExtractResult::Unauthorized("Basic auth not supported");
        }

        match parse_basic_auth_token(raw_token) {
            Some((username, password)) => {
                ExtractResult::Success(auth_type, AuthInfo::Basic(username, password))
            }
            None => {
                error!(raw_token = raw_token, "auth.basic_auth.header_invalid");
                ExtractResult::Invalid
            }
        }
    } else if auth_type == "bearer" {
        ExtractResult::Success(auth_type, AuthInfo::Bearer(raw_token.to_string()))
    } else {
        ExtractResult::Unauthorized("Unknown auth type")
    }
}

/// Result of authentication attempt.
enum AuthResult {
    Success(Account),
    Unauthorized(&'static str),
}

/// Authenticate from extracted auth info.
async fn authenticate(auth: &AuthAPI, _auth_type: String, info: AuthInfo) -> AuthResult {
    let token = match info {
        AuthInfo::Basic(username, password) => {
            match auth.authenticate(&username, &password).await {
                Ok(Some(token)) => token,
                Ok(None) => return AuthResult::Unauthorized("Authentication failed"),
                Err(_) => return AuthResult::Unauthorized("Authentication error"),
            }
        }
        AuthInfo::Bearer(token) => token,
    };

    match auth.get_account(&token).await {
        Ok(Some(account)) => AuthResult::Success(account),
        Ok(None) => AuthResult::Unauthorized("Invalid token"),
        Err(_) => AuthResult::Unauthorized("Token validation error"),
    }
}

/// Parse basic auth token.
fn parse_basic_auth_token(token: &str) -> Option<(String, String)> {
    let decoded = BASE64.decode(token).ok()?;
    let decoded_str = String::from_utf8(decoded).ok()?;

    let parts: Vec<&str> = decoded_str.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }

    Some((parts[0].to_string(), parts[1].to_string()))
}
