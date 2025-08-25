use chain_traits::ChainStaking;

mod testkit;

#[tokio::test]
async fn test_get_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_sui_test_client();
    let apy = client.get_staking_apy().await?;
    assert!(apy.is_some());
    println!("Staking APY: {:?}", apy);
    Ok(())
}

#[tokio::test]
async fn test_get_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_sui_test_client();
    let validators = client.get_staking_validators(Some(5.0)).await?;
    assert!(!validators.is_empty());
    println!("Found {} validators", validators.len());
    Ok(())
}
