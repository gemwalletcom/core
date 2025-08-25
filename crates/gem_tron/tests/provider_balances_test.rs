mod testkit;

use chain_traits::ChainBalances;
use primitives::Chain;

#[tokio::test]
async fn test_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let balance = client.get_balance_coin(testkit::TEST_ADDRESS.to_string()).await?;

    assert_eq!(balance.asset_id.chain, Chain::Tron);
    assert_eq!(balance.asset_id.token_id, None);
    let balance = balance.balance.available.parse::<u64>().unwrap();
    assert!(balance > 0);

    Ok(())
}

#[tokio::test]
async fn test_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let token_ids = vec![
        "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string(), // USDT
    ];

    let balances = client.get_balance_tokens(testkit::TEST_ADDRESS.to_string(), token_ids.clone()).await?;

    assert_eq!(balances.len(), token_ids.len());
    for (i, balance) in balances.iter().enumerate() {
        assert_eq!(balance.asset_id.chain, Chain::Tron);
        assert_eq!(balance.asset_id.token_id, Some(token_ids[i].clone()));
        assert!(balance.balance.available.parse::<u64>().is_ok());
    }

    let balance_value = balances.first().unwrap().balance.available.parse::<u64>().unwrap();
    assert!(balance_value > 0, "USDT balance should be greater than 0");

    Ok(())
}

#[tokio::test]
async fn test_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let staking_balance = client.get_balance_staking(testkit::TEST_ADDRESS.to_string()).await?;

    if let Some(balance) = staking_balance {
        assert_eq!(balance.asset_id.chain, Chain::Tron);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.available.parse::<u64>().is_ok());
        assert!(balance.balance.staked.parse::<u64>().is_ok());
    }

    Ok(())
}
