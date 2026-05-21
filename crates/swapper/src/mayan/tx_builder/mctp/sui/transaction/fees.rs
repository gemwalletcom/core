use crate::{
    SwapperError,
    mayan::{
        cctp_domain::CCTP_TOKEN_DECIMALS,
        model::MayanMctpQuote,
        tx_builder::amount::{fractional_amount, value_to_query},
        tx_builder::mctp::redeem_relayer_fee,
    },
};

pub(super) fn redeem_fee(route: &MayanMctpQuote) -> Result<u64, SwapperError> {
    redeem_relayer_fee(route)
}

pub(in crate::mayan::tx_builder::mctp::sui) fn bridge_amount(route: &MayanMctpQuote, mctp_input_contract: &str) -> Result<u64, SwapperError> {
    if route.from_token.contract.as_str() == mctp_input_contract {
        return route.effective_amount_in64.parse::<u64>().map_err(SwapperError::from);
    }
    fractional_amount(route.min_middle_amount.as_ref().ok_or(SwapperError::InvalidRoute)?, CCTP_TOKEN_DECIMALS)
}

pub(super) fn bridge_fee(route: &MayanMctpQuote) -> Result<u64, SwapperError> {
    value_to_query(route.bridge_fee.as_ref().ok_or(SwapperError::InvalidRoute)?)?
        .parse::<u64>()
        .map_err(SwapperError::from)
}
