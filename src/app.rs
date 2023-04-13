use axum::{Router, routing::get};
use axum::extract::State;
use crate::config::CustomCss;

mod templates;

pub fn router() -> Router<Option<CustomCss>> {
    Router::new()
        .route("/login", get(login))
        .route("/register", get(register))
}

async fn login(State(custom_css): State<Option<CustomCss>>) -> templates::Login {
    templates::Login { custom_css }
}

async fn register(State(custom_css): State<Option<CustomCss>>) -> templates::Register {
    templates::Register { custom_css }
}
