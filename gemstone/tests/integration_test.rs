#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use futures::TryFutureExt;
    use gemstone::{
        network::{provider::AlienProvider, target::*, *},
        swapper::{orca::Orca, *},
    };
    use mayan::mayan_swift_provider::MayanSwiftProvider;
    use primitives::{Asset, AssetId, Chain};
    use reqwest::Client;
    use std::{collections::HashMap, sync::Arc};

    #[derive(Debug)]
    pub struct NativeProvider {
        pub node_config: HashMap<String, String>,
        pub client: Client,
    }

    impl NativeProvider {
        pub fn new(node_config: HashMap<String, String>) -> Self {
            Self {
                node_config,
                client: Client::new(),
            }
        }
    }

    #[async_trait]
    impl AlienProvider for NativeProvider {
        fn get_endpoint(&self, chain: String) -> Result<String, AlienError> {
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
        let node_config = HashMap::from([(Chain::Solana.to_string(), "https://solana-rpc.publicnode.com".into())]);
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
        assert!(quote.to_value.parse::<i32>().unwrap() > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_mayan_swift_quote() -> Result<(), SwapperError> {
        const TEST_WALLET_ADDRESS: &str = "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24";

        let node_config = HashMap::from([(Chain::Base, "https://mainnet.base.org".into())]);
        let network_provider = Arc::new(NativeProvider::new(node_config));

        let mayan_swift_provider = MayanSwiftProvider::new();

        // Create a swap quote request
        let request = SwapQuoteRequest {
            from_asset: AssetId::from_chain(Chain::Base),
            to_asset: AssetId::from_chain(Chain::Ethereum),
            wallet_address: TEST_WALLET_ADDRESS.to_string(),
            destination_address: TEST_WALLET_ADDRESS.to_string(),
            value: "100000000000000000".to_string(),
            mode: GemSwapMode::ExactIn, // Swap mode
            options: None,
        };

        let quote = mayan_swift_provider.fetch_quote(&request, network_provider.clone()).await?;

        assert_eq!(quote.from_value, "100000000000000000");
        // Expect the to_value to be
        assert!(quote.to_value.parse::<i64>().unwrap() > 0);

        // Verify
        assert_eq!(quote.data.routes.len(), 1);
        assert_eq!(quote.data.routes[0].route_type, "swift-order");
        assert_eq!(quote.data.routes[0].input, "base");
        assert_eq!(quote.data.routes[0].output, "ethereum");
        assert_eq!(quote.data.routes[0].fee_tier, "0");
        // assert!(quote.data.routes[0].gas_estimate.is_some();

        // Verify
        // assert_eq!(quote.approval, ApprovalType::None);

        Ok(())
    }
}
