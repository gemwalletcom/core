#![cfg(feature = "yield_integration_tests")]

use std::{collections::HashMap, sync::Arc};

use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::{NativeProvider, RpcProvider};
use primitives::{Chain, EVMChain};
use yielder::{YO_GATEWAY, YO_USDC, YieldDetailsRequest, YieldProviderClient, Yielder, YoApiClient, YoGatewayClient, YoProvider, YoYieldProvider};

fn get_endpoint(provider: &NativeProvider, chain: Chain) -> String {
    provider.get_endpoint(chain).unwrap_or_else(|err| panic!("missing RPC endpoint for chain {chain:?}: {err}"))
}

fn build_gateways(provider: &NativeProvider) -> HashMap<Chain, Arc<dyn YoProvider>> {
    let base_client = EthereumClient::new(JsonRpcClient::new_reqwest(get_endpoint(provider, Chain::Base)), EVMChain::Base);
    let ethereum_client = EthereumClient::new(JsonRpcClient::new_reqwest(get_endpoint(provider, Chain::Ethereum)), EVMChain::Ethereum);

    println!("yielder: using gateway endpoints for Base and Ethereum");
    HashMap::from([
        (Chain::Base, Arc::new(YoGatewayClient::new(base_client, YO_GATEWAY)) as Arc<dyn YoProvider>),
        (Chain::Ethereum, Arc::new(YoGatewayClient::new(ethereum_client, YO_GATEWAY)) as Arc<dyn YoProvider>),
    ])
}

fn build_rpc_provider() -> Arc<NativeProvider> {
    Arc::new(NativeProvider::new().set_debug(false))
}

#[tokio::test]
async fn test_yields_for_asset_with_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rpc_provider = build_rpc_provider();
    let gateways = build_gateways(&rpc_provider);
    let provider: Arc<dyn YieldProviderClient> = Arc::new(YoYieldProvider::new(gateways, rpc_provider));
    let yielder = Yielder::with_providers(vec![provider]);

    let apy_yields = yielder.yields_for_asset_with_apy(&YO_USDC.asset_id()).await?;
    println!("yielder: yields_for_asset_with_apy count={}", apy_yields.len());
    assert!(!apy_yields.is_empty(), "expected at least one Yo vault for asset");
    let apy = apy_yields[0].apy.expect("apy should be computed");
    println!("yielder: first Yo APY={}", apy);
    assert!(apy.is_finite(), "apy should be finite");
    assert!(apy > -1.0, "apy should be > -100%");

    Ok(())
}

#[tokio::test]
async fn test_yo_api_performance() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rpc_provider = build_rpc_provider();
    let api_client = YoApiClient::new(rpc_provider);

    let vault_address = YO_USDC.yo_token.to_string();
    let wallet_address = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";

    println!("yielder: fetch_rewards chain=Base vault={vault_address} wallet={wallet_address}");

    let performance = api_client.fetch_rewards(Chain::Base, &vault_address, wallet_address).await?;

    println!("yielder: rewards total_raw={}", performance.total_rewards_raw(),);
    assert!(performance.total_rewards_raw() > 0, "expected rewards for test address");

    Ok(())
}

#[tokio::test]
async fn test_yo_positions_with_rewards() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rpc_provider = build_rpc_provider();
    let gateways = build_gateways(&rpc_provider);
    let provider = YoYieldProvider::new(gateways, rpc_provider);

    let wallet_address = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
    let request = YieldDetailsRequest {
        asset_id: YO_USDC.asset_id(),
        wallet_address: wallet_address.to_string(),
    };

    let position = provider.positions(&request).await?;
    println!(
        "yielder: position vault_balance={:?} asset_balance={:?} apy={:?} rewards={:?}",
        position.vault_balance_value, position.asset_balance_value, position.apy, position.rewards
    );

    assert!(position.vault_balance_value.is_some(), "vault balance should be present");
    assert!(position.asset_balance_value.is_some(), "asset balance should be present");
    assert!(position.apy.is_some(), "apy should be present");
    assert!(position.rewards.is_some(), "rewards should be present");

    Ok(())
}
