//! Database migration commands.

use clap::Subcommand;
use std::collections::HashSet;
use taxii_db::TaxiiPool;

#[derive(Subcommand)]
pub enum MigrateAction {
    /// Run all pending migrations.
    Run,
    /// Show migration status.
    Status,
    /// List all available migrations.
    Info,
}

pub async fn handle(
    pool: TaxiiPool,
    action: MigrateAction,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        MigrateAction::Run => run_migrations(pool).await,
        MigrateAction::Status => show_status(pool).await,
        MigrateAction::Info => show_info(),
    }
}

async fn run_migrations(pool: TaxiiPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running migrations...");

    match taxii_db::migrations::run(pool.inner()).await {
        Ok(()) => {
            println!("Migrations completed successfully.");
            Ok(())
        }
        Err(e) => {
            eprintln!("Migration failed: {}", e);
            Err(e.into())
        }
    }
}

async fn show_status(pool: TaxiiPool) -> Result<(), Box<dyn std::error::Error>> {
    let all_migrations = taxii_db::migrations::list();
    let applied = taxii_db::migrations::applied(pool.inner())
        .await?
        .into_iter()
        .collect::<HashSet<_>>();

    println!("Migration Status:");
    println!("{:-<60}", "");

    let mut pending_count = 0;
    for migration in &all_migrations {
        let status = if applied.contains(&migration.version) {
            "applied"
        } else {
            pending_count += 1;
            "pending"
        };
        println!(
            "  {} {} [{}]",
            migration.version, migration.description, status
        );
    }

    println!("{:-<60}", "");
    if pending_count == 0 {
        println!("All migrations are up to date.");
    } else {
        println!(
            "{} pending migration(s). Run 'taxii-cli migrate run' to apply.",
            pending_count
        );
    }

    Ok(())
}

fn show_info() -> Result<(), Box<dyn std::error::Error>> {
    let migrations = taxii_db::migrations::list();

    println!("Available Migrations ({} total):", migrations.len());
    println!("{:-<60}", "");

    for migration in migrations {
        println!("  {} {}", migration.version, migration.description);
    }

    Ok(())
}
