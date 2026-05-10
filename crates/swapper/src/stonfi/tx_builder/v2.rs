use super::{
    message::build_jetton_transfer_body,
    model::{NextSwapParams, SwapCellParams, SwapTransactionParams, TxParams},
};
use crate::SwapperError;
use gem_ton::{
    address::Address,
    tvm::{BagOfCells, Cell, CellArc, CellBuilder},
};
use num_bigint::BigUint;
use primitives::unix_timestamp;
use std::str::FromStr;

const V2_SWAP_OPCODE: u32 = 0x6664DE2A;
const V2_CROSS_SWAP_OPCODE: u32 = 0x69CF1A5B;
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
    let from_value = BigUint::from_str(params.from_value)?;
    let body = build_pton_ton_transfer_body(&from_value, &params.wallet_address, Some(swap_body))?;

    let mut value = from_value;
    value += BigUint::from(V2_TON_TO_JETTON_FORWARD_GAS);
    value += BigUint::from(next_swap_forward_gas(&params));
    value += BigUint::from(PTON_V2_TON_TRANSFER_GAS);

    Ok(TxParams {
        to: params.simulation.offer_jetton_wallet.clone(),
        value: value.to_string(),
        data: BagOfCells::from_root(body).to_base64(true)?,
    })
}

fn build_jetton_swap(params: SwapTransactionParams<'_>, swap_body: &CellArc) -> Result<TxParams, SwapperError> {
    let router = Address::parse(&params.simulation.router.address)?;
    let from_value = BigUint::from_str(params.from_value)?;
    let extra_forward_gas = next_swap_forward_gas(&params);
    let body = build_jetton_transfer_body(
        &from_value,
        &router,
        Some(&params.wallet_address),
        &BigUint::from(V2_JETTON_SWAP_FORWARD_GAS + extra_forward_gas),
        Some(swap_body),
    )?;
    let sender_jetton_wallet = params
        .sender_jetton_wallet
        .ok_or_else(|| SwapperError::ComputeQuoteError("missing sender jetton wallet".into()))?;

    Ok(TxParams {
        to: sender_jetton_wallet.to_string(),
        value: (V2_JETTON_SWAP_GAS + extra_forward_gas).to_string(),
        data: BagOfCells::from_root(body).to_base64(true)?,
    })
}

fn build_swap_body(params: &SwapTransactionParams<'_>) -> Result<Cell, SwapperError> {
    let ask_wallet = Address::parse(&params.simulation.ask_jetton_wallet)?;
    let receiver_address = match params.next_swap.as_ref() {
        Some(next_swap) if params.simulation.router == next_swap.simulation.router => Address::parse(&params.simulation.router.address)?,
        Some(next_swap) => Address::parse(&next_swap.simulation.router.address)?,
        None => params.receiver_address,
    };
    let min_ask_amount = BigUint::from_str(&params.simulation.min_ask_units)?;
    let referral_bps = if params.next_swap.is_some() { 0 } else { params.referral.bps };
    let default_deadline_seconds = if params.from_native {
        V2_TON_TO_JETTON_DEADLINE_SECONDS
    } else {
        V2_DEFAULT_DEADLINE_SECONDS
    };
    let deadline = params.deadline.unwrap_or_else(|| unix_timestamp() + default_deadline_seconds);

    let next_payload = params
        .next_swap
        .as_ref()
        .map(|next_swap| build_next_swap_body(params, next_swap))
        .transpose()?
        .map(Cell::into_arc);
    let custom_payload_forward_gas = next_swap_forward_gas(params);

    build_swap_cell(SwapCellParams {
        opcode: V2_SWAP_OPCODE,
        ask_wallet,
        refund_address: params.wallet_address,
        receiver_address,
        min_ask_amount,
        forward_gas: custom_payload_forward_gas,
        next_payload: next_payload.as_ref(),
        referral_bps,
        referral_address: params.referral.address,
        deadline,
    })
}

fn build_next_swap_body(params: &SwapTransactionParams<'_>, next_swap: &NextSwapParams<'_>) -> Result<Cell, SwapperError> {
    let opcode = if params.simulation.router == next_swap.simulation.router {
        V2_CROSS_SWAP_OPCODE
    } else {
        V2_SWAP_OPCODE
    };
    let ask_wallet = Address::parse(&next_swap.simulation.ask_jetton_wallet)?;
    let deadline = params.deadline.unwrap_or_else(|| unix_timestamp() + V2_DEFAULT_DEADLINE_SECONDS);

    build_swap_cell(SwapCellParams {
        opcode,
        ask_wallet,
        refund_address: params.wallet_address,
        receiver_address: params.receiver_address,
        min_ask_amount: next_swap.min_ask_amount.clone(),
        forward_gas: 0,
        next_payload: None,
        referral_bps: params.referral.bps,
        referral_address: params.referral.address,
        deadline,
    })
}

