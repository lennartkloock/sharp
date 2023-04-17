use axum::extract::State;
use axum::response::IntoResponse;
use axum::TypedHeader;
use crate::app::headers::{AcceptLanguage, ContentLanguage};
use crate::app::templates;
use crate::config::CustomCss;
use crate::i18n::I18n;

pub async fn login(
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
