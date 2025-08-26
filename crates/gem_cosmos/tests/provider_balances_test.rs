mod testkit;

use chain_traits::ChainBalances;

const TEST_ADDRESS: &str = "cosmos1cvh8mpz04az0x7vht6h6ekksg8wd650r39ltwj";

#[tokio::test]
async fn test_cosmos_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_cosmos_test_client();
    let address = TEST_ADDRESS.to_string();
    let balance = client.get_balance_coin(address).await?;
    let available = balance.balance.available.parse::<u64>().unwrap();
    assert!(available > 0);
    println!("Balance: {:?} {}", available, balance.asset_id);
    Ok(())
}