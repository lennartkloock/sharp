use crate::{config::CustomCss, i18n};

#[derive(askama::Template)]
#[template(path = "login.html")]
pub struct Login {
    pub i18n: i18n::I18n,
    pub custom_css: Option<CustomCss>,
    pub flashes: Vec<(axum_flash::Level, String)>,
}

#[derive(askama::Template)]
#[template(path = "register.html")]
pub struct Register {
    pub i18n: i18n::I18n,
    pub custom_css: Option<CustomCss>,
    pub flashes: Vec<(axum_flash::Level, String)>,
}

#[derive(askama::Template)]
#[template(path = "reset-password.html")]
pub struct ResetPassword {
    pub i18n: i18n::I18n,
    pub custom_css: Option<CustomCss>,
}
