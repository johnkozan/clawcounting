use clap::Parser;
use tracing_subscriber::EnvFilter;

use clawcounting::app_state::AppState;
use clawcounting::cli::{
    AccountsCommands, Cli, Commands, CurrenciesCommands, JournalEntriesCommands, PeriodsCommands,
    SettingsCommands,
};
use clawcounting::config::Config;
use clawcounting::db::connection::setup_connection;
use clawcounting::db::migrations::run_migrations;
use clawcounting::db::pool::DbPools;
use clawcounting::router::build_router;
use clawcounting::services::{account_service, settings_service};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let default_level = match cli.command {
        Commands::Server => "info",
        _ => "warn",
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| default_level.into()),
        )
        .init();
    let config = Config::from_env(cli.db_path.as_deref());

    // Bootstrap: open connection, set pragmas, register functions, run migrations
    let mut bootstrap_conn =
        setup_connection(&config.db_path).expect("Failed to open database");
    run_migrations(&mut bootstrap_conn).expect("Failed to run migrations");

    // Ensure system user exists
    let system_user_id = settings_service::ensure_system_user(&bootstrap_conn)
        .expect("Failed to create system user");

    match cli.command {
        Commands::Server => {
            // Validate JWT secret is present for server mode
            config.require_jwt_secret();

            // Close bootstrap connection, create pools
            drop(bootstrap_conn);
            let pools =
                DbPools::new(&config.db_path, 4).expect("Failed to create connection pools");

            let state = AppState {
                pools,
                config: config.clone(),
                system_user_id,
            };
            let app = build_router(state);

            let addr = format!("0.0.0.0:{}", config.port);
            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .expect("Failed to bind");
            tracing::info!("Listening on {addr}");
            axum::serve(listener, app).await.expect("Server error");
        }

        Commands::Currencies { command } => {
            let result = match command {
                CurrenciesCommands::List { json } => {
                    clawcounting::cli::currencies::list(&bootstrap_conn, json)
                }
                CurrenciesCommands::Create {
                    code,
                    name,
                    symbol,
                    asset_scale,
                    asset_type,
                    caip19,
                    json,
                } => clawcounting::cli::currencies::create(
                    &bootstrap_conn,
                    &code,
                    &name,
                    &symbol,
                    asset_scale,
                    &asset_type,
                    &caip19,
                    json,
                ),
                CurrenciesCommands::CreateFiat { code, json } => {
                    clawcounting::cli::currencies::create_fiat(&bootstrap_conn, &code, json)
                }
                CurrenciesCommands::Get { id, json } => {
                    clawcounting::cli::currencies::get(&bootstrap_conn, &id, json)
                }
            };
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }

        Commands::Accounts { command } => {
            let result = match command {
                AccountsCommands::List {
                    account_type,
                    currency,
                    json,
                } => clawcounting::cli::accounts::list(
                    &bootstrap_conn,
                    account_type.as_deref(),
                    currency.as_deref(),
                    json,
                ),
                AccountsCommands::Create {
                    name,
                    currency,
                    account_type,
                    normal_balance,
                    number,
                    subledger,
                    parent,
                    entity,
                    json,
                } => clawcounting::cli::accounts::create(
                    &bootstrap_conn,
                    &name,
                    &currency,
                    &account_type,
                    &normal_balance,
                    &number,
                    subledger,
                    parent.as_deref(),
                    entity.as_deref(),
                    json,
                ),
                AccountsCommands::Get { id, json } => {
                    clawcounting::cli::accounts::get(&bootstrap_conn, &id, json)
                }
            };
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }

        Commands::Periods { command } => {
            let result = match command {
                PeriodsCommands::List { json } => {
                    clawcounting::cli::periods::list(&bootstrap_conn, json)
                }
                PeriodsCommands::Create {
                    name,
                    start,
                    end,
                    json,
                } => clawcounting::cli::periods::create(&bootstrap_conn, &name, &start, &end, json),
                PeriodsCommands::Close { id, preview, json } => {
                    clawcounting::cli::periods::close(
                        &mut bootstrap_conn,
                        &id,
                        &system_user_id,
                        preview,
                        json,
                    )
                }
            };
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }

        Commands::JournalEntries { command } => {
            let result = match command {
                JournalEntriesCommands::List { period, json } => {
                    clawcounting::cli::journal_entries::list(&bootstrap_conn, period.as_deref(), json)
                }
                JournalEntriesCommands::Create { file, json } => {
                    clawcounting::cli::journal_entries::create_from_file(
                        &mut bootstrap_conn,
                        &file,
                        &system_user_id,
                        json,
                    )
                }
                JournalEntriesCommands::Get { id, json } => {
                    clawcounting::cli::journal_entries::get(&bootstrap_conn, &id, json)
                }
                JournalEntriesCommands::Reverse { id, date, json } => {
                    clawcounting::cli::journal_entries::reverse(
                        &mut bootstrap_conn,
                        &id,
                        &system_user_id,
                        date.as_deref(),
                        json,
                    )
                }
            };
            if let Err(e) = result {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }

        Commands::Settings { command } => {
            let result = match command {
                SettingsCommands::Set { key, value } => match key.as_str() {
                    "retained-earnings-account" | "retained_earnings_account_id" => {
                        let acct = account_service::get_account(&bootstrap_conn, &value);
                        match acct {
                            Ok(a) => {
                                if a.account_type != "equity" {
                                    eprintln!(
                                        "Error: Account must be equity type, got '{}'",
                                        a.account_type
                                    );
                                    std::process::exit(1);
                                }
                                settings_service::set_setting(
                                    &bootstrap_conn,
                                    "retained_earnings_account_id",
                                    &value,
                                )
                            }
                            Err(e) => Err(e),
                        }
                    }
                    _ => settings_service::set_setting(&bootstrap_conn, &key, &value),
                },
            };
            match result {
                Ok(()) => println!("Setting updated."),
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::VerifyBalances | Commands::Backup | Commands::Restore { .. } => {
            println!("not implemented");
        }
    }
}
