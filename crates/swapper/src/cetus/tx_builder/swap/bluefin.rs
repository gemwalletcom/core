use super::super::{
    constants::{FUNCTION_SWAP, MODULE_BLUEFIN},
    error::tx_error,
    model::SwapStep,
};
use crate::{
    SwapperError,
    cetus::{constants::BLUEFIN_GLOBAL_CONFIG, model::FlattenedPath},
};
use gem_sui::{
    sui_clock_object_input,
    tx_builder::{ObjectResolver, move_call},
};
use sui_transaction_builder::{Argument, TransactionBuilder};

pub(super) fn build_swap(txb: &mut TransactionBuilder, resolver: &ObjectResolver, flattened_path: &FlattenedPath, swap_context: Argument) -> Result<(), SwapperError> {
    let step = SwapStep::try_from(flattened_path)?;
    let global_config = resolver.shared_object(txb, BLUEFIN_GLOBAL_CONFIG, true).map_err(tx_error)?;
    let pool = resolver.shared_object(txb, &step.path.id, true).map_err(tx_error)?;
    let direction = txb.pure(&step.path.direction);
    let amount_in = txb.pure(&step.amount_in);
    let clock = txb.object(sui_clock_object_input());

    move_call(
        txb,
        step.published_at,
        MODULE_BLUEFIN,
        FUNCTION_SWAP,
        &[step.coin_a, step.coin_b],
        vec![swap_context, global_config, pool, direction, amount_in, clock],
    )
    .map_err(tx_error)?;
    Ok(())
}
