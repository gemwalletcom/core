use crate::sui::rpc::CoinAsset;
use anyhow::{anyhow, Result};
use gem_sui::{sui_clock_object, SUI_FRAMEWORK_PACKAGE_ID};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::str::FromStr;

use sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, Command, ObjectArg},
    Identifier, TypeTag,
};

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
const MODULE_COIN: &str = "coin";
const FUNCTION_ZERO: &str = "zero";
const FUNCTION_SPLIT: &str = "split";
const FUNCTION_JOIN: &str = "join";

pub struct TransactionBuilder;

impl TransactionBuilder {
    pub fn call_mint_zero_value_coin(tx_builder: &mut ProgrammableTransactionBuilder, coin_type: &str) -> Result<Argument> {
        let zero_coin_arg = tx_builder.pure(0u64)?;

        // Create the move call command
        let move_call = Command::move_call(
            ObjectID::from_single_byte(SUI_FRAMEWORK_PACKAGE_ID),
            Identifier::from_str(MODULE_COIN)?,
            Identifier::from_str(FUNCTION_ZERO)?,
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

        let amount_total = CoinAssist::calculate_total_balance(&coin_assets);
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

    // Helper function to split a coin
    fn split_coin(tx_builder: &mut ProgrammableTransactionBuilder, coin_arg: Argument, split_amount: BigInt, coin_type: &str) -> Result<Argument> {
        let split_u64 = split_amount.to_u64().ok_or(anyhow!("Failed to convert BigInt to u64"))?;
        let split_amount_arg = tx_builder.pure(split_u64)?;
        Ok(tx_builder.programmable_move_call(
            ObjectID::from_single_byte(SUI_FRAMEWORK_PACKAGE_ID),
            Identifier::from_str(MODULE_COIN)?,
            Identifier::from_str(FUNCTION_SPLIT)?,
            vec![TypeTag::from_str(coin_type)?],
            vec![coin_arg, split_amount_arg],
        ))
    }

    // Helper function to join coins
    fn join_coins(tx_builder: &mut ProgrammableTransactionBuilder, coin_a: Argument, coin_b: Argument, coin_type: &str) -> Result<Argument> {
        Ok(tx_builder.programmable_move_call(
            ObjectID::from_single_byte(SUI_FRAMEWORK_PACKAGE_ID),
            Identifier::from_str(MODULE_COIN)?,
            Identifier::from_str(FUNCTION_JOIN)?,
            vec![TypeTag::from_str(coin_type)?],
            vec![coin_a, coin_b],
        ))
    }

    // Helper function to find exact coin match
    fn find_exact_coin_match(
        tx_builder: &mut ProgrammableTransactionBuilder,
        all_coins: &[CoinAsset],
        sorted_coins: &[CoinAsset],
        amount: &BigInt,
    ) -> Result<Option<BuildCoinResult>> {
        for coin in sorted_coins {
            if coin.balance == *amount {
                let obj_arg = ObjectArg::ImmOrOwnedObject(coin.to_ref());
                let coin_arg = tx_builder.obj(obj_arg)?;

                let mut remain_coins = all_coins.to_vec();
                remain_coins.retain(|c| c.coin_object_id != coin.coin_object_id);

                return Ok(Some(BuildCoinResult {
                    target_coin: coin_arg,
                    remain_coins,
                    is_mint_zero_coin: false,
                    target_coin_amount: amount.to_string(),
                    original_splited_coin: None,
                }));
            }
        }
        Ok(None)
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

        // Try to find exact match first
        if let Some(result) = Self::find_exact_coin_match(tx_builder, all_coins, &sorted_coins, amount)? {
            return Ok(result);
        }

        let mut remaining_amount = amount.clone();
        let mut used_coins = Vec::new();
        let mut coin_args = Vec::new();

        // Otherwise, collect coins until we reach the amount
        for coin in &sorted_coins {
            if remaining_amount == BigInt::from(0u64) {
                break;
            }

            used_coins.push(coin.coin_object_id);
            let obj_arg = ObjectArg::ImmOrOwnedObject(coin.to_ref());
            let coin_arg = tx_builder.obj(obj_arg)?;
            coin_args.push(coin_arg);

            let coin_balance = coin.balance.clone();
            if coin_balance <= remaining_amount {
                remaining_amount -= coin_balance;
            } else {
                // Need to split this coin
                let split_amount = &coin.balance - &remaining_amount;
                let split_coin = Self::split_coin(tx_builder, coin_arg, split_amount, coin_type)?;

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

            Self::join_coins(tx_builder, first_coin, rest_coins, coin_type)?
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
        if let Some(result) = Self::find_exact_coin_match(tx_builder, coin_assets, &sorted_coins, amount)? {
            return Ok(result);
        }

        // Find a coin with sufficient balance
        for coin in &sorted_coins {
            if coin.balance > *amount {
                let obj_arg = ObjectArg::ImmOrOwnedObject(coin.to_ref());
                let coin_arg = tx_builder.obj(obj_arg)?;

                if fix_amount {
                    // Split the coin
                    let split_amount = coin.balance.clone() - amount;
                    let split_result = Self::split_coin(tx_builder, coin_arg, split_amount, coin_type)?;

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

            let coin_balance = coin.balance.clone();
            if coin_balance <= remaining_amount {
                remaining_amount -= coin_balance;
            } else {
                // Need to split this coin
                let split_amount = coin_balance - &remaining_amount;
                let split_coin = Self::split_coin(tx_builder, coin_arg, split_amount, coin_type)?;

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
                Self::join_coins(tx_builder, first_coin, *coin_arg, coin_type)?;
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
        args.push(tx_builder.pure(&params.amount)?);

        // Add amount_limit
        args.push(tx_builder.pure(&params.amount_limit)?);

        // Add sqrt_price_limit
        args.push(tx_builder.pure(sqrt_price_limit)?);

        // Add clock
        args.push(tx_builder.obj(sui_clock_object())?);

        // Make the move call
        tx_builder.programmable_move_call(
            integrate.package_id,
            Identifier::from_str(CLMM_INTEGRATE_POOL_V2_MODULE)?,
            Identifier::from_str(function_name)?,
            type_arguments,
            args,
        );

        Ok(())
    }

    pub fn build_swap_transaction(
        clmm_pool: &ClmmPoolConfig,
        integrate: &IntegrateConfig,
        params: &SwapParams,
        all_coin_asset: &[CoinAsset],
    ) -> Result<ProgrammableTransactionBuilder> {
        let mut tx_builder = ProgrammableTransactionBuilder::new();

        // Calculate the input amounts based on direction and swap mode
        let (amount_a, amount_b) = if params.a2b {
            if params.by_amount_in {
                (params.amount.clone(), BigInt::from(0u64))
            } else {
                (params.amount_limit.clone(), BigInt::from(0u64))
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if params.by_amount_in {
                (BigInt::from(0u64), params.amount.clone())
            } else {
                (BigInt::from(0u64), params.amount_limit.clone())
            }
        };

        // Build coin inputs for both sides of the swap
        let primary_coin_input_a = Self::build_coin_for_amount(&mut tx_builder, all_coin_asset, &amount_a, &params.coin_type_a, false, true)?;
        let primary_coin_input_b = Self::build_coin_for_amount(&mut tx_builder, all_coin_asset, &amount_b, &params.coin_type_b, false, true)?;

        // Build the transaction with the prepared coin inputs
        Self::build_swap_transaction_args(&mut tx_builder, params, clmm_pool, integrate, &primary_coin_input_a, &primary_coin_input_b)?;

        Ok(tx_builder)
    }
}

// Helper structs and implementations
pub struct CoinAssist;

impl CoinAssist {
    pub fn get_coin_assets(coin_type: &str, all_coins: &[CoinAsset]) -> Vec<CoinAsset> {
        all_coins.iter().filter(|asset| asset.coin_type == coin_type).cloned().collect()
    }

    pub fn calculate_total_balance(coin_assets: &[CoinAsset]) -> BigInt {
        coin_assets.iter().map(|asset| asset.balance.clone()).sum()
    }
}

pub fn get_default_sqrt_price_limit(a2b: bool) -> u128 {
    if a2b {
        4295048016_u128
    } else {
        79226673515401279992447579055_u128
    }
}
