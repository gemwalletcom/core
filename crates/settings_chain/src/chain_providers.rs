use std::error::Error;

use chain_traits::{ChainTraits, TransactionsRequest};
use primitives::{Asset, AssetBalance, Chain, DelegationBase, PerpetualPositionsSummary, StakeValidator, Transaction};
use settings::Settings;

use crate::ProviderFactory;

pub struct ChainProviders {
    providers: Vec<Box<dyn ChainTraits>>,
}

impl ChainProviders {
    pub fn new(providers: Vec<Box<dyn ChainTraits>>) -> Self {
        Self { providers }
    }

    pub fn from_settings(settings: &Settings, service_name: &str) -> Self {
        Self::new(ProviderFactory::new_providers_with_user_agent(settings, service_name))
    }

    pub fn for_chain(chain: Chain, settings: &Settings, service_name: &str) -> Self {
        Self::new(vec![ProviderFactory::new_from_settings_with_user_agent(chain, settings, service_name)])
    }

    fn get_provider(&self, chain: Chain) -> Result<&dyn ChainTraits, Box<dyn Error + Send + Sync>> {
        self.providers
            .iter()
            .find(|x| x.get_chain() == chain)
            .map(|provider| provider.as_ref())
            .ok_or_else(|| format!("Provider for chain {} not found", chain.as_ref()).into())
    }

    pub async fn get_token_data(&self, chain: Chain, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_token_data(token_id).await
    }

    pub fn get_is_token_address(&self, chain: Chain, token_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.get_provider(chain)?.get_is_token_address(token_id))
    }

    pub async fn get_balance_coin(&self, chain: Chain, address: String) -> Result<AssetBalance, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_balance_coin(address).await
    }

    pub async fn get_balance_tokens(&self, chain: Chain, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_balance_tokens(address, token_ids).await
    }

    pub async fn get_balance_assets(&self, chain: Chain, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_balance_assets(address).await
    }

    pub async fn get_balance_staking(&self, chain: Chain, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_balance_staking(address).await
    }

    pub async fn get_transactions_in_blocks(&self, chain: Chain, blocks: Vec<u64>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_transactions_in_blocks(blocks).await
    }

    pub async fn get_transactions_by_address(&self, chain: Chain, request: TransactionsRequest) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_transactions_by_address(request).await.map(sort_transactions_by_date)
    }

    pub async fn get_validators(&self, chain: Chain) -> Result<Vec<StakeValidator>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_provider(chain)?.get_staking_validators(None).await?.into_iter().map(|v| v.into()).collect())
    }

    pub async fn get_staking_apy(&self, chain: Chain) -> Result<f64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_provider(chain)?.get_staking_apy().await?.unwrap_or(0.0))
    }

    pub async fn get_latest_block(&self, chain: Chain) -> Result<u64, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_block_latest_number().await
    }

    pub async fn get_transaction_by_hash(&self, chain: Chain, hash: String) -> Result<Option<Transaction>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_transaction_by_hash(hash).await
    }

    pub async fn get_block_transactions(&self, chain: Chain, block_number: u64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_transactions_by_block(block_number).await
    }

    pub async fn get_staking_delegations(&self, chain: Chain, address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_staking_delegations(address).await
    }

    pub async fn get_perpetual_positions(&self, chain: Chain, address: String) -> Result<PerpetualPositionsSummary, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_positions(address).await
    }

    pub async fn get_perpetual_referred_addresses(&self, chain: Chain) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        self.get_provider(chain)?.get_perpetual_referred_addresses().await
    }
}

fn sort_transactions_by_date(mut transactions: Vec<Transaction>) -> Vec<Transaction> {
    transactions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    transactions
}
