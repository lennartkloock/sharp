use crate::i18n::I18n;
use axum::{
    headers::{Error, Header, HeaderName, HeaderValue},
    http::{header::CONTENT_LANGUAGE, StatusCode},
    response::{IntoResponseParts, ResponseParts},
};
use std::{ops::Deref, str::FromStr};
use std::borrow::Cow;
use axum_extra::extract::cookie::{Cookie, SameSite};
use tracing::debug;
use unic_langid::LanguageIdentifier;
use crate::storage::session;

pub const AUTH_COOKIE: &str = "SHARP_token";

pub fn build_auth_cookie<'c, S: Into<Cow<'c, str>>>(value: S) -> Cookie<'c> {
    Cookie::build(AUTH_COOKIE, value.into())
        .max_age(session::MAX_AGE)
        .http_only(true)
        .path("/")
        .same_site(SameSite::Strict)
        .secure(true)
        .finish()
}

pub struct AcceptLanguage(Vec<LanguageIdentifier>);

impl Header for AcceptLanguage {
    fn name() -> &'static HeaderName {
        &axum::http::header::ACCEPT_LANGUAGE
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(Error::invalid)?;
        let lang_ids: Vec<LanguageIdentifier> = value
            .to_str()
            .map_err(|_| Error::invalid())?
            .split(',')
            .filter_map(|lang| {
                lang.split(';')
                    .next()
                    .and_then(|l| LanguageIdentifier::from_str(l).ok())
            })
            .collect();
        debug!(
            "client requests languages: {:?}",
            lang_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
        );
        Ok(Self(lang_ids))
    }

    fn encode<E: Extend<HeaderValue>>(&self, _: &mut E) {
        unimplemented!()
    }
}

impl Deref for AcceptLanguage {
    type Target = Vec<LanguageIdentifier>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<AcceptLanguage> for I18n {
    fn from(lang: AcceptLanguage) -> Self {
        lang.iter()
            .find_map(I18n::from_lang_id)
            .unwrap_or_default()
    }
}

pub struct ContentLanguage(LanguageIdentifier);

impl From<LanguageIdentifier> for ContentLanguage {
    fn from(id: LanguageIdentifier) -> Self {
        Self(id)
    }
}

impl IntoResponseParts for ContentLanguage {
    type Error = (StatusCode, String);

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        match HeaderValue::try_from(&self.0.to_string()) {
            Ok(value) => {
                res.headers_mut().insert(CONTENT_LANGUAGE, value);
                Ok(res)
            }
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to convert lang id to header value: {e}"),
            )),
        }
    }
}
