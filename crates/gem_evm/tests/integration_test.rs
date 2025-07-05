#[cfg(test)]
mod tests {
    use gem_evm::rpc::EthereumClient;
    use num_bigint::BigUint;
    use primitives::{node_config::get_nodes_for_chain, Chain, EVMChain};

    #[tokio::test]
    async fn test_ethereum_client_get_latest_block() {
        let nodes = get_nodes_for_chain(Chain::Ethereum);
        let client = EthereumClient::new(EVMChain::Ethereum, &nodes[0].url);
        let latest_block = client.get_latest_block().await;

        assert!(latest_block.is_ok());

        let block_number = latest_block.unwrap();

        assert!(block_number > 1_000_000);
    }

    #[tokio::test]
    async fn test_ethereum_client_get_block() {
        let nodes = get_nodes_for_chain(Chain::Ethereum);
        let client = EthereumClient::new(EVMChain::Ethereum, &nodes[0].url);
        let block = client.get_block(1).await;

        assert!(block.is_ok());

        let block_data = block.unwrap();

        assert!(block_data.timestamp > BigUint::from(0u32));
    }

    #[tokio::test]
    async fn test_ethereum_client_eth_call_erc20_name() {
        let nodes = get_nodes_for_chain(Chain::Ethereum);
        let client = EthereumClient::new(EVMChain::Ethereum, &nodes[0].url);
        let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let name_call_data = "0x06fdde03";
        let result: Result<String, _> = client.eth_call(usdc_address, name_call_data).await;

        assert!(result.is_ok());

        let name_hex = result.unwrap();

        assert_eq!(name_hex, "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000855534420436f696e000000000000000000000000000000000000000000000000")
    }

    #[tokio::test]
    async fn test_ethereum_client_batch_calls() {
        let nodes = get_nodes_for_chain(Chain::Ethereum);
        let client = EthereumClient::new(EVMChain::Ethereum, &nodes[0].url);
        let latest_block = client.get_latest_block().await.unwrap();
        let prev_block = latest_block - 1;

        let block_numbers = vec![format!("0x{:x}", prev_block), format!("0x{:x}", latest_block)];
        let blocks = client.get_blocks(&block_numbers, false).await;

        assert!(blocks.is_ok());

        let blocks_data = blocks.unwrap();

        assert_eq!(blocks_data.len(), 2);

        for block in &blocks_data {
            assert!(block.timestamp > BigUint::from(0u32));
        }
    }

    #[tokio::test]
    async fn test_ethereum_client_trace_replay_block_transactions() {
        let client = EthereumClient::new(EVMChain::Ethereum, "https://ethereum-public.nodies.app");
        let traces = client.trace_replay_block_transactions(22838462).await;

        assert!(traces.is_ok());

        let traces_data = traces.unwrap();

        assert!(traces_data.len() > 0);
    }
}
