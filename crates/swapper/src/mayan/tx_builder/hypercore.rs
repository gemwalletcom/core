use super::{address::evm_address_bytes, route::is_hypercore_deposit};
use crate::{
    SwapperError,
    mayan::{constants::HYPERCORE_SPOT_USDC_CONTRACT, model::MayanSwiftQuote},
};
use gem_evm::EVM_ZERO_ADDRESS;

pub(super) fn hypercore_custom_payload(route: &MayanSwiftQuote, destination_address: &str) -> Result<Option<Vec<u8>>, SwapperError> {
    if !is_hypercore_deposit(route) {
        return Ok(None);
    }

    let destination = evm_address_bytes(destination_address)?;
    let dex = hypercore_deposit_dex(&route.to_token.contract)?;
    let relayer_fee = route.hc_swift_deposit.as_ref().ok_or(SwapperError::InvalidRoute)?.relayer_fee64.parse::<u64>()?;

    let mut payload = vec![0u8; 32];
    payload[..20].copy_from_slice(&destination);
    payload[20..24].copy_from_slice(&dex.to_be_bytes());
    payload[24..32].copy_from_slice(&relayer_fee.to_be_bytes());
    Ok(Some(payload))
}

pub(super) fn hypercore_deposit_dex(contract: &str) -> Result<u32, SwapperError> {
    if contract != EVM_ZERO_ADDRESS && !contract.eq_ignore_ascii_case(HYPERCORE_SPOT_USDC_CONTRACT) {
        return Err(SwapperError::NotSupportedAsset);
    }
    let bytes = evm_address_bytes(contract)?;
    if bytes[..16].iter().any(|value| *value != 0) {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(u32::from_be_bytes(bytes[16..20].try_into().map_err(|_| SwapperError::InvalidRoute)?))
}
