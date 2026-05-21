pub(in crate::mayan) mod evm;
pub(in crate::mayan) mod solana;

use crate::{
    SwapperError,
    fees::default_referral_address,
    mayan::{
        cctp_domain::CCTP_TOKEN_DECIMALS,
        model::MayanFastMctpQuote,
        tx_builder::{address::native_address_to_bytes32, amount::fractional_amount, swift::token_address_to_bytes32},
        wormhole_chain::{self, WormholeChain, id_for_name as wormhole_chain_id},
    },
};
use gem_evm::EVM_ZERO_ADDRESS;
use gem_solana::SYSTEM_PROGRAM_ID;
use primitives::Chain;

const FAST_MCTP_PAYLOAD_TYPE_DEFAULT: u8 = 1;
const FAST_MCTP_PAYLOAD_TYPE_ORDER: u8 = 3;

fn destination_referrer_address(route: &MayanFastMctpQuote) -> Result<Option<String>, SwapperError> {
    let chain = wormhole_chain::chain_for_name(&route.to_chain)?;
    if chain == Chain::Sui {
        return Ok(None);
    }
    let address = default_referral_address(chain);
    Ok((!address.is_empty()).then_some(address))
}

fn referrer_bytes(route: &MayanFastMctpQuote) -> Result<[u8; 32], SwapperError> {
    let Some(address) = destination_referrer_address(route)? else {
        return native_address_to_bytes32(SYSTEM_PROGRAM_ID, wormhole_chain_id(WormholeChain::Solana.name())?);
    };
    native_address_to_bytes32(&address, wormhole_chain_id(&route.to_chain)?)
}

fn redeem_relayer_fee(route: &MayanFastMctpQuote) -> Result<u64, SwapperError> {
    fractional_amount(route.redeem_relayer_fee.as_ref().ok_or(SwapperError::InvalidRoute)?, CCTP_TOKEN_DECIMALS)
}

fn fast_mctp_input_contract(route: &MayanFastMctpQuote) -> Result<&str, SwapperError> {
    route.fast_mctp_input_contract.as_deref().ok_or(SwapperError::InvalidRoute)
}

fn fast_mctp_contract(route: &MayanFastMctpQuote) -> Result<&str, SwapperError> {
    route.fast_mctp_mayan_contract.as_deref().ok_or(SwapperError::InvalidRoute)
}

fn fast_mctp_min_finality(route: &MayanFastMctpQuote) -> Result<u32, SwapperError> {
    route.fast_mctp_min_finality.ok_or(SwapperError::InvalidRoute)
}

fn circle_max_fee64(route: &MayanFastMctpQuote) -> Result<&str, SwapperError> {
    route.circle_max_fee64.as_deref().ok_or(SwapperError::InvalidRoute)
}

fn refund_relayer_fee64(route: &MayanFastMctpQuote) -> Result<u64, SwapperError> {
    route
        .refund_relayer_fee64
        .as_deref()
        .ok_or(SwapperError::InvalidRoute)?
        .parse::<u64>()
        .map_err(SwapperError::from)
}

fn token_out(route: &MayanFastMctpQuote) -> Result<[u8; 32], SwapperError> {
    if route.to_token.contract == EVM_ZERO_ADDRESS {
        return native_address_to_bytes32(SYSTEM_PROGRAM_ID, wormhole_chain_id(WormholeChain::Solana.name())?);
    }
    if route.to_chain == WormholeChain::Sui.name() {
        return token_address_to_bytes32(route.to_token.verified_address.as_deref().ok_or(SwapperError::InvalidRoute)?, WormholeChain::Sui.name());
    }
    native_address_to_bytes32(&route.to_token.contract, route.to_token.w_chain_id)
}
