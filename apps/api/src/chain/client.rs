use std::error::Error;

use primitives::{Asset, AssetBalance, Chain, ChainAddress, Transaction};
use settings_chain::ChainProviders;

pub struct ChainClient {
    providers: ChainProviders,
}

impl ChainClient {
    pub fn new(providers: ChainProviders) -> Self {
        Self { providers }
    }

    pub async fn get_token_data(&self, chain: Chain, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.providers.get_token_data(chain, token_id).await
    }

    pub async fn get_balances(&self, request: ChainAddress) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.providers.get_assets_balances(request.chain, request.address).await
    }

    pub async fn get_transactions(&self, request: ChainAddress) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.providers.get_transactions_by_address(request.chain, request.address).await
    }

    pub async fn get_validators(&self, chain: Chain) -> Result<Vec<primitives::StakeValidator>, Box<dyn Error + Send + Sync>> {
        self.providers.get_validators(chain).await
    }

    pub async fn get_staking_apy(&self, chain: Chain) -> Result<f64, Box<dyn Error + Send + Sync>> {
        self.providers.get_staking_apy(chain).await
    }

    pub async fn get_block_transactions(
        &self,
        chain: Chain,
        block_number: i64,
        transaction_type: Option<&str>,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.providers.get_block_transactions(chain, block_number).await?;
        Ok(self.filter_transactions(transactions, transaction_type))
    }

    pub async fn get_block_transactions_finalize(
        &self,
        chain: Chain,
        block_number: i64,
        addresses: Vec<String>,
        transaction_type: Option<&str>,
    ) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self
            .get_block_transactions(chain, block_number, None)
            .await?
            .into_iter()
            .map(|x| x.finalize(addresses.clone()))
            .collect::<Vec<Transaction>>();
        Ok(self.filter_transactions(transactions, transaction_type))
    }

    fn filter_transactions(&self, transactions: Vec<Transaction>, transaction_type: Option<&str>) -> Vec<Transaction> {
        if let Some(transaction_type) = transaction_type {
            return transactions
                .into_iter()
                .filter(|x| x.transaction_type.as_ref() == transaction_type)
                .collect::<Vec<Transaction>>();
        }
        transactions
    }

    pub async fn get_latest_block(&self, chain: Chain) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.providers.get_latest_block(chain).await
    }
}
