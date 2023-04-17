use std::sync::Arc;
use crate::{
    app::{
        headers::{AcceptLanguage, ContentLanguage},
        templates,
    },
    config::CustomCss,
    i18n::I18n,
    storage::{Db, NewUser},
};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form, TypedHeader,
};
use axum_flash::{Flash, IncomingFlashes};
use sqlx::AnyPool;
use crate::config::SharpConfig;

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

#[derive(serde::Deserialize)]
pub struct RegisterData {
    email: String,
    username: Option<String>,
    password: String,
    repeat_password: String,
}

impl TryFrom<RegisterData> for NewUser {
    type Error = ();

    fn try_from(value: RegisterData) -> Result<Self, Self::Error> {
        if value.password == value.repeat_password {
            Ok(Self {
                email: value.email,
                username: value.username,
                password: value.password,
            })
        } else {
            Err(())
        }
    }
}

pub async fn submit_register(
    State(db): State<Db<AnyPool>>,
    State(config): State<Arc<SharpConfig>>,
    TypedHeader(accept_lang): TypedHeader<AcceptLanguage>,
    flash: Flash,
    Form(new_user): Form<RegisterData>,
) -> (Flash, Redirect) {
    let i18n: I18n = accept_lang.into();
    let Ok(new_user) = new_user.try_into() else {
        return (flash.error(i18n.register.errors.password_mismatch), Redirect::to("/register"));
    };
    let transaction = db.begin().await.unwrap();
    if let Err(e) = transaction.insert_user(new_user).await {
        transaction.rollback().await;
        return (flash.error(format!("{e}")), Redirect::to("/register"));
    }
    (flash, Redirect::to(&config.redirect_url))
}
