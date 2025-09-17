use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::{error::Error, str::FromStr};

use gem_client::Client;
use primitives::{
    AssetSubtype, FeePriority, FeeRate, GasPriceType, StakeType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput,
    TransactionLoadMetadata, TransactionPreloadInput,
};
use std::collections::HashMap;

use crate::{
    provider::{
        balances_mapper::format_address_parameter,
        preload_mapper::{calculate_stake_fee_rate, calculate_transfer_fee_rate, calculate_transfer_token_fee_rate},
    },
    rpc::client::TronClient,
};

#[async_trait]
impl<C: Client> ChainTransactionLoad for TronClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let (block, chain_parameters, account_usage, is_new_account, votes) = futures::try_join!(
            self.get_tron_block(),
            self.get_chain_parameters(),
            self.get_account_usage(&input.sender_address),
            self.get_is_new_account_for_input_type(&input.destination_address, input.input_type.clone()),
            self.get_votes_for_transaction_input(&input)
        )?;

        let block = block.block_header.raw_data;
        let metadata = TransactionLoadMetadata::Tron {
            block_number: block.number,
            block_version: block.version,
            block_timestamp: block.timestamp,
            transaction_tree_root: block.tx_trie_root.clone(),
            parent_hash: block.parent_hash.clone(),
            witness_address: block.witness_address.clone(),
            votes,
        };

        let fee = match &input.input_type {
            TransactionInputType::Transfer(asset) => match asset.id.token_subtype() {
                AssetSubtype::NATIVE => calculate_transfer_fee_rate(&chain_parameters, &account_usage, is_new_account)?,
                AssetSubtype::TOKEN => {
                    let gas_limit = self
                        .estimate_trc20_transfer_gas(
                            input.sender_address.clone(),
                            asset.id.token_id.clone().unwrap(),
                            format_address_parameter(&input.destination_address)?,
                            input.value.clone(),
                        )
                        .await?;
                    calculate_transfer_token_fee_rate(&chain_parameters, &account_usage, is_new_account, BigInt::from_str(&gas_limit)?)?
                }
            },
            TransactionInputType::Stake(_asset, stake_type) => {
                let account = self.get_account(&input.sender_address).await?;
                let total_staked = account
                    .frozen_v2
                    .as_ref()
                    .map(|frozen_list| frozen_list.iter().map(|frozen| BigInt::from(frozen.amount)).sum::<BigInt>())
                    .unwrap_or_else(|| BigInt::from(0));

                let input_value = BigInt::from_str(&input.value)?;
                calculate_stake_fee_rate(&account_usage, stake_type, &total_staked, &input_value)
            }
            _ => calculate_transfer_fee_rate(&chain_parameters, &account_usage, is_new_account)?,
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

impl<C: Client> TronClient<C> {
    async fn get_is_new_account_for_input_type(&self, address: &str, input_type: TransactionInputType) -> Result<bool, Box<dyn Error + Send + Sync>> {
        match input_type {
            TransactionInputType::Transfer(_) => Ok(self.is_new_account(address).await?),
            _ => Ok(false),
        }
    }

    async fn get_votes_for_transaction_input(&self, input: &TransactionLoadInput) -> Result<HashMap<String, u64>, Box<dyn Error + Send + Sync>> {
        match &input.input_type {
            TransactionInputType::Stake(asset, stake_type) => {
                let account = self.get_account(&input.sender_address).await?;
                let mut current_votes: HashMap<String, u64> = account.votes.unwrap_or_default().into_iter().map(|v| (v.vote_address, v.vote_count)).collect();

                let vote_amount = input.value.parse::<u64>().unwrap_or(0) / 10_u64.pow(asset.decimals as u32);

                match stake_type {
                    StakeType::Stake(validator) => {
                        *current_votes.entry(validator.id.clone()).or_insert(0) += vote_amount;
                    }
                    StakeType::Unstake(delegation) => {
                        if let Some(votes) = current_votes.get_mut(&delegation.base.validator_id) {
                            *votes = votes.saturating_sub(vote_amount);
                        }
                    }
                    StakeType::Redelegate(redelegate_data) => {
                        if let Some(votes) = current_votes.get_mut(&redelegate_data.delegation.base.validator_id) {
                            *votes = votes.saturating_sub(vote_amount);
                        }
                        *current_votes.entry(redelegate_data.to_validator.id.clone()).or_insert(0) += vote_amount;
                    }
                    StakeType::Rewards(_) | StakeType::Withdraw(_) | StakeType::Freeze(_) => {}
                }

                current_votes.retain(|_, &mut v| v > 0);
                Ok(current_votes)
            }
            _ => Ok(HashMap::new()),
        }
    }
}
