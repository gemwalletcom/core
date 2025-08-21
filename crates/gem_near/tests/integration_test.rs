#[cfg(all(test, feature = "reqwest"))]
mod integration_tests {
    use gem_client::ReqwestClient;
    use gem_jsonrpc::{client::JsonRpcClient, new_client};
    use gem_near::rpc::client::NearClient;

    #[tokio::test]
    async fn test_near_client_generic_interface() {
        // This should compile, demonstrating the generic interface works
        let reqwest_client = ReqwestClient::new("https://example.com".to_string(), reqwest::Client::new());
        let jsonrpc_client = JsonRpcClient::new(reqwest_client);
        let near_client: NearClient<ReqwestClient> = NearClient::new(jsonrpc_client);

        // Test that basic properties work
        assert_eq!(near_client.get_chain().to_string(), "near");
    }

    #[tokio::test]
    async fn test_near_genesis_config() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Test actual RPC call
        let jsonrpc_client = new_client("https://rpc.mainnet.near.org".to_string())?;
        let near_client: NearClient<gem_client::ReqwestClient> = NearClient::new(jsonrpc_client);

        let genesis_config = near_client.get_genesis_config().await?;
        assert_eq!(genesis_config.chain_id, "mainnet");

        Ok(())
    }
}
