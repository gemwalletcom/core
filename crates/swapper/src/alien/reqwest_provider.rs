use super::{AlienError, HttpMethod, RpcProvider, Target, provider::Data};
use primitives::{Chain, node_config::get_nodes_for_chain};

use async_trait::async_trait;
use futures::{TryFutureExt, future::try_join_all};
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
impl RpcProvider for NativeProvider {
    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        let nodes = get_nodes_for_chain(chain);
        if nodes.is_empty() {
            return Err(AlienError::ResponseError {
                msg: format!("not supported chain: {chain:?}"),
            });
        }
        Ok(nodes[0].url.clone())
    }

    async fn request(&self, target: Target) -> Result<Data, AlienError> {
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

        let response = req
            .send()
            .map_err(|e| AlienError::ResponseError {
                msg: format!("reqwest send error: {e:?}"),
            })
            .await?;
        let bytes = response
            .bytes()
            .map_err(|e| AlienError::ResponseError {
                msg: format!("request error: {e:?}"),
            })
            .await?;
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
        Ok(bytes.to_vec())
    }

    async fn batch_request(&self, targets: Vec<Target>) -> Result<Vec<Data>, AlienError> {
        let futures = targets.into_iter().map(|target| self.request(target));
        try_join_all(futures).await
    }
}
