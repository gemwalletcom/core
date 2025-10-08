#![cfg(all(feature = "reqwest_provider", feature = "swap_integration_tests"))]

#[cfg(test)]
mod network_tests {
    use gem_solana::{jsonrpc::SolanaRpc, models::blockhash::SolanaBlockhashResult};
    use primitives::{AssetId, Chain};
    use std::{sync::Arc, time::SystemTime};
    use swapper::{
        FetchQuoteData, NativeProvider, Options, QuoteRequest, RpcClient, SwapperError, SwapperMode, SwapperProvider,
        client_factory::create_client_with_chain,
        config::{ReferralFee, ReferralFees, get_swap_config},
    };
    use swapper::{across::Across, cetus::Cetus, uniswap};

    #[tokio::test]
    async fn test_solana_json_rpc() -> Result<(), String> {
        let rpc_client = create_client_with_chain(Arc::new(NativeProvider::default()), Chain::Solana);
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
    async fn test_across_quote() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = Across::boxed(network_provider.clone());
        let mut options = Options {
            slippage: 100.into(),
            fee: Some(ReferralFees::evm(ReferralFee {
                bps: 25,
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
            })),
            preferred_providers: vec![],
        };
        options.fee.as_mut().unwrap().evm_bridge = ReferralFee {
            bps: 25,
            address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
        };

        let request = QuoteRequest {
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
        let swap_provider = uniswap::default::boxed_uniswap_v4(network_provider.clone());
        let options = Options {
            slippage: 100.into(),
            fee: Some(ReferralFees::evm(ReferralFee {
                bps: 25,
                address: "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC".into(),
            })),
            preferred_providers: vec![SwapperProvider::UniswapV4],
        };

        let request = QuoteRequest {
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
        let swap_provider = Cetus::<RpcClient>::boxed(network_provider.clone());
        let config = get_swap_config();
        let options = Options {
            slippage: 50.into(),
            fee: Some(config.referral_fee),
            preferred_providers: vec![],
        };

        let request = QuoteRequest {
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
