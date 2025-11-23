use alloy_primitives::Address;

use super::error::SwapperError;
use primitives::{AssetId, EVMChain};

pub(crate) fn convert_native_to_weth(asset: &AssetId) -> Option<AssetId> {
    if asset.is_native() {
        let evm_chain = EVMChain::from_chain(asset.chain)?;
        let weth = evm_chain.weth_contract()?;
        return AssetId::from_token(asset.chain, weth).into();
    }
    asset.clone().into()
}

pub(crate) fn parse_or_weth_address(asset: &AssetId, evm_chain: EVMChain) -> Result<Address, SwapperError> {
    if let Some(token_id) = &asset.token_id {
        parse_str(token_id)
    } else {
        let weth = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;
        parse_str(weth)
    }
}

pub(crate) fn parse_asset_id(asset: &AssetId) -> Result<Address, SwapperError> {
    if let Some(token_id) = &asset.token_id {
        parse_str(token_id)
    } else {
        Err(SwapperError::InvalidAddress(asset.to_string()))
    }
}

pub(crate) fn parse_str(str: &str) -> Result<Address, SwapperError> {
    str.parse::<Address>().map_err(|_| SwapperError::InvalidAddress(str.to_string()))
}
