mod testkit;

use chain_traits::ChainState;

#[tokio::test]
async fn test_get_xrp_latest_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_xrp_test_client();
    let block_number = client.get_block_latest_number().await?;

    assert!(block_number > 80_000_000, "XRP ledger index should be above 80M, got: {}", block_number);
    println!("XRP latest ledger: {}", block_number);

    Ok(())
}
