use std::error::Error;

use async_trait::async_trait;
use primitives::chart::ChartCandleStick;
use primitives::perpetual::{PerpetualData, PerpetualPositionsSummary};
use primitives::{
    AddressStatus, Asset, AssetBalance, BroadcastOptions, Chain, ChartPeriod, DelegationBase, DelegationValidator, FeeRate, Transaction, TransactionFee,
    TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput, TransactionStateRequest,
    TransactionUpdate, UTXO,
};

pub trait ChainTraits:
    ChainProvider + ChainBalances + ChainStaking + ChainTransactions + ChainState + ChainAccount + ChainPerpetual + ChainToken + ChainTransactionLoad + ChainAddressStatus
{
}

pub trait ChainProvider: Send + Sync {
    fn get_chain(&self) -> Chain;
}

#[async_trait]
pub trait ChainBalances: Send + Sync {
    async fn get_balance_coin(&self, _address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support balance operations".into())
    }
    async fn get_balance_tokens(&self, _address: String, _token_ids: Vec<String>) -> Result<Vec<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support balance operations".into())
    }
    async fn get_balance_staking(&self, _address: String) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support balance operations".into())
    }
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
pub trait ChainStaking: Send + Sync {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }

    async fn get_staking_validators(&self, _apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_staking_delegations(&self, _address: String) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}

#[async_trait]
pub trait ChainTransactions: Send + Sync {
    async fn transaction_broadcast(&self, _data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support transaction broadcasting".into())
    }
    async fn get_transaction_status(&self, _request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support transaction status".into())
    }
    async fn get_transactions_by_block(&self, _block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
    async fn get_transactions_by_address(&self, _address: String, _limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }

    async fn get_transactions_in_blocks(&self, blocks: Vec<u64>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let mut all_transactions = Vec::new();
        for block in blocks {
            match self.get_transactions_by_block(block).await {
                Ok(transactions) => all_transactions.extend(transactions),
                Err(e) => return Err(e),
            }
        }
        Ok(all_transactions)
    }
}

#[async_trait]
pub trait ChainState: Send + Sync {
    async fn get_chain_id(&self) -> Result<String, Box<dyn Error + Sync + Send>>;
    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>>;
}

#[async_trait]
pub trait ChainAccount: Send + Sync {}

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
pub trait ChainTransactionLoad: Send + Sync {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, _input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support transaction loading".into())
    }

    async fn get_transaction_fee_from_data(&self, _data: String) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support transaction fee".into())
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Err("Chain does not support fee rates".into())
    }

    async fn get_utxos(&self, _address: String) -> Result<Vec<UTXO>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}

#[async_trait]
pub trait ChainAddressStatus: Send + Sync {
    async fn get_address_status(&self, _address: String) -> Result<Vec<AddressStatus>, Box<dyn Error + Sync + Send>> {
        Ok(vec![])
    }
}
