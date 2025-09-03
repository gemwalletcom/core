pub const TEST_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";
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
