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
        provider.get_transactions(block_number).await
    }

    pub async fn get_block_finalize(&self, chain: Chain, block_number: i64, addresses: Vec<String>) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Send + Sync>> {
        let transactions = self.get_block(chain, block_number).await?
            .into_iter()
            .map(|x| x.finalize(addresses.clone())).collect::<Vec<Transaction>>();
        Ok(transactions)
    }

    pub async fn get_block_number_latest(&self, chain: Chain) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let provider = settings_chain::ProviderFactory::new(chain, &self.settings);
        provider.get_latest_block().await
    }
}
