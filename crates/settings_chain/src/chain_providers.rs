use std::error::Error;

use gem_chain_rpc::ChainProvider;
use primitives::{Asset, AssetBalance, Chain, Transaction};

pub struct ChainProviders {
    providers: Vec<Box<dyn ChainProvider>>,
}

impl ChainProviders {
    pub fn new(providers: Vec<Box<dyn ChainProvider>>) -> Self {
        Self { providers }
    }

    pub async fn get_token_data(&self, chain: Chain, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.providers.iter().find(|x| x.get_chain() == chain).unwrap().get_token_data(token_id).await
    }

    pub async fn get_assets_balances(&self, chain: Chain, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.providers
            .iter()
            .find(|x| x.get_chain() == chain)
            .unwrap()
            .get_assets_balances(address)
            .await
    }

    pub async fn get_transactions_in_blocks(&self, chain: Chain, blocks: Vec<i64>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.providers
            .iter()
            .find(|x| x.get_chain() == chain)
            .unwrap()
            .get_transactions_in_blocks(blocks)
            .await
    }
}
