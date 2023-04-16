use crate::{
    app::{
        headers::{AcceptLanguage, ContentLanguage},
        templates,
    },
    config::CustomCss,
    i18n::I18n,
    storage::{DbPool, NewUser},
};
use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form, TypedHeader,
};
use axum_flash::{Flash, IncomingFlashes};

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
    State(db): State<DbPool>,
    TypedHeader(accept_lang): TypedHeader<AcceptLanguage>,
    flash: Flash,
    Form(new_user): Form<RegisterData>,
) -> (Flash, Redirect) {
    let i18n: I18n = accept_lang.into();
    let Ok(new_user) = new_user.try_into() else {
        return (flash.error(i18n.register.errors.password_mismatch), Redirect::to("/register"));
    };
    if let Err(e) = db.insert_user(new_user).await {
        return (flash.error(format!("{e}")), Redirect::to("/register"));
    }
    (flash, Redirect::to("/"))
}
