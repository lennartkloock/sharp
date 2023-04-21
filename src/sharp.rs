use crate::{
    config::{CustomCss, SharpConfig},
    sharp::app::AUTH_COOKIE,
    storage::{session, Db},
};
use axum::extract::FromRef;
use axum_extra::extract::CookieJar;
use hyper::{
    body::HttpBody,
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Response, StatusCode,
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
    #[error("infallible")]
    Infallible,
}

pub async fn sharp(config: SharpConfig, db: Db) {
    let in_addr = SocketAddr::new(config.address, config.port);

    info!("Listening on http://{}", in_addr);
    info!("Proxying to http://{}", config.upstream);

    let router = app::router().with_state(AppState {
        db: db.clone(),
        config: Arc::new(config.clone()),
        flash_config: axum_flash::Config::new(axum_flash::Key::generate()),
    });

    axum::Server::bind(&in_addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(make_service_fn(|socket: &AddrStream| {
            // Fixme: Are all the router clones necessary?
            let router = router.clone();
            let db = db.clone();
            let client_addr = socket.remote_addr();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let router = router.clone();
                    let db = db.clone();
                    async move {
                        info!("{client_addr} : {} {}", req.method(), req.uri());

                        let cookies = CookieJar::from_headers(req.headers());
                        let session = match cookies.get(AUTH_COOKIE).map(|c| c.value()) {
                            Some(c) => match session::get_by_token(&db, c).await {
                                // TODO: Verify that session is not too old
                                Ok(s) => Some(s),
                                Err(e) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                                        .body(
                                            format!("{}", e)
                                                .map_err(|_| RoutingError::Infallible)
                                                .boxed_unsync(),
                                        )
                                        .unwrap());
                                }
                            },
                            None => None,
                        };

                        let proxy_through = exceptions::is_exception(&req) || session.is_some();
                        if proxy_through {
                            info!("proxying...");
                            gateway_service::service(req, client_addr, session, config.upstream)
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
