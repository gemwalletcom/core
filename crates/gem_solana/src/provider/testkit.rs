#[cfg(feature = "chain_integration_tests")]
use crate::rpc::client::SolanaClient;
#[cfg(feature = "chain_integration_tests")]
use gem_client::ReqwestClient;
#[cfg(feature = "chain_integration_tests")]
use gem_jsonrpc::JsonRpcClient;
#[cfg(feature = "chain_integration_tests")]
use settings::testkit::get_test_settings;

#[cfg(feature = "chain_integration_tests")]
pub const TEST_ADDRESS: &str = "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H";

#[cfg(feature = "chain_integration_tests")]
pub fn create_solana_test_client() -> SolanaClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new_test_client(settings.chains.solana.url);
    let rpc_client = JsonRpcClient::new(reqwest_client);
    SolanaClient::new(rpc_client)
}
