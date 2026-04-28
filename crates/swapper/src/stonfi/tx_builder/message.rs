use crate::SwapperError;
use gem_ton::{
    address::Address,
    constants::JETTON_TRANSFER_OPCODE,
    signer::cells::{Cell, CellArc, CellBuilder},
};
use num_bigint::BigUint;

const MAX_REFERRAL_BPS: u32 = 100;

pub fn build_jetton_transfer_body(
    amount: &BigUint,
    destination: &Address,
    response_destination: Option<&Address>,
    forward_ton_amount: &BigUint,
    forward_payload: Option<&CellArc>,
) -> Result<Cell, SwapperError> {
    let mut builder = CellBuilder::new();
    builder
        .store_u32(32, JETTON_TRANSFER_OPCODE)?
        .store_u64(64, 0)?
        .store_coins(amount)?
        .store_address(destination)?;
    builder.store_maybe_address(response_destination)?;
    builder.store_maybe_reference(None)?;
    builder.store_coins(forward_ton_amount)?;
    builder.store_maybe_reference(forward_payload)?;
    Ok(builder.build()?)
}

pub fn referral_bps(bps: u32) -> Result<u32, SwapperError> {
    if bps > MAX_REFERRAL_BPS {
        return Err(SwapperError::ComputeQuoteError(format!("STON.fi referral bps must be in range [0, {MAX_REFERRAL_BPS}]")));
    }
    Ok(bps)
}
