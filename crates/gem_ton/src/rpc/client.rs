use std::error::Error;

use primitives::{chain::Chain, Asset, AssetId, AssetType};
use serde_json;

use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainStaking, ChainTraits};
use gem_client::Client;

use crate::models::{ApiResult, BroadcastTransaction, Chainhead, JettonInfo, JettonWalletsResponse, MessageTransactions, SimpleJettonBalance, WalletInfo};

pub struct TonClient<C: Client> {
    pub client: C,
}

impl<C: Client> TonClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_master_head(&self) -> Result<Chainhead, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/api/v3/masterchainInfo").await?)
    }

    pub async fn get_transactions_by_masterchain_block(&self, block_id: String) -> Result<MessageTransactions, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(&format!("/api/v3/transactionsByMasterchainBlock?seqno={}&limit=100&offset=0", block_id))
            .await?)
    }

    pub async fn get_transactions_by_address(&self, address: String, _limit: usize) -> Result<MessageTransactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/transactionsByMessage?msg_hash={}", address)).await?)
    }

    pub async fn get_token_info(&self, token_id: String) -> Result<ApiResult<JettonInfo>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/getTokenData?address={}", token_id)).await?)
    }

    pub async fn get_balance(&self, address: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        let response: ApiResult<String> = self.client.get(&format!("/api/v2/getAddressBalance?address={}", address)).await?;
        Ok(response.result)
    }

    pub async fn get_wallet_information(&self, address: String) -> Result<WalletInfo, Box<dyn Error + Send + Sync>> {
        let response: ApiResult<WalletInfo> = self.client.get(&format!("/api/v2/getWalletInformation?address={}", address)).await?;
        Ok(response.result)
    }

    pub async fn get_token_balance(&self, address: String) -> Result<SimpleJettonBalance, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/getTokenData?address={}", address)).await?)
    }

    pub async fn get_native_balance(&self, address: String) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/getAddressBalance?address={}", address)).await?)
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<ApiResult<BroadcastTransaction>, Box<dyn Error + Send + Sync>> {
        let body = serde_json::json!({ "boc": data });
        Ok(self.client.post("/api/v2/sendBocReturnHash", &body, None).await?)
    }

    pub async fn get_transaction(&self, hash: String) -> Result<MessageTransactions, Box<dyn Error + Send + Sync>> {
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
impl<C: Client> ChainAddressStatus for TonClient<C> {}
impl<C: Client> ChainStaking for TonClient<C> {}
impl<C: Client> chain_traits::ChainProvider for TonClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        Chain::Ton
    }
}
