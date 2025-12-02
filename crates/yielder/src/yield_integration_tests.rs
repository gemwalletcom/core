#![cfg(all(test, feature = "yield_integration_tests"))]

use std::sync::Arc;

use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::client::JsonRpcClient;
use primitives::EVMChain;

use crate::{YO_GATEWAY_BASE_MAINNET, YO_USD, YieldDetailsRequest, YieldProvider, YieldProviderClient, Yielder, YoGatewayClient, YoYieldProvider};

#[tokio::test]
async fn yield_integration_test_fetches_performance_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rpc_url = std::env::var("BASE_RPC_URL").unwrap_or_else(|_| "https://mainnet.base.org".to_string());
    let jsonrpc_client = JsonRpcClient::new_reqwest(rpc_url);
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
