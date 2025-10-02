#![cfg(feature = "reqwest_provider")]

#[cfg(test)]
mod tests {
    use gem_solana::{jsonrpc::SolanaRpc, models::blockhash::SolanaBlockhashResult};
    use gemstone::{
        config::swap_config::{SwapReferralFee, SwapReferralFees, get_swap_config},
        network::{alien_provider::NativeProvider, jsonrpc_client_with_chain},
        swapper::{GemSwapper, across::Across, cetus::Cetus, models::*, uniswap::v4::UniswapV4, *},
    };
    use primitives::{AssetId, Chain};
    use std::{collections::HashMap, sync::Arc, time::SystemTime};

    #[tokio::test]
    async fn test_solana_json_rpc() -> Result<(), String> {
        let rpc_client = jsonrpc_client_with_chain(Arc::new(NativeProvider::default()), Chain::Solana);
        let response: SolanaBlockhashResult = rpc_client.request(SolanaRpc::GetLatestBlockhash).await.map_err(|e| e.to_string())?;
        let recent_blockhash = response.value.blockhash;

        println!("recent_blockhash: {}", recent_blockhash);

        let blockhash = bs58::decode(recent_blockhash)
            .into_vec()
            .map_err(|_| "Failed to decode blockhash".to_string())?;

        let blockhash_array: [u8; 32] = blockhash.try_into().map_err(|_| "Failed to convert blockhash to array".to_string())?;

        assert_eq!(blockhash_array.len(), 32);

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

        let options = SwapperOptions {
            slippage: 100.into(),
            fee: Some(SwapReferralFees::evm(SwapReferralFee {
                bps: 25,
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
            })),
            preferred_providers: vec![],
        };

        let request = SwapperQuoteRequest {
            from_asset: from_asset.into(),
            to_asset: to_asset.into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "20000000000000000".into(), // 0.02 ETH
            mode: SwapperMode::ExactIn,
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
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = Across::boxed(network_provider.clone());
        let mut options = SwapperOptions {
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

        let request = SwapperQuoteRequest {
            from_asset: AssetId::from_chain(Chain::Optimism).into(),
            to_asset: AssetId::from_chain(Chain::Arbitrum).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "20000000000000000".into(), // 0.02 ETH
            mode: SwapperMode::ExactIn,
            options,
        };

        let now = SystemTime::now();
        let quote = swap_provider.fetch_quote(&request).await?;
        let elapsed = SystemTime::now().duration_since(now).unwrap();

        println!("<== elapsed: {:?}", elapsed);
        println!("<== quote: {:?}", quote);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);

        let quote_data = swap_provider.fetch_quote_data(&quote, FetchQuoteData::EstimateGas).await?;
        println!("<== quote_data: {:?}", quote_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_v4_quoter() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = UniswapV4::boxed(network_provider.clone());
        let options = SwapperOptions {
            slippage: 100.into(),
            fee: Some(SwapReferralFees::evm(SwapReferralFee {
                bps: 25,
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
            })),
            preferred_providers: vec![SwapperProvider::UniswapV4],
        };

        let request = SwapperQuoteRequest {
            from_asset: AssetId::from_chain(Chain::Unichain).into(),
            to_asset: AssetId::from(Chain::Unichain, Some("0x078D782b760474a361dDA0AF3839290b0EF57AD6".to_string())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "10000000000000000".into(), // 0.01 ETH
            mode: SwapperMode::ExactIn,
            options,
        };

        let now = SystemTime::now();
        let quote = swap_provider.fetch_quote(&request).await?;
        let elapsed = SystemTime::now().duration_since(now).unwrap();

        println!("<== elapsed: {:?}", elapsed);
        println!("<== quote: {:?}", quote);
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);

        let quote_data = swap_provider.fetch_quote_data(&quote, FetchQuoteData::EstimateGas).await?;
        println!("<== quote_data: {:?}", quote_data);

        Ok(())
    }

    #[tokio::test]
    async fn test_cetus_swap() -> Result<(), Box<dyn std::error::Error>> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = Cetus::boxed(network_provider.clone());
        let config = get_swap_config();
        let options = SwapperOptions {
            slippage: 50.into(),
            fee: Some(config.referral_fee),
            preferred_providers: vec![],
        };

        let request = SwapperQuoteRequest {
            from_asset: AssetId::from_chain(Chain::Sui).into(),
            to_asset: AssetId {
                chain: Chain::Sui,
                token_id: Some("0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC".into()),
            }
            .into(),
            wallet_address: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".into(),
            destination_address: "0xa9bd0493f9bd1f792a4aedc1f99d54535a75a46c38fd56a8f2c6b7c8d75817a1".into(),
            value: "1500000000".into(), // 1.5 SUI
            mode: SwapperMode::ExactIn,
            options,
        };

        let quote = swap_provider.fetch_quote(&request).await?;
        println!("{:?}", quote);

        let quote_data = swap_provider.fetch_quote_data(&quote, FetchQuoteData::None).await?;
        println!("{:?}", quote_data);

        Ok(())
    }
}
