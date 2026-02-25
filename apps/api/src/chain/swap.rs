use std::sync::Arc;

use primitives::swap::SwapResult;
use rocket::{State, get};
use swapper::swapper::GemSwapper;

use crate::params::{ChainParam, SwapProviderParam};
use crate::responders::{ApiError, ApiResponse};

#[get("/chain/swaps/<chain>/<provider>/<hash>")]
pub async fn get_swap_result(chain: ChainParam, provider: SwapProviderParam, hash: &str, swapper: &State<Arc<GemSwapper>>) -> Result<ApiResponse<SwapResult>, ApiError> {
    Ok(swapper.get_swap_result(chain.0, provider.0, hash).await?.into())
}
