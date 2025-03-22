#[cfg(test)]
mod tests {
    use pricer_dex::{pyth::client::PythClient, PriceChainProvider, PythProvider};
    use primitives::Chain;

    #[tokio::test]
    async fn test_get_chain_prices() {
        let provider = PythProvider {
            pyth_client: PythClient::new("http://pythnet.rpcpool.com"),
        };
        let chains = vec![Chain::Bitcoin, Chain::Ethereum];
        let result = provider.get_chain_prices(chains.clone()).await;

        assert!(result.ok().unwrap().len() == chains.len());
    }
}
