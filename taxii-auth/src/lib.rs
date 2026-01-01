//! Authentication for DARWIS TAXII.
//!
//! This crate handles JWT token generation/validation and password hashing.
//! Database operations are delegated to taxii-db.

pub mod error;
pub mod password;

use std::collections::HashMap;
use std::net::IpAddr;

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use tracing::warn;

pub use error::{AuthError, AuthResult};

use taxii_core::Account as AccountEntity;
use taxii_db::{Account, AccountActivity, EventType, TaxiiPool, validate_permissions};

/// Client information for activity logging.
#[derive(Debug, Clone, Default)]
pub struct ClientInfo {
    /// Client IP address.
    pub ip_address: Option<IpAddr>,
    /// Client user agent string.
    pub user_agent: Option<String>,
}

impl ClientInfo {
    /// Create new client info.
    #[must_use]
    pub fn new(ip_address: Option<IpAddr>, user_agent: Option<String>) -> Self {
        Self {
            ip_address,
            user_agent,
        }
    }
}

/// Convert Account (database model) to AccountEntity (domain entity).
fn account_to_entity(account: &Account) -> AccountEntity {
    AccountEntity {
        id: account.id,
        username: account.username.clone(),
        is_admin: account.is_admin,
        permissions: account.permissions(),
        details: HashMap::new(),
    }
}

/// JWT token claims.
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    account_id: i32,
    exp: i64,
}

/// SQL Database Auth API.
///
/// This struct handles authentication logic (JWT, password verification).
/// Database operations are delegated to taxii-db.
pub struct AuthAPI {
    pool: TaxiiPool,
    secret: String,
    /// Token TTL in seconds.
    token_ttl_secs: i64,
}

/// Default token TTL: 1 hour in seconds.
pub const DEFAULT_TOKEN_TTL_SECS: i64 = 60 * 60;

impl AuthAPI {
    /// Create a new auth API.
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `secret` - JWT signing secret (must not be empty)
    /// * `token_ttl_secs` - Token time-to-live in seconds (defaults to 1 hour)
    pub fn new(pool: TaxiiPool, secret: String, token_ttl_secs: Option<i64>) -> AuthResult<Self> {
        if secret.is_empty() {
            return Err(AuthError::Config("Secret is not defined".to_string()));
        }

        Ok(Self {
            pool,
            secret,
            token_ttl_secs: token_ttl_secs.unwrap_or(DEFAULT_TOKEN_TTL_SECS),
        })
    }

    /// Get pool reference.
    #[must_use]
    pub fn pool(&self) -> &TaxiiPool {
        &self.pool
    }

    /// Authenticate user and return JWT token.
    ///
    /// This is a simple version without activity logging.
    /// Use `authenticate_with_logging` when client info is available.
    pub async fn authenticate(&self, username: &str, password: &str) -> AuthResult<Option<String>> {
        self.authenticate_internal(username, password, None).await
    }

    /// Authenticate user with activity logging.
    ///
    /// Logs successful and failed login attempts in the background
    /// without blocking the authentication response.
    pub async fn authenticate_with_logging(
        &self,
        username: &str,
        password: &str,
        client_info: ClientInfo,
    ) -> AuthResult<Option<String>> {
        self.authenticate_internal(username, password, Some(client_info))
            .await
    }

    /// Internal authentication logic.
    async fn authenticate_internal(
        &self,
        username: &str,
        password: &str,
        client_info: Option<ClientInfo>,
    ) -> AuthResult<Option<String>> {
        let account = Account::find_by_username(&self.pool, username).await?;

        let account = match account {
            Some(a) => a,
            None => {
                // Log failed attempt for unknown username (fire-and-forget)
                if let Some(info) = client_info {
                    let pool = self.pool.clone();
                    let username = username.to_string();
                    tokio::spawn(async move {
                        let _ = AccountActivity::log_failed_by_username(
                            &pool,
                            &username,
                            info.ip_address,
                            info.user_agent.as_deref(),
                        )
                        .await;
                    });
                }
                return Ok(None);
            }
        };

        if !password::check_password_hash(&account.password_hash, password) {
            // Log failed login attempt (fire-and-forget)
            if let Some(info) = client_info {
                let pool = self.pool.clone();
                let account_id = account.id;
                tokio::spawn(async move {
                    let _ = AccountActivity::log(
                        &pool,
                        account_id,
                        EventType::LoginFailed,
                        info.ip_address,
                        info.user_agent.as_deref(),
                    )
                    .await;
                });
            }
            return Ok(None);
        }

        // Log successful login (fire-and-forget)
        if let Some(info) = client_info {
            let pool = self.pool.clone();
            let account_id = account.id;
            tokio::spawn(async move {
                let _ = AccountActivity::log(
                    &pool,
                    account_id,
                    EventType::LoginSuccess,
                    info.ip_address,
                    info.user_agent.as_deref(),
                )
                .await;
            });
        }

        let token = self.generate_token(account.id, Some(self.token_ttl_secs))?;
        Ok(Some(token))
    }

