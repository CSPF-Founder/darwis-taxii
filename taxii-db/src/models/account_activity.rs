//! Account activity model for tracking credential usage.

use std::net::IpAddr;

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::error::DatabaseResult;
use crate::pool::TaxiiPool;

/// Event types for account activity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// Successful login.
    LoginSuccess,
    /// Failed login attempt.
    LoginFailed,
}

impl EventType {
    /// Convert to database string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LoginSuccess => "login_success",
            Self::LoginFailed => "login_failed",
        }
    }
}

/// Account activity database row.
///
/// Table: account_activity
#[derive(Debug, Clone, FromRow)]
pub struct AccountActivity {
    /// Primary key.
    pub id: i64,

    /// Account ID (foreign key to accounts).
    pub account_id: i32,

    /// Event type (login_success, login_failed).
    pub event_type: String,

    /// Client IP address (optional).
    #[sqlx(try_from = "String")]
    pub ip_address: Option<String>,

    /// Client user agent (optional).
    pub user_agent: Option<String>,

    /// Timestamp of the event.
    pub created_at: DateTime<Utc>,
}

/// Account usage summary for reporting.
#[derive(Debug, Clone)]
pub struct AccountUsageSummary {
    /// Account ID.
    pub account_id: i32,
    /// Username.
    pub username: String,
    /// Whether account is admin.
    pub is_admin: bool,
    /// Last successful login timestamp.
    pub last_login: Option<DateTime<Utc>>,
    /// Last login IP address.
    pub last_ip: Option<String>,
    /// Total successful logins in period.
    pub login_count: i64,
    /// Total failed login attempts in period.
    pub failed_count: i64,
}

