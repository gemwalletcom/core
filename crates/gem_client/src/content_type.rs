use std::str::FromStr;

pub const CONTENT_TYPE: &str = "Content-Type";
pub const APPLICATION_JSON: &str = "application/json";
const TEXT_PLAIN: &str = "text/plain";
const APPLICATION_FORM_URL_ENCODED: &str = "application/x-www-form-urlencoded";
const APPLICATION_X_BINARY: &str = "application/x-binary";

#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    ApplicationJson,
    TextPlain,
    ApplicationFormUrlEncoded,
    ApplicationXBinary,
}

impl ContentType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ContentType::ApplicationJson => APPLICATION_JSON,
            ContentType::TextPlain => TEXT_PLAIN,
            ContentType::ApplicationFormUrlEncoded => APPLICATION_FORM_URL_ENCODED,
            ContentType::ApplicationXBinary => APPLICATION_X_BINARY,
        }
    }
}

impl FromStr for ContentType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            APPLICATION_JSON => Ok(ContentType::ApplicationJson),
            TEXT_PLAIN => Ok(ContentType::TextPlain),
            APPLICATION_FORM_URL_ENCODED => Ok(ContentType::ApplicationFormUrlEncoded),
            APPLICATION_X_BINARY => Ok(ContentType::ApplicationXBinary),
            _ => Err("Unknown content type"),
        }
    }
}
