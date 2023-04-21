use crate::config::SharpConfigBuilder;
use clap::Parser;
use sqlx::{any::AnyPoolOptions, sqlite::SqliteConnectOptions, ConnectOptions, Connection};
use std::{path::PathBuf, str::FromStr};
use tracing::{debug, error, info, Level};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

const VERSION_STRING: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

mod config;

mod i18n {
    i18n_langid_codegen::i18n!("locales");
}

mod sharp;
mod storage;

// TODO: Improve slogan, include in README

/// [s]elf-[h]osted [a]uthentication [r]everse [p]roxy
///
/// Simple user management for your web backend
#[derive(clap::Parser)]
#[command(author, version, about, long_about)]
struct Args {
    /// Relative path to the config file
    #[arg(short, long, default_value_os_t = PathBuf::from("sharp.toml"))]
    config: PathBuf,
    /// Log level
    #[arg(short, long, default_value_t = Level::INFO)]
    log_level: Level,
    /// Check config file for errors
    #[arg(long)]
    check: bool,
    /// Create the necessary tables in the database
    #[arg(long)]
    setup_db: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::from_level(args.log_level).into())
                .with_env_var("SHARP_LOG")
                .from_env_lossy(),
        )
        .init();

    info!("{VERSION_STRING} - show help with '--help'");

    let config_res = if args.check {
        config::read_config(|| SharpConfigBuilder::from_file(args.config)).await
    } else {
        config::read_run_config(args.config).await
    };
    match config_res {
        Ok(config) => {
            debug!("read config: {config:?}");

            if let Ok(opt) = SqliteConnectOptions::from_str(&config.database_url) {
                if let Ok(con) = opt.create_if_missing(true).connect().await {
                    debug!("established first connection to sqlite database");
                    con.close();
                }
            }

            match AnyPoolOptions::new()
                .max_connections(config.database_max_connections)
                .connect(&config.database_url)
                .await
            {
                Ok(db) if args.check || args.setup_db => {
                    if args.check {
                        info!("config is OK");
                    }
                    if args.setup_db {
                        match storage::setup(&db).await {
                            Ok(_) => info!("successfully set up database"),
                            Err(e) => error!("failed to setup database: {e}"),
                        }
                    }
                }
                Ok(db) => sharp::sharp(config, db).await,
                Err(e) => error!("failed to connect to database: {e}"),
            }
        }
        Err(e) => error!("{e}"),
    }
}
