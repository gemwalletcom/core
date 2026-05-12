mod bluefin;
mod cetus;
mod deepbook;

use super::{
    constants::{FUNCTION_CONFIRM_SWAP, FUNCTION_NEW_SWAP_CONTEXT, MODULE_ROUTER},
    error::tx_error,
    model::SwapLimits,
};
use crate::{
    Quote, SwapperError,
    cetus::{
        constants::{BLUEFIN, BLUEFIN_GLOBAL_CONFIG, CETUS, CETUS_GLOBAL_CONFIG, CETUS_PARTNER, DEEPBOOK_V3, DEEPBOOK_V3_GLOBAL_CONFIG},
        model::{ProcessedRouterData, RouterData},
    },
    fees::ReferralFee,
};
use gem_sui::tx_builder::{ObjectResolver, move_call};
use std::collections::BTreeSet;
use sui_transaction_builder::{Argument, TransactionBuilder};

pub(super) fn shared_object_ids(router: &RouterData) -> Result<Vec<String>, SwapperError> {
    let mut object_ids = BTreeSet::new();
    for path in &router.paths {
        match path.provider.as_str() {
            CETUS => {
                object_ids.insert(CETUS_GLOBAL_CONFIG.to_string());
                object_ids.insert(path.id.clone());
                object_ids.insert(CETUS_PARTNER.to_string());
            }
            BLUEFIN => {
                object_ids.insert(BLUEFIN_GLOBAL_CONFIG.to_string());
                object_ids.insert(path.id.clone());
            }
            DEEPBOOK_V3 => {
                object_ids.insert(DEEPBOOK_V3_GLOBAL_CONFIG.to_string());
                object_ids.insert(path.id.clone());
                if path
                    .extended_details
                    .as_ref()
                    .and_then(|details| details.deepbookv3_need_add_deep_price_point)
                    .unwrap_or(false)
                {
                    let reference_pool_id = path
                        .extended_details
                        .as_ref()
                        .and_then(|details| details.deepbookv3_reference_pool_id.as_ref())
                        .ok_or(SwapperError::InvalidRoute)?;
                    object_ids.insert(reference_pool_id.clone());
                }
            }
            provider => return Err(SwapperError::TransactionError(format!("Unsupported Cetus route provider: {provider}"))),
        }
    }
    Ok(object_ids.into_iter().collect())
}

pub(super) fn build_swap(
    txb: &mut TransactionBuilder,
    resolver: &ObjectResolver,
    quote: &Quote,
    router: &RouterData,
    referral_fee: &ReferralFee,
    input_coin: Argument,
) -> Result<Argument, SwapperError> {
    let processed = ProcessedRouterData::try_from(router)?;
    let limits = SwapLimits::new(quote, router, referral_fee)?;
    let request_id = txb.pure(&processed.request_id);
    let expected_amount_out = txb.pure(&limits.expected_amount_out);
    let amount_out_limit = txb.pure(&limits.amount_out_limit);
    let fee_rate = txb.pure(&limits.fee_rate);
    let fee_recipient = txb.pure(&limits.fee_recipient);
    let swap_context = move_call(
        txb,
        &router.aggregator_v3(),
        MODULE_ROUTER,
        FUNCTION_NEW_SWAP_CONTEXT,
        &[&processed.from_coin_type, &processed.target_coin_type],
        vec![request_id, expected_amount_out, amount_out_limit, input_coin, fee_rate, fee_recipient],
    )
    .map_err(tx_error)?;

    for flattened_path in &processed.flattened_paths {
        match flattened_path.path.provider.as_str() {
            CETUS => cetus::build_swap(txb, resolver, flattened_path, swap_context)?,
            BLUEFIN => bluefin::build_swap(txb, resolver, flattened_path, swap_context)?,
            DEEPBOOK_V3 => deepbook::build_swap(txb, resolver, flattened_path, swap_context)?,
            provider => return Err(SwapperError::TransactionError(format!("Unsupported Cetus route provider: {provider}"))),
        }
    }

    move_call(
        txb,
        &router.aggregator_v3(),
        MODULE_ROUTER,
        FUNCTION_CONFIRM_SWAP,
        &[&processed.target_coin_type],
        vec![swap_context],
    )
    .map_err(tx_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cetus::testkit::{route_path, router};

    #[test]
    fn test_shared_object_ids() {
        let mut router_data = router(1000);
        router_data.paths = vec![route_path(true, Some("0x1".to_string()))];

        let object_ids = shared_object_ids(&router_data).unwrap();

        assert_eq!(object_ids, vec![CETUS_PARTNER.to_string(), "0x1".to_string(), CETUS_GLOBAL_CONFIG.to_string()]);
    }
}
