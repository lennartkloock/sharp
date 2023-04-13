#[derive(askama::Template)]
#[template(path = "login.html")]
pub struct Login {}

#[derive(askama::Template)]
#[template(path = "register.html")]
pub struct Register {}
