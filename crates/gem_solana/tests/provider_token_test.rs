mod testkit;

use chain_traits::ChainToken;
use primitives::{AssetType, Chain};

#[tokio::test]
async fn test_get_token_data_usdc_spl_token() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string();

    let asset = client.get_token_data(usdc_mint.clone()).await?;

    assert_eq!(asset.chain, Chain::Solana);
    assert_eq!(asset.token_id, Some(usdc_mint));
    assert_eq!(asset.symbol, "USDC");
    assert_eq!(asset.name, "USD Coin");
    assert_eq!(asset.decimals, 6);
    assert_eq!(asset.asset_type, AssetType::TOKEN);

    Ok(())
}

#[tokio::test]
async fn test_get_token_data_spl_token_2022() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = testkit::create_test_client();
    let spl2022_mint = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo".to_string();

    let asset = client.get_token_data(spl2022_mint.clone()).await?;

    assert_eq!(asset.chain, Chain::Solana);
    assert_eq!(asset.token_id, Some(spl2022_mint));
    assert_eq!(asset.symbol, "PYUSD");
    assert_eq!(asset.name, "PayPal USD");
    assert_eq!(asset.decimals, 6);
    assert_eq!(asset.asset_type, AssetType::TOKEN);

    Ok(())
}
