pub type AlienTarget = swapper::Target;
pub type AlienHttpMethod = swapper::HttpMethod;
pub use gem_jsonrpc::X_CACHE_TTL;

use std::collections::HashMap;

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
