use reqwest::header;

pub const JSON_CONTENT_TYPE: &str = "application/json";
pub const JSON_HEADER: header::HeaderValue = header::HeaderValue::from_static(JSON_CONTENT_TYPE);
