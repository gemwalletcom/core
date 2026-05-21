use std::sync::Arc;

use primitives::{AssetId, swap::SwapResult};
use rocket::{State, get};
use swapper::{Options, QuoteRequest, SwapQuotes, SwapperQuoteAsset, config::get_default_slippage, cross_chain::VaultAddresses, swapper::GemSwapper};

use crate::params::{AddressParam, AssetIdParam, ChainParam, SwapProviderParam};
use crate::responders::{ApiError, ApiResponse};

#[get("/chain/swaps/<provider>/transaction/<hash>?<chain>")]
pub async fn get_swap_result(provider: SwapProviderParam, hash: &str, chain: ChainParam, swapper: &State<Arc<GemSwapper>>) -> Result<ApiResponse<SwapResult>, ApiError> {
    Ok(swapper.get_swap_result(chain.0, provider.0, hash).await?.into())
}

#[get("/chain/swaps/<provider>/vault_addresses")]
pub async fn get_vault_addresses(provider: SwapProviderParam, swapper: &State<Arc<GemSwapper>>) -> Result<ApiResponse<VaultAddresses>, ApiError> {
    Ok(swapper.get_vault_addresses(&provider.0, None).await?.into())
}

#[get("/chain/swaps/quote?<from_asset>&<to_asset>&<value>&<wallet_address>&<destination_address>")]
pub async fn get_swap_quote(
    from_asset: AssetIdParam,
    to_asset: AssetIdParam,
    value: &str,
    wallet_address: AddressParam,
    destination_address: AddressParam,
    swapper: &State<Arc<GemSwapper>>,
) -> Result<ApiResponse<SwapQuotes>, ApiError> {
    let request = build_quote_request(from_asset.0, to_asset.0, value, wallet_address.0, destination_address.0);
    Ok(swapper.get_quotes(&request).await?.into())
}

fn build_quote_request(from_asset_id: AssetId, to_asset_id: AssetId, value: &str, wallet_address: String, destination_address: String) -> QuoteRequest {
    let from_asset = SwapperQuoteAsset::from(from_asset_id.clone());
    let to_asset = SwapperQuoteAsset::from(to_asset_id);

    QuoteRequest {
        from_asset,
        to_asset,
        wallet_address,
        destination_address,
        value: value.to_string(),
        options: Options {
            slippage: get_default_slippage(&from_asset_id.chain),
            use_max_amount: false,
        },
    }
}
