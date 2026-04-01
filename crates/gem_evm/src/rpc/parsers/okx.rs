use alloy_primitives::Address;
use num_bigint::BigUint;

use crate::{
    address::ethereum_address_from_topic,
    ethereum_address_checksum,
    rpc::{mapper::TRANSFER_TOPIC, model::Log, staking_mapper::ethereum_value_from_log_data},
};
use primitives::{AssetId, SwapProvider, Transaction as PrimitivesTransaction, TransactionSwapMetadata};

use super::{ParseContext, ProtocolParser, make_swap_transaction, try_map_balance_diff_swap};

pub(crate) const FUNCTION_OKX_DAG_SWAP_BY_ORDER_ID: &str = "0xf2c42696";
const OKX_SWAP_EVENT_TOPIC: &str = "0x1bb43f2da90e35f7b0cf38521ca95a49e68eb42fac49924930a5bd73cdf7576c";
const NATIVE_TOKEN_ADDRESS: &str = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const EVENT_WORD_SIZE: usize = 64;

pub struct OkxParser;

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

impl ProtocolParser for OkxParser {
    fn matches(&self, context: &ParseContext<'_>) -> bool {
        context.transaction.input.starts_with(FUNCTION_OKX_DAG_SWAP_BY_ORDER_ID)
    }

    fn parse(&self, context: &ParseContext<'_>) -> Option<PrimitivesTransaction> {
        let metadata = Self::try_map_receipt_swap(context)
            .or_else(|| try_map_balance_diff_swap(context.chain, &context.transaction.from, context.trace, context.receipt, Some(Self::provider())))?;

        make_swap_transaction(context.chain, context.transaction, context.receipt, &metadata, context.created_at)
    }
}

impl OkxParser {
    fn provider() -> String {
        SwapProvider::Okx.id().to_string()
    }

    fn try_map_receipt_swap(context: &ParseContext<'_>) -> Option<TransactionSwapMetadata> {
        Self::try_map_receipt_event(context).or_else(|| Self::try_map_transfer_swap(context))
    }

    fn try_map_receipt_event(context: &ParseContext<'_>) -> Option<TransactionSwapMetadata> {
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
            provider: Some(Self::provider()),
        })
    }

    fn try_map_transfer_swap(context: &ParseContext<'_>) -> Option<TransactionSwapMetadata> {
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
                provider: Some(Self::provider()),
            }),
            (true, [], [received]) => Some(TransactionSwapMetadata {
                from_asset: AssetId::from_chain(*context.chain),
                from_value: context.transaction.value.to_string(),
                to_asset: AssetId::from_token(*context.chain, &received.token),
                to_value: received.value.clone(),
                provider: Some(Self::provider()),
            }),
            _ => None,
        }
    }

    fn asset_id_from_token(context: &ParseContext<'_>, token: &str) -> Option<AssetId> {
        let token = ethereum_address_checksum(token).ok()?;
        if token == ethereum_address_checksum(NATIVE_TOKEN_ADDRESS).ok()? || token == Address::ZERO.to_checksum(None) {
            return Some(AssetId::from_chain(*context.chain));
        }

        Some(AssetId::from_token(*context.chain, &token))
    }
}

