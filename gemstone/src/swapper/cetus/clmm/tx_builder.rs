use crate::sui::rpc::{CoinAsset, SuiClient};
use anyhow::{anyhow, Result};
use bcs;
use gem_sui::sui_clock_object;
use num_bigint::BigInt;
use std::str::FromStr;
use sui_types::{
    base_types::{ObjectDigest, ObjectID, ObjectRef, SequenceNumber, SuiAddress},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, ObjectArg, ProgrammableTransaction, TransactionData},
    Identifier, TypeTag,
};

// Helper function to create ObjectRef from ObjectID
#[allow(dead_code)]
fn object_id_to_ref(id: ObjectID) -> (ObjectID, SequenceNumber, ObjectDigest) {
    (id, SequenceNumber::from_u64(0), ObjectDigest::new([0; 32]))
}

#[derive(Debug, Clone)]
pub struct SwapParams {
    pub pool_ref: ObjectRef,
    pub a2b: bool,
    pub by_amount_in: bool,
    pub amount: BigInt,
    pub amount_limit: BigInt,
    pub coin_type_a: String,
    pub coin_type_b: String,
    pub swap_partner: Option<ObjectRef>,
}

#[derive(Debug, Clone)]
pub struct ClmmPoolConfig {
    pub package_id: ObjectID,
    pub published_at: String,
    pub global_config_id: ObjectID,
    pub global_config_shared_version: SequenceNumber,
}

#[derive(Debug, Clone)]
pub struct IntegrateConfig {
    pub package_id: ObjectID,
    pub published_at: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BuildCoinResult {
    pub target_coin: Argument,
    pub remain_coins: Vec<CoinAsset>,
    pub is_mint_zero_coin: bool,
    pub target_coin_amount: String,
    pub original_splited_coin: Option<ObjectID>,
}

// Constants
const CLMM_INTEGRATE_POOL_V2_MODULE: &str = "clmm_integrate";
const SWAP_WITH_PARTNER_A2B: &str = "swap_a2b_with_partner";
const SWAP_WITH_PARTNER_B2A: &str = "swap_b2a_with_partner";
const SWAP_A2B: &str = "swap_a2b";
const SWAP_B2A: &str = "swap_b2a";

pub struct TransactionBuilder;

impl TransactionBuilder {
    pub fn call_mint_zero_value_coin(tx_builder: &mut ProgrammableTransactionBuilder, coin_type: &str) -> Result<Argument> {
        let zero_coin_arg = tx_builder.pure(0u64)?;

        // Create the move call command
        let move_call = sui_types::transaction::Command::move_call(
            ObjectID::from_str("0x2").unwrap(),
            Identifier::new("coin").unwrap(),
            Identifier::new("zero").unwrap(),
            vec![TypeTag::from_str(coin_type)?],
            vec![zero_coin_arg],
        );

        Ok(tx_builder.command(move_call))
    }

    pub fn build_zero_value_coin(
        all_coins: &[CoinAsset],
        tx_builder: &mut ProgrammableTransactionBuilder,
        coin_type: &str,
        build_vector: bool,
    ) -> Result<BuildCoinResult> {
        let zero_coin = Self::call_mint_zero_value_coin(tx_builder, coin_type)?;

        let target_coin = if build_vector {
            // We can't use make_obj_vec directly with Argument, so we'll just use the zero_coin directly
            zero_coin
        } else {
            zero_coin
        };

        Ok(BuildCoinResult {
            target_coin,
            remain_coins: all_coins.to_vec(),
            is_mint_zero_coin: true,
            target_coin_amount: "0".to_string(),
            original_splited_coin: None,
        })
    }

