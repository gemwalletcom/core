use crate::SwapperError;
use alloy_primitives::U256;
use gem_jsonrpc::types::{JsonRpcError, JsonRpcResponse, JsonRpcResult, JsonRpcResults};

#[derive(Debug)]
pub struct QuoteResult {
    pub amount_out: U256,
    pub route_idx: usize,
    pub batch_idx: usize,
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
                    .filter_map(|(route_idx, result)| match result {
                        JsonRpcResult::Value(value) => decoder(value).ok().map(|quoter_tuple| QuoteResult {
                            amount_out: quoter_tuple.0,
                            route_idx,
                            batch_idx,
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

pub fn get_selected_candidate<'a, T>(candidates: &'a [Vec<T>], quote: &QuoteResult) -> Result<&'a T, SwapperError> {
    candidates
        .get(quote.batch_idx)
        .and_then(|batch| batch.get(quote.route_idx))
        .ok_or(SwapperError::InvalidRoute)
}
