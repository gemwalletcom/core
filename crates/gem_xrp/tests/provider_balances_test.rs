mod testkit;

use chain_traits::ChainBalances;

#[tokio::test]
async fn test_xrp_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_xrp_test_client();
    let address = testkit::TEST_ADDRESS.to_string();
    let balance = client.get_balance_coin(address).await?;
    let available = balance.balance.available.parse::<u64>().unwrap();
    assert!(available > 0);
    println!("Balance: {:?} {}", available, balance.asset_id);
    Ok(())
}