    pub fn build_coin_for_amount(
        tx_builder: &mut ProgrammableTransactionBuilder,
        all_coins: &[CoinAsset],
        amount: &BigInt,
        coin_type: &str,
        build_vector: bool,
        fix_amount: bool,
    ) -> Result<BuildCoinResult> {
        let coin_assets = CoinAssist::get_coin_assets(coin_type, all_coins);

        // Mint zero coin if amount is 0
        if amount == &BigInt::from(0u64) {
            return Self::build_zero_value_coin(all_coins, tx_builder, coin_type, build_vector);
        }

        let amount_total = BigInt::from(CoinAssist::calculate_total_balance(&coin_assets));
        if amount_total < *amount {
            return Err(anyhow!(
                "The amount({}) is Insufficient balance for {}, expect {}",
                amount_total,
                coin_type,
                amount
            ));
        }

        Self::build_coin(tx_builder, all_coins, &coin_assets, amount, coin_type, build_vector, fix_amount)
    }

    fn build_coin(
        tx_builder: &mut ProgrammableTransactionBuilder,
        all_coins: &[CoinAsset],
        coin_assets: &[CoinAsset],
        amount: &BigInt,
        coin_type: &str,
        build_vector: bool,
        fix_amount: bool,
    ) -> Result<BuildCoinResult> {
        if build_vector {
            Self::build_vector_coin(tx_builder, all_coins, coin_assets, amount, coin_type, fix_amount)
        } else {
            Self::build_one_coin(tx_builder, coin_assets, amount, coin_type, fix_amount)
        }
    }

    fn build_vector_coin(
        tx_builder: &mut ProgrammableTransactionBuilder,
        all_coins: &[CoinAsset],
        coin_assets: &[CoinAsset],
        amount: &BigInt,
        coin_type: &str,
        fix_amount: bool,
    ) -> Result<BuildCoinResult> {
        // Sort coins by balance (ascending)
        let mut sorted_coins = coin_assets.to_vec();
        sorted_coins.sort_by(|a, b| a.balance.cmp(&b.balance));

        let mut remaining_amount = amount.clone();
        let mut used_coins = Vec::new();
        let mut coin_args = Vec::new();

        // Try to find exact match first
        for coin in &sorted_coins {
            if BigInt::from(coin.balance) == *amount {
                let obj_arg = ObjectArg::ImmOrOwnedObject(coin.to_ref());
                let coin_arg = tx_builder.obj(obj_arg)?;
                // We can't use make_obj_vec with Argument, so we'll just use the coin_arg directly
                let vec_arg = coin_arg;

                let mut remain_coins = all_coins.to_vec();
                remain_coins.retain(|c| c.coin_object_id != coin.coin_object_id);

                return Ok(BuildCoinResult {
                    target_coin: vec_arg,
                    remain_coins,
                    is_mint_zero_coin: false,
                    target_coin_amount: amount.to_string(),
                    original_splited_coin: None,
                });
            }
        }

        // Otherwise, collect coins until we reach the amount
        for coin in &sorted_coins {
            if remaining_amount == BigInt::from(0u64) {
                break;
            }

            used_coins.push(coin.coin_object_id);
            let obj_arg = ObjectArg::ImmOrOwnedObject(coin.to_ref());
            let coin_arg = tx_builder.obj(obj_arg)?;
            coin_args.push(coin_arg);

            let coin_balance = BigInt::from(coin.balance);
            if coin_balance <= remaining_amount {
                remaining_amount -= coin_balance;
            } else {
                // Need to split this coin
                // Convert BigInt to u64 for the split operation
                let split_amount_u64 = u64::from_str(&(coin.balance - u64::from_str(&remaining_amount.to_string()).unwrap_or(0)).to_string()).unwrap_or(0);
                let split_amount = tx_builder.pure(split_amount_u64)?;
                let split_coin = tx_builder.programmable_move_call(
                    ObjectID::from_str("0x2").unwrap(),
                    Identifier::new("coin").unwrap(),
                    Identifier::new("split").unwrap(),
                    vec![TypeTag::from_str(coin_type)?],
                    vec![coin_arg, split_amount],
                );

                // Replace the last coin with the split result
                coin_args.pop();
                coin_args.push(split_coin);
                remaining_amount = BigInt::from(0u64);
            }
        }

        if remaining_amount > BigInt::from(0u64) {
            return Err(anyhow!("Insufficient balance: needed {} more of {}", remaining_amount, coin_type));
        }

        // If we need to merge coins
        let merged_coin = if coin_args.len() > 1 && fix_amount {
            let first_coin = coin_args[0];
            // We can't use make_obj_vec with Argument, so we'll handle this differently
            // For now, just use the first coin
            let rest_coins = if coin_args.len() > 1 { coin_args[1] } else { first_coin };

            tx_builder.programmable_move_call(
                ObjectID::from_str("0x2").unwrap(),
                Identifier::new("coin").unwrap(),
                Identifier::new("join").unwrap(),
                vec![TypeTag::from_str(coin_type)?],
                vec![first_coin, rest_coins],
            )
        } else if coin_args.len() == 1 {
            coin_args[0]
        } else {
            // We can't use make_obj_vec with Argument, so we'll just use the first coin if available
            coin_args[0]
        };

        let mut remain_coins = all_coins.to_vec();
        remain_coins.retain(|c| !used_coins.contains(&c.coin_object_id));

        Ok(BuildCoinResult {
            target_coin: merged_coin,
            remain_coins,
            is_mint_zero_coin: false,
            target_coin_amount: amount.to_string(),
            original_splited_coin: None,
        })
    }

