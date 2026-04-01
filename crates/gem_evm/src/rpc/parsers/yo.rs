use crate::{
    address::{ethereum_address_checksum, ethereum_address_from_topic},
    rpc::{mapper::TRANSFER_TOPIC, staking_mapper::ethereum_value_from_log_data},
};
use primitives::{AssetId, Transaction as PrimitivesTransaction, TransactionType, contract_constants::ETHEREUM_YO_PROTOCOL_CONTRACT};

use super::{ParseContext, ProtocolParser};

pub(crate) const FUNCTION_YO_DEPOSIT: &str = "0x82b78ba7";
pub(crate) const FUNCTION_YO_WITHDRAW: &str = "0x99519ab8";

pub struct YoParser;

impl ProtocolParser for YoParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        let Some(to) = context.transaction.to.as_ref().and_then(|to| ethereum_address_checksum(to).ok()) else {
            return false;
        };
        let Some(contract) = ethereum_address_checksum(ETHEREUM_YO_PROTOCOL_CONTRACT).ok() else {
            return false;
        };

        to == contract && (context.transaction.input.starts_with(FUNCTION_YO_DEPOSIT) || context.transaction.input.starts_with(FUNCTION_YO_WITHDRAW))
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let from = ethereum_address_checksum(&context.transaction.from).ok()?;
        let to = ethereum_address_checksum(context.transaction.to.as_ref()?).ok()?;
        let (transaction_type, topic_index) = if context.transaction.input.starts_with(FUNCTION_YO_DEPOSIT) {
            (TransactionType::EarnDeposit, 1)
        } else if context.transaction.input.starts_with(FUNCTION_YO_WITHDRAW) {
            (TransactionType::EarnWithdraw, 2)
        } else {
            return None;
        };

        let log = context.receipt.logs.iter().find(|log| {
            log.topics.len() == 3
                && log.topics.first().is_some_and(|topic| topic == TRANSFER_TOPIC)
                && log
                    .topics
                    .get(topic_index)
                    .and_then(|topic| ethereum_address_from_topic(topic))
                    .is_some_and(|address| address == from)
        })?;
        let token_id = ethereum_address_checksum(&log.address).ok()?;
        let value = ethereum_value_from_log_data(&log.data, 0, 64)?;

        Some(PrimitivesTransaction::new(
            context.transaction.hash.clone(),
            AssetId::from_token(*context.chain, &token_id),
            from,
            to,
            None,
            transaction_type,
            context.receipt.get_state(),
            context.receipt.get_fee().to_string(),
            AssetId::from_chain(*context.chain),
            value.to_string(),
            None,
            None,
            context.created_at,
        ))
    }
}
