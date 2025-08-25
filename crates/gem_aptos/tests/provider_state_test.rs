use chain_traits::ChainState;

mod testkit;

#[tokio::test]
async fn test_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_aptos_test_client();
    let chain_id = client.get_chain_id().await?;
    assert!(!chain_id.is_empty());
    println!("Aptos chain ID: {}", chain_id);
    Ok(())
}

#[tokio::test]
async fn test_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_aptos_test_client();
    let latest_block = client.get_block_latest_number().await?;
    assert!(latest_block > 0);
    println!("Latest block: {}", latest_block);
    Ok(())
}
