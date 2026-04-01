pub mod okx;
pub mod yo;

use chrono::{DateTime, Utc};

use super::{
    balance_differ::BalanceDiffer,
    model::{Transaction, TransactionReciept, TransactionReplayTrace},
};
use crate::ethereum_address_checksum;
use chain_primitives::SwapMapper as BalanceSwapMapper;
use primitives::{AssetId, Chain, Transaction as PrimitivesTransaction, TransactionSwapMetadata, TransactionType};

use self::{okx::OkxParser, yo::YoParser};

pub struct ParseContext<'a> {
    pub chain: &'a Chain,
    pub transaction: &'a Transaction,
    pub receipt: &'a TransactionReciept,
    pub trace: Option<&'a TransactionReplayTrace>,
    pub created_at: DateTime<Utc>,
}

pub trait ProtocolParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool;
    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction>;
}

pub struct ProtocolParsers;

impl ProtocolParsers {
    fn parsers() -> [&'static dyn ProtocolParser; 2] {
        [&OkxParser, &YoParser]
    }

    pub fn map_transaction(
        chain: &Chain,
        transaction: &Transaction,
        receipt: &TransactionReciept,
        trace: Option<&TransactionReplayTrace>,
        created_at: DateTime<Utc>,
    ) -> Option<PrimitivesTransaction> {
        let context = ParseContext {
            chain,
            transaction,
            receipt,
            trace,
            created_at,
        };

        Self::parsers()
            .into_iter()
            .filter(|parser| parser.matches(&context))
            .find_map(|parser| parser.parse(&context))
    }
}

pub fn make_swap_transaction(
    chain: &Chain,
    transaction: &Transaction,
    receipt: &TransactionReciept,
    metadata: &TransactionSwapMetadata,
    created_at: DateTime<Utc>,
) -> Option<PrimitivesTransaction> {
    let from_checksum = ethereum_address_checksum(&transaction.from).ok()?;
    let contract_checksum = transaction.to.as_ref().and_then(|to| ethereum_address_checksum(to).ok());

    Some(PrimitivesTransaction::new(
        transaction.hash.clone(),
        metadata.from_asset.clone(),
        from_checksum.clone(),
        from_checksum,
        contract_checksum,
        TransactionType::Swap,
        receipt.get_state(),
        receipt.get_fee().to_string(),
        AssetId::from_chain(*chain),
        transaction.value.to_string(),
        None,
        serde_json::to_value(metadata).ok(),
        created_at,
    ))
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
