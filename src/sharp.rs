use crate::{
    config::{CustomCss, SharpConfig},
    storage::Db,
};
use axum::extract::FromRef;
use axum_extra::extract::CookieJar;
use hyper::{
    body::HttpBody,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
};
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tower::ServiceExt;
use tracing::{error, info};

mod app;
mod exceptions;
mod gateway_service;

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

#[derive(Debug, thiserror::Error)]
enum RoutingError {
    #[error("hyper error: {0}")]
    Hyper(#[from] hyper::Error),
    #[error("axum error: {0}")]
    Axum(#[from] axum::Error),
}

pub async fn sharp(config: SharpConfig, db: Db) {
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
