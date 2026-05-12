use super::super::model::SwapSimulation;
use gem_ton::{address::Address, tvm::CellArc};
use num_bigint::BigUint;

#[derive(Debug, Clone, Copy)]
pub struct ReferralParams {
    pub address: Address,
    pub bps: u32,
}

#[derive(Debug, Clone)]
pub struct NextSwapParams<'a> {
    pub simulation: &'a SwapSimulation,
    pub min_ask_amount: BigUint,
}

#[derive(Debug, Clone)]
pub struct SwapTransactionParams<'a> {
    pub simulation: &'a SwapSimulation,
    pub next_swap: Option<NextSwapParams<'a>>,
    pub from_native: bool,
    pub to_native: bool,
    pub sender_jetton_wallet: Option<&'a str>,
    pub from_value: &'a str,
    pub wallet_address: Address,
    pub receiver_address: Address,
    pub referral: ReferralParams,
    pub deadline: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TxParams {
    pub to: String,
    pub value: String,
    pub data: String,
}

pub(super) struct SwapCellParams<'a> {
    pub opcode: u32,
    pub ask_wallet: Address,
    pub refund_address: Address,
    pub receiver_address: Address,
    pub min_ask_amount: BigUint,
    pub forward_gas: u64,
    pub next_payload: Option<&'a CellArc>,
    pub referral_bps: u32,
    pub referral_address: Address,
    pub deadline: u64,
}
