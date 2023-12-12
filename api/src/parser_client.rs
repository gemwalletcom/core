use primitives::{Chain, Transaction};
use settings::Settings;

pub struct ParserClient {
    settings: Settings,
}

impl ParserClient {
    pub async fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub async fn get_block(
        &self,
        chain: Chain,
        block_number: i64,
        transaction_type: Option<&str>,
    ) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Send + Sync>> {
        let provider = settings_chain::ProviderFactory::new_provider(chain, &self.settings);
        let transactions = provider.get_transactions(block_number).await?;
        Ok(self.filter_transactions(transactions, transaction_type))
    }

    pub async fn get_block_finalize(
        &self,
        chain: Chain,
        block_number: i64,
        addresses: Vec<String>,
        transaction_type: Option<&str>,
    ) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Send + Sync>> {
        let transactions = self
            .get_block(chain, block_number, None)
            .await?
            .into_iter()
            .map(|x| x.finalize(addresses.clone()))
            .collect::<Vec<Transaction>>();
        Ok(self.filter_transactions(transactions, transaction_type))
    }

    pub fn filter_transactions(
        &self,
        transactions: Vec<Transaction>,
        transaction_type: Option<&str>,
    ) -> Vec<Transaction> {
        if let Some(transaction_type) = transaction_type {
            return transactions
                .into_iter()
                .filter(|x| x.transaction_type.to_string() == transaction_type)
                .collect::<Vec<Transaction>>();
        }

        transactions
    }

    pub async fn get_block_number_latest(
        &self,
        chain: Chain,
    ) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let provider = settings_chain::ProviderFactory::new_provider(chain, &self.settings);
        provider.get_latest_block().await
    }
}
