#[cfg(all(feature = "rpc", feature = "reqwest"))]
use primitives::FeeRate;

pub const TEST_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";
pub const TOKEN_USDC_ADDRESS: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
pub const TOKEN_DAI_ADDRESS: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
pub const TEST_SMARTCHAIN_STAKING_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub fn create_ethereum_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    use crate::rpc::client::EthereumClient;
    use gem_jsonrpc::JsonRpcClient;
    use primitives::EVMChain;
    let client = JsonRpcClient::new_reqwest("https://eth.llamarpc.com".to_string());
    EthereumClient::new(client, EVMChain::Ethereum)
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub fn create_polygon_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    use crate::rpc::client::EthereumClient;
    use gem_jsonrpc::JsonRpcClient;
    use primitives::EVMChain;
    let client = JsonRpcClient::new_reqwest("https://polygon.llamarpc.com".to_string());
    EthereumClient::new(client, EVMChain::Polygon)
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub fn create_arbitrum_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    use crate::rpc::client::EthereumClient;
    use gem_jsonrpc::JsonRpcClient;
    use primitives::EVMChain;
    let client = JsonRpcClient::new_reqwest("https://arb1.arbitrum.io/rpc".to_string());
    EthereumClient::new(client, EVMChain::Arbitrum)
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub fn create_smartchain_test_client() -> crate::rpc::client::EthereumClient<gem_client::ReqwestClient> {
    use crate::rpc::client::EthereumClient;
    use gem_jsonrpc::JsonRpcClient;
    use primitives::EVMChain;
    let client = JsonRpcClient::new_reqwest("https://bsc-dataseed4.bnbchain.org".to_string());
    EthereumClient::new(client, EVMChain::SmartChain)
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
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