    /// Create a new account.
    pub async fn create_account(
        &self,
        username: &str,
        password: &str,
        is_admin: bool,
    ) -> AuthResult<AccountEntity> {
        let password_hash = password::generate_password_hash(password);
        let account = Account::create(&self.pool, username, &password_hash, is_admin).await?;
        Ok(account_to_entity(&account))
    }

    /// Get account from token.
    pub async fn get_account(&self, token: &str) -> AuthResult<Option<AccountEntity>> {
        let account_id = match self.get_account_id(token) {
            Some(id) => id,
            None => return Ok(None),
        };

        let account = Account::find(&self.pool, account_id).await?;
        Ok(account.as_ref().map(account_to_entity))
    }

    /// Delete an account.
    pub async fn delete_account(&self, username: &str) -> AuthResult<()> {
        Account::delete_by_username(&self.pool, username).await?;
        Ok(())
    }

    /// Get all accounts.
    pub async fn get_accounts(&self) -> AuthResult<Vec<AccountEntity>> {
        let accounts = Account::find_all(&self.pool).await?;
        Ok(accounts.iter().map(account_to_entity).collect())
    }

    /// Update an account.
    pub async fn update_account(
        &self,
        account_entity: &AccountEntity,
        password: Option<&str>,
    ) -> AuthResult<AccountEntity> {
        // Validate permissions
        validate_permissions(&account_entity.permissions).map_err(AuthError::InvalidPermission)?;

        let permissions_json = serde_json::to_string(&account_entity.permissions)?;

        // Check if exists
        let existing = Account::find_by_username(&self.pool, &account_entity.username).await?;
        let is_new = existing.is_none();

        let updated = if let Some(existing) = existing {
            // Update existing
            if let Some(pw) = password {
                let password_hash = password::generate_password_hash(pw);
                Account::update_with_password(
                    &self.pool,
                    existing.id,
                    &password_hash,
                    account_entity.is_admin,
                    &permissions_json,
                )
                .await?
            } else {
                Account::update(
                    &self.pool,
                    existing.id,
                    account_entity.is_admin,
                    &permissions_json,
                )
                .await?
            }
        } else {
            // Create new
            let password_hash = password
                .map(password::generate_password_hash)
                .unwrap_or_default();

            Account::create(
                &self.pool,
                &account_entity.username,
                &password_hash,
                account_entity.is_admin,
            )
            .await?
        };

        // If we just created, we need to update permissions since create uses empty {}
        if is_new && !account_entity.permissions.is_empty() {
            let updated = Account::update(
                &self.pool,
                updated.id,
                account_entity.is_admin,
                &permissions_json,
            )
            .await?;
            return Ok(account_to_entity(&updated));
        }

        Ok(account_to_entity(&updated))
    }

    /// Generate JWT token.
    fn generate_token(&self, account_id: i32, ttl_secs: Option<i64>) -> AuthResult<String> {
        let ttl_secs = ttl_secs.unwrap_or(self.token_ttl_secs);
        let exp = Utc::now() + Duration::seconds(ttl_secs);

        let claims = Claims {
            account_id,
            exp: exp.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?;

        Ok(token)
    }

    /// Get account ID from token.
    fn get_account_id(&self, token: &str) -> Option<i32> {
        let validation = Validation::default();

        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        ) {
            Ok(data) => Some(data.claims.account_id),
            Err(e) => {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        warn!(token = token, "Invalid token used");
                    }
                    _ => {
                        warn!(token = token, "Can not decode a token");
                    }
                }
                None
            }
        }
    }
}
