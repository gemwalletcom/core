use super::{
    message::{build_jetton_transfer_body, referral_bps},
    model::{SwapTransactionParams, TxParams},
};
use crate::SwapperError;
use gem_ton::{
    address::Address,
    signer::cells::{BagOfCells, Cell, CellArc, CellBuilder},
};
use num_bigint::BigUint;
use primitives::unix_timestamp;
use std::str::FromStr;

const V2_SWAP_OPCODE: u32 = 0x6664DE2A;
const V2_JETTON_SWAP_GAS: u64 = 300_000_000;
const V2_JETTON_SWAP_FORWARD_GAS: u64 = 240_000_000;
const V2_TON_TO_JETTON_FORWARD_GAS: u64 = 300_000_000;
const V2_DEFAULT_DEADLINE_SECONDS: u64 = 15 * 60;
const V2_TON_TO_JETTON_DEADLINE_SECONDS: u64 = 60;
const PTON_V2_TON_TRANSFER_OPCODE: u32 = 0x01F3835D;
const PTON_V2_TON_TRANSFER_GAS: u64 = 10_000_000;

pub fn build_swap_transaction(params: SwapTransactionParams<'_>) -> Result<TxParams, SwapperError> {
    let swap_body = build_swap_body(&params)?.into_arc();
    if params.from_native {
        return build_ton_to_jetton(params, &swap_body);
    }
    build_jetton_swap(params, &swap_body)
}

fn build_ton_to_jetton(params: SwapTransactionParams<'_>, swap_body: &CellArc) -> Result<TxParams, SwapperError> {
    let wallet = Address::parse(params.wallet_address)?;
    let from_value = BigUint::from_str(params.from_value)?;
    let body = build_pton_ton_transfer_body(&from_value, &wallet, Some(swap_body))?;

    let mut value = from_value;
    value += BigUint::from(V2_TON_TO_JETTON_FORWARD_GAS);
    value += BigUint::from(PTON_V2_TON_TRANSFER_GAS);

    Ok(TxParams {
        to: params.simulation.offer_jetton_wallet.clone(),
        value: value.to_string(),
        data: BagOfCells::from_root(body).to_base64(true)?,
    })
}

fn build_jetton_swap(params: SwapTransactionParams<'_>, swap_body: &CellArc) -> Result<TxParams, SwapperError> {
    let router = Address::parse(&params.simulation.router.address)?;
    let wallet = Address::parse(params.wallet_address)?;
    let from_value = BigUint::from_str(params.from_value)?;
    let body = build_jetton_transfer_body(&from_value, &router, Some(&wallet), &BigUint::from(V2_JETTON_SWAP_FORWARD_GAS), Some(swap_body))?;

    Ok(TxParams {
        to: params.simulation.offer_jetton_wallet.clone(),
        value: V2_JETTON_SWAP_GAS.to_string(),
        data: BagOfCells::from_root(body).to_base64(true)?,
    })
}

fn build_swap_body(params: &SwapTransactionParams<'_>) -> Result<Cell, SwapperError> {
    let ask_wallet = Address::parse(&params.simulation.ask_jetton_wallet)?;
    let refund_address = Address::parse(params.wallet_address)?;
    let receiver_address = Address::parse(params.receiver_address)?;
    let min_ask_amount = BigUint::from_str(params.min_ask_amount)?;
    let referral_address = Address::parse(params.referral.address)?;
    let referral_bps = referral_bps(params.referral.bps)?;
    let default_deadline_seconds = if params.from_native {
        V2_TON_TO_JETTON_DEADLINE_SECONDS
    } else {
        V2_DEFAULT_DEADLINE_SECONDS
    };
    let deadline = params.deadline.unwrap_or_else(|| unix_timestamp() + default_deadline_seconds);

    let mut details = CellBuilder::new();
    details.store_coins(&min_ask_amount)?.store_address(&receiver_address)?.store_coins(&BigUint::from(0u64))?;
    details.store_maybe_reference(None)?;
    details.store_coins(&BigUint::from(0u64))?;
    details.store_maybe_reference(None)?;
    details.store_u32(16, referral_bps)?;
    details.store_address(&referral_address)?;
    let details = details.build()?.into_arc();

    let mut builder = CellBuilder::new();
    builder
        .store_u32(32, V2_SWAP_OPCODE)?
        .store_address(&ask_wallet)?
        .store_address(&refund_address)?
        .store_address(&refund_address)?
        .store_u64(64, deadline)?
        .store_reference(&details)?;
    Ok(builder.build()?)
}

fn build_pton_ton_transfer_body(amount: &BigUint, refund_address: &Address, forward_payload: Option<&CellArc>) -> Result<Cell, SwapperError> {
    let mut builder = CellBuilder::new();
    builder
        .store_u32(32, PTON_V2_TON_TRANSFER_OPCODE)?
        .store_u64(64, 0)?
        .store_coins(amount)?
        .store_address(refund_address)?;
    builder.store_maybe_reference(forward_payload)?;
    Ok(builder.build()?)
}
