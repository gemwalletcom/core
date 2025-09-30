use crate::sui::rpc::CoinAsset;
use gem_sui::{ObjectId, SUI_COIN_TYPE_FULL, SUI_FRAMEWORK_PACKAGE_ID, sui_clock_object_input};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::error::Error;
use std::str::FromStr;

use sui_transaction_builder::{Function, Serialized, TransactionBuilder as ProgrammableTransactionBuilder, unresolved::Input};
use sui_types::{Address, Argument, Identifier, TypeTag};

use super::models::{CetusConfig, SwapParams};

// Constants
const SWAP_WITH_PARTNER_A2B: &str = "swap_a2b_with_partner";
const SWAP_WITH_PARTNER_B2A: &str = "swap_b2a_with_partner";
const SWAP_A2B: &str = "swap_a2b";
const SWAP_B2A: &str = "swap_b2a";
const MODULE_COIN: &str = "coin";
const MODULE_POOL_SCRIPT_V2: &str = "pool_script_v2";
const FUNCTION_ZERO: &str = "zero";
const MIN_PRICE_SQRT_LIMIT: u128 = 4295048016_u128;
const MAX_PRICE_SQRT_LIMIT: u128 = 79226673515401279992447579055_u128;

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct BuildCoinResult {
    pub target_coin: Argument,
    pub remain_coins: Vec<CoinAsset>,
    pub is_mint_zero_coin: bool,
    pub target_coin_amount: String,
    pub original_splited_coin: Option<Address>,
}

pub struct TransactionBuilder;

impl TransactionBuilder {
    pub fn build_zero_value_coin(
        all_coins: &[CoinAsset],
        ptb: &mut ProgrammableTransactionBuilder,
        coin_type: &str,
    ) -> Result<BuildCoinResult, Box<dyn Error + Send + Sync>> {
        let function = Function::new(
            ObjectId::from(SUI_FRAMEWORK_PACKAGE_ID).into(),
            Identifier::from_str(MODULE_COIN)?,
            Identifier::from_str(FUNCTION_ZERO)?,
            vec![TypeTag::from_str(coin_type)?],
        );
        let target_coin = ptb.move_call(function, vec![]);

        Ok(BuildCoinResult {
            target_coin,
            remain_coins: all_coins.to_vec(),
            is_mint_zero_coin: true,
            target_coin_amount: "0".to_string(),
            original_splited_coin: None,
        })
    }

    pub fn build_coin_for_amount(
        ptb: &mut ProgrammableTransactionBuilder,
        all_coins: &[CoinAsset],
        amount: &BigInt,
        coin_type: &str,
        fix_amount: bool,
    ) -> Result<BuildCoinResult, Box<dyn Error + Send + Sync>> {
        let coin_assets = CoinAssist::get_coin_assets(coin_type, all_coins);
        // Mint zero coin if amount is 0
        if amount == &BigInt::from(0u64) {
            return Self::build_zero_value_coin(all_coins, ptb, coin_type);
        }

        let amount_total = CoinAssist::calculate_total_balance(&coin_assets);
        if amount_total < *amount {
            return Err(format!("The amount({}) is Insufficient balance for {}, expect {}", amount_total, coin_type, amount).into());
        }

        Self::build_one_coin(ptb, &coin_assets, amount, coin_type, fix_amount)
    }

    fn build_one_coin(
        ptb: &mut ProgrammableTransactionBuilder,
        coin_assets: &[CoinAsset],
        amount: &BigInt,
        coin_type: &str,
        fix_amount: bool,
    ) -> Result<BuildCoinResult, Box<dyn Error + Send + Sync>> {
        if coin_type == SUI_COIN_TYPE_FULL {
            if amount == &BigInt::from(0) && coin_assets.len() > 1 {
                let results = CoinAssist::select_coins_gte(coin_assets, amount);
                let target_coin = &results.0[0];
                return Ok(BuildCoinResult {
                    target_coin: ptb.input(Input::owned(target_coin.coin_object_id, target_coin.version, target_coin.digest)),
                    remain_coins: results.1,
                    is_mint_zero_coin: false,
                    target_coin_amount: target_coin.balance.to_string(),
                    original_splited_coin: None,
                });
            }
            // split gas coin
            let amount_argument = ptb.input(Serialized(&amount.to_u64().unwrap_or(0)));
            let target_coin = ptb.split_coins(ptb.gas(), vec![amount_argument]);
            return Ok(BuildCoinResult {
                target_coin,
                remain_coins: vec![],
                is_mint_zero_coin: false,
                target_coin_amount: amount.to_string(),
                original_splited_coin: None,
            });
        }
        Self::build_split_target_coin(ptb, coin_assets, amount, fix_amount)
    }

    fn build_split_target_coin(
        ptb: &mut ProgrammableTransactionBuilder,
        coin_assets: &[CoinAsset],
        amount: &BigInt,
        fix_amount: bool,
    ) -> Result<BuildCoinResult, Box<dyn Error + Send + Sync>> {
        let (selected_coins, remain_coins) = CoinAssist::select_coins_gte(coin_assets, amount);

        if selected_coins.is_empty() {
            return Err("No coins selected for splitting".into());
        }

        let total_selected_amount = CoinAssist::calculate_total_balance(&selected_coins);

        // Split into primary coin and merge coins
        let mut coins_iter = selected_coins.iter();
        let primary_coin = coins_iter.next().unwrap();
        let merge_coins: Vec<_> = coins_iter.collect();

        let mut target_coin = ptb.input(Input::owned(primary_coin.coin_object_id, primary_coin.version, primary_coin.digest));
        let mut original_splited_coin = None;

        // Merge additional coins if any
        if !merge_coins.is_empty() {
            let merge_coin_args: Vec<Argument> = merge_coins
                .iter()
                .map(|coin| ptb.input(Input::owned(coin.coin_object_id, coin.version, coin.digest)))
                .collect::<Vec<_>>();

            ptb.merge_coins(target_coin, merge_coin_args);
        }

        // Split coin if needed
        if fix_amount && total_selected_amount > *amount {
            original_splited_coin = Some(primary_coin.coin_object_id);
            let amount_arg = ptb.input(Serialized(&amount.to_u64().unwrap_or(0)));
            target_coin = ptb.split_coins(target_coin, vec![amount_arg]);
        }

        Ok(BuildCoinResult {
            target_coin,
            remain_coins,
            is_mint_zero_coin: false,
            target_coin_amount: total_selected_amount.to_string(),
            original_splited_coin,
        })
    }

