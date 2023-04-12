use axum_extra::extract::CookieJar;
use hyper::{
    body::HttpBody,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
};
use std::{convert::Infallible, net::SocketAddr};
use tower::ServiceExt;
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

const VERSION_STRING: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

mod app;
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
                .from_env_lossy(),
        )
        .init();

    info!("{VERSION_STRING}");

    let in_addr: SocketAddr = ([127, 0, 0, 1], 3001).into();
    let out_addr: SocketAddr = ([127, 0, 0, 1], 8000).into();

    info!("Listening on http://{}", in_addr);
    info!("Proxying to http://{}", out_addr);

    let router = app::router();

    axum::Server::bind(&in_addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(make_service_fn(|socket: &AddrStream| {
            // Fixme: Are all the router clones necessary?
            let router = router.clone();
            let remote_addr = socket.remote_addr();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let router = router.clone();
                    async move {
                        info!("{remote_addr} : {} {}", req.method(), req.uri());
                        let cookies = CookieJar::from_headers(req.headers());
                        match (exceptions::is_exception(&req), cookies.get("SHARP_session")) {
                            (true, _) => {
                                info!("`{}` is an exception, proxying...", req.uri());
                                gateway_service::service(req, remote_addr, out_addr)
                                    .await
                                    .map(|res| {
                                        res.map(|b| b.map_err(RoutingError::from).boxed_unsync())
                                    })
                            }
                            (_, Some(cookie)) => {
                                info!("cookie was set, proxying...");
                                // TODO: Check cookie
                                gateway_service::service(req, remote_addr, out_addr)
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
