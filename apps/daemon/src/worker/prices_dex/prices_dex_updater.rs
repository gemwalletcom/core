use gem_tracing::{error_with_fields, info_with_fields};
use pricer_dex::providers::pyth::client::PythClient;
use pricer_dex::{JupiterProvider, PriceChainAssetsProvider, PriceChainProvider, PythProvider};
use primitives::{AssetId, Chain};

pub struct PricesDexUpdater {
    pyth_provider: PythProvider,
    jupiter_provider: JupiterProvider,
}

impl PricesDexUpdater {
    pub fn new(hermes_url: &str, jupiter_url: &str) -> Self {
        let pyth_client = PythClient::new(hermes_url);
        let pyth_provider = PythProvider { pyth_client };

        let jupiter_provider = JupiterProvider::new(jupiter_url);

        Self {
            pyth_provider,
            jupiter_provider,
        }
    }

    pub async fn update_prices(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let chains = Chain::all();

        println!("🔹 Fetching prices from Pyth Hermes API...");

        match self.pyth_provider.get_chain_prices(chains.clone()).await {
            Ok(prices) => {
                println!("\n✅ Successfully fetched {} prices from Pyth:", prices.len());
                for (chain, price) in chains.iter().zip(prices.iter()) {
                    println!("  {:?}: ${:.2}", chain, price.price);
                }
            }
            Err(e) => {
                eprintln!("❌ Error fetching prices from Pyth: {}", e);
            }
        }

        println!("\n🔹 Fetching prices from Jupiter API...");

        let solana_asset_ids = vec![
            AssetId::from_token(Chain::Solana, "So11111111111111111111111111111111111111112"),
            AssetId::from_token(Chain::Solana, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            AssetId::from_token(Chain::Solana, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
            AssetId::from_token(Chain::Solana, "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN"),
        ];

        match self.jupiter_provider.get_asset_prices(Chain::Solana, solana_asset_ids).await {
            Ok(prices) => {
                println!("\n✅ Successfully fetched {} prices from Jupiter:", prices.len());
                for price in prices.iter() {
                    println!("  {}: ${:.6}", price.asset_id, price.price);
                }
            }
            Err(e) => {
                eprintln!("❌ Error fetching prices from Jupiter: {}", e);
            }
        }

        Ok(())
    }

    pub async fn update_real_time_price(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::time::{Duration, sleep};

        loop {
            let pyth_price = match self.pyth_provider.get_chain_prices(vec![Chain::Solana]).await {
                Ok(prices) => prices.first().map(|p| (p.price, p.price_change_percentage_24h)),
                Err(e) => {
                    error_with_fields!("pyth_fetch_error", e.as_ref(), chain = "solana");
                    None
                }
            };

            let solana_wrapped_sol = AssetId::from_token(Chain::Solana, "So11111111111111111111111111111111111111112");

            let jupiter_price = match self.jupiter_provider.get_asset_prices(Chain::Solana, vec![solana_wrapped_sol]).await {
                Ok(prices) => prices.first().map(|p| (p.price, p.price_change_percentage_24h)),
                Err(e) => {
                    error_with_fields!("jupiter_fetch_error", e.as_ref(), chain = "solana");
                    None
                }
            };

            if let (Some((pyth, pyth_change)), Some((jupiter, jupiter_change))) = (pyth_price, jupiter_price) {
                let diff = jupiter - pyth;
                let diff_pct = (diff / pyth) * 100.0;
                info_with_fields!("sol", pyth = pyth, jupiter = jupiter, diff = diff);
            }

            sleep(Duration::from_secs(5)).await;
        }
    }
}
