use crate::network::{JsonRpcError, JsonRpcResponse, JsonRpcResult, JsonRpcResults};
use crate::swapper::SwapperError;
use alloy_primitives::U256;

#[derive(Debug)]
pub struct QuoteResult {
    pub amount_out: U256,
    pub fee_tier_idx: usize,
    pub batch_idx: usize,
    pub gas_estimate: Option<String>,
}

pub fn get_best_quote<F>(batch_results: &[Result<JsonRpcResults<String>, JsonRpcError>], decoder: F) -> Result<QuoteResult, SwapperError>
where
    F: Fn(&JsonRpcResponse<String>) -> Result<(U256, U256), SwapperError>,
{
    batch_results
        .iter()
        .enumerate()
        .filter_map(|(batch_idx, batch_result)| {
            batch_result.as_ref().ok().map(|results| {
                results
                    .0
                    .iter()
                    .enumerate()
                    .filter_map(|(fee_idx, result)| match result {
                        JsonRpcResult::Value(value) => decoder(value).ok().map(|quoter_tuple| QuoteResult {
                            amount_out: quoter_tuple.0,
                            fee_tier_idx: fee_idx,
                            batch_idx,
                            gas_estimate: Some(quoter_tuple.1.to_string()),
                        }),
                        _ => None,
                    })
                    .max_by_key(|quote| quote.amount_out)
            })
        })
        .flatten()
        .max_by_key(|quote| quote.amount_out)
        .ok_or(SwapperError::NoQuoteAvailable)
}
