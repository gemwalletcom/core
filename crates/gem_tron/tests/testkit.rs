use gem_client::ReqwestClient;
use gem_tron::rpc::client::TronClient;

pub const TEST_ADDRESS: &str = "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb";

pub fn create_test_client() -> TronClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://api.trongrid.io".to_string(), reqwest::Client::new());
    TronClient::new(reqwest_client)
}
