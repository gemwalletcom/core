mod testkit;

use chain_traits::ChainBalances;
use primitives::Chain;

#[tokio::test]
async fn test_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let address = "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string();
    
    let balance = client.get_balance_coin(address).await?;
    
    assert_eq!(balance.asset_id.chain, Chain::Tron);
    assert_eq!(balance.asset_id.token_id, None);
    assert!(balance.balance.available.parse::<u64>().is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let address = "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string();
    let token_ids = vec![
        "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string(), // USDT
        "TEkxiTehnzSmSe2XqrBj4w32RUN966rdz8".to_string(), // USDC
    ];
    
    let balances = client.get_balance_tokens(address, token_ids.clone()).await?;
    
    assert_eq!(balances.len(), token_ids.len());
    for (i, balance) in balances.iter().enumerate() {
        assert_eq!(balance.asset_id.chain, Chain::Tron);
        assert_eq!(balance.asset_id.token_id, Some(token_ids[i].clone()));
        assert!(balance.balance.available.parse::<u64>().is_ok());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let address = "TEB39Rt69QkgD1BKhqaRNqGxfQzCarkRCb".to_string();
    
    let staking_balance = client.get_balance_staking(address).await?;
    
    if let Some(balance) = staking_balance {
        assert_eq!(balance.asset_id.chain, Chain::Tron);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.available.parse::<u64>().is_ok());
        assert!(balance.balance.staked.parse::<u64>().is_ok());
    }
    
    Ok(())
}