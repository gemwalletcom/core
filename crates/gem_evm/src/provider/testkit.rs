pub use crate::testkit::{TEST_ADDRESS, TEST_MONAD_ADDRESS, TEST_SMARTCHAIN_STAKING_ADDRESS, TEST_TRANSACTION_ID, TOKEN_DAI_ADDRESS, TOKEN_USDC_ADDRESS};

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use primitives::FeeRate;

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
fn build_test_client(chain: primitives::EVMChain, rpc_url: &str) -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    use crate::rpc::{ankr::AnkrClient, client::EthereumClient};
    use gem_jsonrpc::JsonRpcClient;

    let settings = get_test_settings();
    let rpc_client = JsonRpcClient::new_reqwest(rpc_url.to_string());

    let ankr_client = AnkrClient::new(JsonRpcClient::new_reqwest(format!("https://rpc.ankr.com/multichain/{}", settings.ankr.key.secret)), chain);

    EthereumClient::new(rpc_client, chain).with_ankr_client(ankr_client)
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
pub fn create_monad_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    let settings = get_test_settings();
    build_test_client(primitives::EVMChain::Monad, &settings.chains.monad.url)
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
