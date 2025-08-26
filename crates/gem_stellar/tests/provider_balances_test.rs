mod testkit;

use chain_traits::ChainBalances;
use primitives::Chain;

#[tokio::test]
async fn test_stellar_get_balance_coin() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let balance = client.get_balance_coin(testkit::TEST_ADDRESS.to_string()).await?;

    assert_eq!(balance.asset_id.chain, Chain::Stellar);
    assert_eq!(balance.asset_id.token_id, None);
    assert!(balance.balance.available.parse::<u64>().is_ok());

    Ok(())
}

#[tokio::test]
async fn test_get_balance_tokens() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let token_ids = vec![
        "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN::USDC".to_string(), // USDC token
    ];

    let balances = client.get_balance_tokens(testkit::TEST_ADDRESS.to_string(), token_ids.clone()).await?;

    assert_eq!(balances.len(), token_ids.len());
    for (i, balance) in balances.iter().enumerate() {
        assert_eq!(balance.asset_id.chain, Chain::Stellar);
        assert_eq!(balance.asset_id.token_id, Some(token_ids[i].clone()));
        assert!(balance.balance.available.parse::<u64>().is_ok());
    }

    Ok(())
}
