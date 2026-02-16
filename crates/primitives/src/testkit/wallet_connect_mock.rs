use crate::WalletConnectRequest;

impl WalletConnectRequest {
    pub fn mock(method: &str, params: &str, chain_id: Option<&str>) -> Self {
        Self {
            topic: "test-topic".to_string(),
            method: method.to_string(),
            params: params.to_string(),
            chain_id: chain_id.map(|v| v.to_string()),
            domain: "example.com".to_string(),
        }
    }
}
