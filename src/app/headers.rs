use axum::{
    headers::{Error, Header, HeaderName, HeaderValue},
};
use std::str::FromStr;
use unic_langid::LanguageIdentifier;

pub struct AcceptLanguage(pub Vec<LanguageIdentifier>);

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
        Ok(Self(
            value
                .to_str()
                .map_err(|_| Error::invalid())?
                .split(';')
                .filter_map(|lang| LanguageIdentifier::from_str(lang).ok())
                .collect(),
        ))
    }

    fn encode<E: Extend<HeaderValue>>(&self, _: &mut E) {
        unimplemented!()
    }
}
