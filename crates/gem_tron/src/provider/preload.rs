use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;

use gem_client::Client;
use primitives::{
    AssetSubtype, FeePriority, FeeRate, GasPriceType, StakeType, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput,
    TransactionLoadMetadata, TransactionPreloadInput,
};

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
            TransactionInputType::Transfer(asset) | TransactionInputType::TransferNft(asset, _) | TransactionInputType::Account(asset, _) => {
                match &asset.id.token_id {
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
                }
            }
            TransactionInputType::Stake(_asset, stake_type) => {
                TransactionFee::new_from_fee(calculate_stake_fee_rate(&chain_parameters, &account_usage, stake_type)?)
            }
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
        let (total_fee, _chargeable_energy, energy_price) =
            calculate_transfer_token_fee_rate(chain_parameters, account_usage, BigInt::from_str(&estimated_energy)?)?;
        let gas_price_type = GasPriceType::regular(energy_price);

        Ok(TransactionFee::new_gas_price_type(gas_price_type, total_fee.clone(), total_fee, HashMap::new()))
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
