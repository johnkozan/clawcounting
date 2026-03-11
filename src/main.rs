mod app_state;
mod cli;
mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod router;
mod services;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::app_state::AppState;
use crate::cli::{Cli, Commands};
use crate::config::Config;
use crate::db::connection::setup_connection;
use crate::db::migrations::run_migrations;
use crate::db::pool::DbPools;
use crate::router::build_router;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();
    let config = Config::from_env(cli.db_path.as_deref());

    // Bootstrap: open connection, set pragmas, register functions, run migrations
    let mut bootstrap_conn =
        setup_connection(&config.db_path).expect("Failed to open database");
    run_migrations(&mut bootstrap_conn).expect("Failed to run migrations");

    match cli.command {
        Commands::Server => {
            // Validate JWT secret is present for server mode
            config.require_jwt_secret();

            // Close bootstrap connection, create pools
            drop(bootstrap_conn);
            let pools = DbPools::new(&config.db_path, 4).expect("Failed to create connection pools");

            let state = AppState {
                pools,
                config: config.clone(),
            };
            let app = build_router(state);

            let addr = format!("0.0.0.0:{}", config.port);
            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .expect("Failed to bind");
            tracing::info!("Listening on {addr}");
            axum::serve(listener, app)
                .await
                .expect("Server error");
        }

        Commands::Accounts { command: _ }
        | Commands::JournalEntries { command: _ }
        | Commands::Periods { command: _ }
        | Commands::Currencies { command: _ }
        | Commands::VerifyBalances
        | Commands::Backup
        | Commands::Restore { .. } => {
            println!("not implemented");
        }
    }
}
