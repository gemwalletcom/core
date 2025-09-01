#[cfg(test)]
mod tests {
    use gem_chain_rpc::{ChainStakeProvider, ChainTokenDataProvider, GenericProvider, SmartChainProvider};
    use gem_client::ReqwestClient;
    use gem_cosmos::rpc::CosmosClient;
    use gem_evm::rpc::EthereumClient;
    use gem_jsonrpc::JsonRpcClient;
    use gem_solana::rpc::client::SolanaClient;
    use primitives::{chain_cosmos::CosmosChain, EVMChain};

    #[tokio::test]
    async fn test_get_solana_token_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = SolanaClient::new(JsonRpcClient::new(ReqwestClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
            reqwest::Client::new(),
        )));
        let provider = GenericProvider::new(client);
        let token_data = provider.get_token_data("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()).await?;

        assert_eq!(token_data.symbol, "USDC");
        assert_eq!(token_data.decimals, 6);

        Ok(())
    }

    #[tokio::test]
    async fn test_solana_get_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = SolanaClient::new(JsonRpcClient::new(ReqwestClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
            reqwest::Client::new(),
        )));
        let provider = GenericProvider::new(client);

        let validators = provider.get_validators().await?;
        assert!(!validators.is_empty());

        // Check that validators have valid data
        for validator in validators.iter().take(5) {
            assert!(!validator.id.is_empty());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_cosmos_get_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = CosmosClient::new(
            CosmosChain::Cosmos,
            ReqwestClient::new("https://cosmos-rest.publicnode.com".to_string(), reqwest::Client::new()),
        );
        let provider = GenericProvider::new(client);

        let validators = provider.get_validators().await?;
        assert!(!validators.is_empty());

        // Check that validators have valid data
        for validator in validators.iter().take(5) {
            assert!(!validator.id.is_empty());
            assert!(!validator.name.is_empty());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_validators_and_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = JsonRpcClient::new(ReqwestClient::new("https://bsc-dataseed.binance.org".to_string(), reqwest::Client::new()));
        let ethereum_client = EthereumClient::new(client, EVMChain::SmartChain);
        let provider = SmartChainProvider::new(ethereum_client);

        let validators = provider.get_validators().await?;
        assert!(!validators.is_empty());

        // Check that validators have valid data
        for validator in validators.iter().take(5) {
            assert!(!validator.id.is_empty());
            assert!(!validator.name.is_empty());
        }

        let apy = provider.get_staking_apy().await?;
        assert!(apy >= 0.0);

        Ok(())
    }
}
