use gem_client::ReqwestClient;
use gem_tron::rpc::client::TronClient;

pub fn create_test_client() -> TronClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://api.trongrid.io".to_string(), reqwest::Client::new());
    TronClient::new(reqwest_client)
}
