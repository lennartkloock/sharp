use axum::{body::HttpBody, routing::get, Router};
use axum_extra::extract::CookieJar;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Body,
};
use std::{convert::Infallible, net::SocketAddr};
use tower::ServiceExt;
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

const VERSION_STRING: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

mod gateway_service;

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
    info!("Proxying on http://{}", out_addr);

    let router = Router::new().route(
        "/",
        get(|| async { format!("Hello from {VERSION_STRING}") }),
    );

    axum::Server::bind(&in_addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(make_service_fn(|socket: &AddrStream| {
            // TODO: Are all the router clones necessary?
            let router = router.clone();
            let remote_addr = socket.remote_addr();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let router = router.clone();
                    async move {
                        let cookies = CookieJar::from_headers(req.headers());
                        if let Some(_) = cookies.get("SHARP_session") {
                            gateway_service::service(req, remote_addr, out_addr)
                                .await
                                .map(|res| res.map(|b| b.boxed_unsync()))
                        } else {
                            router.oneshot(req).await
                        }
                    }
                }))
            }
        }))
        .await
        .unwrap();
}
