use super::{AlienError, HttpMethod, Target};
use primitives::{Chain, node_config::get_nodes_for_chain};

use async_trait::async_trait;
use futures::TryFutureExt;
use gem_jsonrpc::{RpcProvider as GenericRpcProvider, RpcResponse};
use reqwest::Client;

#[derive(Debug)]
pub struct NativeProvider {
    pub client: Client,
    debug: bool,
}

impl NativeProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            debug: true,
        }
    }

    pub fn set_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }
}

impl Default for NativeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GenericRpcProvider for NativeProvider {
    type Error = AlienError;

    fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error> {
        let nodes = get_nodes_for_chain(chain);
        if nodes.is_empty() {
            return Err(Self::Error::response_error(format!("not supported chain: {chain:?}")));
        }
        Ok(nodes[0].url.clone())
    }

    async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error> {
        if self.debug {
            println!("==> request: url: {:?}, method: {:?}", target.url, target.method);
        }
        let mut req = match target.method {
            HttpMethod::Get => self.client.get(target.url),
            HttpMethod::Post => self.client.post(target.url),
            HttpMethod::Put => self.client.put(target.url),
            HttpMethod::Delete => self.client.delete(target.url),
            HttpMethod::Head => self.client.head(target.url),
            HttpMethod::Patch => self.client.patch(target.url),
            HttpMethod::Options => todo!(),
        };
        if let Some(headers) = target.headers {
            for (key, value) in headers.iter() {
                req = req.header(key, value);
            }
        }
        if let Some(body) = target.body {
            if self.debug && body.len() <= 4096 {
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
                    println!("=== json: {json:?}");
                } else {
                    println!("=== body: {:?}", String::from_utf8(body.to_vec()).unwrap());
                }
            }
            req = req.body(body);
        }

        let response = req.send().map_err(|e| Self::Error::response_error(format!("reqwest send error: {e}"))).await?;
        let status = response.status();
        let bytes = response.bytes().map_err(|e| Self::Error::response_error(format!("request error: {e}"))).await?;
        if self.debug {
            println!("<== response body size: {:?}", bytes.len());
        }
        if self.debug && bytes.len() <= 4096 {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                println!("=== json: {json:?}");
            } else {
                println!("=== body: {:?}", String::from_utf8(bytes.to_vec()).unwrap());
            }
        }
        if !status.is_success() {
            return Err(Self::Error::http_error(status.as_u16(), bytes.len()));
        }
        Ok(RpcResponse {
            status: Some(status.as_u16()),
            data: bytes.to_vec(),
        })
    }
}
