pub const TEST_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";
pub const TOKEN_USDC_ADDRESS: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
pub const TOKEN_DAI_ADDRESS: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
pub const TEST_SMARTCHAIN_STAKING_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use primitives::FeeRate;

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
fn build_test_client(chain: primitives::EVMChain, rpc_url: &str) -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    use crate::rpc::{AlchemyClient, ankr::AnkrClient, client::EthereumClient};
    use gem_client::ReqwestClient;
    use gem_jsonrpc::JsonRpcClient;

    let settings = get_test_settings();
    let rpc_client = JsonRpcClient::new_reqwest(rpc_url.to_string());

    let alchemy_client = AlchemyClient::new(
        ReqwestClient::new_test_client(format!("https://api.g.alchemy.com/data/v1/{}", settings.alchemy.key.secret)),
        chain,
    );

    let ankr_client = AnkrClient::new(
        JsonRpcClient::new_reqwest(format!("https://rpc.ankr.com/multichain/{}", settings.ankr.key.secret)),
        chain,
    );

    EthereumClient::new(rpc_client, chain)
        .with_alchemy_client(alchemy_client)
        .with_ankr_client(ankr_client)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_ethereum_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(primitives::EVMChain::Ethereum, &settings.chains.ethereum.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_smartchain_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(primitives::EVMChain::SmartChain, &settings.chains.smartchain.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_polygon_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(primitives::EVMChain::Polygon, &settings.chains.polygon.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_arbitrum_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(primitives::EVMChain::Arbitrum, &settings.chains.arbitrum.url)
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn print_fee_rates(fee_rates: Vec<FeeRate>) {
    for fee_rate in &fee_rates {
        use crate::ether_conv;
        println!(
            "Fee rate: {:?} total: {}, gas_price: {}, priority_fee: {}",
            fee_rate.priority,
            ether_conv::EtherConv::to_gwei(&fee_rate.gas_price_type.total_fee()),
            ether_conv::EtherConv::to_gwei(&fee_rate.gas_price_type.gas_price()),
            ether_conv::EtherConv::to_gwei(&fee_rate.gas_price_type.priority_fee())
        );
    }
}
