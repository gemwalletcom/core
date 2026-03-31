pub mod okx;

use super::{
    balance_differ::BalanceDiffer,
    model::{Transaction, TransactionReciept, TransactionReplayTrace},
};
use crate::ethereum_address_checksum;
use chain_primitives::SwapMapper as BalanceSwapMapper;
use primitives::{Chain, TransactionSwapMetadata};

pub struct SwapParseContext<'a> {
    pub chain: &'a Chain,
    pub transaction: &'a Transaction,
    pub receipt: &'a TransactionReciept,
    pub trace: Option<&'a TransactionReplayTrace>,
}

pub trait SwapParser {
    fn matches(&self, context: &SwapParseContext<'_>) -> bool;
    fn parse(&self, context: &SwapParseContext<'_>) -> Option<TransactionSwapMetadata>;
}

pub fn try_map_balance_diff_swap(
    chain: &Chain,
    from: &str,
    trace: Option<&TransactionReplayTrace>,
    receipt: &TransactionReciept,
    provider: Option<String>,
) -> Option<TransactionSwapMetadata> {
    let trace = trace?;
    let from = ethereum_address_checksum(from).ok()?;
    let differ = BalanceDiffer::new(*chain);
    let diff_map = differ.calculate(trace, receipt);
    let diff = diff_map.get(&from)?;
    let native_asset_id = chain.as_asset_id();
    let fee = receipt.get_fee();
    BalanceSwapMapper::map_swap(diff, &fee, &native_asset_id, provider)
}
