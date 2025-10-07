use std::{collections::HashMap, fmt::Debug};

pub const X_CACHE_TTL: &str = "x-cache-ttl";

#[derive(Debug, Clone)]
pub struct Target {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}

impl Target {
    pub fn get(url: &str) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Get,
            headers: None,
            body: None,
        }
    }

    pub fn post_json(url: &str, body: serde_json::Value) -> Self {
        Self {
            url: url.into(),
            method: HttpMethod::Post,
            headers: Some(HashMap::from([("Content-Type".into(), "application/json".into())])),
            body: Some(serde_json::to_vec(&body).unwrap()),
        }
    }

    pub fn set_cache_ttl(mut self, ttl: u64) -> Self {
        if self.headers.is_none() {
            self.headers = Some(HashMap::new());
        }
        if let Some(headers) = self.headers.as_mut() {
            headers.insert(X_CACHE_TTL.into(), ttl.to_string());
        }
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

impl From<HttpMethod> for String {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Patch => "PATCH",
        }
        .into()
    }
}
