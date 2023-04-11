use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tower::make::Shared;
use tower::service_fn;
use tracing_subscriber::filter::LevelFilter;

const VERSION_STRING: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));

mod gateway_service;

#[tokio::main]
async fn main() -> hyper::Result<()> {
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

    axum::Server::bind(&in_addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(Shared::new(service_fn(move |req| gateway_service::service(req, out_addr))))
        .await
}
