use std::error::Error;

use primitives::{chain::Chain, Asset, AssetId, AssetType};
use serde_json;

use chain_traits::{ChainAccount, ChainPerpetual, ChainStaking, ChainTraits};
use gem_client::Client;

use crate::rpc::model::{
    AccountInfo, JettonInfo, JettonWalletsResponse, RunGetMethod, TonBroadcastTransaction, TonJettonBalance, TonMessageTransactions, TonResult, TonWalletInfo,
};

use super::model::{Blocks, Chainhead, Shards, Transactions};

pub struct TonClient<C: Client> {
    pub client: C,
}

impl<C: Client> TonClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_master_head(&self) -> Result<TonResult<Chainhead>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/api/v2/getMasterchainInfo").await?)
    }

    pub async fn get_shards(&self, sequence: i64) -> Result<Shards, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/blockchain/masterchain/{}/shards", sequence)).await?)
    }

    pub async fn get_blocks(&self, sequence: i64) -> Result<Blocks, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/blockchain/masterchain/{}/blocks", sequence)).await?)
    }

    pub async fn get_transactions(&self, block_id: String) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v2/blockchain/masterchain/{}/transactions", block_id)).await?)
    }

    pub async fn get_transactions_by_address(&self, address: String, _limit: usize) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/transactionsByMessage?msg_hash={}", address)).await?)
    }

    pub async fn get_token_info(&self, token_id: String) -> Result<TonResult<JettonInfo>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/getTokenData?address={}", token_id)).await?)
    }

    pub async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.get_master_head().await?.result.last.seqno)
    }

    pub async fn get_balance(&self, address: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        let response: TonResult<String> = self.client.get(&format!("/api/v2/getAddressBalance?address={}", address)).await?;
        Ok(response.result)
    }

    pub async fn get_account_info(&self, address: String) -> Result<AccountInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/getAddressBalance?address={}", address)).await?)
    }

    pub async fn get_wallet_information(&self, address: String) -> Result<TonWalletInfo, Box<dyn Error + Send + Sync>> {
        let response: TonResult<TonWalletInfo> = self.client.get(&format!("/api/v2/getWalletInformation?address={}", address)).await?;
        Ok(response.result)
    }

    pub async fn get_token_balance(&self, address: String) -> Result<TonJettonBalance, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/getTokenData?address={}", address)).await?)
    }

    pub async fn run_get_method(&self, address: String, method: String, stack: Vec<String>) -> Result<RunGetMethod, Box<dyn Error + Send + Sync>> {
        let stack_json = serde_json::json!([stack]);
        let body = serde_json::json!({
            "id": "1",
            "jsonrpc": "2.0",
            "method": "runGetMethod",
            "params": {
                "address": address,
                "method": method,
                "stack": stack_json
            }
        });
        Ok(self.client.post("/api/v2/jsonRPC", &body, None).await?)
    }

    pub async fn get_native_balance(&self, address: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/getAddressBalance?address={}", address)).await?)
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<TonResult<TonBroadcastTransaction>, Box<dyn Error + Send + Sync>> {
        let body = serde_json::json!({ "boc": data });
        Ok(self.client.post("/api/v2/sendBocReturnHash", &body, None).await?)
    }

    pub async fn get_transaction(&self, hash: String) -> Result<TonMessageTransactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/transactionsByMessage?msg_hash={}", hash)).await?)
    }

    pub async fn get_jetton_wallets(&self, address: String) -> Result<JettonWalletsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(&format!("/api/v3/jetton/wallets?owner_address={}&limit=100&offset=0", address))
            .await?)
    }

    pub async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let token_info = self.get_token_info(token_id.clone()).await?.result;
        let decimals = token_info.jetton_content.data.decimals as i32;
        Ok(Asset::new(
            AssetId::from_token(Chain::Ton, &token_id),
            token_info.jetton_content.data.name,
            token_info.jetton_content.data.symbol,
            decimals,
            AssetType::JETTON,
        ))
    }
}

impl<C: Client> ChainTraits for TonClient<C> {}
impl<C: Client> ChainAccount for TonClient<C> {}
impl<C: Client> ChainPerpetual for TonClient<C> {}
impl<C: Client> ChainStaking for TonClient<C> {}
