#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use futures::TryFutureExt;
    use gemstone::swapper::across::Across;
    use gemstone::{
        network::{provider::AlienProvider, target::*, *},
        swapper::{orca::Orca, *},
    };
    use num_bigint::BigInt;
    use primitives::{AssetId, Chain};
    use reqwest::Client;
    use std::str::FromStr;
    use std::{collections::HashMap, sync::Arc};

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
                println!("==> request body size: {:?}", body.len());
                println!("==> request body: {:?}", String::from_utf8(body.clone()).unwrap());
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

    #[tokio::test]
    async fn test_orca_get_quote_by_input() -> Result<(), SwapperError> {
        let node_config = HashMap::from([(Chain::Solana, "https://solana-rpc.publicnode.com".into())]);
        let swap_provider: Box<dyn GemSwapProvider> = Box::new(Orca::default());
        let network_provider = Arc::new(NativeProvider::new(node_config));

        let request = SwapQuoteRequest {
            from_asset: AssetId::from(Chain::Solana, None),
            to_asset: AssetId::from(Chain::Solana, Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into())),
            wallet_address: "G7B17AigRCGvwnxFc5U8zY5T3NBGduLzT7KYApNU2VdR".into(),
            destination_address: "G7B17AigRCGvwnxFc5U8zY5T3NBGduLzT7KYApNU2VdR".into(),
            value: "1000000".into(),
            mode: GemSwapMode::ExactIn,
            options: None,
        };
        let quote = swap_provider.fetch_quote(&request, network_provider.clone()).await?;

        assert_eq!(quote.from_value, "1000000");
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_across_get_quote_by_input() -> Result<(), SwapperError> {
        let node_config = HashMap::from([(Chain::Base, "https://app.across.to".into())]);
        let swap_provider: Box<dyn GemSwapProvider> = Box::new(Across::default());
        let network_provider = Arc::new(NativeProvider::new(node_config));

        let request = SwapQuoteRequest {
            from_asset: AssetId::from(Chain::Base, Some("0x4200000000000000000000000000000000000006".into())),
            to_asset: AssetId::from(Chain::Optimism, Some("0x4200000000000000000000000000000000000006".into())),
            wallet_address: "G7B17AigRCGvwnxFc5U8zY5T3NBGduLzT7KYApNU2VdR".into(),
            destination_address: "G7B17AigRCGvwnxFc5U8zY5T3NBGduLzT7KYApNU2VdR".into(),
            value: "30000000000000000".into(),
            mode: GemSwapMode::ExactIn,
            options: None,
        };
        let quote = swap_provider.fetch_quote(&request, network_provider.clone()).await?;

        assert_eq!(quote.from_value, "30000000000000000");
        let value = BigInt::from_str(quote.to_value.as_str()).unwrap();
        println!("value: {}", value);
        assert!(value > BigInt::from(0));

        Ok(())
    }
}
