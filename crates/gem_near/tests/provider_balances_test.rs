mod testkit;

use chain_traits::ChainBalances;

#[tokio::test]
async fn test_near_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_near_test_client()?;
    let address = testkit::TEST_ADDRESS.to_string();
    let balance = client.get_balance_coin(address).await?;
    let available = balance.balance.available.parse::<u128>();
    assert!(available.is_ok());
    println!("Balance: {} {}", balance.balance.available, balance.asset_id);
    Ok(())
}