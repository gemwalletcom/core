use super::contracts::MayanSwiftV2;
use crate::{
    Quote, SwapperError,
    mayan::{
        model::{MayanSwiftQuote, QuoteType},
        tx_builder::{
            address::native_address_to_bytes32,
            amount::{bps_u8, gas_drop_amount, min_amount_out},
            route::{is_hypercore_deposit, swift_destination_chain, swift_destination_chain_id},
            swift::{referrer_bytes, swift_payload_type, swift_random_key, swift_to_token},
        },
    },
};
use alloy_primitives::FixedBytes;

pub(super) fn swift_order(
    quote: &Quote,
    route: &MayanSwiftQuote,
    source_chain_id: u16,
    destination_address: &str,
    custom_payload: Option<&[u8]>,
) -> Result<MayanSwiftV2::OrderParams, SwapperError> {
    let destination_chain_id = swift_destination_chain_id(route)?;
    if !is_hypercore_deposit(route) && route.to_token.w_chain_id != destination_chain_id {
        return Err(SwapperError::InvalidRoute);
    }
    let referrer_addr = FixedBytes::from(referrer_bytes(&route.from_chain)?);
    let destination_chain = swift_destination_chain(route);
    let amount_out_min = min_amount_out(&route.min_amount_out, route.to_token.decimals, destination_chain, &QuoteType::Swift)?;
    let gas_drop = gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::Swift, is_hypercore_deposit(route))?;

    Ok(MayanSwiftV2::OrderParams {
        payloadType: swift_payload_type(custom_payload),
        trader: FixedBytes::from(native_address_to_bytes32(&quote.request.wallet_address, source_chain_id)?),
        destAddr: FixedBytes::from(native_address_to_bytes32(destination_address, destination_chain_id)?),
        destChainId: destination_chain_id,
        referrerAddr: referrer_addr,
        tokenOut: FixedBytes::from(swift_to_token(route)?),
        minAmountOut: amount_out_min,
        gasDrop: gas_drop,
        cancelFee: route.cancel_relayer_fee64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?,
        refundFee: route.refund_relayer_fee64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?,
        deadline: route.deadline64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?,
        referrerBps: bps_u8(route.referrer_bps)?,
        auctionMode: route.swift_auction_mode.ok_or(SwapperError::InvalidRoute)?,
        random: FixedBytes::from(swift_random_key(route)?),
    })
}
