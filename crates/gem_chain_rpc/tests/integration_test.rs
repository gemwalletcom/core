#[cfg(test)]
mod tests {
    use gem_chain_rpc::solana::provider::SolanaProvider;
    use gem_solana::client::SolanaRpc;
    use primitives::Chain;

    #[tokio::test]
    async fn test_get_solana_token_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = SolanaProvider::new("https://solana-rpc.publicnode.com");
        let token_data = provider.get_token_data("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()).await?;

        assert_eq!(token_data.symbol, "USDC");
        assert_eq!(token_data.decimals, 6);

        Ok(())
    }
}
