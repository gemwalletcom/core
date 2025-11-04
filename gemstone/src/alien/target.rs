use std::collections::HashMap;

pub use gem_client::X_CACHE_TTL;
pub use gem_jsonrpc::RpcResponse as AlienResponse;
pub type AlienTarget = swapper::Target;
pub type AlienHttpMethod = swapper::HttpMethod;

#[uniffi::remote(Record)]
pub struct AlienTarget {
    pub url: String,
    pub method: AlienHttpMethod,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}

#[uniffi::remote(Record)]
pub struct AlienResponse {
    pub status: Option<u16>,
    pub data: Vec<u8>,
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