    fn build_one_coin(
        tx_builder: &mut ProgrammableTransactionBuilder,
        coin_assets: &[CoinAsset],
        amount: &BigInt,
        coin_type: &str,
        fix_amount: bool,
    ) -> Result<BuildCoinResult> {
        // Sort coins by balance (ascending)
        let mut sorted_coins = coin_assets.to_vec();
        sorted_coins.sort_by(|a, b| a.balance.cmp(&b.balance));

        // Try to find exact match first
        for coin in &sorted_coins {
            if BigInt::from(coin.balance) == *amount {
                let obj_arg = ObjectArg::ImmOrOwnedObject(coin.to_ref());
                let coin_arg = tx_builder.obj(obj_arg)?;

                let mut remain_coins = coin_assets.to_vec();
                remain_coins.retain(|c| c.coin_object_id != coin.coin_object_id);

                return Ok(BuildCoinResult {
                    target_coin: coin_arg,
                    remain_coins,
                    is_mint_zero_coin: false,
                    target_coin_amount: amount.to_string(),
                    original_splited_coin: None,
                });
            }
        }

        // Find a coin with sufficient balance
        for coin in &sorted_coins {
            if BigInt::from(coin.balance) > *amount {
                let obj_arg = ObjectArg::ImmOrOwnedObject(coin.to_ref());
                let coin_arg = tx_builder.obj(obj_arg)?;

                if fix_amount {
                    // Split the coin
                    // Convert BigInt to u64 for split operation
                    let amount_u64 = u64::from_str(&amount.to_string()).unwrap_or(0);
                    let split_amount = tx_builder.pure(coin.balance - amount_u64)?;
                    let split_result = tx_builder.programmable_move_call(
                        ObjectID::from_str("0x2").unwrap(),
                        Identifier::new("coin").unwrap(),
                        Identifier::new("split").unwrap(),
                        vec![TypeTag::from_str(coin_type)?],
                        vec![coin_arg, split_amount],
                    );

                    let mut remain_coins = coin_assets.to_vec();
                    remain_coins.retain(|c| c.coin_object_id != coin.coin_object_id);

                    return Ok(BuildCoinResult {
                        target_coin: split_result,
                        remain_coins,
                        is_mint_zero_coin: false,
                        target_coin_amount: amount.to_string(),
                        original_splited_coin: Some(coin.coin_object_id),
                    });
                } else {
                    let mut remain_coins = coin_assets.to_vec();
                    remain_coins.retain(|c| c.coin_object_id != coin.coin_object_id);

                    return Ok(BuildCoinResult {
                        target_coin: coin_arg,
                        remain_coins,
                        is_mint_zero_coin: false,
                        target_coin_amount: coin.balance.to_string(),
                        original_splited_coin: None,
                    });
                }
            }
        }

        // If no single coin is sufficient, we need to merge coins
        let mut remaining_amount = amount.clone();
        let mut used_coins = Vec::new();
        let mut coin_args = Vec::new();

        for coin in &sorted_coins {
            if remaining_amount == BigInt::from(0u64) {
                break;
            }

            used_coins.push(coin.coin_object_id);
            let obj_arg = ObjectArg::ImmOrOwnedObject(coin.to_ref());
            let coin_arg = tx_builder.obj(obj_arg)?;
            coin_args.push(coin_arg);

            let coin_balance = BigInt::from(coin.balance);
            if coin_balance <= remaining_amount {
                remaining_amount -= coin_balance;
            } else {
                // Need to split this coin
                // Convert BigInt to u64 for split operation
                let split_amount_u64 = u64::from_str(&(coin.balance - u64::from_str(&remaining_amount.to_string()).unwrap_or(0)).to_string()).unwrap_or(0);
                let split_amount = tx_builder.pure(split_amount_u64)?;
                let split_coin = tx_builder.programmable_move_call(
                    ObjectID::from_str("0x2").unwrap(),
                    Identifier::new("coin").unwrap(),
                    Identifier::new("split").unwrap(),
                    vec![TypeTag::from_str(coin_type)?],
                    vec![coin_arg, split_amount],
                );

                // Replace the last coin with the split result
                coin_args.pop();
                coin_args.push(split_coin);
                remaining_amount = BigInt::from(0u64);
            }
        }

        if remaining_amount > BigInt::from(0u64) {
            return Err(anyhow!("Insufficient balance: needed {} more of {}", remaining_amount, coin_type));
        }

        // Merge coins if needed
        let merged_coin = if coin_args.len() > 1 {
            let first_coin = coin_args[0];

            for coin_arg in coin_args.iter().skip(1) {
                tx_builder.programmable_move_call(
                    ObjectID::from_str("0x2").unwrap(),
                    Identifier::new("coin").unwrap(),
                    Identifier::new("join").unwrap(),
                    vec![TypeTag::from_str(coin_type)?],
                    vec![first_coin, *coin_arg],
                );
            }

            first_coin
        } else {
            coin_args[0]
        };

        let mut remain_coins = coin_assets.to_vec();
        remain_coins.retain(|c| !used_coins.contains(&c.coin_object_id));

        Ok(BuildCoinResult {
            target_coin: merged_coin,
            remain_coins,
            is_mint_zero_coin: false,
            target_coin_amount: amount.to_string(),
            original_splited_coin: None,
        })
    }

