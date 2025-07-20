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
}
