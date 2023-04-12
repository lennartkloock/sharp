use axum::{http::StatusCode, routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/*path", get(|| async { StatusCode::UNAUTHORIZED }))
}
