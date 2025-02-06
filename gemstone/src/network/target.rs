use std::{collections::HashMap, fmt::Debug};

pub const X_CACHE_TTL: &str = "x-cache-ttl";

#[derive(Debug, Clone, uniffi::Record)]
pub struct AlienTarget {
    pub url: String,
    pub method: AlienHttpMethod,
    pub headers: Option<HashMap<String, String>>,
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
            headers: Some(HashMap::from([("Content-Type".into(), "application/json".into())])),
            body: Some(serde_json::to_vec(&body).unwrap()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
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
