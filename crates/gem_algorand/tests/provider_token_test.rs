use chain_traits::ChainToken;

mod testkit;

#[tokio::test]
async fn test_get_token_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_algorand_test_client();
    let token_data = client.get_token_data("31566704".to_string()).await?;
    assert!(!token_data.name.is_empty());
    assert!(token_data.decimals > 0);
    println!("Token data: {:?}", token_data);
    Ok(())
}
