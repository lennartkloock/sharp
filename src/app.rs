use axum::{Router, routing::get};

mod templates;

pub fn router() -> Router {
    Router::new()
        .route("/login", get(login))
        .route("/register", get(register))
}

async fn login() -> templates::Login {
    templates::Login {}
}

async fn register() -> templates::Register {
    templates::Register {}
}
