#![cfg(feature = "yield_integration_tests")]

use std::{collections::HashMap, sync::Arc};

use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_jsonrpc::NativeProvider;
use primitives::{Chain, EVMChain};
use yielder::{YO_GATEWAY, YO_USDC, YieldDetailsRequest, YieldProviderClient, Yielder, YoGatewayClient, YoProvider, YoYieldProvider};

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

#[tokio::test]
async fn test_yields_for_asset() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let provider = NativeProvider::new().set_debug(false);
    let gateways = build_gateways(&provider);
    let yo_provider: Arc<dyn YieldProviderClient> = Arc::new(YoYieldProvider::new(gateways));

    let yields = yo_provider.yields(&YO_USDC.asset_id());
    println!("yielder: yields_for_asset count={}", yields.len());
    assert!(!yields.is_empty(), "expected at least one Yo vault for asset");

    Ok(())
}

#[tokio::test]
async fn test_yields_for_asset_with_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let provider = NativeProvider::new().set_debug(false);
    let gateways = build_gateways(&provider);
    let yo_provider: Arc<dyn YieldProviderClient> = Arc::new(YoYieldProvider::new(gateways));

    let yields = yo_provider.yields_with_apy(&YO_USDC.asset_id()).await?;
    println!("yielder: yields_for_asset_with_apy count={}", yields.len());
    assert!(!yields.is_empty(), "expected at least one Yo vault for asset");

    let apy = yields[0].apy.expect("apy should be computed");
    println!("yielder: first Yo APY={:.2}%", apy);
    assert!(apy.is_finite(), "apy should be finite");
    assert!(apy > -1.0, "apy should be > -100%");

    Ok(())
}

#[tokio::test]
async fn test_yo_positions() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let provider = NativeProvider::new().set_debug(false);
    let gateways = build_gateways(&provider);
    let yo_provider = YoYieldProvider::new(gateways);

    let wallet_address = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
    let request = YieldDetailsRequest {
        asset_id: YO_USDC.asset_id(),
        wallet_address: wallet_address.to_string(),
    };

    let position = yo_provider.positions(&request).await?;
    println!(
        "yielder: position vault_balance={:?} asset_balance={:?}",
        position.vault_balance_value, position.asset_balance_value
    );

    assert!(position.vault_balance_value.is_some(), "vault balance should be present");
    assert!(position.asset_balance_value.is_some(), "asset balance should be present");

    Ok(())
}
