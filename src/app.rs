use axum::http::StatusCode;
use axum::Router;
use axum::routing::get;

pub fn router() -> Router {
    Router::new().route(
        "/*path",
        get(|| async { StatusCode::UNAUTHORIZED }),
    )
}
