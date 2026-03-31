use alloy_primitives::Address;
use num_bigint::BigUint;

use crate::{
    address::ethereum_address_from_topic,
    ethereum_address_checksum,
    rpc::{mapper::TRANSFER_TOPIC, model::Log, staking_mapper::ethereum_value_from_log_data},
};
use primitives::{AssetId, SwapProvider, TransactionSwapMetadata};

use super::{SwapParseContext, SwapParser, try_map_balance_diff_swap};

pub(crate) const FUNCTION_OKX_DAG_SWAP_BY_ORDER_ID: &str = "0xf2c42696";
const OKX_SWAP_EVENT_TOPIC: &str = "0x1bb43f2da90e35f7b0cf38521ca95a49e68eb42fac49924930a5bd73cdf7576c";
const NATIVE_TOKEN_ADDRESS: &str = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const EVENT_WORD_SIZE: usize = 64;

pub struct OkxSwapParser;

struct ReceiptTransfer {
    token: String,
    from: String,
    to: String,
    value: String,
}

struct OkxSwapEvent {
    from_token: String,
    to_token: String,
    user: String,
    from_amount: BigUint,
    to_amount: BigUint,
}

impl SwapParser for OkxSwapParser {
    fn matches(&self, context: &SwapParseContext<'_>) -> bool {
        context.transaction.input.starts_with(FUNCTION_OKX_DAG_SWAP_BY_ORDER_ID)
    }

    fn parse(&self, context: &SwapParseContext<'_>) -> Option<TransactionSwapMetadata> {
        Self::try_map_receipt_swap(context).or_else(|| {
            try_map_balance_diff_swap(
                context.chain,
                &context.transaction.from,
                context.trace,
                context.receipt,
                Some(SwapProvider::Okx.id().to_string()),
            )
        })
    }
}

impl OkxSwapParser {
    fn try_map_receipt_swap(context: &SwapParseContext<'_>) -> Option<TransactionSwapMetadata> {
        Self::try_map_receipt_event(context).or_else(|| Self::try_map_transfer_swap(context))
    }

    fn try_map_receipt_event(context: &SwapParseContext<'_>) -> Option<TransactionSwapMetadata> {
        let event = context
            .receipt
            .logs
            .iter()
            .find(|log| log.topics.len() == 1 && log.topics.first().is_some_and(|topic| topic == OKX_SWAP_EVENT_TOPIC))
            .and_then(|log| OkxSwapEvent::decode(&log.data))?;
        let from = ethereum_address_checksum(&context.transaction.from).ok()?;
        if event.user != from {
            return None;
        }

        let from_asset = Self::asset_id_from_token(context, &event.from_token)?;
        let to_asset = Self::asset_id_from_token(context, &event.to_token)?;
        if from_asset == to_asset {
            return None;
        }

        Some(TransactionSwapMetadata {
            from_asset,
            from_value: event.from_amount.to_string(),
            to_asset,
            to_value: event.to_amount.to_string(),
            provider: Some(SwapProvider::Okx.id().to_string()),
        })
    }

    fn try_map_transfer_swap(context: &SwapParseContext<'_>) -> Option<TransactionSwapMetadata> {
        let from = ethereum_address_checksum(&context.transaction.from).ok()?;
        let transfers: Vec<ReceiptTransfer> = context.receipt.logs.iter().filter_map(ReceiptTransfer::from_log).collect();
        let outgoing: Vec<&ReceiptTransfer> = transfers.iter().filter(|transfer| transfer.from == from && transfer.value != "0").collect();
        let incoming: Vec<&ReceiptTransfer> = transfers.iter().filter(|transfer| transfer.to == from && transfer.value != "0").collect();

        match (context.transaction.value > BigUint::from(0u8), outgoing.as_slice(), incoming.as_slice()) {
            (_, [sent], [received]) if sent.token != received.token => Some(TransactionSwapMetadata {
                from_asset: AssetId::from_token(*context.chain, &sent.token),
                from_value: sent.value.clone(),
                to_asset: AssetId::from_token(*context.chain, &received.token),
                to_value: received.value.clone(),
                provider: Some(SwapProvider::Okx.id().to_string()),
            }),
            (true, [], [received]) => Some(TransactionSwapMetadata {
                from_asset: AssetId::from_chain(*context.chain),
                from_value: context.transaction.value.to_string(),
                to_asset: AssetId::from_token(*context.chain, &received.token),
                to_value: received.value.clone(),
                provider: Some(SwapProvider::Okx.id().to_string()),
            }),
            _ => None,
        }
    }

    fn asset_id_from_token(context: &SwapParseContext<'_>, token: &str) -> Option<AssetId> {
        let token = ethereum_address_checksum(token).ok()?;
        if token == ethereum_address_checksum(NATIVE_TOKEN_ADDRESS).ok()? || token == Address::ZERO.to_checksum(None) {
            return Some(AssetId::from_chain(*context.chain));
        }

        Some(AssetId::from_token(*context.chain, &token))
    }
}

impl ReceiptTransfer {
    fn from_log(log: &Log) -> Option<Self> {
        if log.topics.len() != 3 || !log.topics.first().is_some_and(|topic| topic == TRANSFER_TOPIC) {
            return None;
        }

        Some(Self {
            token: ethereum_address_checksum(&log.address).ok()?,
            from: ethereum_address_from_topic(log.topics.get(1)?)?,
            to: ethereum_address_from_topic(log.topics.get(2)?)?,
            value: ethereum_value_from_log_data(&log.data, 0, EVENT_WORD_SIZE)?.to_string(),
        })
    }
}

impl OkxSwapEvent {
    fn decode(data: &str) -> Option<Self> {
        let data = data.trim_start_matches("0x");
        let words = (0..data.len())
            .step_by(EVENT_WORD_SIZE)
            .map(|start| data.get(start..start + EVENT_WORD_SIZE))
            .collect::<Option<Vec<_>>>()?;
        let [from_token, to_token, user, from_amount, to_amount] = words.as_slice() else {
            return None;
        };

        let from_token = ethereum_address_from_topic(&format!("0x{from_token}"))?;
        let to_token = ethereum_address_from_topic(&format!("0x{to_token}"))?;
        let user = ethereum_address_from_topic(&format!("0x{user}"))?;
        let from_amount = ethereum_value_from_log_data(from_amount, 0, EVENT_WORD_SIZE)?;
        let to_amount = ethereum_value_from_log_data(to_amount, 0, EVENT_WORD_SIZE)?;

        Some(Self {
            from_token,
            to_token,
            user,
            from_amount,
            to_amount,
        })
    }
}
