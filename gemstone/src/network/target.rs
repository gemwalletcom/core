use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Clone, uniffi::Record)]
pub struct AlienTarget {
    pub url: String,
    pub method: AlienHttpMethod,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
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
