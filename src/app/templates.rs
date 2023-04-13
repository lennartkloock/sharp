use crate::config::CustomCss;

#[derive(askama::Template)]
#[template(path = "login.html")]
pub struct Login {
    pub custom_css: Option<CustomCss>,
}

#[derive(askama::Template)]
#[template(path = "register.html")]
pub struct Register {
    pub custom_css: Option<CustomCss>,
}
