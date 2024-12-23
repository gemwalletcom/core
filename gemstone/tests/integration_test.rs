#[cfg(test)]
mod tests {
    use across::Across;
    use async_trait::async_trait;
    use futures::TryFutureExt;
    use gemstone::{
        config::swap_config::{SwapReferralFee, SwapReferralFees},
        network::{provider::AlienProvider, target::*, *},
        swapper::{orca::Orca, *},
    };
    use primitives::{AssetId, Chain};
    use reqwest::Client;
    use std::{collections::HashMap, sync::Arc, time::SystemTime};

    pub fn print_json(bytes: &[u8]) {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(bytes) {
            println!("=== json: {:?}", json);
        } else {
            println!("=== body: {:?}", String::from_utf8(bytes.to_vec()).unwrap());
        }
    }

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
                print_json(&body);
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
                print_json(&bytes);
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
            options: GemSwapOptions::default(),
        };
        let quote = swap_provider.fetch_quote(&request, network_provider.clone()).await?;

        assert_eq!(quote.from_value, "1000000");
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_across_quote() -> Result<(), SwapperError> {
        let swap_provider = Across::boxed();
        let network_provider = Arc::new(NativeProvider::default());
        let options = GemSwapOptions {
            slippage_bps: 100,
            fee: Some(SwapReferralFees::evm(SwapReferralFee {
                bps: 25,
                address: "0x3d83ec320541ae96c4c91e9202643870458fb290".into(),
            })),
            preferred_providers: vec![],
        };

        let request = SwapQuoteRequest {
            from_asset: AssetId::from_chain(Chain::Optimism),
            to_asset: AssetId::from_chain(Chain::Arbitrum),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "20000000000000000".into(), // 0.02 ETH
            mode: GemSwapMode::ExactIn,
            options,
        };

        let now = SystemTime::now();
        let quote = swap_provider.fetch_quote(&request, network_provider.clone()).await?;
        let elapsed = SystemTime::now().duration_since(now).unwrap();

        println!("<== elapsed: {:?}", elapsed);
        println!("<== quote: {:?}", quote);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);

        let quote_data = swap_provider
            .fetch_quote_data(&quote, network_provider.clone(), FetchQuoteData::EstimateGas)
            .await?;
        println!("<== quote_data: {:?}", quote_data);

        Ok(())
    }
}
