use axum::{Router, routing::get};
use axum::extract::State;
use axum::response::Redirect;
use crate::config::CustomCss;
use crate::i18n::Locale;

mod templates;

pub fn router() -> Router<Option<CustomCss>> {
    Router::new()
        .route("/", get(|| async { Redirect::to("/login") }))
        .route("/login", get(login))
        .route("/register", get(register))
        .route("/reset-password", get(reset_password))
}

async fn login(State(custom_css): State<Option<CustomCss>>) -> templates::Login {
    templates::Login { i18n: Locale::De, custom_css }
}

async fn register(State(custom_css): State<Option<CustomCss>>) -> templates::Register {
    templates::Register { i18n: Locale::De,custom_css }
}

async fn reset_password(State(custom_css): State<Option<CustomCss>>) -> templates::ResetPassword {
    templates::ResetPassword { i18n: Locale::De,custom_css }
}
