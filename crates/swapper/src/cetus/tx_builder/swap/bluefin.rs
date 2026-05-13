use super::{finalize_swap, prepare_swap_inputs};
use super::super::constants::MODULE_BLUEFIN;
use crate::{
    SwapperError,
    cetus::{constants::BLUEFIN_GLOBAL_CONFIG, model::FlattenedPath},
};
use gem_sui::tx_builder::ObjectResolver;
use sui_transaction_builder::{Argument, TransactionBuilder};

// Move sig: `<published_at>::bluefin::swap<A, B>(swap_context, global_config, pool, direction, amount_in, clock)`
// Source: https://github.com/CetusProtocol/aggregator/blob/main/src/movecall/bluefin.ts
pub(super) fn build_swap(txb: &mut TransactionBuilder, resolver: &ObjectResolver, flattened_path: &FlattenedPath, swap_context: Argument) -> Result<(), SwapperError> {
    let s = prepare_swap_inputs(txb, resolver, flattened_path, BLUEFIN_GLOBAL_CONFIG)?;
    finalize_swap(txb, &s.step, MODULE_BLUEFIN, vec![swap_context, s.global_config, s.pool, s.direction, s.amount_in, s.clock])
}
