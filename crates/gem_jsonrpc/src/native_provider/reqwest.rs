use gem_client::ClientError;
use primitives::{Chain, node_config::get_nodes_for_chain};
use reqwest::Client;

use crate::{HttpMethod, RpcProvider, RpcResponse, Target};

#[derive(Debug)]
pub struct NativeProvider {
    client: Client,
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

#[async_trait::async_trait]
impl RpcProvider for NativeProvider {
    type Error = ClientError;

    fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error> {
        let nodes = get_nodes_for_chain(chain);
        if nodes.is_empty() {
            return Err(ClientError::Network(format!("not supported chain: {chain:?}")));
        }
        Ok(nodes[0].url.clone())
    }

    async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error> {
        if self.debug {
            println!("==> request: url: {:?}, method: {:?}", target.url, target.method);
        }

        let mut request = match target.method {
            HttpMethod::Get => self.client.get(target.url),
            HttpMethod::Post => self.client.post(target.url),
            HttpMethod::Put => self.client.put(target.url),
            HttpMethod::Delete => self.client.delete(target.url),
            HttpMethod::Head => self.client.head(target.url),
            HttpMethod::Patch => self.client.patch(target.url),
            HttpMethod::Options => self.client.request(reqwest::Method::OPTIONS, target.url),
        };

        if let Some(headers) = target.headers {
            for (key, value) in headers {
                request = request.header(&key, value);
            }
        }

        if let Some(body) = target.body {
            if self.debug && body.len() <= 4096 {
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
                    println!("=== json: {json:?}");
                } else {
                    println!("=== body: {:?}", String::from_utf8_lossy(&body));
                }
            }
            request = request.body(body);
        }

        let response = request.send().await.map_err(|e| ClientError::Network(format!("reqwest send error: {e}")))?;
        let status = response.status();
        let bytes = response.bytes().await.map_err(|e| ClientError::Network(format!("request error: {e}")))?;

        if self.debug {
            println!("<== response body size: {:?}", bytes.len());
        }
        if self.debug && bytes.len() <= 4096 {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                println!("=== json: {json:?}");
            } else {
                println!("=== body: {:?}", String::from_utf8_lossy(&bytes));
            }
        }

        Ok(RpcResponse {
            status: Some(status.as_u16()),
            data: bytes.to_vec(),
        })
    }
}
