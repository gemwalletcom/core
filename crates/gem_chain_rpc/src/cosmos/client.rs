use std::error::Error;

use super::model::{BlockResponse, MessageSend, TransactionResponse};
use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use hex;
use primitives::{Asset, AssetId, Chain, TransactionState, TransactionType};
use reqwest_middleware::ClientWithMiddleware;
use sha2::{Digest, Sha256};

const MESSAGE_DELEGATE: &str = "/cosmos.staking.v1beta1.MsgDelegate";
const MESSAGE_UNDELEGATE: &str = "/cosmos.staking.v1beta1.MsgUndelegate";
const MESSAGE_REDELEGATE: &str = "/cosmos.staking.v1beta1.MsgBeginRedelegate";
const MESSAGE_SEND_BETA: &str = "/cosmos.bank.v1beta1.MsgSend";
const MESSAGE_REWARD_BETA: &str = "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward";
const MESSAGE_SEND: &str = "/types.MsgSend"; // thorchain

const MESSAGES: &[&str] = &[
    MESSAGE_SEND,
    MESSAGE_SEND_BETA,
    MESSAGE_DELEGATE,
    MESSAGE_UNDELEGATE,
    MESSAGE_REDELEGATE,
    MESSAGE_REWARD_BETA,
];

pub struct CosmosClient {
    chain: Chain,
    url: String,
    client: ClientWithMiddleware,
}

#[derive(Debug, Clone)]
pub struct TransactionDecode {
    pub hash: String,
    pub body: String,
    pub tx: cosmos_sdk_proto::cosmos::tx::v1beta1::Tx,
    pub tx_body: cosmos_sdk_proto::cosmos::tx::v1beta1::TxBody,
}

impl CosmosClient {
    pub fn new(chain: Chain, client: ClientWithMiddleware, url: String) -> Self {
        Self { chain, url, client }
    }

    pub fn get_hash(&self, bytes: Vec<u8>) -> String {
        hex::encode(Sha256::digest(bytes.clone())).to_uppercase()
    }

    pub fn map_transaction_decode(&self, body: String) -> Option<TransactionDecode> {
        let bytes = general_purpose::STANDARD.decode(body.clone()).ok()?;
        let tx: cosmos_sdk_proto::cosmos::tx::v1beta1::Tx = cosmos_sdk_proto::prost::Message::decode(&*bytes).ok()?;
        let tx_body = tx.clone().body?;

        // only decode supported transactions.
        if tx_body
            .clone()
            .messages
            .into_iter()
            .filter(|x| MESSAGES.contains(&x.type_url.as_str()))
            .collect::<Vec<_>>()
            .is_empty()
        {
            return None;
        }
        Some(TransactionDecode {
            hash: self.get_hash(bytes.clone()),
            body: body.clone(),
            tx,
            tx_body,
        })
    }

    pub fn get_amount(&self, coins: Vec<cosmos_sdk_proto::cosmos::base::v1beta1::Coin>) -> Option<String> {
        Some(
            coins
                .into_iter()
                .filter(|x| x.denom == self.chain.clone().as_denom().unwrap_or_default())
                .collect::<Vec<_>>()
                .first()?
                .amount
                .clone(),
        )
    }

    pub fn map_transaction(&self, transaction: TransactionDecode, reciept: TransactionResponse) -> Option<primitives::Transaction> {
        let chain = self.get_chain();
        let tx_auth = transaction.tx.auth_info.clone()?;
        let sequence = tx_auth.signer_infos.first()?.sequence;
        let default_denom = self.chain.as_denom()?;
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
                MESSAGE_SEND | MESSAGE_SEND_BETA => {
                    // special handling for thorchain as it uses a different message type and decoding does not work
                    let message: MessageSend = serde_json::from_value(reciept.tx.body.messages.first()?.clone()).ok()?;
                    let amount = message.amount.first()?.clone();

                    asset_id = if amount.denom == self.chain.as_denom()? {
                        self.get_chain().as_asset_id()
                    } else {
                        AssetId::from(self.chain, Some(amount.denom.clone()))
                    };
                    transaction_type = TransactionType::Transfer;
                    value = if asset_id.is_native() {
                        message.get_amount(self.chain.as_denom()?)?.to_string()
                    } else {
                        message.get_amount(&asset_id.token_id.clone()?)?.to_string()
                    };
                    from_address = message.from_address;
                    to_address = message.to_address;
                }
                MESSAGE_DELEGATE => {
                    let message: cosmos_sdk_proto::cosmos::staking::v1beta1::MsgDelegate = cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

                    asset_id = chain.as_asset_id();
                    value = message.amount?.amount.clone();
                    transaction_type = TransactionType::StakeDelegate;
                    from_address = message.delegator_address;
                    to_address = message.validator_address;
                }
                MESSAGE_UNDELEGATE => {
                    let message: cosmos_sdk_proto::cosmos::staking::v1beta1::MsgUndelegate = cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

                    asset_id = chain.as_asset_id();
                    transaction_type = TransactionType::StakeUndelegate;
                    value = message.amount?.amount.clone();
                    from_address = message.delegator_address;
                    to_address = message.validator_address;
                }
                MESSAGE_REDELEGATE => {
                    let message: cosmos_sdk_proto::cosmos::staking::v1beta1::MsgBeginRedelegate =
                        cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

                    asset_id = chain.as_asset_id();
                    transaction_type = TransactionType::StakeRedelegate;
                    value = message.amount?.amount.clone();
                    from_address = message.delegator_address;
                    to_address = message.validator_dst_address;
                }
                MESSAGE_REWARD_BETA => {
                    let message: cosmos_sdk_proto::cosmos::distribution::v1beta1::MsgWithdrawDelegatorReward =
                        cosmos_sdk_proto::prost::Message::decode(&*message.value).ok()?;

                    asset_id = chain.as_asset_id();
                    value = reciept.get_rewards_value(self.chain.as_denom()?).unwrap_or_default().to_string();
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
                Utc::now().naive_utc(),
            );
            return Some(transaction);
        }
        None
    }

    pub async fn get_transaction(&self, hash: String) -> Result<TransactionResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/cosmos/tx/v1beta1/txs/{}", self.url, hash);
        let transaction = self.client.get(url).send().await?.json::<TransactionResponse>().await?;
        Ok(transaction)
    }

    pub async fn get_block(&self, block: &str) -> Result<BlockResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/cosmos/base/tendermint/v1beta1/blocks/{}", self.url, block);
        let block = self.client.get(url).send().await?.json::<BlockResponse>().await?;
        Ok(block)
    }
}

#[async_trait]
impl ChainBlockProvider for CosmosClient {
    fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block = self.get_block("latest").await?;
        let block_number = block.block.header.height.parse::<i64>()?;
        return Ok(block_number);
    }

    async fn get_transactions(&self, block: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let response = self.get_block(block.to_string().as_str()).await?;
        let transactions = response.block.data.txs;

        let txs = transactions
            .clone()
            .into_iter()
            .flat_map(|x| self.map_transaction_decode(x))
            .collect::<Vec<_>>();
        let txs_futures = txs.clone().into_iter().map(|x| self.get_transaction(x.hash));
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
impl ChainTokenDataProvider for CosmosClient {
    async fn get_token_data(&self, _chain: Chain, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
