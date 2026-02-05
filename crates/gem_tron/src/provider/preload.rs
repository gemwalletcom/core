use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;

use gem_client::Client;
use number_formatter::BigNumberFormatter;
use primitives::{
    AssetSubtype, FeePriority, FeeRate, GasPriceType, StakeType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput, TronStakeData, TronVote,
};

use crate::{
    provider::{
        balances_mapper::format_address_parameter,
        preload_mapper::{calculate_stake_fee_rate, calculate_transfer_fee_rate, calculate_transfer_token_fee_rate, calculate_unfreeze_amounts},
    },
    rpc::client::TronClient,
};

#[async_trait]
impl<C: Client> ChainTransactionLoad for TronClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Send + Sync>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let (block, chain_parameters, account_usage, is_new_account, stake_data) = futures::try_join!(
            self.get_tron_block(),
            self.get_chain_parameters(),
            self.get_account_usage(&input.sender_address),
            self.get_is_new_account_for_input_type(&input.destination_address, input.input_type.clone()),
            self.get_stake_data(&input)
        )?;

        let block = block.block_header.raw_data;
        let metadata = TransactionLoadMetadata::Tron {
            block_number: block.number,
            block_version: block.version,
            block_timestamp: block.timestamp,
            transaction_tree_root: block.tx_trie_root.clone(),
            parent_hash: block.parent_hash.clone(),
            witness_address: block.witness_address.clone(),
            stake_data,
        };

        let fee = match &input.input_type {
            TransactionInputType::Transfer(asset) | TransactionInputType::TransferNft(asset, _) | TransactionInputType::Account(asset, _) => match &asset.id.token_id {
                None => TransactionFee::new_from_fee(calculate_transfer_fee_rate(&chain_parameters, &account_usage, is_new_account)?),
                Some(token_id) => {
                    self.estimate_token_transfer_fee(
                        input.sender_address.clone(),
                        input.destination_address.clone(),
                        token_id.clone(),
                        input.value.clone(),
                        &chain_parameters,
                        &account_usage,
                    )
                    .await?
                }
            },
            TransactionInputType::Stake(_asset, stake_type) => TransactionFee::new_from_fee(calculate_stake_fee_rate(&chain_parameters, &account_usage, stake_type)?),
            TransactionInputType::Swap(from_asset, _, swap_data) => match &from_asset.id.token_id {
                None => TransactionFee::new_from_fee(calculate_transfer_fee_rate(&chain_parameters, &account_usage, is_new_account)?),
                Some(token_id) => {
                    self.estimate_token_transfer_fee(
                        input.sender_address.clone(),
                        swap_data.data.to.clone(),
                        token_id.clone(),
                        input.value.clone(),
                        &chain_parameters,
                        &account_usage,
                    )
                    .await?
                }
            },
            _ => TransactionFee::new_from_fee(calculate_transfer_fee_rate(&chain_parameters, &account_usage, is_new_account)?),
        };

        Ok(TransactionLoadData { fee, metadata })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Send + Sync>> {
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(BigInt::from(1)))])
    }
}

impl<C: Client> TronClient<C> {
    async fn estimate_token_transfer_fee(
        &self,
        sender_address: String,
        destination_address: String,
        token_id: String,
        value: String,
        chain_parameters: &[crate::models::ChainParameter],
        account_usage: &crate::models::account::TronAccountUsage,
    ) -> Result<TransactionFee, Box<dyn Error + Send + Sync>> {
        let estimated_energy = self
            .estimate_trc20_transfer_gas(sender_address, token_id, format_address_parameter(&destination_address)?, value)
            .await?;
        let token_fee = calculate_transfer_token_fee_rate(chain_parameters, account_usage, estimated_energy)?;

        Ok(TransactionFee::new_gas_price_type(
            GasPriceType::regular(BigInt::from(token_fee.energy_price)),
            BigInt::from(token_fee.fee),
            BigInt::from(token_fee.fee_limit),
            HashMap::new(),
        ))
    }

