use std::fmt::Debug;

pub const X_CACHE_TTL: &str = "x-cache-ttl";

#[derive(Debug, Clone, uniffi::Record)]
pub struct AlienHeader {
    pub key: String,
    pub value: String,
}

impl AlienHeader {
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct AlienTarget {
    pub url: String,
    pub method: AlienHttpMethod,
    pub headers: Option<Vec<AlienHeader>>,
    pub body: Option<Vec<u8>>,
}

impl AlienTarget {
    pub fn get(url: &str) -> Self {
        Self {
            url: url.into(),
            method: AlienHttpMethod::Get,
            headers: None,
            body: None,
        }
    }

    pub fn post_json(url: &str, body: serde_json::Value) -> Self {
        Self {
            url: url.into(),
            method: AlienHttpMethod::Post,
            headers: Some(vec![AlienHeader {
                key: "Content-Type".into(),
                value: "application/json".into(),
            }]),
            body: Some(serde_json::to_vec(&body).unwrap()),
        }
    }

    pub fn set_cache_ttl(mut self, ttl: u64) -> Self {
        if self.headers.is_none() {
            self.headers = Some(vec![]);
        }
        if let Some(headers) = self.headers.as_mut() {
            headers.push(AlienHeader {
                key: X_CACHE_TTL.into(),
                value: ttl.to_string(),
            });
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum AlienHttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

impl From<AlienHttpMethod> for String {
    fn from(value: AlienHttpMethod) -> Self {
        match value {
            AlienHttpMethod::Get => "GET",
            AlienHttpMethod::Post => "POST",
            AlienHttpMethod::Put => "PUT",
            AlienHttpMethod::Delete => "DELETE",
            AlienHttpMethod::Head => "HEAD",
            AlienHttpMethod::Options => "OPTIONS",
            AlienHttpMethod::Patch => "PATCH",
        }
        .into()
    }
}

#[uniffi::export]
fn alien_method_to_string(method: AlienHttpMethod) -> String {
    method.into()
}
