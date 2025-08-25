use gem_client::ReqwestClient;
use gem_jsonrpc::client::JsonRpcClient;
use gem_sui::SuiClient;

pub fn create_sui_test_client() -> SuiClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://fullnode.mainnet.sui.io:443".to_string(), reqwest::Client::new());
    SuiClient::new(JsonRpcClient::new(reqwest_client))
}
