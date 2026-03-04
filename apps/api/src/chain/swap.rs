use std::sync::Arc;

use primitives::swap::SwapResult;
use rocket::{State, get};
use swapper::swapper::GemSwapper;

use crate::params::{ChainParam, SwapProviderParam};
use crate::responders::{ApiError, ApiResponse};

#[get("/chain/swaps/<provider>/transaction/<hash>?<chain>")]
pub async fn get_swap_result(provider: SwapProviderParam, hash: &str, chain: ChainParam, swapper: &State<Arc<GemSwapper>>) -> Result<ApiResponse<SwapResult>, ApiError> {
    Ok(swapper.get_swap_result(chain.0, provider.0, hash).await?.into())
}

#[get("/chain/swaps/<provider>/vault_addresses")]
pub async fn get_vault_addresses(provider: SwapProviderParam, swapper: &State<Arc<GemSwapper>>) -> Result<ApiResponse<Vec<String>>, ApiError> {
    Ok(swapper.get_vault_addresses(&provider.0, None).await?.into())
}
