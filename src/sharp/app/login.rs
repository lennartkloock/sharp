use crate::{
    config::CustomCss,
    i18n::I18n,
    sharp::app::{
        headers::{build_auth_cookie, AcceptLanguage, ContentLanguage},
        templates,
    },
    storage::{error::StorageError, session, session::NewSession, user, user::User, Db},
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, Params, PasswordHash, PasswordHasher,
};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form, TypedHeader,
};
use axum_extra::extract::CookieJar;
use axum_flash::{Flash, IncomingFlashes};
use tracing::{info, warn};

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
        templates::Login {
            i18n,
            custom_css,
            flashes,
        },
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
    match login_user(&db, &login).await {
        Ok(token) => (
            flash,
            cookies.add(build_auth_cookie(token)),
            Redirect::to("/"),
        ),
        Err(LoginError::Argon2(e)) => {
            warn!("password hashing error during login attempt: {e}");
            (
                flash.error(format!(
                    "{}: failed to validate password",
                    i18n.internal_error
                )),
                cookies,
                Redirect::to("/login"),
            )
        }
        Err(LoginError::Storage(e)) => (
            flash.error(format!("{}: {e}", i18n.internal_error)),
            cookies,
            Redirect::to("/login"),
        ),
        Err(LoginError::WrongEmailPassword) => (
            flash.error(i18n.login.wrong_creds_error),
            cookies,
            Redirect::to("/login"),
        ),
    }
}

enum LoginError {
    Argon2(argon2::password_hash::Error),
    Storage(StorageError),
    WrongEmailPassword,
}

impl From<argon2::password_hash::Error> for LoginError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::Argon2(value)
    }
}

impl From<StorageError> for LoginError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

async fn login_user(db: &Db, data: &LoginData) -> Result<String, LoginError> {
    let user = get_user_and_verify(db, data).await?;
    info!("successfully validated login credentials");
    let new_session = NewSession::generate(user.id);
    session::insert(db, &new_session).await?;
    Ok(new_session.token)
}

/// Do not use argon's verify_password
///
/// The difference is that this method hashes the given password, even when there was no user found.
/// This is important since a client could find out if a user exists in the database just by analyzing the server response time.
async fn get_user_and_verify(db: &Db, data: &LoginData) -> Result<User, LoginError> {
    let user = user::get(db, &data.email).await?;
    let password_hash = user.as_ref().map(|u| u.password_hash.clone());
    let (stored_hash, params) = if let Some(hash) = &password_hash {
        let hash = PasswordHash::new(hash)?;
        let params = Params::try_from(&hash)?;
        (Some(hash), params)
    } else {
        (None, Params::default())
    };
    let random_salt = SaltString::generate(&mut OsRng);
    let salt = stored_hash
        .as_ref()
        .and_then(|h| h.salt)
        .unwrap_or(random_salt.as_salt());
    let hash = Argon2::default()
        .hash_password_customized(
            data.password.as_bytes(),
            stored_hash.as_ref().map(|h| h.algorithm),
            stored_hash.as_ref().and_then(|h| h.version),
            params,
            salt,
        )?
        .hash
        .ok_or(LoginError::WrongEmailPassword)?;
    let user = user.ok_or(LoginError::WrongEmailPassword)?;
    let stored_hash = stored_hash
        .and_then(|h| h.hash)
        .ok_or(LoginError::WrongEmailPassword)?;
    (stored_hash == hash)
        .then_some(user)
        .ok_or(LoginError::WrongEmailPassword)
}
