#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use futures::TryFutureExt;
    use gemstone::{
        config::swap_config::{get_swap_config, SwapReferralFee, SwapReferralFees},
        network::{provider::AlienProvider, target::*, *},
        swapper::{across::Across, cetus::Cetus, mayan::MayanSwiftProvider, models::*, orca::Orca, uniswap::v4::UniswapV4, GemSwapper, *},
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
                (Chain::Sui, "https://fullnode.mainnet.sui.io".into()),
                (Chain::Abstract, "https://api.mainnet.abs.xyz".into()),
                (Chain::Unichain, "https://mainnet.unichain.org".into()),
                (Chain::SmartChain, "https://binance.llamarpc.com".into()),
                (Chain::Linea, "https://rpc.linea.build".into()),
                (Chain::Base, "https://mainnet.base.org".into()),
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
        let swap_provider = Orca::boxed();
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
    async fn test_swapper_get_quote_by_output() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swapper = GemSwapper::new(network_provider);

        let trade_pairs: HashMap<Chain, (AssetId, AssetId)> = HashMap::from([
            (
                Chain::Abstract,
                (
                    AssetId::from_chain(Chain::Abstract),
                    AssetId::from(Chain::Abstract, Some("0x84A71ccD554Cc1b02749b35d22F684CC8ec987e1".to_string())),
                ),
            ),
            (
                Chain::Ethereum,
                (
                    AssetId::from_chain(Chain::Ethereum),
                    AssetId::from(Chain::Ethereum, Some("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string())),
                ),
            ),
        ]);

        let (from_asset, to_asset) = trade_pairs.get(&Chain::Abstract).cloned().unwrap();

        let options = GemSwapOptions {
            slippage: 100.into(),
            fee: Some(SwapReferralFees::evm(SwapReferralFee {
                bps: 25,
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
            })),
            preferred_providers: vec![],
        };

        let request = SwapQuoteRequest {
            from_asset,
            to_asset,
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "20000000000000000".into(), // 0.02 ETH
            mode: GemSwapMode::ExactIn,
            options,
        };

        let quotes = swapper.fetch_quote(&request).await?;
        assert_eq!(quotes.len(), 1);

        let quote = &quotes[0];
        println!("<== quote: {:?}", quote);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);

        let quote_data = swapper.fetch_quote_data(&quote, FetchQuoteData::EstimateGas).await?;
        println!("<== quote_data: {:?}", quote_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_across_quote() -> Result<(), SwapperError> {
        let swap_provider = Across::boxed();
        let network_provider = Arc::new(NativeProvider::default());
        let mut options = GemSwapOptions {
            slippage: 100.into(),
            fee: Some(SwapReferralFees::evm(SwapReferralFee {
                bps: 25,
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
            })),
            preferred_providers: vec![],
        };
        options.fee.as_mut().unwrap().evm_bridge = SwapReferralFee {
            bps: 25,
            address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
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

    #[tokio::test]
    async fn test_v4_quoter() -> Result<(), SwapperError> {
        let swap_provider = UniswapV4::boxed();
        let network_provider = Arc::new(NativeProvider::default());
        let options = GemSwapOptions {
            slippage: 100.into(),
            fee: Some(SwapReferralFees::evm(SwapReferralFee {
                bps: 25,
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
            })),
            preferred_providers: vec![SwapProvider::UniswapV4],
        };

        let request = SwapQuoteRequest {
            from_asset: AssetId::from_chain(Chain::Unichain),
            to_asset: AssetId::from(Chain::Unichain, Some("0x078D782b760474a361dDA0AF3839290b0EF57AD6".to_string())),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "10000000000000000".into(), // 0.01 ETH
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

    #[tokio::test]
    async fn test_cetus_swap() -> Result<(), Box<dyn std::error::Error>> {
        let swap_provider = Cetus::boxed();
        let network_provider = Arc::new(NativeProvider::default());
        let config = get_swap_config();
        let options = GemSwapOptions {
            slippage: 50.into(),
            fee: Some(config.referral_fee),
            preferred_providers: vec![],
        };

        let request = SwapQuoteRequest {
            from_asset: AssetId::from_chain(Chain::Sui),
            to_asset: AssetId {
                chain: Chain::Sui,
                token_id: Some("0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC".into()),
            },
            wallet_address: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".into(),
            destination_address: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".into(),
            value: "1500000000".into(), // 1.5 SUI
            mode: GemSwapMode::ExactIn,
            options,
        };

        let quote = swap_provider.fetch_quote(&request, network_provider.clone()).await?;
        println!("{:?}", quote);

        let quote_data = swap_provider.fetch_quote_data(&quote, network_provider.clone(), FetchQuoteData::None).await?;
        println!("{:?}", quote_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_mayan_swift_quote() -> Result<(), SwapperError> {
        const TEST_WALLET_ADDRESS: &str = "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24";

        let swap_provider = MayanSwiftProvider::default();
        let network_provider = Arc::new(NativeProvider::default());

        // Create a swap quote request
        let request = SwapQuoteRequest {
            from_asset: AssetId::from(Chain::Base, Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string())),
            to_asset: AssetId::from_chain(Chain::Optimism),
            wallet_address: TEST_WALLET_ADDRESS.to_string(),
            destination_address: TEST_WALLET_ADDRESS.to_string(),
            value: "9000000".to_string(),
            mode: GemSwapMode::ExactIn,
            options: GemSwapOptions {
                slippage: 10.into(),
                fee: None,
                preferred_providers: vec![],
            },
        };

        let quote = swap_provider.fetch_quote(&request, network_provider.clone()).await?;

        assert_eq!(quote.from_value, "9000000");
        // Expect the to_value to be
        assert!(quote.to_value.parse::<i64>().unwrap() > 0);

        // Verify
        assert_eq!(quote.data.routes.len(), 1);
        assert_eq!(
            quote.data.routes[0].input,
            AssetId::from(Chain::Base, Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string())),
        );
        assert_eq!(quote.data.routes[0].output, AssetId::from_chain(Chain::Optimism));

        Ok(())
    }
}
