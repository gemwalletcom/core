use gem_client::ReqwestClient;
use gem_stellar::rpc::client::StellarClient;

pub fn create_test_client() -> StellarClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://horizon.stellar.org".to_string(), reqwest::Client::new());
    StellarClient::new(reqwest_client)
}