    pub fn build_swap_transaction_args(
        tx_builder: &mut ProgrammableTransactionBuilder,
        params: &SwapParams,
        clmm_pool: &ClmmPoolConfig,
        integrate: &IntegrateConfig,
        primary_coin_input_a: &BuildCoinResult,
        primary_coin_input_b: &BuildCoinResult,
    ) -> Result<()> {
        let sqrt_price_limit = get_default_sqrt_price_limit(params.a2b);
        let type_arguments = vec![TypeTag::from_str(&params.coin_type_a)?, TypeTag::from_str(&params.coin_type_b)?];

        let has_swap_partner = params.swap_partner.is_some();

        let function_name = if has_swap_partner {
            if params.a2b {
                SWAP_WITH_PARTNER_A2B
            } else {
                SWAP_WITH_PARTNER_B2A
            }
        } else if params.a2b {
            SWAP_A2B
        } else {
            SWAP_B2A
        };

        let mut args = Vec::new();

        // Add global config
        let global_obj_arg = ObjectArg::SharedObject {
            id: clmm_pool.global_config_id,
            initial_shared_version: clmm_pool.global_config_shared_version,
            mutable: true,
        };
        args.push(tx_builder.obj(global_obj_arg)?);

        // Add pool id
        let pool_obj_arg = ObjectArg::ImmOrOwnedObject(params.pool_ref);
        args.push(tx_builder.obj(pool_obj_arg)?);

        // Add swap partner if needed
        if has_swap_partner {
            let partner_obj_arg = ObjectArg::ImmOrOwnedObject(params.swap_partner.unwrap());
            args.push(tx_builder.obj(partner_obj_arg)?);
        }

        // Add coin inputs
        args.push(primary_coin_input_a.target_coin);
        args.push(primary_coin_input_b.target_coin);

        // Add by_amount_in
        args.push(tx_builder.pure(params.by_amount_in)?);

        // Add amount
        args.push(tx_builder.pure(u64::from_str(&params.amount.to_string()).unwrap_or(0))?);

        // Add amount_limit
        args.push(tx_builder.pure(u64::from_str(&params.amount_limit.to_string()).unwrap_or(0))?);

        // Add sqrt_price_limit
        args.push(tx_builder.pure(u128::from_str(&sqrt_price_limit).unwrap_or(0))?);

        // Add clock
        args.push(tx_builder.obj(sui_clock_object())?);

        // Make the move call
        tx_builder.programmable_move_call(
            integrate.package_id,
            Identifier::from_str(CLMM_INTEGRATE_POOL_V2_MODULE).unwrap(),
            Identifier::from_str(function_name).unwrap(),
            type_arguments,
            args,
        );

        Ok(())
    }

