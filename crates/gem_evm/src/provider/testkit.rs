pub const TEST_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";
pub const TOKEN_USDC_ADDRESS: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
pub const TOKEN_DAI_ADDRESS: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
pub const TEST_SMARTCHAIN_STAKING_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
use primitives::FeeRate;

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
fn get_test_settings() -> settings::Settings {
    let settings_path = std::env::current_dir().expect("Failed to get current directory").join("../../Settings.yaml");
    settings::Settings::new_setting_path(settings_path).expect("Failed to load settings for tests")
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
fn build_test_client(chain: primitives::EVMChain, rpc_url: &str) -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    use crate::rpc::{alchemy::client::alchemy_url, ankr::AnkrClient, client::EthereumClient, AlchemyClient};
    use gem_client::ReqwestClient;
    use gem_jsonrpc::JsonRpcClient;

    let settings = get_test_settings();
    let rpc_client = JsonRpcClient::new_reqwest(rpc_url.to_string());

    let http_client = reqwest::Client::builder().build().expect("Failed to build reqwest client for tests");
    let alchemy_client = AlchemyClient::new(ReqwestClient::new(alchemy_url(&settings.alchemy.key.secret), http_client.clone()), chain);

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
    build_test_client(primitives::EVMChain::Ethereum, "https://eth.llamarpc.com")
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_smartchain_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    build_test_client(primitives::EVMChain::SmartChain, "https://bsc-dataseed4.bnbchain.org")
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_polygon_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    build_test_client(primitives::EVMChain::Polygon, "https://polygon.llamarpc.com")
}

#[cfg(all(test, feature = "rpc", feature = "reqwest"))]
pub fn create_arbitrum_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    build_test_client(primitives::EVMChain::Arbitrum, "https://arb1.arbitrum.io/rpc")
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
