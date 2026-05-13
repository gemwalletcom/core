mod bluefin;
mod cetus;
mod deepbook;

use super::{
    constants::{FUNCTION_CONFIRM_SWAP, FUNCTION_NEW_SWAP_CONTEXT, FUNCTION_SWAP, MODULE_ROUTER},
    error::tx_error,
    model::{SwapLimits, SwapStep},
};
use crate::{
    Quote, SwapperError,
    cetus::{
        constants::{
            BLUEFIN, BLUEFIN_GLOBAL_CONFIG, CETUS, CETUS_DLMM, CETUS_DLMM_GLOBAL_CONFIG, CETUS_DLMM_PARTNER, CETUS_DLMM_VERSIONED, CETUS_GLOBAL_CONFIG, CETUS_PARTNER,
            DEEPBOOK_V3, DEEPBOOK_V3_GLOBAL_CONFIG,
        },
        model::{FlattenedPath, ProcessedRouterData, RouterData},
    },
    fees::ReferralFee,
};
use gem_sui::{
    sui_clock_object_input,
    tx_builder::{ObjectResolver, move_call},
};
use std::collections::BTreeSet;
use sui_transaction_builder::{Argument, TransactionBuilder};

pub(super) struct SwapInputs<'a> {
    pub step: SwapStep<'a>,
    pub global_config: Argument,
    pub pool: Argument,
    pub direction: Argument,
    pub amount_in: Argument,
    pub clock: Argument,
}

pub(super) fn prepare_swap_inputs<'a>(
    txb: &mut TransactionBuilder,
    resolver: &ObjectResolver,
    flattened_path: &'a FlattenedPath,
    global_config_id: &str,
) -> Result<SwapInputs<'a>, SwapperError> {
    let step = SwapStep::try_from(flattened_path)?;
    let global_config = resolver.shared_object(txb, global_config_id, true).map_err(tx_error)?;
    let pool = resolver.shared_object(txb, &step.path.id, true).map_err(tx_error)?;
    let direction = txb.pure(&step.path.direction);
    let amount_in = txb.pure(&step.amount_in);
    let clock = txb.object(sui_clock_object_input());
    Ok(SwapInputs {
        step,
        global_config,
        pool,
        direction,
        amount_in,
        clock,
    })
}

pub(super) fn finalize_swap(txb: &mut TransactionBuilder, step: &SwapStep<'_>, module: &str, args: Vec<Argument>) -> Result<(), SwapperError> {
    move_call(txb, step.published_at, module, FUNCTION_SWAP, &[step.coin_a, step.coin_b], args).map_err(tx_error)?;
    Ok(())
}

pub(super) fn shared_object_ids(router: &RouterData) -> Result<Vec<String>, SwapperError> {
    let mut object_ids = BTreeSet::new();
    for path in &router.paths {
        match path.provider.as_str() {
            CETUS => {
                object_ids.insert(CETUS_GLOBAL_CONFIG.to_string());
                object_ids.insert(path.id.clone());
                object_ids.insert(CETUS_PARTNER.to_string());
            }
            CETUS_DLMM => {
                object_ids.insert(CETUS_DLMM_GLOBAL_CONFIG.to_string());
                object_ids.insert(path.id.clone());
                object_ids.insert(CETUS_DLMM_PARTNER.to_string());
                object_ids.insert(CETUS_DLMM_VERSIONED.to_string());
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
            CETUS => cetus::build_clmm_swap(txb, resolver, flattened_path, swap_context)?,
            CETUS_DLMM => cetus::build_dlmm_swap(txb, resolver, flattened_path, swap_context)?,
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
    use crate::cetus::{
        constants::DEEPBOOK_V3,
        model::{ExtendedDetails, RouterResponse},
        testkit::{route_path, router},
    };

    fn fixture_router() -> RouterData {
        match serde_json::from_str::<RouterResponse>(include_str!("../../testdata/router_response.json")).unwrap() {
            RouterResponse::Ok { data } => data,
            RouterResponse::Err { .. } => panic!("Expected router response"),
        }
    }

    #[test]
    fn test_shared_object_ids() {
        let mut router_data = router(1000);
        router_data.paths = vec![route_path(true, Some("0x1".to_string()))];

        let object_ids: BTreeSet<String> = shared_object_ids(&router_data).unwrap().into_iter().collect();
        let expected: BTreeSet<String> = [CETUS_PARTNER, "0x1", CETUS_GLOBAL_CONFIG].iter().map(|s| s.to_string()).collect();

        assert_eq!(object_ids, expected);

        let fixture_ids: BTreeSet<String> = shared_object_ids(&fixture_router()).unwrap().into_iter().collect();
        let expected_fixture: BTreeSet<String> = [
            BLUEFIN_GLOBAL_CONFIG,
            CETUS_GLOBAL_CONFIG,
            CETUS_PARTNER,
            "0xpool1",
            "0xpool2",
            "0xpool3",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(fixture_ids, expected_fixture);

        let mut deepbook_router = router(1000);
        let mut path = route_path(true, Some("0xdb".to_string()));
        path.provider = DEEPBOOK_V3.to_string();
        path.id = "0xdb_pool".to_string();
        path.extended_details = Some(ExtendedDetails {
            deepbookv3_need_add_deep_price_point: Some(true),
            deepbookv3_reference_pool_id: Some("0xref_pool".to_string()),
            deepbookv3_reference_pool_base_type: None,
            deepbookv3_reference_pool_quote_type: None,
        });
        deepbook_router.paths = vec![path];
        let deepbook_ids: BTreeSet<String> = shared_object_ids(&deepbook_router).unwrap().into_iter().collect();
        let expected_deepbook: BTreeSet<String> = [DEEPBOOK_V3_GLOBAL_CONFIG, "0xdb_pool", "0xref_pool"].iter().map(|s| s.to_string()).collect();
        assert_eq!(deepbook_ids, expected_deepbook);

        let mut dlmm_router = router(1000);
        let mut dlmm_path = route_path(true, Some("0xdlmm".to_string()));
        dlmm_path.provider = CETUS_DLMM.to_string();
        dlmm_path.id = "0xdlmm_pool".to_string();
        dlmm_router.paths = vec![dlmm_path];
        let dlmm_ids: BTreeSet<String> = shared_object_ids(&dlmm_router).unwrap().into_iter().collect();
        let expected_dlmm: BTreeSet<String> = [CETUS_DLMM_GLOBAL_CONFIG, CETUS_DLMM_PARTNER, CETUS_DLMM_VERSIONED, "0xdlmm_pool"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(dlmm_ids, expected_dlmm);

        let mut bad_router = router(1000);
        let mut bad_path = route_path(true, Some("0xdb".to_string()));
        bad_path.provider = DEEPBOOK_V3.to_string();
        bad_path.extended_details = Some(ExtendedDetails {
            deepbookv3_need_add_deep_price_point: Some(true),
            deepbookv3_reference_pool_id: None,
            deepbookv3_reference_pool_base_type: None,
            deepbookv3_reference_pool_quote_type: None,
        });
        bad_router.paths = vec![bad_path];
        assert!(matches!(shared_object_ids(&bad_router), Err(SwapperError::InvalidRoute)));
    }
}