    async fn get_is_new_account_for_input_type(&self, address: &str, input_type: TransactionInputType) -> Result<bool, Box<dyn Error + Send + Sync>> {
        match input_type {
            TransactionInputType::Transfer(asset)
            | TransactionInputType::TransferNft(asset, _)
            | TransactionInputType::Account(asset, _)
            | TransactionInputType::Swap(asset, _, _) => match asset.id.token_subtype() {
                AssetSubtype::NATIVE => Ok(self.is_new_account(address).await?),
                AssetSubtype::TOKEN => Ok(false),
            },
            _ => Ok(false),
        }
    }

    async fn get_stake_data(&self, input: &TransactionLoadInput) -> Result<TronStakeData, Box<dyn Error + Send + Sync>> {
        match &input.input_type {
            TransactionInputType::Stake(asset, stake_type) => {
                let account = self.get_account(&input.sender_address).await?;
                let amount = BigNumberFormatter::value_as_u64(&input.value, asset.decimals as u32)?;
                let mut votes: HashMap<String, u64> = account
                    .votes
                    .as_ref()
                    .map(|v| v.iter().map(|v| (v.vote_address.clone(), v.vote_count)).collect())
                    .unwrap_or_default();

                match stake_type {
                    StakeType::Stake(v) => *votes.entry(v.id.clone()).or_default() += amount,
                    StakeType::Unstake(d) => {
                        votes.entry(d.base.validator_id.clone()).and_modify(|v| *v = v.saturating_sub(amount));
                        votes.retain(|_, v| *v > 0);
                        if votes.is_empty() {
                            return Ok(TronStakeData::Unfreeze(calculate_unfreeze_amounts(
                                account.frozen_v2.as_ref(),
                                BigNumberFormatter::value_as_u64(&input.value, 0)?,
                            )));
                        }
                    }
                    StakeType::Redelegate(r) => {
                        votes.entry(r.delegation.base.validator_id.clone()).and_modify(|v| *v = v.saturating_sub(amount));
                        *votes.entry(r.to_validator.id.clone()).or_default() += amount;
                    }
                    StakeType::Rewards(_) | StakeType::Withdraw(_) | StakeType::Freeze(_) => {}
                }

                let votes = votes
                    .into_iter()
                    .filter(|(_, count)| *count > 0)
                    .map(|(validator, count)| TronVote { validator, count })
                    .collect();
                Ok(TronStakeData::Votes(votes))
            }
            _ => Ok(TronStakeData::Votes(vec![])),
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_USDT_TOKEN_ID, create_test_client};
    use chain_traits::ChainTransactionLoad;
    use num_bigint::BigInt;
    use primitives::{Asset, AssetId, AssetType, Chain};

    #[tokio::test]
    async fn test_get_transaction_load_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let asset = Asset::from_chain(Chain::Tron);

        let input = TransactionLoadInput::mock_with_input_type(TransactionInputType::Transfer(asset));
        let input = TransactionLoadInput {
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: "TGas3vJWx6R9wZEq66T3p7T5QAkXHRzh2q".to_string(),
            ..input
        };

        let result = client.get_transaction_load(input).await?;

        assert!(result.fee.fee > BigInt::from(0), "Transfer fee should be calculated");

        if let TransactionLoadMetadata::Tron { block_number, .. } = result.metadata {
            assert!(block_number > 0, "Block number should be positive");
        } else {
            panic!("Expected Tron metadata");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_get_transaction_load_token_transfer() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();
        let asset_id = AssetId::from(Chain::Tron, Some(TEST_USDT_TOKEN_ID.to_string()));
        let asset = Asset::new(asset_id, "Tether USD".to_string(), "USDT".to_string(), 6, AssetType::TRC20);

        let input = TransactionLoadInput::mock_with_input_type(TransactionInputType::Transfer(asset));
        let input = TransactionLoadInput {
            sender_address: TEST_ADDRESS.to_string(),
            destination_address: "TGas3vJWx6R9wZEq66T3p7T5QAkXHRzh2q".to_string(),
            ..input
        };

        let result = client.get_transaction_load(input).await?;

        assert!(result.fee.gas_limit > result.fee.fee, "Fee limit should be greater than estimated fee");
        assert!(result.fee.gas_limit > BigInt::from(0), "Gas limit should be greater than 0");

        if let TransactionLoadMetadata::Tron { block_number, .. } = result.metadata {
            assert!(block_number > 0, "Block number should be positive");
        } else {
            panic!("Expected Tron metadata");
        }

        Ok(())
    }
}
