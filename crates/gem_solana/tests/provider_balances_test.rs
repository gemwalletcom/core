mod testkit;

use chain_traits::ChainBalances;
use primitives::Chain;

#[tokio::test]
async fn test_solana_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let balance = client.get_balance_coin(testkit::TEST_ADDRESS.to_string()).await?;

    assert_eq!(balance.asset_id.chain, Chain::Solana);
    assert_eq!(balance.asset_id.token_id, None);
    assert!(balance.balance.available.parse::<u64>().is_ok());

    Ok(())
}

#[tokio::test]
async fn test_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let token_ids = vec![
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
    ];

    let balances = client.get_balance_tokens(testkit::TEST_ADDRESS.to_string(), token_ids.clone()).await?;

    assert_eq!(balances.len(), token_ids.len());
    for (i, balance) in balances.iter().enumerate() {
        assert_eq!(balance.asset_id.chain, Chain::Solana);
        assert_eq!(balance.asset_id.token_id, Some(token_ids[i].clone()));
        assert!(balance.balance.available.parse::<u64>().is_ok());
    }

    Ok(())
}

#[tokio::test]
async fn test_get_balance_staking() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let staking_balance = client.get_balance_staking(testkit::TEST_ADDRESS.to_string()).await?;

    if let Some(balance) = staking_balance {
        assert_eq!(balance.asset_id.chain, Chain::Solana);
        assert_eq!(balance.asset_id.token_id, None);
        assert!(balance.balance.available.parse::<u64>().is_ok());
        assert!(balance.balance.staked.parse::<u64>().is_ok());
    }

    Ok(())
}
