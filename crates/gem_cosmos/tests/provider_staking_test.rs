mod testkit;

use chain_traits::ChainStaking;

#[tokio::test]
async fn test_get_osmosis_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_osmosis_test_client();
    let apy = client.get_staking_apy().await?;

    assert!(apy.is_some());
    let apy_value = apy.unwrap();

    assert!(apy_value > 1.0 && apy_value < 2.0, "APY should be between 1% and 2%, got: {}", apy_value);
    assert_ne!(apy_value, 14.0);

    println!("Osmosis staking APY: {}%", apy_value);

    Ok(())
}

#[tokio::test]
async fn test_get_cosmos_staking_apy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_cosmos_test_client();
    let apy = client.get_staking_apy().await?;

    assert!(apy.is_some());
    let apy_value = apy.unwrap();

    assert!(apy_value > 5.0 && apy_value < 25.0);

    println!("Cosmos staking APY: {}%", apy_value);

    Ok(())
}

#[tokio::test]
async fn test_get_cosmos_staking_validators() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_cosmos_test_client();
    let apy = client.get_staking_apy().await?;
    let validators = client.get_staking_validators(apy).await?;

    assert!(!validators.is_empty());
    assert!(validators.len() <= 200);

    for validator in validators.iter().take(5) {
        assert!(!validator.id.is_empty());
        assert!(!validator.name.is_empty());
        assert!(validator.commision >= 0.0 && validator.commision <= 100.0);
        if validator.is_active {
            assert!(validator.apr >= 0.0);
        }
    }

    Ok(())
}
