use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::{error::Error, str::FromStr};

use gem_client::Client;
use primitives::{
    AssetSubtype, FeePriority, FeeRate, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput,
};

use crate::{
    provider::{
        balances_mapper::format_address_parameter,
        preload_mapper::{calculate_transfer_fee_rate, calculate_transfer_token_fee_rate},
    },
    rpc::client::TronClient,
};

#[async_trait]
impl<C: Client> ChainTransactionLoad for TronClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let (block, chain_parameters, account_usage, is_new_account) = futures::try_join!(
            self.get_tron_block(),
            self.get_chain_parameters(),
            self.get_account_usage(&input.sender_address),
            self.is_new_account(&input.destination_address)
        )?;
        let block = block.block_header.raw_data;
        let metadata = TransactionLoadMetadata::Tron {
            block_number: block.number,
            block_version: block.version,
            block_timestamp: block.timestamp,
            transaction_tree_root: block.tx_trie_root.clone(),
            parent_hash: block.parent_hash.clone(),
            witness_address: block.witness_address.clone(),
        };

        let fee = match input.input_type.get_asset().id.token_subtype() {
            AssetSubtype::NATIVE => calculate_transfer_fee_rate(&chain_parameters, &account_usage, is_new_account)?,
            AssetSubtype::TOKEN => {
                let gas_limit = self
                    .estimate_trc20_transfer_gas(
                        input.sender_address.clone(),
                        input.input_type.get_asset().id.token_id.clone().unwrap(),
                        format_address_parameter(&input.destination_address)?,
                        input.value.clone(),
                    )
                    .await?;
                calculate_transfer_token_fee_rate(&chain_parameters, &account_usage, is_new_account, BigInt::from_str(&gas_limit)?)?
            }
        };

        Ok(TransactionLoadData {
            fee: TransactionFee::new_from_fee(fee),
            metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Send + Sync>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(1)))])
    }
}
