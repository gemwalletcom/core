use std::error::Error;

use super::model::{BlockResponse, TransactionResponse};
use base64::{engine::general_purpose, Engine as _};
use hex;
use primitives::Chain;
use reqwest_middleware::ClientWithMiddleware;
use sha2::{Digest, Sha256};

pub const MESSAGE_DELEGATE: &str = "/cosmos.staking.v1beta1.MsgDelegate";
pub const MESSAGE_UNDELEGATE: &str = "/cosmos.staking.v1beta1.MsgUndelegate";
pub const MESSAGE_REDELEGATE: &str = "/cosmos.staking.v1beta1.MsgBeginRedelegate";
pub const MESSAGE_SEND_BETA: &str = "/cosmos.bank.v1beta1.MsgSend";
pub const MESSAGE_REWARD_BETA: &str = "/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward";
pub const MESSAGE_SEND: &str = "/types.MsgSend"; // thorchain

pub const MESSAGES: &[&str] = &[
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

    pub fn get_chain(&self) -> Chain {
        self.chain
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
