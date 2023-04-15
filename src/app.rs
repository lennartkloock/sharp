use crate::{app::headers::AcceptLanguage, config::CustomCss, i18n::I18n};
use axum::{extract::State, response::Redirect, routing::get, Router, TypedHeader};

mod headers;
mod templates;

pub fn router() -> Router<Option<CustomCss>> {
    Router::new()
        .route("/", get(|| async { Redirect::to("/login") }))
        .route("/login", get(login))
        .route("/register", get(register))
        .route("/reset-password", get(reset_password))
}

async fn login(
    State(custom_css): State<Option<CustomCss>>,
    TypedHeader(accept_lang): TypedHeader<AcceptLanguage>,
) -> templates::Login {
    let i18n = accept_lang.0.iter().find_map(|id| I18n::from_lang_id(id)).unwrap_or_default();
    templates::Login {
        i18n,
        custom_css,
    }
}

async fn register(
    State(custom_css): State<Option<CustomCss>>,
    TypedHeader(accept_lang): TypedHeader<AcceptLanguage>,
) -> templates::Register {
    let i18n = accept_lang.0.iter().find_map(|id| I18n::from_lang_id(id)).unwrap_or_default();
    templates::Register {
        i18n,
        custom_css,
    }
}

async fn reset_password(
    State(custom_css): State<Option<CustomCss>>,
    TypedHeader(accept_lang): TypedHeader<AcceptLanguage>,
) -> templates::ResetPassword {
    let i18n = accept_lang.0.iter().find_map(|id| I18n::from_lang_id(id)).unwrap_or_default();
    templates::ResetPassword {
        i18n,
        custom_css,
    }
}
