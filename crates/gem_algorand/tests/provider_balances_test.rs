use chain_traits::ChainBalances;
use primitives::Chain;

mod testkit;

#[tokio::test]
async fn test_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_algorand_test_client();
    let balance = client.get_balance_coin(testkit::TEST_ADDRESS.to_string()).await?;
    assert_eq!(balance.asset_id.chain, Chain::Algorand);
    println!("Balance: {:?}", balance);
    Ok(())
}
