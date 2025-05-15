use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use chrono::Utc;
use primitives::{Asset, AssetId, Chain, TransactionState, TransactionType};

use super::client::CosmosClient;
use super::model::TransactionResponse;
use super::client::TransactionDecode;

pub struct CosmosProvider {
    client: CosmosClient,
}

impl CosmosProvider {
    pub fn new(client: CosmosClient) -> Self {
        Self { client }
    }

    pub fn map_transaction(&self, transaction: TransactionDecode, reciept: TransactionResponse) -> Option<primitives::Transaction> {
        let chain = self.get_chain();
        let tx_auth = transaction.tx.auth_info.clone()?;
        let sequence = tx_auth.signer_infos.first()?.sequence;
        let default_denom = chain.as_denom()?;
        let fee = tx_auth.fee?.amount.into_iter().filter(|x| x.denom == default_denom).collect::<Vec<_>>();
        let fee = fee.first()?.amount.clone();
        let memo = transaction.tx_body.memo.clone();
        let block_number = reciept.tx_response.height.clone();
        let hash = reciept.tx_response.txhash.clone();
        let state = if reciept.tx_response.code == 0 {
            TransactionState::Confirmed
        } else {
            TransactionState::Reverted
        };

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
                        let message: cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend =
                            cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

                        asset_id = chain.as_asset_id();
                        transaction_type = TransactionType::Transfer;
                        let values = self.client.get_amount(message.amount)?;
                        value = values;
                        from_address = message.from_address;
                        to_address = message.to_address;
                    }
                }
                super::client::MESSAGE_DELEGATE => {
                    let message: cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate =
                        cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

                    asset_id = chain.as_asset_id();
                    transaction_type = TransactionType::StakeDelegate;
                    value = message.amount?.amount.clone();
                    from_address = message.delegator_address;
                    to_address = message.validator_address;
                }
                super::client::MESSAGE_UNDELEGATE => {
                    let message: cosmos_sdk_proto::cosmos::staking::v1beta1::MsgUndelegate =
                        cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

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
                    value = reciept.get_rewards_value(chain.as_denom()?).unwrap_or_default().to_string();
                    transaction_type = TransactionType::StakeRewards;
                    from_address = message.delegator_address;
                    to_address = message.validator_address;
                }
                _ => {
                    continue;
                }
            }
            let transaction = primitives::Transaction::new(
                hash,
                asset_id.clone(),
                from_address,
                to_address,
                None,
                transaction_type,
                state,
                block_number,
                sequence.to_string(),
                fee,
                asset_id.clone(),
                value,
                Some(memo),
                None,
                Utc::now(),
            );
            return Some(transaction);
        }
        None
    }
}

#[async_trait]
impl ChainBlockProvider for CosmosProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block("latest").await?;
        let block_number = block.block.header.height.parse::<i64>()?;
        return Ok(block_number);
    }

    async fn get_transactions(&self, block: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let response = self.client.get_block(block.to_string().as_str()).await?;
        let transactions = response.block.data.txs;

        let txs = transactions
            .clone()
            .into_iter()
            .flat_map(|x| self.client.map_transaction_decode(x))
            .collect::<Vec<_>>();
        let txs_futures = txs.clone().into_iter().map(|x| self.client.get_transaction(x.hash));
        let reciepts = futures::future::try_join_all(txs_futures).await?;

        let transactions = txs
            .clone()
            .into_iter()
            .zip(reciepts.iter())
            .filter_map(|(transaction, receipt)| self.map_transaction(transaction, receipt.clone()))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for CosmosProvider {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
