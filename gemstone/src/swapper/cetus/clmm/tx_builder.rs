use crate::sui::rpc::CoinAsset;
use anyhow::{anyhow, Result};
use gem_sui::{sui_clock_object, SUI_COIN_TYPE_FULL, SUI_FRAMEWORK_PACKAGE_ID};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use std::str::FromStr;

use sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{Argument, Command, ObjectArg},
    Identifier, TypeTag,
};

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
pub struct SwapParams {
    pub pool_object_shared: SharedObject,
    pub a2b: bool,
    pub by_amount_in: bool,
    pub amount: BigInt,
    pub amount_limit: BigInt,
    pub coin_type_a: String,
    pub coin_type_b: String,
    pub swap_partner: Option<ObjectRef>,
}

#[derive(Debug, Clone)]
pub struct CetusConfig {
    pub global_config: SharedObject,
    pub clmm_pool: ObjectID,
    pub router: ObjectID,
}

#[derive(Debug, Clone)]
pub struct SharedObject {
    pub id: ObjectID,
    pub shared_version: u64,
}

impl SharedObject {
    pub fn initial_shared_version(&self) -> SequenceNumber {
        SequenceNumber::from_u64(self.shared_version)
    }
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

pub struct TransactionBuilder;

impl TransactionBuilder {
    pub fn build_zero_value_coin(all_coins: &[CoinAsset], ptb: &mut ProgrammableTransactionBuilder, coin_type: &str) -> Result<BuildCoinResult> {
        let move_call = Command::move_call(
            ObjectID::from_single_byte(SUI_FRAMEWORK_PACKAGE_ID),
            Identifier::from_str(MODULE_COIN)?,
            Identifier::from_str(FUNCTION_ZERO)?,
            vec![TypeTag::from_str(coin_type)?],
            vec![],
        );
        let target_coin = ptb.command(move_call);

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
    ) -> Result<BuildCoinResult> {
        let coin_assets = CoinAssist::get_coin_assets(coin_type, all_coins);
        // Mint zero coin if amount is 0
        if amount == &BigInt::from(0u64) {
            return Self::build_zero_value_coin(all_coins, ptb, coin_type);
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

        Self::build_one_coin(ptb, &coin_assets, amount, coin_type, fix_amount)
    }

    fn build_one_coin(
        ptb: &mut ProgrammableTransactionBuilder,
        coin_assets: &[CoinAsset],
        amount: &BigInt,
        coin_type: &str,
        _fix_amount: bool,
    ) -> Result<BuildCoinResult> {
        if coin_type == SUI_COIN_TYPE_FULL {
            if coin_assets.len() > 1 {
                let results = CoinAssist::select_coins_gte(coin_assets, amount);
                let target_coin = &results.0[0];
                return Ok(BuildCoinResult {
                    target_coin: ptb.obj(ObjectArg::ImmOrOwnedObject(target_coin.to_ref()))?,
                    remain_coins: results.1,
                    is_mint_zero_coin: false,
                    target_coin_amount: target_coin.balance.to_string(),
                    original_splited_coin: None,
                });
            } else {
                // split gas coin
                let amount_arg = ptb.pure(amount.to_u64().unwrap())?;
                let target_coin = ptb.command(Command::SplitCoins(Argument::GasCoin, vec![amount_arg]));
                return Ok(BuildCoinResult {
                    target_coin,
                    remain_coins: vec![],
                    is_mint_zero_coin: false,
                    target_coin_amount: amount.to_string(),
                    original_splited_coin: None,
                });
            }
        }
        todo!("build_split_target_coin")
    }

    pub fn build_swap_transaction_args(
        ptb: &mut ProgrammableTransactionBuilder,
        params: &SwapParams,
        cetus_config: &CetusConfig,
        primary_coin_input_a: &BuildCoinResult,
        primary_coin_input_b: &BuildCoinResult,
    ) -> Result<()> {
        let sqrt_price_limit = if params.a2b { MIN_PRICE_SQRT_LIMIT } else { MAX_PRICE_SQRT_LIMIT };
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
            id: cetus_config.global_config.id,
            initial_shared_version: cetus_config.global_config.initial_shared_version(),
            mutable: true,
        };
        args.push(ptb.obj(global_obj_arg)?);

        // Add pool object
        let pool_obj_arg = ObjectArg::SharedObject {
            id: params.pool_object_shared.id,
            initial_shared_version: params.pool_object_shared.initial_shared_version(),
            mutable: true,
        };
        args.push(ptb.obj(pool_obj_arg)?);

        // Add swap partner if needed
        if has_swap_partner {
            let partner_obj_arg = ObjectArg::ImmOrOwnedObject(params.swap_partner.unwrap());
            args.push(ptb.obj(partner_obj_arg)?);
        }

        // Add coin inputs
        args.push(primary_coin_input_a.target_coin);
        args.push(primary_coin_input_b.target_coin);

        // Add by_amount_in
        args.push(ptb.pure(params.by_amount_in)?);

        // Add amount
        args.push(ptb.pure(params.amount.to_u64().unwrap_or(0))?);

        // Add amount_limit
        args.push(ptb.pure(params.amount_limit.to_u64().unwrap_or(0))?);

        // Add sqrt_price_limit
        args.push(ptb.pure(sqrt_price_limit)?);

        // Add clock
        args.push(ptb.obj(sui_clock_object())?);

        // Make the move call
        ptb.programmable_move_call(
            cetus_config.router,
            Identifier::from_str(MODULE_POOL_SCRIPT_V2)?,
            Identifier::from_str(function_name)?,
            type_arguments,
            args,
        );

        Ok(())
    }

    pub fn build_swap_transaction(cetus_config: &CetusConfig, params: &SwapParams, all_coin_asset: &[CoinAsset]) -> Result<ProgrammableTransactionBuilder> {
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
    pub fn select_coins_gte(_coin_assets: &[CoinAsset], _amount: &BigInt) -> (Vec<CoinAsset>, Vec<CoinAsset>) {
        todo!()
    }
}
