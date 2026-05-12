use super::{
    constants::{FUNCTION_TRANSFER_OR_DESTROY_COIN, MODULE_ROUTER},
    error::{sui_error, tx_error},
    swap::build_swap,
};
use crate::{Quote, SwapperError, cetus::model::RouterData, fees::ReferralFee};
use gem_sui::{
    is_sui_coin,
    models::{CoinAsset, TxOutput},
    tx_builder::{ObjectResolver, TransactionBuilderInput, build_input_coin, finish_transaction, move_call},
};
use sui_transaction_builder::TransactionBuilder;

#[derive(Clone)]
pub(super) struct BuildInput<'a> {
    pub(super) transaction: TransactionBuilderInput,
    pub(super) from_coin_type: &'a str,
    pub(super) target_coin_type: &'a str,
    pub(super) amount: u64,
    pub(super) from_coins: &'a [CoinAsset],
    pub(super) target_merge_coin: Option<&'a CoinAsset>,
}

impl BuildInput<'_> {
    pub(super) fn with_gas_budget(&self, gas_budget: u64) -> Self {
        Self {
            transaction: self.transaction.with_gas_budget(gas_budget),
            ..self.clone()
        }
    }
}

pub(super) fn build_transaction(
    resolver: &ObjectResolver,
    quote: &Quote,
    router: &RouterData,
    referral_fee: &ReferralFee,
    input: &BuildInput<'_>,
) -> Result<TxOutput, SwapperError> {
    let mut txb = TransactionBuilder::new();
    let input_coin = build_input_coin(&mut txb, input.from_coin_type, input.amount, input.from_coins).map_err(sui_error)?;
    let target_coin = build_swap(&mut txb, resolver, quote, router, referral_fee, input_coin)?;

    if is_sui_coin(input.target_coin_type) {
        let gas = txb.gas();
        txb.merge_coins(gas, vec![target_coin]);
    } else if let Some(merge_coin) = input.target_merge_coin {
        let coin = txb.object(merge_coin.to_input());
        txb.merge_coins(coin, vec![target_coin]);
    } else {
        move_call(
            &mut txb,
            &router.aggregator_v3(),
            MODULE_ROUTER,
            FUNCTION_TRANSFER_OR_DESTROY_COIN,
            &[input.target_coin_type],
            vec![target_coin],
        )
        .map_err(tx_error)?;
    }

    finish_transaction(txb, input.transaction.clone()).map_err(sui_error)
}
