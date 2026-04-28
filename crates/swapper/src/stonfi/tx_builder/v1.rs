use super::{
    message::build_jetton_transfer_body,
    model::{SwapTransactionParams, TxParams},
};
use crate::SwapperError;
use gem_ton::{
    address::Address,
    signer::cells::{BagOfCells, Cell, CellArc, CellBuilder},
};
use num_bigint::BigUint;
use std::str::FromStr;

const V1_SWAP_OPCODE: u32 = 0x25938561;
const V1_JETTON_TO_JETTON_GAS: u64 = 220_000_000;
const V1_JETTON_TO_JETTON_FORWARD_GAS: u64 = 175_000_000;
const V1_JETTON_TO_TON_GAS: u64 = 170_000_000;
const V1_JETTON_TO_TON_FORWARD_GAS: u64 = 125_000_000;
const V1_TON_TO_JETTON_FORWARD_GAS: u64 = 185_000_000;

pub fn build_swap_transaction(params: SwapTransactionParams<'_>) -> Result<TxParams, SwapperError> {
    let swap_body = build_swap_body(&params)?.into_arc();
    if params.from_native {
        return build_ton_to_jetton(params, &swap_body);
    }
    build_jetton_swap(params, &swap_body)
}

fn build_ton_to_jetton(params: SwapTransactionParams<'_>, swap_body: &CellArc) -> Result<TxParams, SwapperError> {
    let router = Address::parse(&params.simulation.router.address)?;
    let from_value = BigUint::from_str(params.from_value)?;
    let forward_gas = BigUint::from(V1_TON_TO_JETTON_FORWARD_GAS);
    let body = build_jetton_transfer_body(&from_value, &router, None, &forward_gas, Some(swap_body))?;

    let mut value = from_value;
    value += forward_gas;

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
    let (gas, forward_gas) = if params.to_native {
        (V1_JETTON_TO_TON_GAS, V1_JETTON_TO_TON_FORWARD_GAS)
    } else {
        (V1_JETTON_TO_JETTON_GAS, V1_JETTON_TO_JETTON_FORWARD_GAS)
    };
    let body = build_jetton_transfer_body(&from_value, &router, Some(&wallet), &BigUint::from(forward_gas), Some(swap_body))?;

    Ok(TxParams {
        to: params.simulation.offer_jetton_wallet.clone(),
        value: gas.to_string(),
        data: BagOfCells::from_root(body).to_base64(true)?,
    })
}

fn build_swap_body(params: &SwapTransactionParams<'_>) -> Result<Cell, SwapperError> {
    let ask_wallet = Address::parse(&params.simulation.ask_jetton_wallet)?;
    let wallet = Address::parse(params.wallet_address)?;
    let min_ask_amount = BigUint::from_str(params.min_ask_amount)?;
    let referral_address = Address::parse(params.referral.address)?;

    let mut builder = CellBuilder::new();
    builder
        .store_u32(32, V1_SWAP_OPCODE)?
        .store_address(&ask_wallet)?
        .store_coins(&min_ask_amount)?
        .store_address(&wallet)?
        .store_bit(true)?
        .store_address(&referral_address)?;

    Ok(builder.build()?)
}
