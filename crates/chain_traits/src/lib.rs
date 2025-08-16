use std::error::Error;

use async_trait::async_trait;
use primitives::chart::ChartCandleStick;
use primitives::perpetual::{PerpetualData, PerpetualPositionsSummary};
use primitives::{Asset, AssetBalance, ChartPeriod, DelegationBase, DelegationValidator, FeePriorityValue, TransactionPreload, TransactionPreloadInput, TransactionStateRequest, TransactionUpdate, UTXO};

pub trait ChainTraits: ChainBalances + ChainStaking + ChainTransactions + ChainState + ChainAccount + ChainPerpetual + ChainToken + ChainPreload {}

#[async_trait]
pub trait ChainBalances: Send + Sync {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>>;
    async fn get_balance_tokens(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>>;
    async fn get_balance_staking(&self, address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>>;
}

#[async_trait]
pub trait ChainStaking: Send + Sync {
    async fn get_staking_validators(&self) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_staking_delegations(&self, _address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}

#[async_trait]
pub trait ChainTransactions: Send + Sync {
    async fn transaction_broadcast(&self, data: String) -> Result<String, Box<dyn Error + Sync + Send>>;
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>>;
}

#[async_trait]
pub trait ChainState: Send + Sync {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>>;
    async fn get_block_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>>;
    async fn get_fees(&self) -> Result<Vec<FeePriorityValue>, Box<dyn Error + Sync + Send>>;
}

#[async_trait]
pub trait ChainAccount: Send + Sync {
    async fn get_utxos(&self, _address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}

#[async_trait]
pub trait ChainPerpetual: Send + Sync {
    async fn get_positions(&self, _address: String) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support perpetual trading".into())
    }

    async fn get_perpetuals_data(&self) -> Result<Vec<PerpetualData>, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support perpetual trading".into())
    }

    async fn get_candlesticks(&self, _symbol: String, _period: ChartPeriod) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support perpetual trading".into())
    }
}

#[async_trait]
pub trait ChainToken: Send + Sync {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support tokens".into())
    }
    
    fn get_is_token_address(&self, _token_id: &str) -> bool {
        false
    }
}

#[async_trait]
pub trait ChainPreload: Send + Sync {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        Ok(TransactionPreload::default())
    }
}
