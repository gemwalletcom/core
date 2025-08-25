use chain_traits::ChainTransactions;

mod testkit;

#[tokio::test]
async fn test_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_sui_test_client();
    let latest_block = client.get_block_latest_number().await?;
    let transactions = client.get_transactions_by_block(latest_block - 1).await?;
    println!("Transactions in block {}: {}", latest_block - 1, transactions.len());
    Ok(())
}