    pub fn build_swap_transaction(
        _client: &SuiClient,
        clmm_pool: &ClmmPoolConfig,
        integrate: &IntegrateConfig,
        params: &SwapParams,
        all_coin_asset: &[CoinAsset],
    ) -> Result<ProgrammableTransactionBuilder> {
        let mut tx_builder = ProgrammableTransactionBuilder::new();

        let amount_a = if params.a2b {
            if params.by_amount_in {
                params.amount.clone()
            } else {
                params.amount_limit.clone()
            }
        } else {
            BigInt::from(0u64)
        };

        let primary_coin_input_a = Self::build_coin_for_amount(
            &mut tx_builder,
            all_coin_asset,
            &amount_a,
            &params.coin_type_a,
            false,
            true,
        )?;

        let amount_b = if !params.a2b {
            if params.by_amount_in {
                params.amount.clone()
            } else {
                params.amount_limit.clone()
            }
        } else {
            BigInt::from(0u64)
        };

        let primary_coin_input_b = Self::build_coin_for_amount(
            &mut tx_builder,
            all_coin_asset,
            &amount_b,
            &params.coin_type_b,
            false,
            true,
        )?;

        Self::build_swap_transaction_args(&mut tx_builder, params, clmm_pool, integrate, &primary_coin_input_a, &primary_coin_input_b)?;

        Ok(tx_builder)
    }

