use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider};
use async_trait::async_trait;
use gem_tron::rpc::trongrid::client::TronGridClient;
use primitives::{chain::Chain, Asset, AssetBalance, Transaction};

use gem_tron::rpc::trongrid::mapper::TronGridMapper;
use gem_tron::rpc::TronClient;
use gem_tron::rpc::TronMapper;

pub struct TronProvider {
    client: TronClient,
    assets_provider: Box<dyn ChainAssetsProvider>,
    transactions_provider: Box<dyn ChainTransactionsProvider>,
}

impl TronProvider {
    pub fn new(client: TronClient, assets_provider: Box<dyn ChainAssetsProvider>, transactions_provider: Box<dyn ChainTransactionsProvider>) -> Self {
        Self {
            client,
            assets_provider,
            transactions_provider,
        }
    }
}

#[async_trait]
impl ChainBlockProvider for TronProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        self.client.get_latest_block().await
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block_tranactions(block_number).await?;
        let reciepts = self.client.get_block_tranactions_reciepts(block_number).await?;

        Ok(TronMapper::map_transactions(self.get_chain(), block, reciepts))
    }
}

#[async_trait]
impl ChainTokenDataProvider for TronProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        self.client.get_token_data(token_id).await
    }
}

#[async_trait]
impl ChainAssetsProvider for TronProvider {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.assets_provider.get_assets_balances(address).await
    }
}

#[async_trait]
impl ChainTransactionsProvider for TronProvider {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.transactions_provider.get_transactions_by_address(address).await
    }
}

#[async_trait]
impl ChainStakeProvider for TronProvider {
    async fn get_validators(&self) -> Result<Vec<primitives::StakeValidator>, Box<dyn Error + Send + Sync>> {
        let witnesses = self.client.get_witnesses_list().await?;
        Ok(TronMapper::map_validators(witnesses))
    }

    async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let params = self.client.get_chain_parameters().await?;
        let witnesses = self.client.get_witnesses_list().await?;

        let block_reward = params
            .iter()
            .find(|p| p.key == "getWitnessPayPerBlock")
            .and_then(|p| p.value)
            .unwrap_or(16_000_000) as f64
            / 1_000_000.0;

        let voting_reward = params
            .iter()
            .find(|p| p.key == "getWitness127PayPerBlock")
            .and_then(|p| p.value)
            .unwrap_or(160_000_000) as f64
            / 1_000_000.0;

        let blocks_per_year = 365.25 * 24.0 * 60.0 * 60.0 / 3.0;
        let annual_rewards = (block_reward + voting_reward) * blocks_per_year;

        let total_votes: i64 = witnesses.witnesses.iter().map(|x| x.vote_count.unwrap_or(0)).sum();
        let total_staked_trx = total_votes as f64;

        if total_staked_trx == 0.0 {
            return Ok(0.0);
        }

        let apy = (annual_rewards / total_staked_trx) * 100.0;

        Ok(apy)
    }
}

// Tron Grid
#[async_trait]
impl ChainAssetsProvider for TronGridClient {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let accounts = self.get_accounts_by_address(&address).await?.data;
        if let Some(account) = accounts.first() {
            Ok(TronGridMapper::map_asset_balances(account.clone()))
        } else {
            Ok(vec![])
        }
    }
}

#[async_trait]
impl ChainTransactionsProvider for TronGridClient {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let native_transactions = self.get_transactions_by_address(&address, 16).await?.data;
        let token_transactions = self.get_token_transactions(&address, 16).await?;

        let transactions = native_transactions
            .clone()
            .into_iter()
            .chain(token_transactions)
            .collect::<Vec<gem_tron::rpc::model::Transaction>>();

        if transactions.is_empty() {
            return Ok(vec![]);
        }
        let transaction_ids = transactions.iter().map(|x| x.tx_id.clone()).collect::<Vec<String>>();
        let reciepts = self.get_transactions_reciepts(transaction_ids).await?;

        Ok(TronGridMapper::map_transactions(transactions.clone(), reciepts))
    }
}
