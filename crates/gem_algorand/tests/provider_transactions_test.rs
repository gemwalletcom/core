use chain_traits::{ChainState, ChainTransactions};

mod testkit;

#[tokio::test]
async fn test_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_algorand_test_client();
    let latest_block = client.get_block_latest_number().await?;
    let transactions = client.get_transactions_by_block(latest_block - 1).await?;
    println!("Transactions in block {}: {}", latest_block - 1, transactions.len());
    Ok(())
}

#[tokio::test]
async fn test_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_algorand_test_client();
    let transactions = client.get_transactions_by_address(testkit::TEST_ADDRESS.to_string()).await?;
    println!("Address: {}, transactions count: {}", testkit::TEST_ADDRESS, transactions.len());
    Ok(())
}