impl AccountActivity {
    /// Log an account activity event (fire-and-forget safe).
    ///
    /// This method is designed to be called from a spawned task
    /// and will not fail the caller if logging fails.
    pub async fn log(
        pool: &TaxiiPool,
        account_id: i32,
        event_type: EventType,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> DatabaseResult<()> {
        let ip_str = ip_address.map(|ip| ip.to_string());

        // Use text cast to avoid requiring ipnetwork feature
        sqlx::query!(
            r#"INSERT INTO account_activity (account_id, event_type, ip_address, user_agent)
               VALUES ($1, $2, $3::text::inet, $4)"#,
            account_id,
            event_type.as_str(),
            ip_str,
            user_agent,
        )
        .execute(pool.inner())
        .await?;

        Ok(())
    }

    /// Log a failed login attempt by username (account may not exist).
    ///
    /// Returns Ok(false) if the username doesn't exist.
    pub async fn log_failed_by_username(
        pool: &TaxiiPool,
        username: &str,
        ip_address: Option<IpAddr>,
        user_agent: Option<&str>,
    ) -> DatabaseResult<bool> {
        let ip_str = ip_address.map(|ip| ip.to_string());

        // Use text cast to avoid requiring ipnetwork feature
        let result = sqlx::query!(
            r#"INSERT INTO account_activity (account_id, event_type, ip_address, user_agent)
               SELECT id, 'login_failed', $2::text::inet, $3
               FROM accounts WHERE username = $1"#,
            username,
            ip_str,
            user_agent,
        )
        .execute(pool.inner())
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get account usage summary for all accounts.
    ///
    /// Returns usage statistics including last login and counts.
    pub async fn get_usage_summary(pool: &TaxiiPool) -> DatabaseResult<Vec<AccountUsageSummary>> {
        // Use a single query with subqueries for efficiency
        let rows = sqlx::query!(
            r#"
            SELECT
                a.id as "account_id!",
                a.username as "username!",
                a.is_admin as "is_admin!",
                (
                    SELECT created_at
                    FROM account_activity
                    WHERE account_id = a.id AND event_type = 'login_success'
                    ORDER BY created_at DESC
                    LIMIT 1
                ) as last_login,
                (
                    SELECT ip_address::text
                    FROM account_activity
                    WHERE account_id = a.id AND event_type = 'login_success'
                    ORDER BY created_at DESC
                    LIMIT 1
                ) as last_ip,
                COALESCE((
                    SELECT COUNT(*)
                    FROM account_activity
                    WHERE account_id = a.id AND event_type = 'login_success'
                ), 0) as "login_count!",
                COALESCE((
                    SELECT COUNT(*)
                    FROM account_activity
                    WHERE account_id = a.id AND event_type = 'login_failed'
                ), 0) as "failed_count!"
            FROM accounts a
            ORDER BY a.username
            "#
        )
        .fetch_all(pool.inner())
        .await?;

        let summaries = rows
            .into_iter()
            .map(|row| AccountUsageSummary {
                account_id: row.account_id,
                username: row.username,
                is_admin: row.is_admin,
                last_login: row.last_login,
                last_ip: row.last_ip,
                login_count: row.login_count,
                failed_count: row.failed_count,
            })
            .collect();

        Ok(summaries)
    }

    /// Get accounts that have never logged in.
    pub async fn get_unused_accounts(pool: &TaxiiPool) -> DatabaseResult<Vec<(i32, String)>> {
        let rows = sqlx::query!(
            r#"
            SELECT a.id, a.username
            FROM accounts a
            WHERE NOT EXISTS (
                SELECT 1 FROM account_activity
                WHERE account_id = a.id AND event_type = 'login_success'
            )
            ORDER BY a.username
            "#
        )
        .fetch_all(pool.inner())
        .await?;

        Ok(rows.into_iter().map(|r| (r.id, r.username)).collect())
    }

    /// Get accounts inactive for specified number of days.
    pub async fn get_inactive_accounts(
        pool: &TaxiiPool,
        days: i32,
    ) -> DatabaseResult<Vec<AccountUsageSummary>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                a.id as "account_id!",
                a.username as "username!",
                a.is_admin as "is_admin!",
                (
                    SELECT created_at
                    FROM account_activity
                    WHERE account_id = a.id AND event_type = 'login_success'
                    ORDER BY created_at DESC
                    LIMIT 1
                ) as last_login,
                (
                    SELECT ip_address::text
                    FROM account_activity
                    WHERE account_id = a.id AND event_type = 'login_success'
                    ORDER BY created_at DESC
                    LIMIT 1
                ) as last_ip,
                COALESCE((
                    SELECT COUNT(*)
                    FROM account_activity
                    WHERE account_id = a.id AND event_type = 'login_success'
                ), 0) as "login_count!",
                COALESCE((
                    SELECT COUNT(*)
                    FROM account_activity
                    WHERE account_id = a.id AND event_type = 'login_failed'
                ), 0) as "failed_count!"
            FROM accounts a
            WHERE NOT EXISTS (
                SELECT 1 FROM account_activity
                WHERE account_id = a.id
                  AND event_type = 'login_success'
                  AND created_at > NOW() - ($1 || ' days')::interval
            )
            ORDER BY a.username
            "#,
            days.to_string()
        )
        .fetch_all(pool.inner())
        .await?;

        let summaries = rows
            .into_iter()
            .map(|row| AccountUsageSummary {
                account_id: row.account_id,
                username: row.username,
                is_admin: row.is_admin,
                last_login: row.last_login,
                last_ip: row.last_ip,
                login_count: row.login_count,
                failed_count: row.failed_count,
            })
            .collect();

        Ok(summaries)
    }

    /// Delete activity records older than specified number of days.
    ///
    /// Returns the number of records deleted.
    pub async fn cleanup_old_records(pool: &TaxiiPool, retention_days: i32) -> DatabaseResult<u64> {
        let result = sqlx::query!(
            r#"DELETE FROM account_activity
               WHERE created_at < NOW() - ($1 || ' days')::interval"#,
            retention_days.to_string()
        )
        .execute(pool.inner())
        .await?;

        Ok(result.rows_affected())
    }
}
