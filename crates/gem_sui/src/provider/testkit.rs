#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::SuiClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_jsonrpc::client::JsonRpcClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "0x93f65b8c16c263343bbf66cf9f8eef69cb1dbc92d13f0c331b0dcaeb76b4aab6";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_TOKEN_ADDRESS: &str = "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_TRANSACTION_ID: &str = "CJ16PEqq49KFp758iEVwxEkd3CwP7zDfqGYLuLuu9Z63";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_sui_test_client() -> SuiClient<ReqwestClient> {
    let settings = get_test_settings();
    let reqwest_client = ReqwestClient::new(settings.chains.sui.url, reqwest::Client::new());
    SuiClient::new(JsonRpcClient::new(reqwest_client))
}
