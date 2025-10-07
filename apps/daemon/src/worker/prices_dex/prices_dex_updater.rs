use pricer_dex::providers::pyth::client::PythClient;
use pricer_dex::{PriceChainProvider, PythProvider};
use primitives::Chain;

pub struct PricesDexUpdater {
    pyth_provider: PythProvider,
}

impl PricesDexUpdater {
    pub fn new(hermes_url: &str) -> Self {
        let pyth_client = PythClient::new(hermes_url);
        let pyth_provider = PythProvider { pyth_client };
        Self { pyth_provider }
    }

    pub async fn update_prices(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let chains = Chain::all();

        println!("🔹 Fetching prices from Pyth Hermes API...");

        match self.pyth_provider.get_chain_prices(chains.clone()).await {
            Ok(prices) => {
                println!("\n✅ Successfully fetched {} prices:", prices.len());
                for (chain, price) in chains.iter().zip(prices.iter()) {
                    println!("  {:?}: ${:.2}", chain, price.price);
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Error fetching prices: {}", e);
                Err(e)
            }
        }
    }
}
