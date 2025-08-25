use chain_traits::{ChainState, ChainTransactions};

mod testkit;

#[tokio::test]
async fn test_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let xrp_client = testkit::create_xrp_test_client();

    let latest_block = xrp_client.get_block_latest_number().await? - 5;
    let transactions = xrp_client.get_transactions_by_block(latest_block).await?;

    println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());
    assert!(latest_block > 0);
    Ok(())
}

#[tokio::test]
async fn test_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let xrp_client = testkit::create_xrp_test_client();

    let transactions = xrp_client.get_transactions_by_address(testkit::TEST_ADDRESS.to_string()).await?;

    println!("Address: {}, transactions count: {}", testkit::TEST_ADDRESS, transactions.len());
    Ok(())
}
