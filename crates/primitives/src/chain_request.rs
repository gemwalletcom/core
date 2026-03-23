#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainRequestProtocol {
    JsonRpc,
    Http,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainRequestType {
    Unknown,
    Broadcast,
}

#[derive(Debug, Clone, Copy)]
pub struct ChainRequest<'a> {
    pub protocol: ChainRequestProtocol,
    pub method: &'a str,
    pub path: &'a str,
    pub body: &'a [u8],
}

impl<'a> ChainRequest<'a> {
    pub fn new(protocol: ChainRequestProtocol, method: &'a str, path: &'a str, body: &'a [u8]) -> Self {
        Self { protocol, method, path, body }
    }

    pub fn is_json_rpc_method(&self, method: &str) -> bool {
        self.protocol == ChainRequestProtocol::JsonRpc && self.method == method
    }

    pub fn is_http_path(&self, method: &str, path: &str) -> bool {
        self.protocol == ChainRequestProtocol::Http && self.method == method && self.path == path
    }

    pub fn is_http_post_path(&self, path: &str) -> bool {
        self.is_http_path("POST", path)
    }

    pub fn body_utf8(&self) -> Option<&'a str> {
        std::str::from_utf8(self.body).ok()
    }
}
