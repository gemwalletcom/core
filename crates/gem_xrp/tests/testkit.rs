use gem_client::ReqwestClient;
use gem_xrp::rpc::XRPClient;

pub const TEST_ADDRESS: &str = "rnZmVGX6f4pUYyS4oXYJzoLdRojQV8y297";

pub fn create_xrp_test_client() -> XRPClient<ReqwestClient> {
    let reqwest_client = ReqwestClient::new("https://s1.ripple.com:51234/".to_string(), reqwest::Client::new());
    XRPClient::new(reqwest_client)
}
