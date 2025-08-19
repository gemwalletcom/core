mod testkit;

use chain_traits::ChainStaking;

#[tokio::test]
async fn test_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let apy = client.get_staking_apy().await?;
    assert!(apy.is_some());
    Ok(())
}

#[tokio::test]
async fn test_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let validators = client.get_staking_validators(None).await?;
    assert!(!validators.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_get_staking_delegations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let address = "6sbzC1eH4FTujJXWj51eQe25cYvr4xfXbJ1vAj7j2k5J".to_string();
    let delegations = client.get_staking_delegations(address).await?;
    assert!(delegations.len() <= 100);
    Ok(())
}