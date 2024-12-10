#[cfg(test)]
mod tests {
    use gem_chain_rpc::{ChainTokenDataProvider, SolanaClient};
    use primitives::Chain;

    #[tokio::test]
    async fn test_get_solana_token_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = SolanaClient::new("https://solana-rpc.publicnode.com".into());
        let provider: Box<dyn ChainTokenDataProvider> = Box::new(client);
        let token_data = provider
            .get_token_data(Chain::Solana, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into())
            .await?;

        assert_eq!(token_data.symbol, "USDC");
        assert_eq!(token_data.decimals, 6);

        Ok(())
    }
}
