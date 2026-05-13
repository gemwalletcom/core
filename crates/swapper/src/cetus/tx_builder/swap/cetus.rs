use super::{finalize_swap, prepare_swap_inputs};
use super::super::{
    constants::{MODULE_CETUS, MODULE_CETUS_DLMM},
    error::tx_error,
};
use crate::{
    SwapperError,
    cetus::{
        constants::{CETUS_DLMM_GLOBAL_CONFIG, CETUS_DLMM_PARTNER, CETUS_DLMM_VERSIONED, CETUS_GLOBAL_CONFIG, CETUS_PARTNER},
        model::FlattenedPath,
    },
};
use gem_sui::tx_builder::ObjectResolver;
use sui_transaction_builder::{Argument, TransactionBuilder};

// Move sig: `<published_at>::cetus::swap<A, B>(swap_context, global_config, pool, partner, direction, amount_in, clock)`
// Source: https://github.com/CetusProtocol/aggregator/blob/main/src/movecall/cetus.ts
pub(super) fn build_clmm_swap(txb: &mut TransactionBuilder, resolver: &ObjectResolver, flattened_path: &FlattenedPath, swap_context: Argument) -> Result<(), SwapperError> {
    let s = prepare_swap_inputs(txb, resolver, flattened_path, CETUS_GLOBAL_CONFIG)?;
    let partner = resolver.shared_object(txb, CETUS_PARTNER, true).map_err(tx_error)?;
    finalize_swap(txb, &s.step, MODULE_CETUS, vec![swap_context, s.global_config, s.pool, partner, s.direction, s.amount_in, s.clock])
}

// Move sig: `<published_at>::cetus_dlmm::swap<A, B>(swap_context, global_config, pool, partner, direction, amount_in, versioned, clock)`
// Source: https://github.com/CetusProtocol/aggregator/blob/main/src/movecall/cetus_dlmm.ts
pub(super) fn build_dlmm_swap(txb: &mut TransactionBuilder, resolver: &ObjectResolver, flattened_path: &FlattenedPath, swap_context: Argument) -> Result<(), SwapperError> {
    let s = prepare_swap_inputs(txb, resolver, flattened_path, CETUS_DLMM_GLOBAL_CONFIG)?;
    let partner = resolver.shared_object(txb, CETUS_DLMM_PARTNER, true).map_err(tx_error)?;
    let versioned = resolver.shared_object(txb, CETUS_DLMM_VERSIONED, false).map_err(tx_error)?;
    finalize_swap(
        txb,
        &s.step,
        MODULE_CETUS_DLMM,
        vec![swap_context, s.global_config, s.pool, partner, s.direction, s.amount_in, versioned, s.clock],
    )
}
