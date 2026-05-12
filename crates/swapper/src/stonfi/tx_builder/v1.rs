use super::{
    message::build_jetton_transfer_body,
    model::{SwapTransactionParams, TxParams},
};
use crate::SwapperError;
use gem_ton::{
    address::Address,
    tvm::{BagOfCells, Cell, CellArc, CellBuilder},
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
    let from_value = BigUint::from_str(params.from_value)?;
    let (gas, forward_gas) = if params.to_native {
        (V1_JETTON_TO_TON_GAS, V1_JETTON_TO_TON_FORWARD_GAS)
    } else {
        (V1_JETTON_TO_JETTON_GAS, V1_JETTON_TO_JETTON_FORWARD_GAS)
    };
    let body = build_jetton_transfer_body(&from_value, &router, Some(&params.wallet_address), &BigUint::from(forward_gas), Some(swap_body))?;
    let sender_jetton_wallet = params
        .sender_jetton_wallet
        .ok_or_else(|| SwapperError::ComputeQuoteError("missing sender jetton wallet".into()))?;

    Ok(TxParams {
        to: sender_jetton_wallet.to_string(),
        value: gas.to_string(),
        data: BagOfCells::from_root(body).to_base64(true)?,
    })
}

fn build_swap_body(params: &SwapTransactionParams<'_>) -> Result<Cell, SwapperError> {
    let ask_wallet = Address::parse(&params.simulation.ask_jetton_wallet)?;
    let min_ask_amount = BigUint::from_str(&params.simulation.min_ask_units)?;

    let mut builder = CellBuilder::new();
    builder
        .store_u32(32, V1_SWAP_OPCODE)?
        .store_address(&ask_wallet)?
        .store_coins(&min_ask_amount)?
        .store_address(&params.wallet_address)?
        .store_bit(true)?
        .store_address(&params.referral.address)?;

    Ok(builder.build()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stonfi::{model::SwapSimulation, tx_builder::SwapTransactionParams};

    #[test]
    fn test_build_v1_swap_transaction() {
        let simulation: SwapSimulation = serde_json::from_str(include_str!("../testdata/v1_simulation.json")).unwrap();

        let transaction = build_swap_transaction(SwapTransactionParams::mock(&simulation)).unwrap();

        assert_eq!(transaction.to, simulation.offer_jetton_wallet);
        assert_eq!(transaction.value, "1185000000");
        assert_eq!(
            transaction.data,
            "te6cckEBAgEAqAABbQ+KfqUAAAAAAAAAAEO5rKAIAO87mQKicbKgHIk4pSPP4k5xhHqutqYgAB7USnesDnCcECwbgQMBANclk4VhgAndkkNzqarUGyjOwC2pOE1nNjryA0/Cp8zAZ+KNQRDehid7ywAM6FKWpQGl51ZuTKImFkWYLixc2NCsRYy79zJEauQV8/AAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz5OFmmt"
        );
        assert!(BagOfCells::parse_base64(&transaction.data).is_ok());
    }
}
