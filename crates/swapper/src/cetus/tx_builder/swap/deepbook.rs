use super::super::{
    constants::{FUNCTION_ADD_DEEP_PRICE_POINT, FUNCTION_SWAP, MODULE_DEEPBOOK_V3},
    error::tx_error,
    model::SwapStep,
};
use crate::{
    SwapperError,
    cetus::{
        constants::{DEEPBOOK_V3_DEEP_FEE_TYPE, DEEPBOOK_V3_GLOBAL_CONFIG},
        model::{ExtendedDetails, FlattenedPath, Path},
    },
};
use gem_sui::{
    sui_clock_object_input,
    tx_builder::{ObjectResolver, move_call, zero_coin},
};
use sui_transaction_builder::{Argument, TransactionBuilder};

pub(super) fn build_swap(txb: &mut TransactionBuilder, resolver: &ObjectResolver, flattened_path: &FlattenedPath, swap_context: Argument) -> Result<(), SwapperError> {
    let step = SwapStep::try_from(flattened_path)?;

    if step
        .path
        .extended_details
        .as_ref()
        .and_then(|details| details.deepbookv3_need_add_deep_price_point)
        .unwrap_or(false)
    {
        add_deep_price_point(txb, resolver, step.path, step.path.extended_details.as_ref().ok_or(SwapperError::InvalidRoute)?)?;
    }

    let deep_coin = zero_coin(txb, DEEPBOOK_V3_DEEP_FEE_TYPE).map_err(tx_error)?;
    let global_config = resolver.shared_object(txb, DEEPBOOK_V3_GLOBAL_CONFIG, true).map_err(tx_error)?;
    let pool = resolver.shared_object(txb, &step.path.id, true).map_err(tx_error)?;
    let amount_in = txb.pure(&step.amount_in);
    let direction = txb.pure(&step.path.direction);
    let clock = txb.object(sui_clock_object_input());

    move_call(
        txb,
        step.published_at,
        MODULE_DEEPBOOK_V3,
        FUNCTION_SWAP,
        &[step.coin_a, step.coin_b],
        vec![swap_context, global_config, pool, amount_in, direction, deep_coin, clock],
    )
    .map_err(tx_error)?;
    Ok(())
}

fn add_deep_price_point(txb: &mut TransactionBuilder, resolver: &ObjectResolver, path: &Path, details: &ExtendedDetails) -> Result<(), SwapperError> {
    let published_at = path.published_at.as_deref().ok_or(SwapperError::InvalidRoute)?;
    let reference_pool_id = details.deepbookv3_reference_pool_id.as_deref().ok_or(SwapperError::InvalidRoute)?;
    let reference_pool_base_type = details.deepbookv3_reference_pool_base_type.as_deref().ok_or(SwapperError::InvalidRoute)?;
    let reference_pool_quote_type = details.deepbookv3_reference_pool_quote_type.as_deref().ok_or(SwapperError::InvalidRoute)?;
    let (coin_a, coin_b) = if path.direction {
        (path.from.as_str(), path.target.as_str())
    } else {
        (path.target.as_str(), path.from.as_str())
    };
    let pool = resolver.shared_object(txb, &path.id, true).map_err(tx_error)?;
    let reference_pool = resolver.shared_object(txb, reference_pool_id, true).map_err(tx_error)?;
    let clock = txb.object(sui_clock_object_input());

    move_call(
        txb,
        published_at,
        MODULE_DEEPBOOK_V3,
        FUNCTION_ADD_DEEP_PRICE_POINT,
        &[coin_a, coin_b, reference_pool_base_type, reference_pool_quote_type],
        vec![pool, reference_pool, clock],
    )
    .map_err(tx_error)?;
    Ok(())
}
