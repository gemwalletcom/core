use super::models::SwapperError;
use gem_evm::address::EthereumAddress;
use primitives::{AssetId, EVMChain};

pub(crate) fn get_address_by_asset(asset: &AssetId, evm_chain: EVMChain) -> Result<String, SwapperError> {
    let str = match &asset.token_id {
        Some(token_id) => token_id.to_string(),
        None => evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?.to_string(),
    };
    Ok(str)
}

pub(crate) fn normalize_asset(asset: &AssetId) -> Option<AssetId> {
    if !asset.is_native() {
        return Some(asset.clone());
    }
    let evm_chain = EVMChain::from_chain(asset.chain)?;
    let weth = evm_chain.weth_contract()?;
    Some(AssetId::from(asset.chain, Some(weth.to_string())))
}

pub(crate) fn parse_into_address(asset: &AssetId, evm_chain: EVMChain) -> Result<EthereumAddress, SwapperError> {
    let str = get_address_by_asset(asset, evm_chain)?;
    EthereumAddress::parse(&str).ok_or(SwapperError::InvalidAddress { address: str })
}
