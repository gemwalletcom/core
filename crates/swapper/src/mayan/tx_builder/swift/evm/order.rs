use super::contracts::MayanSwiftV2;
use crate::{
    Quote, SwapperError,
    mayan::{
        model::MayanSwiftQuote,
        tx_builder::{
            address::native_address_to_bytes32,
            swift::{SwiftOrderFields, swift_payload_type, swift_random_key},
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
    let fields = SwiftOrderFields::new(route)?;

    Ok(MayanSwiftV2::OrderParams {
        payloadType: swift_payload_type(custom_payload),
        trader: FixedBytes::from(native_address_to_bytes32(&quote.request.wallet_address, source_chain_id)?),
        destAddr: FixedBytes::from(native_address_to_bytes32(destination_address, fields.destination_chain_id)?),
        destChainId: fields.destination_chain_id,
        referrerAddr: FixedBytes::from(fields.referrer),
        tokenOut: FixedBytes::from(fields.token_out),
        minAmountOut: fields.amount_out_min,
        gasDrop: fields.gas_drop,
        cancelFee: fields.cancel_fee,
        refundFee: fields.refund_fee,
        deadline: fields.deadline,
        referrerBps: fields.referrer_bps,
        auctionMode: fields.auction_mode,
        random: FixedBytes::from(swift_random_key(route)?),
    })
}
