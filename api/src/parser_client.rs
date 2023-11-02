use primitives::{Chain, Transaction};
use settings::Settings;

pub struct ParserClient {
    settings: Settings,
}

impl ParserClient {
    pub async fn new(
        settings: Settings,
    ) -> Self {
        Self {
            settings,
        }
    }

    pub async fn get_block(&self, chain: Chain, block_number: i64) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Send + Sync>> {
        let provider = settings_chain::ProviderFactory::new(chain, &self.settings);
        return provider.get_transactions(block_number).await
    }

    pub async fn get_block_number_latest(&self, chain: Chain) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let provider = settings_chain::ProviderFactory::new(chain, &self.settings);
        return provider.get_latest_block().await
    }
}