    #[allow(dead_code)]
    pub async fn adjust_transaction_for_gas(
        client: &SuiClient,
        all_coins: &[CoinAsset],
        amount: u64,
        sender_address: &str,
        tx: &ProgrammableTransaction,
    ) -> Result<(u64, Option<ProgrammableTransactionBuilder>)> {
        // Get amount coins
        let amount_coins = CoinAssist::select_coin_asset_greater_than_or_equal(all_coins, amount, None);
        if amount_coins.is_empty() {
            return Err(anyhow!("Insufficient balance"));
        }

        let total_amount = CoinAssist::calculate_total_balance(all_coins);

        // If the remaining coin balance is greater than 1000000000, no gas fee correction will be done
        if total_amount - amount > 1000000000 {
            return Ok((amount, None));
        }

        // Estimate gas consumption
        let estimate_gas = (Self::serialize_and_estimate_gas(client, sender_address, tx.clone()).await).unwrap_or(500000);

        // Find gas coins
        let exclude_ids = amount_coins.iter().map(|coin| coin.coin_object_id).collect::<Vec<_>>();
        let gas_coins = CoinAssist::select_coin_asset_greater_than_or_equal(all_coins, estimate_gas, Some(exclude_ids));

        // There is not enough gas and the amount needs to be adjusted
        if gas_coins.is_empty() {
            // Readjust the amount, reserve 500 gas for the split
            let new_gas = estimate_gas + 500;
            if total_amount - amount < new_gas {
                let new_amount = if amount > new_gas {
                    amount - new_gas
                } else {
                    return Err(anyhow!("Gas insufficient balance"));
                };

                let new_tx_builder = ProgrammableTransactionBuilder::new();
                return Ok((new_amount, Some(new_tx_builder)));
            }
        }

        Ok((amount, None))
    }

    #[allow(dead_code)]
    async fn serialize_and_estimate_gas(client: &SuiClient, sender_address: &str, tx: ProgrammableTransaction) -> Result<u64> {
        // Create a dummy transaction data with the builder
        // We need to use a dummy gas budget and gas price for the serialization
        let sender = SuiAddress::from_str(sender_address)?;

        // Use empty coin references for estimation purposes
        let coin_refs = Vec::new();

        // Use dummy values for gas budget and gas price
        let gas_budget = 50000000; // Dummy value
        let gas_price = 1000; // Dummy value
                              // Create the transaction data
        let tx_data = TransactionData::new_programmable(sender, coin_refs, tx, gas_budget, gas_price);

        // Serialize the transaction data to bytes using bincode
        let tx_bytes = bcs::to_bytes(&tx_data).map_err(|e| anyhow!("Failed to serialize transaction data: {}", e))?;

        // Estimate the gas cost
        client
            .estimate_gas_budget(sender_address, &tx_bytes)
            .await
            .map_err(|e| anyhow!("Failed to estimate gas: {}", e))
    }
}

// Helper structs and implementations
pub struct CoinAssist;

impl CoinAssist {
    pub fn get_coin_assets(coin_type: &str, all_coins: &[CoinAsset]) -> Vec<CoinAsset> {
        all_coins.iter().filter(|asset| asset.coin_type == coin_type).cloned().collect()
    }

    pub fn calculate_total_balance(coin_assets: &[CoinAsset]) -> u64 {
        coin_assets.iter().map(|asset| asset.balance).sum()
    }

    #[allow(dead_code)]
    pub fn select_coin_asset_greater_than_or_equal(all_coins: &[CoinAsset], amount: u64, exclude_ids: Option<Vec<ObjectID>>) -> Vec<CoinAsset> {
        let mut result = Vec::new();
        let mut total = 0u64;

        // Filter out excluded coins
        let filtered_coins = if let Some(ids) = &exclude_ids {
            all_coins.iter().filter(|coin| !ids.contains(&coin.coin_object_id)).cloned().collect::<Vec<_>>()
        } else {
            all_coins.to_vec()
        };

        // Sort coins by balance (descending)
        let mut sorted_coins = filtered_coins;
        sorted_coins.sort_by(|a, b| b.balance.cmp(&a.balance));

        // Try to find a single coin with sufficient balance
        for coin in &sorted_coins {
            if coin.balance >= amount {
                return vec![coin.clone()];
            }
        }

        // Otherwise, collect coins until we reach the amount
        for coin in sorted_coins {
            if total >= amount {
                break;
            }

            result.push(coin.clone());
            total += coin.balance;
        }

        if total < amount {
            // Not enough coins to reach the amount
            return Vec::new();
        }

        result
    }
}

pub fn get_default_sqrt_price_limit(a2b: bool) -> String {
    if a2b {
        "4295048016".to_string()
    } else {
        "79226673515401279992447579055".to_string()
    }
}