fn build_swap_cell(params: SwapCellParams<'_>) -> Result<Cell, SwapperError> {
    let mut details = CellBuilder::new();
    details
        .store_coins(&params.min_ask_amount)?
        .store_address(&params.receiver_address)?
        .store_coins(&BigUint::from(params.forward_gas))?;
    details.store_maybe_reference(params.next_payload)?;
    details.store_coins(&BigUint::from(0u64))?;
    details.store_maybe_reference(None)?;
    details.store_u32(16, params.referral_bps)?;
    details.store_address(&params.referral_address)?;
    let details = details.build()?.into_arc();

    let mut builder = CellBuilder::new();
    builder
        .store_u32(32, params.opcode)?
        .store_address(&params.ask_wallet)?
        .store_address(&params.refund_address)?
        .store_address(&params.refund_address)?
        .store_u64(64, params.deadline)?
        .store_reference(&details)?;
    Ok(builder.build()?)
}

fn next_swap_forward_gas(params: &SwapTransactionParams<'_>) -> u64 {
    // Same-router cross-swaps route internally; inter-router hops need extra forward gas.
    match params.next_swap.as_ref() {
        Some(next_swap) if params.simulation.router != next_swap.simulation.router => V2_JETTON_SWAP_FORWARD_GAS,
        _ => 0,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stonfi::{
        model::SwapSimulation,
        tx_builder::{NextSwapParams, SwapTransactionParams},
    };

    const TEST_SENDER_JETTON_WALLET: &str = "EQAlgB03OjJKdXrlwZiGJD5snSzPKF2VL5bErJn_cqJANGH9";

    #[test]
    fn test_build_v2_ton_to_jetton_swap_transaction() {
        let simulation: SwapSimulation = serde_json::from_str(include_str!("../testdata/v2_simulation.json")).unwrap();

        let transaction = build_swap_transaction(SwapTransactionParams::mock(&simulation)).unwrap();

        assert_eq!(transaction.to, simulation.offer_jetton_wallet);
        assert_eq!(transaction.value, "1310000000");
        assert_eq!(
            transaction.data,
            "te6cckEBAwEA9QABZAHzg10AAAAAAAAAAEO5rKAIAGdClLUoDS86s3JlETCyLMFxYubGhWIsZd+5kiNXIK+fAQHhZmTeKoASRaxPr7HbegHJxHe2GKlO3cvD6MrnQ16ILwr/R8R9I/AAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz4AGdClLUoDS86s3JlETCyLMFxYubGhWIsZd+5kiNXIK+eAAAAAMqn4gEACAJMxNleIAGdClLUoDS86s3JlETCyLMFxYubGhWIsZd+5kiNXIK+eAAAZQAM6FKWpQGl51ZuTKImFkWYLixc2NCsRYy79zJEauQV8+IF8mPY="
        );
        assert!(BagOfCells::parse_base64(&transaction.data).is_ok());
    }

    #[test]
    fn test_build_v2_jetton_to_ton_swap_transaction() {
        let simulation: SwapSimulation = serde_json::from_str(include_str!("../testdata/v2_simulation.json")).unwrap();

        let transaction = build_swap_transaction(SwapTransactionParams {
            from_native: false,
            to_native: true,
            sender_jetton_wallet: Some(TEST_SENDER_JETTON_WALLET),
            from_value: "1000000",
            ..SwapTransactionParams::mock(&simulation)
        })
        .unwrap();

        assert_eq!(transaction.to, TEST_SENDER_JETTON_WALLET);
        assert_eq!(transaction.value, "300000000");
        assert_eq!(
            transaction.data,
            "te6cckECAwEAARoAAa4Pin6lAAAAAAAAAAAw9CQIASXCgjXKjRJeZ2WRUT1SByGx/pn3ci9Mh3I85+4N+3OjAAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXzyBycOAEBAeFmZN4qgBJFrE+vsdt6AcnEd7YYqU7dy8PoyudDXogvCv9HxH0j8ADOhSlqUBpedWbkyiJhZFmC4sXNjQrEWMu/cyRGrkFfPgAZ0KUtSgNLzqzcmURMLIswXFi5saFYixl37mSI1cgr54AAAAAyqfiAQAIAkzE2V4gAZ0KUtSgNLzqzcmURMLIswXFi5saFYixl37mSI1cgr54AABlAAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXz4W/Xfwg=="
        );
        assert!(BagOfCells::parse_base64(&transaction.data).is_ok());
    }

    #[test]
    fn test_build_v2_2_hop_swap_transactions() {
        let swap_to_intermediary = SwapSimulation::mock(
            "EQCSIMGBps_qzRG3uPYhON8bucyCtu0mYdL1-u4gSz77IBa3",
            "EQCSLWJ9fY7b0A5OI72wxUp27l4fRlc6GvRBeFf6PiPpH4p3",
            "260238",
            "257635",
        );
        let swap_from_intermediary = SwapSimulation::mock(
            "EQCSLWJ9fY7b0A5OI72wxUp27l4fRlc6GvRBeFf6PiPpH4p3",
            "EQCSIMGBps_qzRG3uPYhON8bucyCtu0mYdL1-u4gSz77IBa3",
            "709",
            "702",
        );
        let cross_transaction = build_swap_transaction(SwapTransactionParams {
            next_swap: Some(NextSwapParams {
                simulation: &swap_from_intermediary,
                min_ask_amount: BigUint::from(694u32),
            }),
            ..SwapTransactionParams::mock(&swap_to_intermediary)
        })
        .unwrap();

        assert_eq!(cross_transaction.to, swap_to_intermediary.offer_jetton_wallet);
        assert_eq!(cross_transaction.value, "1310000000");
        assert_eq!(
            cross_transaction.data,
            "te6cckECBQEAAbUAAWQB84NdAAAAAAAAAABDuaygCABnQpS1KA0vOrNyZREwsizBcWLmxoViLGXfuZIjVyCvnwEB4WZk3iqAEkWsT6+x23oBycR3thipTt3Lw+jK50NeiC8K/0fEfSPwAM6FKWpQGl51ZuTKImFkWYLixc2NCsRYy79zJEauQV8+ABnQpS1KA0vOrNyZREwsizBcWLmxoViLGXfuZIjVyCvngAAAADKp+IBAAgGTMD7mOAElwoI1yo0SXmdlkVE9Ugchsf6Z93IvTIdyPOfuDftzohAAAEADOhSlqUBpedWbkyiJhZFmC4sXNjQrEWMu/cyRGrkFfPgDAeFpzxpbgBJEGDA02f1Zojb3HsQnG+N3OZBW3aTMOl6/XcQJZ99kEADOhSlqUBpedWbkyiJhZFmC4sXNjQrEWMu/cyRGrkFfPgAZ0KUtSgNLzqzcmURMLIswXFi5saFYixl37mSI1cgr54AAAAAyqfiAQAQAkSAraABnQpS1KA0vOrNyZREwsizBcWLmxoViLGXfuZIjVyCvngAAGUADOhSlqUBpedWbkyiJhZFmC4sXNjQrEWMu/cyRGrkFfPhOi4cX"
        );
        assert!(BagOfCells::parse_base64(&cross_transaction.data).is_ok());

        let mut swap_from_intermediary_on_other_router = swap_from_intermediary.clone();
        swap_from_intermediary_on_other_router.router.address = "EQDx--jUU9PUtHltPYZX7wdzIi0SPY3KZ8nvOs0iZvQJd6Ql".to_string();
        let forward_transaction = build_swap_transaction(SwapTransactionParams {
            next_swap: Some(NextSwapParams {
                simulation: &swap_from_intermediary_on_other_router,
                min_ask_amount: BigUint::from(694u32),
            }),
            from_native: false,
            sender_jetton_wallet: Some(TEST_SENDER_JETTON_WALLET),
            from_value: "1000000",
            ..SwapTransactionParams::mock(&swap_to_intermediary)
        })
        .unwrap();

        assert_eq!(forward_transaction.to, TEST_SENDER_JETTON_WALLET);
        assert_eq!(forward_transaction.value, "540000000");
        assert_eq!(
            forward_transaction.data,
            "te6cckECBQEAAd4AAa4Pin6lAAAAAAAAAAAw9CQIASXCgjXKjRJeZ2WRUT1SByGx/pn3ci9Mh3I85+4N+3OjAAzoUpalAaXnVm5MoiYWRZguLFzY0KxFjLv3MkRq5BXzyDk4cAEBAeFmZN4qgBJFrE+vsdt6AcnEd7YYqU7dy8PoyudDXogvCv9HxH0j8ADOhSlqUBpedWbkyiJhZFmC4sXNjQrEWMu/cyRGrkFfPgAZ0KUtSgNLzqzcmURMLIswXFi5saFYixl37mSI1cgr54AAAAAyqfiAQAIBmzA+5jgB4/fRqKenqWjy2nsMr94O5kRaJHsblM+T3nWaRM3oEu6BycOAEAAAQAM6FKWpQGl51ZuTKImFkWYLixc2NCsRYy79zJEauQV8+AMB4WZk3iqAEkQYMDTZ/VmiNvcexCcb43c5kFbdpMw6Xr9dxAln32QQAM6FKWpQGl51ZuTKImFkWYLixc2NCsRYy79zJEauQV8+ABnQpS1KA0vOrNyZREwsizBcWLmxoViLGXfuZIjVyCvngAAAADKp+IBABACRICtoAGdClLUoDS86s3JlETCyLMFxYubGhWIsZd+5kiNXIK+eAAAZQAM6FKWpQGl51ZuTKImFkWYLixc2NCsRYy79zJEauQV8+J6Wy8o="
        );
        assert!(BagOfCells::parse_base64(&forward_transaction.data).is_ok());
    }
}