    pub fn build_swap_transaction_args(
        ptb: &mut ProgrammableTransactionBuilder,
        params: &SwapParams,
        cetus_config: &CetusConfig,
        primary_coin_input_a: &BuildCoinResult,
        primary_coin_input_b: &BuildCoinResult,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let sqrt_price_limit = if params.a2b { MIN_PRICE_SQRT_LIMIT } else { MAX_PRICE_SQRT_LIMIT };
        let type_arguments = vec![TypeTag::from_str(&params.coin_type_a)?, TypeTag::from_str(&params.coin_type_b)?];
        let has_swap_partner = params.swap_partner.is_some();

        let function_name = if has_swap_partner {
            if params.a2b { SWAP_WITH_PARTNER_A2B } else { SWAP_WITH_PARTNER_B2A }
        } else if params.a2b {
            SWAP_A2B
        } else {
            SWAP_B2A
        };

        let mut args = Vec::new();

        // Add global config
        let global_config = cetus_config.global_config.clone();
        args.push(ptb.input(Input::shared(global_config.id, global_config.shared_version, true)));

        // Add pool object
        let pool_obj = params.pool_object_shared.clone();
        args.push(ptb.input(Input::shared(pool_obj.id, pool_obj.shared_version, true)));

        // Add swap partner if needed
        if has_swap_partner {
            let partner_obj = params.swap_partner.clone().unwrap();
            args.push(ptb.input(Input::shared(partner_obj.id, partner_obj.shared_version, true)));
        }

        // Add coin inputs
        args.push(primary_coin_input_a.target_coin);
        args.push(primary_coin_input_b.target_coin);

        // Add by_amount_in
        args.push(ptb.input(Serialized(&params.by_amount_in)));

        // Add amount
        args.push(ptb.input(Serialized(&params.amount.to_u64().unwrap_or(0))));

        // Add amount_limit
        args.push(ptb.input(Serialized(&params.amount_limit.to_u64().unwrap_or(0))));

        // Add sqrt_price_limit
        args.push(ptb.input(Serialized(&sqrt_price_limit)));

        // Add clock
        args.push(ptb.input(sui_clock_object_input()));

        // Make the move call
        let function = Function::new(
            cetus_config.router,
            Identifier::from_str(MODULE_POOL_SCRIPT_V2)?,
            Identifier::from_str(function_name)?,
            type_arguments,
        );
        ptb.move_call(function, args);

        Ok(())
    }

    pub fn build_swap_transaction(
        cetus_config: &CetusConfig,
        params: &SwapParams,
        all_coin_asset: &[CoinAsset],
    ) -> Result<ProgrammableTransactionBuilder, Box<dyn Error + Send + Sync>> {
        let mut ptb = ProgrammableTransactionBuilder::new();

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
        let primary_coin_input_a = Self::build_coin_for_amount(&mut ptb, all_coin_asset, &amount_a, &params.coin_type_a, true)?;
        let primary_coin_input_b = Self::build_coin_for_amount(&mut ptb, all_coin_asset, &amount_b, &params.coin_type_b, true)?;

        // Build the transaction with the prepared coin inputs
        Self::build_swap_transaction_args(&mut ptb, params, cetus_config, &primary_coin_input_a, &primary_coin_input_b)?;

        Ok(ptb)
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

    // (selected(object_id, balance), remains_coins)
    pub fn select_coins_gte(coin_assets: &[CoinAsset], amount: &BigInt) -> (Vec<CoinAsset>, Vec<CoinAsset>) {
        // Sort coins by balance in descending order
        let mut sorted_coins: Vec<CoinAsset> = coin_assets.to_vec();
        sorted_coins.sort_by(|a, b| b.balance.cmp(&a.balance));

        let total = Self::calculate_total_balance(&sorted_coins);

        // If total is less than amount, return empty selected and all coins as remaining
        if total < *amount {
            return (vec![], sorted_coins);
        }

        // If total equals amount, return all coins as selected and empty remaining
        if total == *amount {
            return (sorted_coins, vec![]);
        }

        let mut sum = BigInt::from(0u64);
        let mut selected_coins = Vec::new();
        let mut remaining_coins = sorted_coins.clone();

        while sum < total {
            let target = amount - &sum;

            // Find coin with smallest sufficient balance
            if let Some((idx, _)) = remaining_coins.iter().enumerate().find(|(_, coin)| coin.balance >= target) {
                selected_coins.push(remaining_coins.remove(idx));
                break;
            }

            // If no coin with sufficient balance found, take the largest one
            if let Some(coin) = remaining_coins.pop() {
                if coin.balance > BigInt::from(0u64) {
                    sum += &coin.balance;
                    selected_coins.push(coin);
                }
            } else {
                break;
            }
        }

        // Sort both vectors by balance
        selected_coins.sort_by(|a, b| b.balance.cmp(&a.balance));
        remaining_coins.sort_by(|a, b| b.balance.cmp(&a.balance));

        (selected_coins, remaining_coins)
    }
}
