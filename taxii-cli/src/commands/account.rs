//! Account management commands.

use clap::Subcommand;
use taxii_auth::AuthAPI;
use taxii_db::TaxiiPool;

/// Account management actions.
#[derive(Subcommand)]
pub enum AccountAction {
    /// List all accounts with their permissions.
    List,

    /// Delete an account.
    Delete {
        /// Username of the account to delete.
        #[arg(short, long)]
        username: String,
    },
}

/// Handle account commands.
pub async fn handle(
    pool: TaxiiPool,
    auth_secret: &str,
    action: AccountAction,
) -> Result<(), Box<dyn std::error::Error>> {
    let auth = AuthAPI::new(pool, auth_secret.to_string(), None)?;

    match action {
        AccountAction::List => {
            list_accounts(&auth).await?;
        }
        AccountAction::Delete { username } => {
            delete_account(&auth, &username).await?;
        }
    }

    Ok(())
}

/// Delete an account.
async fn delete_account(auth: &AuthAPI, username: &str) -> Result<(), Box<dyn std::error::Error>> {
    auth.delete_account(username).await?;
    println!("Account '{username}' deleted successfully");
    Ok(())
}

/// List all accounts with permissions.
async fn list_accounts(auth: &AuthAPI) -> Result<(), Box<dyn std::error::Error>> {
    let accounts = auth.get_accounts().await?;

    if accounts.is_empty() {
        println!("No accounts found.");
        return Ok(());
    }

    println!("{:<5} {:<20} {:<8} Permissions", "ID", "Username", "Admin");
    println!("{}", "-".repeat(70));

    for account in accounts {
        let permissions_str = if account.permissions.is_empty() {
            "-".to_string()
        } else {
            account
                .permissions
                .iter()
                .map(|(col, perm)| {
                    let perm_str = match perm {
                        taxii_core::PermissionValue::Taxii1(s) => s.clone(),
                        taxii_core::PermissionValue::Taxii2(list) => list.join("+"),
                    };
                    format!("{col}:{perm_str}")
                })
                .collect::<Vec<_>>()
                .join(", ")
        };

        println!(
            "{:<5} {:<20} {:<8} {}",
            account.id,
            account.username,
            if account.is_admin { "Yes" } else { "No" },
            permissions_str
        );
    }

    Ok(())
}
