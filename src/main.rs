use crate::{
    config::{CustomCss, SharpConfig, SharpConfigBuilder},
    storage::Db,
};
use axum::extract::FromRef;
use axum_extra::extract::CookieJar;
use clap::Parser;
use hyper::{
    body::HttpBody,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
};
use std::{convert::Infallible, net::SocketAddr, path::PathBuf, sync::Arc};
use tower::ServiceExt;
use tracing::{debug, error, info, Level};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

const VERSION_STRING: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

mod app;
mod config;
mod exceptions;

mod i18n {
    i18n_langid_codegen::i18n!("locales");
}

mod gateway_service;
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

#[derive(Debug, thiserror::Error)]
enum RoutingError {
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),
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
            match Db::connect(&config.database_url).await {
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
                Ok(db) => sharp(config, db).await,
                Err(e) => error!("failed to connect to database: {e}"),
            }
        }
        Err(e) => error!("{e}"),
    }
}

#[derive(Clone)]
pub struct AppState {
    db: Db,
    config: Arc<SharpConfig>,
    flash_config: axum_flash::Config,
}

impl FromRef<AppState> for Db {
    fn from_ref(input: &AppState) -> Self {
        input.db.clone()
    }
}

impl FromRef<AppState> for Arc<SharpConfig> {
    fn from_ref(input: &AppState) -> Self {
        Arc::clone(&input.config)
    }
}

impl FromRef<AppState> for Option<CustomCss> {
    fn from_ref(input: &AppState) -> Self {
        input.config.custom_css.clone()
    }
}

impl FromRef<AppState> for axum_flash::Config {
    fn from_ref(input: &AppState) -> Self {
        input.flash_config.clone()
    }
}

async fn sharp(config: SharpConfig, db: Db) {
    let in_addr = SocketAddr::new(config.address, config.port);

    info!("Listening on http://{}", in_addr);
    info!("Proxying to http://{}", config.upstream);

    let router = app::router().with_state(AppState {
        db,
        config: Arc::new(config.clone()),
        flash_config: axum_flash::Config::new(axum_flash::Key::generate()),
    });

    axum::Server::bind(&in_addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(make_service_fn(|socket: &AddrStream| {
            // Fixme: Are all the router clones necessary?
            let router = router.clone();
            let client_addr = socket.remote_addr();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let router = router.clone();
                    async move {
                        info!("{client_addr} : {} {}", req.method(), req.uri());
                        let cookies = CookieJar::from_headers(req.headers());
                        let proxy_through = exceptions::is_exception(&req)
                            || cookies.get("SHARP_session").map(|c| c.value() == "true")
                                == Some(true);
                        if proxy_through {
                            info!("proxying...");
                            gateway_service::service(req, client_addr, config.upstream)
                                .await
                                .map(|res| {
                                    res.map(|b| b.map_err(RoutingError::from).boxed_unsync())
                                })
                        } else {
                            info!("client couldn't authorize");
                            Ok(router
                                .oneshot(req)
                                .await
                                .unwrap() // Is Infallible
                                .map(|b| b.map_err(RoutingError::from).boxed_unsync()))
                        }
                    }
                }))
            }
        }))
        .await
        .unwrap();
}
