use crate::{
    SwapperError,
    fees::default_referral_address,
    mayan::{cctp_domain::CCTP_TOKEN_DECIMALS, model::MayanMctpQuote, tx_builder::amount::fractional_amount, wormhole_chain},
};

pub(in crate::mayan) mod evm;
pub(in crate::mayan) mod solana;
pub(in crate::mayan) mod sui;

fn destination_referrer_address(route: &MayanMctpQuote) -> Result<Option<String>, SwapperError> {
    let chain = wormhole_chain::chain_for_name(&route.to_chain)?;
    let address = default_referral_address(chain);
    Ok((!address.is_empty()).then_some(address))
}

fn redeem_relayer_fee(route: &MayanMctpQuote) -> Result<u64, SwapperError> {
    fractional_amount(route.redeem_relayer_fee.as_ref().ok_or(SwapperError::InvalidRoute)?, CCTP_TOKEN_DECIMALS)
}
