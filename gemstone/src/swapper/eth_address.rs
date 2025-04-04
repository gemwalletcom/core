use alloy_primitives::Address;

use super::error::SwapperError;
use primitives::{AssetId, EVMChain};

pub(crate) fn normalize_weth_asset(asset: &AssetId) -> Option<AssetId> {
    if !asset.is_native() {
        return Some(asset.clone());
    }
    let evm_chain = EVMChain::from_chain(asset.chain)?;
    let weth = evm_chain.weth_contract()?;
    Some(AssetId::from(asset.chain, Some(weth.to_string())))
}

pub(crate) fn normalize_weth_address(asset: &AssetId, evm_chain: EVMChain) -> Result<Address, SwapperError> {
    if asset.is_native() {
        let weth = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;
        parse_str(weth)
    } else {
        parse_asset_id(asset)
    }
}

pub(crate) fn parse_asset_id(asset: &AssetId) -> Result<Address, SwapperError> {
    match &asset.token_id {
        Some(token_id) => token_id.parse::<Address>().map_err(|_| SwapperError::InvalidAddress(token_id.to_string())),
        None => Err(SwapperError::InvalidAddress(asset.to_string())),
    }
}

pub(crate) fn parse_str(str: &str) -> Result<Address, SwapperError> {
    str.parse::<Address>().map_err(|_| SwapperError::InvalidAddress(str.to_string()))
}
