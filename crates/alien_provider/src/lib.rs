use async_trait::async_trait;
use futures::TryFutureExt;
use gemstone::network::{provider::AlienProvider, target::*, *};
use primitives::Chain;
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug)]
pub struct NativeProvider {
    pub node_config: HashMap<Chain, String>,
    pub client: Client,
}

impl NativeProvider {
    pub fn new(node_config: HashMap<Chain, String>) -> Self {
        Self {
            node_config,
            client: Client::new(),
        }
    }
}

impl Default for NativeProvider {
    fn default() -> Self {
        Self::new(HashMap::from([
            (Chain::Ethereum, "https://eth.llamarpc.com".into()),
            (Chain::Optimism, "https://optimism.llamarpc.com".into()),
            (Chain::Arbitrum, "https://arbitrum.llamarpc.com".into()),
            (Chain::Solana, "https://solana-rpc.publicnode.com".into()),
            (Chain::Sui, "https://fullnode.mainnet.sui.io".into()),
            (Chain::Abstract, "https://api.mainnet.abs.xyz".into()),
            (Chain::Unichain, "https://mainnet.unichain.org".into()),
            (Chain::SmartChain, "https://binance.llamarpc.com".into()),
            (Chain::Linea, "https://rpc.linea.build".into()),
        ]))
    }
}

#[async_trait]
impl AlienProvider for NativeProvider {
    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
        Ok(self
            .node_config
            .get(&chain)
            .ok_or(AlienError::ResponseError {
                msg: "not supported chain".into(),
            })?
            .to_string())
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
            for (key, value) in headers.iter() {
                req = req.header(key, value);
            }
        }
        if let Some(body) = target.body {
            if body.len() <= 4096 {
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
        if bytes.len() <= 4096 {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                println!("=== json: {:?}", json);
            } else {
                println!("=== body: {:?}", String::from_utf8(bytes.to_vec()).unwrap());
            }
        }
        Ok(bytes.to_vec())
    }

    async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
        let mut futures = vec![];
        for target in targets.iter() {
            let future = self.request(target.clone());
            futures.push(future);
        }
        let responses = futures::future::join_all(futures).await;
        let error = responses.iter().find_map(|x| x.as_ref().err());
        if let Some(err) = error {
            return Err(err.clone());
        }
        let responses = responses.into_iter().filter_map(|x| x.ok()).collect();
        Ok(responses)
    }
}
