use chain_traits::ChainToken;

mod testkit;

#[tokio::test]
async fn test_get_token_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_aptos_test_client();
    let token_data = client
        .get_token_data("0xf22bede237a07e121b56d91a491eb7bcdfd1f5907926a9e58338f964a01b17fa::asset::USDC".to_string())
        .await?;
    assert!(!token_data.name.is_empty());
    assert!(token_data.decimals > 0);
    println!("Token data: {:?}", token_data);
    Ok(())
}
