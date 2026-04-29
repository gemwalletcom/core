use super::super::model::SwapSimulation;

#[derive(Debug, Clone, Copy)]
pub struct ReferralParams<'a> {
    pub address: &'a str,
    pub bps: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SwapTransactionParams<'a> {
    pub simulation: &'a SwapSimulation,
    pub from_native: bool,
    pub to_native: bool,
    pub from_value: &'a str,
    pub min_ask_amount: &'a str,
    pub wallet_address: &'a str,
    pub receiver_address: &'a str,
    pub referral: ReferralParams<'a>,
    pub deadline: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TxParams {
    pub to: String,
    pub value: String,
    pub data: String,
}
