use argon2::{Argon2, PasswordHash, PasswordVerifier};
use argon2::password_hash::Error;
use crate::{
    app::{
        headers::{AcceptLanguage, ContentLanguage},
        templates,
    },
    config::CustomCss,
    i18n::I18n,
};
use axum::{extract::State, Form, response::IntoResponse, TypedHeader};
use axum::response::Redirect;
use axum_extra::extract::CookieJar;
use axum_flash::{Flash, IncomingFlashes};
use tracing::{info, warn};
use crate::app::headers::build_auth_cookie;
use crate::storage::session::NewSession;
use crate::storage::{Db, session, user};
use crate::storage::user::User;

pub async fn login(
    State(custom_css): State<Option<CustomCss>>,
    incoming_flashes: IncomingFlashes,
    TypedHeader(accept_lang): TypedHeader<AcceptLanguage>,
) -> impl IntoResponse {
    let i18n: I18n = accept_lang.into();
    let flashes = incoming_flashes
        .iter()
        .map(|(l, m)| (l, m.to_string()))
        .collect();
    (
        incoming_flashes,
        ContentLanguage::from(i18n.lang_id.clone()),
        templates::Login { i18n, custom_css, flashes },
    )
}

#[derive(serde::Deserialize)]
pub struct LoginData {
    email: String,
    password: String,
}

pub async fn submit_login(
    State(db): State<Db>,
    TypedHeader(accept_lang): TypedHeader<AcceptLanguage>,
    cookies: CookieJar,
    flash: Flash,
    Form(login): Form<LoginData>,
) -> impl IntoResponse {
    let i18n: I18n = accept_lang.into();
    match user::get(&db, &login.email).await
        .map(|u| u.map(|User { id, password_hash, .. }| PasswordHash::new(&password_hash)
            .map(|hash| Argon2::default().verify_password(login.password.as_bytes(), &hash)
                .map(|_| id)))
        )
    {
        Ok(Some(Ok(Ok(user_id)))) => {
            info!("successfully validated login credentials");
            let new_session = NewSession::generate(user_id);
            match session::insert(&db, &new_session).await {
                Ok(_) => (
                    flash,
                    cookies.add(build_auth_cookie(new_session.token)),
                    Redirect::to("/"),
                ),
                Err(e) => (
                    flash.error(format!("{}: {e}", i18n.internal_error)),
                    cookies,
                    Redirect::to("/login"),
                ),
            }
        }
        Ok(Some(Ok(Err(Error::Password)))) | Ok(None) => {
            (
                flash.error(i18n.login.wrong_creds_error),
                cookies,
                Redirect::to("/login"),
            )
        }
        Ok(Some(Ok(Err(e)))) => {
            warn!("failed to verify password: {e}");
            (
                flash.error(format!("{}: failed to validate password", i18n.internal_error)),
                cookies,
                Redirect::to("/login"),
            )
        }
        Ok(Some(Err(e))) => {
            warn!("failed to parse stored password hash: {e}");
            (
                flash.error(format!("{}: failed to validate password", i18n.internal_error)),
                cookies,
                Redirect::to("/login"),
            )
        },
        Err(e) => (
            flash.error(format!("{}: {e}", i18n.internal_error)),
            cookies,
            Redirect::to("/login"),
        ),
    }
}
