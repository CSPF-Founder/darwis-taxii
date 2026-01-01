//! Account activity commands for credential usage tracking.

use chrono::{DateTime, Local, Utc};
use clap::Subcommand;
use taxii_db::{AccountActivity, TaxiiPool};

/// Activity management actions.
#[derive(Subcommand)]
pub enum ActivityAction {
    /// Show credential usage summary for all accounts.
    Usage {
        /// Show only accounts inactive for specified days.
        #[arg(long)]
        inactive_days: Option<i32>,

        /// Show only accounts that have never logged in.
        #[arg(long, default_value = "false")]
        unused: bool,
    },

    /// Clean up old activity records.
    Cleanup {
        /// Number of days to retain (default: 30).
        #[arg(long, default_value = "30")]
        retention_days: i32,

        /// Actually delete records (without this flag, only shows what would be deleted).
        #[arg(long, default_value = "false")]
        confirm: bool,
    },
}

/// Handle activity commands.
pub async fn handle(
    pool: TaxiiPool,
    action: ActivityAction,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        ActivityAction::Usage {
            inactive_days,
            unused,
        } => {
            if unused {
                show_unused_accounts(&pool).await?;
            } else if let Some(days) = inactive_days {
                show_inactive_accounts(&pool, days).await?;
            } else {
                show_usage_summary(&pool).await?;
            }
        }
        ActivityAction::Cleanup {
            retention_days,
            confirm,
        } => {
            cleanup_old_records(&pool, retention_days, confirm).await?;
        }
    }

    Ok(())
}

/// Format datetime for display in local timezone.
fn format_datetime(dt: Option<DateTime<Utc>>) -> String {
    match dt {
        Some(dt) => {
            let local: DateTime<Local> = dt.into();
            local.format("%Y-%m-%d %H:%M").to_string()
        }
        None => "Never".to_string(),
    }
}

/// Show credential usage summary for all accounts.
async fn show_usage_summary(pool: &TaxiiPool) -> Result<(), Box<dyn std::error::Error>> {
    let summaries = AccountActivity::get_usage_summary(pool).await?;

    if summaries.is_empty() {
        println!("No accounts found.");
        return Ok(());
    }

    println!(
        "{:<5} {:<20} {:<8} {:<18} {:<16} {:<8} {:<8}",
        "ID", "Username", "Admin", "Last Login", "Last IP", "Logins", "Failed"
    );
    println!("{}", "-".repeat(95));

    for summary in summaries {
        let last_login = format_datetime(summary.last_login);
        let last_ip = summary.last_ip.unwrap_or_else(|| "-".to_string());

        println!(
            "{:<5} {:<20} {:<8} {:<18} {:<16} {:<8} {:<8}",
            summary.account_id,
            truncate(&summary.username, 20),
            if summary.is_admin { "Yes" } else { "No" },
            last_login,
            truncate(&last_ip, 16),
            summary.login_count,
            summary.failed_count
        );
    }

    Ok(())
}

/// Show accounts that have never logged in.
async fn show_unused_accounts(pool: &TaxiiPool) -> Result<(), Box<dyn std::error::Error>> {
    let unused = AccountActivity::get_unused_accounts(pool).await?;

    if unused.is_empty() {
        println!("All accounts have been used at least once.");
        return Ok(());
    }

    println!("Accounts that have never logged in:");
    println!("{:<5} {:<30}", "ID", "Username");
    println!("{}", "-".repeat(40));

    let count = unused.len();
    for (id, username) in unused {
        println!("{id:<5} {username:<30}");
    }

    println!("\nTotal: {count} unused account(s)");

    Ok(())
}

/// Show accounts inactive for specified number of days.
async fn show_inactive_accounts(
    pool: &TaxiiPool,
    days: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let summaries = AccountActivity::get_inactive_accounts(pool, days).await?;

    if summaries.is_empty() {
        println!("All accounts have been active in the last {days} days.");
        return Ok(());
    }

    println!("Accounts inactive for {days}+ days:");
    println!(
        "{:<5} {:<20} {:<8} {:<18} {:<16}",
        "ID", "Username", "Admin", "Last Login", "Last IP"
    );
    println!("{}", "-".repeat(75));

    for summary in &summaries {
        let last_login = format_datetime(summary.last_login);
        let last_ip = summary.last_ip.clone().unwrap_or_else(|| "-".to_string());

        println!(
            "{:<5} {:<20} {:<8} {:<18} {:<16}",
            summary.account_id,
            truncate(&summary.username, 20),
            if summary.is_admin { "Yes" } else { "No" },
            last_login,
            truncate(&last_ip, 16)
        );
    }

    println!("\nTotal: {} inactive account(s)", summaries.len());

    Ok(())
}

/// Clean up old activity records.
async fn cleanup_old_records(
    pool: &TaxiiPool,
    retention_days: i32,
    confirm: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !confirm {
        // Dry run - just show what would be deleted
        println!("Would delete activity records older than {retention_days} days.");
        println!("Run with --confirm to actually delete records.");
        return Ok(());
    }

    let deleted = AccountActivity::cleanup_old_records(pool, retention_days).await?;

    if deleted > 0 {
        println!("Deleted {deleted} activity record(s) older than {retention_days} days.");
    } else {
        println!("No records older than {retention_days} days found.");
    }

    Ok(())
}

/// Truncate string to specified length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