impl ReceiptTransfer {
    fn from_log(log: &Log) -> Option<Self> {
        if log.topics.len() != 3 || log.topics.first().is_none_or(|topic| topic != TRANSFER_TOPIC) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::model::{Log, Transaction, TransactionReciept, TransactionReplayTrace};
    use crate::rpc::parsers::ProtocolParsers;
    use chrono::DateTime;
    use num_bigint::BigUint;
    use primitives::{
        Chain, SwapProvider, TransactionState,
        asset_constants::{BASE_USDC_TOKEN_ID, SMARTCHAIN_CAKE_ASSET_ID},
        testkit::json_rpc::load_json_rpc_result,
    };

    fn erc20_transfer_log(token: &str, from: &str, to: &str, value: &str) -> Log {
        let value = BigUint::parse_bytes(value.as_bytes(), 10).unwrap();

        Log {
            address: token.to_string(),
            topics: vec![
                TRANSFER_TOPIC.to_string(),
                format!("0x{:0>64}", from.trim_start_matches("0x")),
                format!("0x{:0>64}", to.trim_start_matches("0x")),
            ],
            data: format!("0x{:0>64}", value.to_str_radix(16)),
            transaction_hash: None,
        }
    }

    fn map_transaction(chain: &Chain, transaction: &Transaction, receipt: &TransactionReciept, trace: Option<&TransactionReplayTrace>) -> PrimitivesTransaction {
        ProtocolParsers::map_transaction(chain, transaction, receipt, trace, DateTime::from_timestamp(1743373403, 0).unwrap()).unwrap()
    }

    #[test]
    fn test_map_okx_transactions() {
        let receipt_tx = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/okx_base_swap_tx.json"));
        let receipt_only = load_json_rpc_result::<TransactionReciept>(include_str!("../../../testdata/okx_base_swap_tx_receipt.json"));
        let swap_tx = map_transaction(&Chain::Base, &receipt_tx, &receipt_only, None);
        let swap_metadata: TransactionSwapMetadata = serde_json::from_value(swap_tx.metadata.clone().unwrap()).unwrap();
        assert_eq!(swap_tx.transaction_type, primitives::TransactionType::Swap);
        assert_eq!(swap_tx.state, TransactionState::Confirmed);
        assert_eq!(swap_metadata.provider, Some(SwapProvider::Okx.id().to_string()));
        assert_eq!(
            swap_metadata.from_asset,
            AssetId {
                chain: Chain::Base,
                token_id: Some(BASE_USDC_TOKEN_ID.to_string()),
            }
        );
        assert_eq!(
            swap_metadata.to_asset,
            AssetId {
                chain: Chain::Base,
                token_id: Some("0x0000000f2eB9f69274678c76222B35eEc7588a65".to_string()),
            }
        );
        assert_eq!(swap_metadata.from_value, "995000");
        assert_eq!(swap_metadata.to_value, "928345");

        let balance_tx = load_json_rpc_result::<Transaction>(include_str!("../../../testdata/okx_bsc_swap_tx.json"));
        let balance_receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../testdata/okx_bsc_swap_tx_receipt.json"));
        let balance_trace = load_json_rpc_result::<TransactionReplayTrace>(include_str!("../../../testdata/okx_bsc_swap_tx_trace.json"));
        let balance_swap_tx = map_transaction(&Chain::SmartChain, &balance_tx, &balance_receipt, Some(&balance_trace));
        let balance_metadata: TransactionSwapMetadata = serde_json::from_value(balance_swap_tx.metadata.clone().unwrap()).unwrap();
        assert_eq!(balance_swap_tx.transaction_type, primitives::TransactionType::Swap);
        assert_eq!(balance_metadata.provider, Some(SwapProvider::Okx.id().to_string()));
        assert_eq!(balance_metadata.from_asset, SMARTCHAIN_CAKE_ASSET_ID.clone());
        assert_eq!(
            balance_metadata.to_asset,
            AssetId {
                chain: Chain::SmartChain,
                token_id: None,
            }
        );
        assert_eq!(balance_metadata.from_value, "1000000000000000000");
        assert_eq!(balance_metadata.to_value, "2255593079375436");

        let transfer_tx = Transaction {
            from: "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F".to_string(),
            gas: 750000,
            hash: "0x77144af6766c014ad05b0ae90979dc5df9978ecb5829c89925659445b8630dd2".to_string(),
            input: FUNCTION_OKX_DAG_SWAP_BY_ORDER_ID.to_string(),
            to: Some("0x4409921ae43a39a11d90f7b7f96cfd0b8093d9fc".to_string()),
            block_number: BigUint::from(1u32),
            value: BigUint::from(0u8),
        };
        let transfer_receipt = TransactionReciept {
            gas_used: BigUint::from(318420u32),
            effective_gas_price: BigUint::from(10_000_000u64),
            l1_fee: None,
            logs: vec![
                erc20_transfer_log(
                    BASE_USDC_TOKEN_ID,
                    "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F",
                    "0x4409921ae43a39a11d90f7b7f96cfd0b8093d9fc",
                    "995000",
                ),
                erc20_transfer_log(
                    "0x0000000f2eB9f69274678c76222B35eEc7588a65",
                    "0x4409921ae43a39a11d90f7b7f96cfd0b8093d9fc",
                    "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F",
                    "928345",
                ),
            ],
            status: "0x1".to_string(),
            block_number: BigUint::from(1u32),
        };
        let transfer_swap_tx = map_transaction(&Chain::Base, &transfer_tx, &transfer_receipt, None);
        let transfer_metadata: TransactionSwapMetadata = serde_json::from_value(transfer_swap_tx.metadata.clone().unwrap()).unwrap();
        assert_eq!(transfer_swap_tx.transaction_type, primitives::TransactionType::Swap);
        assert_eq!(transfer_metadata.provider, Some(SwapProvider::Okx.id().to_string()));
        assert_eq!(
            transfer_metadata.from_asset,
            AssetId {
                chain: Chain::Base,
                token_id: Some(BASE_USDC_TOKEN_ID.to_string()),
            }
        );
        assert_eq!(
            transfer_metadata.to_asset,
            AssetId {
                chain: Chain::Base,
                token_id: Some("0x0000000f2eB9f69274678c76222B35eEc7588a65".to_string()),
            }
        );
        assert_eq!(transfer_metadata.from_value, "995000");
        assert_eq!(transfer_metadata.to_value, "928345");

        let mut reverted_receipt = load_json_rpc_result::<TransactionReciept>(include_str!("../../../testdata/okx_base_swap_tx_receipt.json"));
        reverted_receipt.status = "0x0".to_string();
        let reverted_tx = map_transaction(&Chain::Base, &receipt_tx, &reverted_receipt, None);
        assert_eq!(reverted_tx.transaction_type, primitives::TransactionType::Swap);
        assert_eq!(reverted_tx.state, TransactionState::Reverted);
    }
}
