use super::super::{
    constants::{FUNCTION_SWAP, MODULE_CETUS_DLMM},
    error::tx_error,
    model::SwapStep,
};
use crate::{
    SwapperError,
    cetus::{
        constants::{CETUS_DLMM_GLOBAL_CONFIG, CETUS_DLMM_PARTNER, CETUS_DLMM_VERSIONED},
        model::FlattenedPath,
    },
};
use gem_sui::{
    sui_clock_object_input,
    tx_builder::{ObjectResolver, move_call},
};
use sui_transaction_builder::{Argument, TransactionBuilder};

// Move sig: `<published_at>::cetus_dlmm::swap<A, B>(swap_context, global_config, pool, partner, direction, amount_in, versioned, clock)`
// Source: https://github.com/CetusProtocol/aggregator/blob/main/src/movecall/cetus_dlmm.ts
pub(super) fn build_swap(txb: &mut TransactionBuilder, resolver: &ObjectResolver, flattened_path: &FlattenedPath, swap_context: Argument) -> Result<(), SwapperError> {
    let step = SwapStep::try_from(flattened_path)?;
    let global_config = resolver.shared_object(txb, CETUS_DLMM_GLOBAL_CONFIG, true).map_err(tx_error)?;
    let pool = resolver.shared_object(txb, &step.path.id, true).map_err(tx_error)?;
    let partner = resolver.shared_object(txb, CETUS_DLMM_PARTNER, true).map_err(tx_error)?;
    let direction = txb.pure(&step.path.direction);
    let amount_in = txb.pure(&step.amount_in);
    let versioned = resolver.shared_object(txb, CETUS_DLMM_VERSIONED, false).map_err(tx_error)?;
    let clock = txb.object(sui_clock_object_input());

    move_call(
        txb,
        step.published_at,
        MODULE_CETUS_DLMM,
        FUNCTION_SWAP,
        &[step.coin_a, step.coin_b],
        vec![swap_context, global_config, pool, partner, direction, amount_in, versioned, clock],
    )
    .map_err(tx_error)?;
    Ok(())
}
