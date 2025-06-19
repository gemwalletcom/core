use chrono::DateTime;
use primitives::{AssetId, Chain, Transaction, TransactionState, TransactionType};

use super::client::TransactionDecode;
use super::model::TransactionResponse;

pub struct CosmosMapper;

impl CosmosMapper {
    pub fn map_transaction(chain: Chain, transaction: TransactionDecode, receipt: TransactionResponse) -> Option<Transaction> {
        let tx_auth = transaction.tx.auth_info.clone()?;
        let default_denom = chain.as_denom()?;
        let fee = tx_auth.fee?.amount.into_iter().filter(|x| x.denom == default_denom).collect::<Vec<_>>();
        let fee = fee.first()?.amount.clone();
        let memo = transaction.tx_body.memo.clone();
        let hash = receipt.tx_response.txhash.clone();
        let state = if receipt.tx_response.code == 0 {
            TransactionState::Confirmed
        } else {
            TransactionState::Reverted
        };
        let created_at = DateTime::parse_from_rfc3339(&receipt.tx_response.timestamp).ok()?.into();

        for message in transaction.clone().tx_body.messages {
            let asset_id: AssetId;
            let transaction_type: TransactionType;
            let value: String;
            let from_address: String;
            let to_address: String;

            match message.type_url.as_str() {
                super::client::MESSAGE_SEND | super::client::MESSAGE_SEND_BETA => {
                    // special handling for thorchain as it uses a different message type and decoding does not work
                    if message.type_url.as_str() == super::client::MESSAGE_SEND {
                        let message: super::model::MessageSend = serde_json::from_slice(&message.value).ok()?;
                        asset_id = chain.as_asset_id();
                        transaction_type = TransactionType::Transfer;
                        value = message.amount.first()?.amount.clone();
                        from_address = message.from_address;
                        to_address = message.to_address;
                    } else {
                        let message: cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend = cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

                        asset_id = chain.as_asset_id();
                        transaction_type = TransactionType::Transfer;
                        // Getting the amount requires client helper, so users of this mapper will need to extract the amount
                        value = Self::extract_amount(&message.amount)?;
                        from_address = message.from_address;
                        to_address = message.to_address;
                    }
                }
                super::client::MESSAGE_DELEGATE => {
                    let message: cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate = cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

                    asset_id = chain.as_asset_id();
                    transaction_type = TransactionType::StakeDelegate;
                    value = message.amount?.amount.clone();
                    from_address = message.delegator_address;
                    to_address = message.validator_address;
                }
                super::client::MESSAGE_UNDELEGATE => {
                    let message: cosmos_sdk_proto::cosmos::staking::v1beta1::MsgUndelegate = cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

                    asset_id = chain.as_asset_id();
                    transaction_type = TransactionType::StakeUndelegate;
                    value = message.amount?.amount.clone();
                    from_address = message.delegator_address;
                    to_address = message.validator_address;
                }
                super::client::MESSAGE_REDELEGATE => {
                    let message: cosmos_sdk_proto::cosmos::staking::v1beta1::MsgBeginRedelegate =
                        cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

                    asset_id = chain.as_asset_id();
                    transaction_type = TransactionType::StakeRedelegate;
                    value = message.amount?.amount.clone();
                    from_address = message.delegator_address;
                    to_address = message.validator_dst_address;
                }
                super::client::MESSAGE_REWARD_BETA => {
                    let message: cosmos_sdk_proto::cosmos::distribution::v1beta1::MsgWithdrawDelegatorReward =
                        cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

                    asset_id = chain.as_asset_id();
                    value = receipt.get_rewards_value(chain.as_denom()?).unwrap_or_default().to_string();
                    transaction_type = TransactionType::StakeRewards;
                    from_address = message.delegator_address;
                    to_address = message.validator_address;
                }
                _ => {
                    continue;
                }
            }
            let transaction = Transaction::new(
                hash,
                asset_id.clone(),
                from_address,
                to_address,
                None,
                transaction_type,
                state,
                fee,
                asset_id.clone(),
                value,
                Some(memo),
                None,
                created_at,
            );
            return Some(transaction);
        }
        None
    }

    // Helper method to extract amount from coin list
    fn extract_amount(amounts: &[cosmos_sdk_proto::cosmos::base::v1beta1::Coin]) -> Option<String> {
        if amounts.is_empty() {
            return None;
        }
        Some(amounts[0].amount.clone())
    }
}
