mod testkit;

use chain_traits::ChainState;

#[tokio::test]
async fn test_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_polkadot_test_client();
    let chain_id = client.get_chain_id().await?;
    assert!(!chain_id.is_empty());
    println!("Chain ID: {}", chain_id);
    Ok(())
}

#[tokio::test]
async fn test_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_polkadot_test_client();
    let block_number = client.get_block_latest_number().await?;
    assert!(block_number > 0);
    println!("Latest block: {}", block_number);
    Ok(())
}
