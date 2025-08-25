use chain_traits::ChainBalances;
use primitives::Chain;

mod testkit;

#[tokio::test]
async fn test_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_sui_test_client();
    let balance = client
        .get_balance_coin("0x1c6ffe96e9beec00749dfc2fc3a65b69b46c5bd0987b47e0c9d4b98a1bbcd1f0".to_string())
        .await?;
    assert_eq!(balance.asset_id.chain, Chain::Sui);
    println!("Balance: {:?}", balance);
    Ok(())
}
