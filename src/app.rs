use crate::{
    app::headers::{AcceptLanguage, ContentLanguage},
    config::CustomCss,
    i18n::I18n,
    AppState,
};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    routing::get,
    Router, TypedHeader,
};

mod headers;
mod templates;

mod register;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { Redirect::to("/login") }))
        .route("/login", get(login))
        .route(
            "/register",
            get(register::register).post(register::submit_register),
        )
        .route("/reset-password", get(reset_password))
        .fallback(get(|| async { Redirect::to("/") }))
}

async fn login(
    State(custom_css): State<Option<CustomCss>>,
    TypedHeader(accept_lang): TypedHeader<AcceptLanguage>,
) -> impl IntoResponse {
    let i18n: I18n = accept_lang.into();
    (
        ContentLanguage::from(i18n.lang_id.clone()),
        templates::Login { i18n, custom_css },
    )
}

// async fn submit_login(Form(login): Form<NewUser>, State(db): State<DbPool>) -> Redirect {
//     Redirect::to("/")
// }

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
