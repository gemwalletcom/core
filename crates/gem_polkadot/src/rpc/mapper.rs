use chrono::{DateTime, Utc};
use primitives::{chain::Chain, Transaction, TransactionState, TransactionType};

use super::model::{Block, Extrinsic, ExtrinsicArguments, TRANSACTION_TYPE_TRANSFER_ALLOW_DEATH, TRANSACTION_TYPE_TRANSFER_KEEP_ALIVE};

pub struct PolkadotMapper;

impl PolkadotMapper {
    pub fn map_transactions(chain: Chain, block: Block) -> Vec<Transaction> {
        let first = block.extrinsics.first();
        let created_at = match first {
            Some(Extrinsic {
                args: ExtrinsicArguments::Timestamp(timestamp),
                ..
            }) => DateTime::from_timestamp_millis(timestamp.now as i64),
            _ => None,
        }
        .expect("Timestamp not found");

        block
            .extrinsics
            .iter()
            .flat_map(|x| Self::map_transaction(chain, block.clone(), x.clone(), created_at))
            .flatten()
            .collect()
    }

    pub fn map_transaction(chain: Chain, block: Block, transaction: Extrinsic, created_at: DateTime<Utc>) -> Vec<Option<Transaction>> {
        match &transaction.args.clone() {
            ExtrinsicArguments::Transfer(transfer) => {
                vec![Self::map_transfer(
                    chain,
                    block,
                    transaction.clone(),
                    transaction.method.method.clone(),
                    transfer.dest.id.clone(),
                    transfer.value.clone(),
                    created_at,
                )]
            }
            ExtrinsicArguments::Transfers(transfers) => transfers
                .calls
                .iter()
                .map(|x| {
                    Self::map_transfer(
                        chain,
                        block.clone(),
                        transaction.clone(),
                        x.method.method.clone(),
                        x.args.dest.id.clone(),
                        x.args.value.clone(),
                        created_at,
                    )
                })
                .collect(),
            _ => vec![],
        }
    }

    fn map_transfer(
        chain: Chain,
        block: Block,
        transaction: Extrinsic,
        method: String,
        to_address: String,
        value: String,
        created_at: DateTime<Utc>,
    ) -> Option<Transaction> {
        if method != TRANSACTION_TYPE_TRANSFER_ALLOW_DEATH && method != TRANSACTION_TYPE_TRANSFER_KEEP_ALIVE {
            return None;
        }

        let from_address = transaction.signature?.signer.id.clone();
        let state = if transaction.success {
            TransactionState::Confirmed
        } else {
            TransactionState::Failed
        };

        Some(Transaction::new(
            transaction.hash.clone(),
            chain.as_asset_id(),
            from_address,
            to_address,
            None,
            TransactionType::Transfer,
            state,
            block.number.to_string(),
            transaction.nonce.unwrap_or_default().clone(),
            transaction.info.partial_fee.unwrap_or("0".to_string()),
            chain.as_asset_id(),
            value,
            None,
            None,
            created_at,
        ))
    }
}
