#![cfg(feature = "yield_integration_tests")]

use std::sync::Arc;

use alloy_primitives::U256;
use gem_client::ReqwestClient;
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use primitives::EVMChain;
use yielder::{
    YO_GATEWAY_BASE_MAINNET, YO_USD, YieldDetailsRequest, YieldProvider, YieldProviderClient, Yielder, YoGatewayClient, YoYieldProvider,
};

fn base_rpc_url() -> String {
    std::env::var("BASE_RPC_URL").unwrap_or_else(|_| "https://mainnet.base.org".to_string())
}

#[tokio::test]
async fn test_yields_for_asset_with_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let jsonrpc_client = JsonRpcClient::new_reqwest(base_rpc_url());
    let ethereum_client = EthereumClient::new(jsonrpc_client, EVMChain::Base);
    let gateway_client = YoGatewayClient::new(ethereum_client, YO_GATEWAY_BASE_MAINNET);
    let provider: Arc<dyn YieldProviderClient> = Arc::new(YoYieldProvider::new(Arc::new(gateway_client)));
    let yielder = Yielder::with_providers(vec![provider]);

    let apy_yields = yielder.yields_for_asset_with_apy(&YO_USD.asset_id()).await?;
    assert!(!apy_yields.is_empty(), "expected at least one Yo vault for asset");
    let apy = apy_yields[0].apy.expect("apy should be computed");
    assert!(apy.is_finite(), "apy should be finite");
    assert!(apy > -1.0, "apy should be > -100%");

    let details = yielder
        .positions(
            YieldProvider::Yo,
            &YieldDetailsRequest {
                asset_id: YO_USD.asset_id(),
                wallet_address: "0x0000000000000000000000000000000000000000".to_string(),
            },
        )
        .await?;

    assert!(details.apy.is_some(), "apy should be present in details");

    Ok(())
}

#[tokio::test]
async fn test_yo_positions() {
    let http_client = ReqwestClient::new_test_client(base_rpc_url());
    let jsonrpc_client = JsonRpcClient::new(http_client);
    let eth_client = EthereumClient::new(jsonrpc_client, EVMChain::Base);
    let gateway = Arc::new(YoGatewayClient::base_mainnet(eth_client.clone()));
    let gateway_client = YoGatewayClient::base_mainnet(eth_client);
    let provider = YoYieldProvider::new(gateway);

    let wallet_address = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
    let asset_id = YO_USD.asset_id();

    let request = YieldDetailsRequest {
        asset_id: asset_id.clone(),
        wallet_address: wallet_address.to_string(),
    };

    let position = provider.positions(&request).await.expect("should fetch positions");

    println!("Position for {wallet_address}:");
    println!("  Asset ID: {}", position.asset_id);
    println!("  Provider: {:?}", position.provider);
    println!("  Vault Token: {}", position.vault_token_address);
    println!("  Asset Token: {}", position.asset_token_address);
    println!("  Vault Balance (yoUSD shares): {:?}", position.vault_balance_value);
    println!("  Asset Balance (USDC): {:?}", position.asset_balance_value);
    println!("  APY: {:?}", position.apy);

    let mut total_usd = 0.0;

    if let Some(vault_balance) = &position.vault_balance_value {
        let shares: u128 = vault_balance.parse().unwrap_or(0);
        let shares_formatted = shares as f64 / 1_000_000.0;

        let shares_u256 = U256::from(shares);
        let assets = gateway_client
            .quote_convert_to_assets(YO_USD.yo_token, shares_u256)
            .await
            .expect("should convert shares to assets");
        let assets_value: u128 = assets.to_string().parse().unwrap_or(0);
        let assets_usd = assets_value as f64 / 1_000_000.0;

        println!("\n  yoUSD shares: {:.6} = ${:.6} USDC", shares_formatted, assets_usd);
        total_usd += assets_usd;
    }

    if let Some(asset_balance) = &position.asset_balance_value {
        let usdc: u128 = asset_balance.parse().unwrap_or(0);
        let usdc_formatted = usdc as f64 / 1_000_000.0;
        println!("  USDC balance: ${:.6}", usdc_formatted);
        total_usd += usdc_formatted;
    }

    println!("\n  TOTAL USD: ${:.2}", total_usd);
}
