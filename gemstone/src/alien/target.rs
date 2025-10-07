pub type AlienTarget = gem_swapper::Target;
pub type AlienHttpMethod = gem_swapper::HttpMethod;

use std::collections::HashMap;

pub const X_CACHE_TTL: &str = "x-cache-ttl";

#[uniffi::remote(Record)]
pub struct AlienTarget {
    pub url: String,
    pub method: AlienHttpMethod,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}

#[uniffi::remote(Enum)]
pub enum AlienHttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

#[uniffi::export]
fn alien_method_to_string(method: AlienHttpMethod) -> String {
    method.into()
}
