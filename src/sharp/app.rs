use crate::{
    config::CustomCss,
    i18n::I18n,
    sharp::{
        app::headers::{AcceptLanguage, ContentLanguage},
        AppState,
    },
};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    routing::get,
    Router, TypedHeader,
};

mod headers;
mod templates;

mod login;
mod register;

pub const AUTH_COOKIE: &str = "SHARP_token";

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { Redirect::to("/login") }))
        .route("/login", get(login::login))
        .route(
            "/register",
            get(register::register).post(register::submit_register),
        )
        .route("/reset-password", get(reset_password))
        .fallback(get(|| async { Redirect::to("/") }))
}

async fn reset_password(
    State(custom_css): State<Option<CustomCss>>,
    TypedHeader(accept_lang): TypedHeader<AcceptLanguage>,
) -> impl IntoResponse {
    let i18n: I18n = accept_lang.into();
    (
        ContentLanguage::from(i18n.lang_id.clone()),
        templates::ResetPassword { i18n, custom_css },
    )
}
