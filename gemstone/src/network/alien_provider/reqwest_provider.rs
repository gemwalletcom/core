use super::{provider::Data, AlienError, AlienHttpMethod, AlienProvider, AlienTarget};
use primitives::{node_config::get_nodes_for_chain, Chain};

use async_trait::async_trait;
use futures::{future::try_join_all, TryFutureExt};
use reqwest::Client;

#[derive(Debug)]
pub struct NativeProvider {
    pub client: Client,
}

impl NativeProvider {
    pub fn new() -> Self {
        Self { client: Client::new() }
    }
}

impl Default for NativeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AlienProvider for NativeProvider {
    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        let nodes = get_nodes_for_chain(chain);
        if nodes.is_empty() {
            return Err(AlienError::ResponseError {
                msg: format!("not supported chain: {:?}", chain),
            });
        }
        Ok(nodes[0].url.clone())
    }

    async fn request(&self, target: AlienTarget) -> Result<Data, AlienError> {
        println!("==> request: url: {:?}, method: {:?}", target.url, target.method);
        let mut req = match target.method {
            AlienHttpMethod::Get => self.client.get(target.url),
            AlienHttpMethod::Post => self.client.post(target.url),
            AlienHttpMethod::Put => self.client.put(target.url),
            AlienHttpMethod::Delete => self.client.delete(target.url),
            AlienHttpMethod::Head => self.client.head(target.url),
            AlienHttpMethod::Patch => self.client.patch(target.url),
            AlienHttpMethod::Options => todo!(),
        };
        if let Some(headers) = target.headers {
            for header in headers.iter() {
                req = req.header(&header.key, &header.value);
            }
        }
        if let Some(body) = target.body {
            if cfg!(debug_assertions) && body.len() <= 4096 {
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
                    println!("=== json: {:?}", json);
                } else {
                    println!("=== body: {:?}", String::from_utf8(body.to_vec()).unwrap());
                }
            }
            req = req.body(body);
        }

        let response = req
            .send()
            .map_err(|e| AlienError::ResponseError {
                msg: format!("reqwest send error: {:?}", e),
            })
            .await?;
        let bytes = response
            .bytes()
            .map_err(|e| AlienError::ResponseError {
                msg: format!("request error: {:?}", e),
            })
            .await?;
        println!("<== response body size: {:?}", bytes.len());
        if cfg!(debug_assertions) && bytes.len() <= 4096 {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                println!("=== json: {:?}", json);
            } else {
                println!("=== body: {:?}", String::from_utf8(bytes.to_vec()).unwrap());
            }
        }
        Ok(bytes.to_vec())
    }

    async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
        let futures = targets.into_iter().map(|target| self.request(target));
        try_join_all(futures).await
    }
}
