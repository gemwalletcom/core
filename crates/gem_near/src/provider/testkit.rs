#[cfg(all(test, feature = "integration_tests"))]
use crate::rpc::client::NearClient;
#[cfg(all(test, feature = "integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "integration_tests"))]
use gem_jsonrpc::new_client;

#[cfg(all(test, feature = "integration_tests"))]
pub const TEST_ADDRESS: &str = "75b4f90dc729b28ce1a3d44b2c96b3943136f1d7ced0b5df1fc23662439e3e3c";

#[cfg(all(test, feature = "integration_tests"))]
pub fn create_near_test_client() -> Result<NearClient<ReqwestClient>, Box<dyn std::error::Error + Send + Sync>> {
    let jsonrpc_client = new_client("https://rpc.mainnet.near.org".to_string())?;
    Ok(NearClient::new(jsonrpc_client))
}
