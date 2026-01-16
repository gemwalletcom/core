use std::str::FromStr;

pub const CONTENT_TYPE: &str = "Content-Type";
const APPLICATION_JSON: &str = "application/json";
const TEXT_PLAIN: &str = "text/plain";
const APPLICATION_FORM_URL_ENCODED: &str = "application/x-www-form-urlencoded";
const APPLICATION_X_BINARY: &str = "application/x-binary";
const APPLICATION_APTOS_BCS: &str = "application/x.aptos.signed_transaction+bcs";

#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    ApplicationJson,
    TextPlain,
    ApplicationFormUrlEncoded,
    ApplicationXBinary,
    ApplicationAptosBcs,
}

impl ContentType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            ContentType::ApplicationJson => APPLICATION_JSON,
            ContentType::TextPlain => TEXT_PLAIN,
            ContentType::ApplicationFormUrlEncoded => APPLICATION_FORM_URL_ENCODED,
            ContentType::ApplicationXBinary => APPLICATION_X_BINARY,
            ContentType::ApplicationAptosBcs => APPLICATION_APTOS_BCS,
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
            APPLICATION_APTOS_BCS => Ok(ContentType::ApplicationAptosBcs),
            _ => Err("Unknown content type"),
        }
    }
}
