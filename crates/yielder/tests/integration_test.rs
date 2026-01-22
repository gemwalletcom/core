#![cfg(feature = "yield_integration_tests")]

use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use gem_client::ReqwestClient;
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use primitives::{Chain, EVMChain};
use yielder::{
    YO_GATEWAY, YO_USD, YieldDetailsRequest, YieldError, YieldProvider, YieldProviderClient, Yielder, YoApiProvider, YoGatewayClient, YoPerformanceData,
    YoProvider, YoYieldProvider, build_performance_url, parse_performance_response,
};

fn base_rpc_url() -> String {
    std::env::var("BASE_RPC_URL").unwrap_or_else(|_| "https://gemnodes.com/base".to_string())
}

#[tokio::test]
async fn test_yields_for_asset_with_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let jsonrpc_client = JsonRpcClient::new_reqwest(base_rpc_url());
    let ethereum_client = EthereumClient::new(jsonrpc_client, EVMChain::Base);
    let gateway_client: Arc<dyn YoProvider> = Arc::new(YoGatewayClient::new(ethereum_client, YO_GATEWAY));
    let mut gateways = HashMap::new();
    gateways.insert(Chain::Base, gateway_client);
    let provider: Arc<dyn YieldProviderClient> = Arc::new(YoYieldProvider::new(gateways));
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
async fn test_yo_api_performance() {
    let url = build_performance_url(
        Chain::Base,
        "0x0000000f2eB9f69274678c76222B35eEc7588a65",
        "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7",
    )
    .expect("should build URL");

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await.expect("should fetch API");
    let data = response.bytes().await.expect("should get bytes");

    let performance = parse_performance_response(&data).expect("should parse response");

    println!("Yo API Performance:");
    println!("  Realized: {} (raw: {})", performance.realized.formatted, performance.realized.raw);
    println!("  Unrealized: {} (raw: {})", performance.unrealized.formatted, performance.unrealized.raw);
    println!("  Total rewards: {}", performance.total_rewards_raw());

    assert!(performance.total_rewards_raw() > 0, "should have some rewards");
}

struct ReqwestYoApiClient;

#[async_trait]
impl YoApiProvider for ReqwestYoApiClient {
    async fn get_user_performance(&self, chain: Chain, vault_address: &str, user_address: &str) -> Result<YoPerformanceData, YieldError> {
        let url = build_performance_url(chain, vault_address, user_address)?;
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await.map_err(|e| YieldError::new(e.to_string()))?;
        let data = response.bytes().await.map_err(|e| YieldError::new(e.to_string()))?;
        parse_performance_response(&data)
    }
}

#[tokio::test]
async fn test_yo_positions_with_rewards() {
    let http_client = ReqwestClient::new_test_client(base_rpc_url());
    let jsonrpc_client = JsonRpcClient::new(http_client);
    let eth_client = EthereumClient::new(jsonrpc_client, EVMChain::Base);
    let gateway: Arc<dyn YoProvider> = Arc::new(YoGatewayClient::new(eth_client, YO_GATEWAY));
    let mut gateways = HashMap::new();
    gateways.insert(Chain::Base, gateway);

    let api_client: Arc<dyn YoApiProvider> = Arc::new(ReqwestYoApiClient);
    let provider = YoYieldProvider::new(gateways).with_api_client(api_client);

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
    println!("  Rewards: {:?}", position.rewards);

    assert!(position.rewards.is_some(), "rewards should be present");
}
