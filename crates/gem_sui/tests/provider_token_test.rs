use chain_traits::ChainToken;

mod testkit;

#[tokio::test]
async fn test_get_token_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_sui_test_client();
    let token_data = client
        .get_token_data("0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC".to_string())
        .await?;
    assert!(!token_data.name.is_empty());
    assert!(token_data.decimals > 0);
    println!("Token data: {:?}", token_data);
    Ok(())
}
