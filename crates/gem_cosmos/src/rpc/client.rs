use std::error::Error;

use crate::rpc::model::TransactionsResponse;

use super::model::{BlockResponse, TransactionResponse};
use primitives::Chain;
use reqwest_middleware::ClientWithMiddleware;

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

impl CosmosClient {
    pub fn new(chain: Chain, client: ClientWithMiddleware, url: String) -> Self {
        Self { chain, url, client }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
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
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_block(&self, block: &str) -> Result<BlockResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/cosmos/base/tendermint/v1beta1/blocks/{}", self.url, block);
        Ok(self.client.get(url).send().await?.json().await?)
    }

    pub async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<TransactionResponse>, Box<dyn Error + Send + Sync>> {
        let inbound = self.get_transactions_by_query(format!("message.sender='{}'", address), limit).await?;
        let outbound = self.get_transactions_by_query(format!("message.recipient='{}'", address), limit).await?;
        let responses = inbound.tx_responses.into_iter().chain(outbound.tx_responses.into_iter()).collect::<Vec<_>>();
        let txs = inbound.txs.into_iter().chain(outbound.txs.into_iter()).collect::<Vec<_>>();
        Ok(responses
            .into_iter()
            .zip(txs)
            .map(|(response, tx)| TransactionResponse { tx, tx_response: response })
            .collect::<Vec<_>>())
    }

    pub async fn get_transactions_by_query(&self, query: String, limit: usize) -> Result<TransactionsResponse, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/cosmos/tx/v1beta1/txs", self.url);
        let query = [("query", query), ("pagination.limit", limit.to_string()), ("page", 1.to_string())];
        Ok(self.client.get(url).query(&query).send().await?.json().await?)
    }
}
