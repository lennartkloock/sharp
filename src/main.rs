use crate::config::SharpConfig;
use axum_extra::extract::CookieJar;
use hyper::{
    body::HttpBody,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
};
use std::{convert::Infallible, net::SocketAddr};
use tower::ServiceExt;
use tracing::{error, info};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

const VERSION_STRING: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

mod app;
mod config;
mod exceptions;
mod gateway_service;

#[derive(Debug, thiserror::Error)]
enum RoutingError {
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("SHARP_LOG")
                .from_env_lossy(),
        )
        .init();

    info!("{VERSION_STRING}");

    match config::read_config("sharp.toml").await {
        Ok(config) => sharp(config).await,
        Err(e) => error!("{e}"),
    }
}

async fn sharp(config: SharpConfig) {
    let in_addr = SocketAddr::new(config.address, config.port);

    info!("Listening on http://{}", in_addr);
    info!("Proxying to http://{}", config.upstream);

    let router = app::router();

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
                        match (exceptions::is_exception(&req), cookies.get("SHARP_session")) {
                            (true, _) => {
                                info!("`{}` is an exception, proxying...", req.uri());
                                gateway_service::service(req, client_addr, config.upstream)
                                    .await
                                    .map(|res| {
                                        res.map(|b| b.map_err(RoutingError::from).boxed_unsync())
                                    })
                            }
                            (_, Some(cookie)) => {
                                info!("cookie was set, proxying...");
                                // TODO: Check cookie
                                gateway_service::service(req, client_addr, config.upstream)
                                    .await
                                    .map(|res| {
                                        res.map(|b| b.map_err(RoutingError::from).boxed_unsync())
                                    })
                            }
                            (_, _) => {
                                info!("client couldn't authorize");
                                Ok(router
                                    .oneshot(req)
                                    .await
                                    .unwrap() // Is Infallible
                                    .map(|b| b.map_err(RoutingError::from).boxed_unsync()))
                            }
                        }
                    }
                }))
            }
        }))
        .await
        .unwrap();
}
