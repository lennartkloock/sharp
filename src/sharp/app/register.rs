use crate::{
    config::{CustomCss, SharpConfig},
    i18n::I18n,
    sharp::app::{
        headers::{build_auth_cookie, AcceptLanguage, ContentLanguage},
        templates,
    },
    storage::{error::StorageResult, session, session::NewSession, user, user::NewUser, Db},
};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form, TypedHeader,
};
use axum_extra::extract::CookieJar;
use axum_flash::{Flash, IncomingFlashes};
use std::sync::Arc;

pub async fn register(
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
        templates::Register {
            i18n,
            custom_css,
            flashes,
        },
    )
}

pub enum RegisterError {
    PasswordTooShort,
    PasswordMismatch,
}

#[derive(serde::Deserialize)]
pub struct RegisterData {
    email: String,
    username: Option<String>,
    password: String,
    repeat_password: String,
}

impl TryFrom<RegisterData> for NewUser {
    type Error = RegisterError;

    fn try_from(value: RegisterData) -> Result<Self, Self::Error> {
        if value.password.len() < 8 {
            return Err(RegisterError::PasswordTooShort);
        }
        if value.password != value.repeat_password {
            return Err(RegisterError::PasswordMismatch);
        }
        Ok(Self {
            email: value.email,
            username: value.username,
            password: value.password,
        })
    }
}

pub async fn submit_register(
    State(db): State<Db>,
    State(config): State<Arc<SharpConfig>>,
    TypedHeader(accept_lang): TypedHeader<AcceptLanguage>,
    cookies: CookieJar,
    flash: Flash,
    Form(new_user): Form<RegisterData>,
) -> (Flash, CookieJar, Redirect) {
    let i18n: I18n = accept_lang.into();
    match new_user.try_into() {
        Ok(new_user) => match register_new_user(&db, new_user).await {
            Ok(token) => (
                flash,
                cookies.add(build_auth_cookie(token)),
                Redirect::to(&config.redirect_url),
            ),
            Err(e) => (
                flash.error(format!("{e}")),
                cookies,
                Redirect::to("/register"),
            ),
        },
        Err(e) => {
            let e = match e {
                RegisterError::PasswordTooShort => i18n.register.password_too_short_error,
                RegisterError::PasswordMismatch => i18n.register.password_mismatch_error,
            };
            (flash.error(e), cookies, Redirect::to("/register"))
        }
    }
}

async fn register_new_user(db: &Db, new_user: NewUser) -> StorageResult<String> {
    // Transaction is dropped when error occurs, causing a rollback
    let mut transaction = db.begin().await?;
    let id = user::insert(&mut transaction, &new_user).await?;
    let session = NewSession::generate(id);
    session::insert(&mut transaction, &session).await?;
    transaction.commit().await?;
    Ok(session.token)
}
