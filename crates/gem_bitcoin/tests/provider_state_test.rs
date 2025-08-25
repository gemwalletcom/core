mod testkit;

use chain_traits::ChainState;

#[tokio::test]
async fn test_get_bitcoin_latest_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_bitcoin_test_client();
    let block_number = client.get_block_latest_number().await?;

    assert!(block_number > 800_000, "Bitcoin block number should be above 800k, got: {}", block_number);
    println!("Bitcoin latest block: {}", block_number);

    Ok(())
}

#[tokio::test]
async fn test_get_bitcoin_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_bitcoin_test_client();
    let chain_id = client.get_chain_id().await?;

    assert!(!chain_id.is_empty());
    assert!(chain_id.len() == 64); // Bitcoin block hashes are 64 characters
    println!("Bitcoin chain ID: {}", chain_id);

    Ok(())
}
