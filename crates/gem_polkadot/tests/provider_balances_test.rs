mod testkit;

use chain_traits::ChainBalances;

#[tokio::test]
async fn test_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_polkadot_test_client();
    let address = testkit::TEST_ADDRESS.to_string();
    let balance = client.get_balance_coin(address).await?;
    assert_eq!(balance.asset_id.chain.to_string(), "polkadot");
    println!("Balance: {:?} {}", balance.balance, balance.asset_id);
    Ok(())
}
