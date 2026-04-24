use std::error::Error;

use primitives::{Asset, AssetId, AssetType, chain::Chain};
use serde_json;

use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainStaking, ChainTraits};
use gem_client::{Client, ClientExt};

use crate::models::{
    ApiResult, BroadcastTransaction, Chainhead, JettonInfo, JettonOffchainMetadata, JettonWalletsResponse, MessageTransactions, NftCollection, NftCollectionsResponse, NftItem,
    NftItemsResponse, SimpleJettonBalance, WalletInfo,
};

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

    pub async fn get_transactions_by_address(&self, address: String, limit: usize) -> Result<MessageTransactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/transactions?account={}&limit={}", address, limit)).await?)
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
        Ok(self.client.post("/api/v2/sendBocReturnHash", &body).await?)
    }

    pub async fn get_transaction(&self, hash: String) -> Result<MessageTransactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/transactionsByMessage?msg_hash={}", hash)).await?)
    }

    pub async fn get_jetton_wallets(&self, address: String) -> Result<JettonWalletsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/jetton/wallets?owner_address={}&limit=100&offset=0", address)).await?)
    }

    pub async fn get_nft_items_by_owner(&self, owner_address: &str) -> Result<Vec<NftItem>, Box<dyn Error + Send + Sync>> {
        let response: NftItemsResponse = self.client.get(&format!("/api/v3/nft/items?owner_address={}&limit=1000&offset=0", owner_address)).await?;
        Ok(response.nft_items)
    }

    pub async fn get_nft_item(&self, address: &str) -> Result<NftItem, Box<dyn Error + Send + Sync>> {
        let response: NftItemsResponse = self.client.get(&format!("/api/v3/nft/items?address={}", address)).await?;
        response.nft_items.into_iter().next().ok_or_else(|| "NFT item not found".into())
    }

    pub async fn get_nft_collection(&self, collection_address: &str) -> Result<NftCollection, Box<dyn Error + Send + Sync>> {
        let response: NftCollectionsResponse = self.client.get(&format!("/api/v3/nft/collections?collection_address={}", collection_address)).await?;
        response.nft_collections.into_iter().next().ok_or_else(|| "NFT collection not found".into())
    }

    pub async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let token_info = self.get_token_info(token_id.clone()).await?.result;
        let data = &token_info.jetton_content.data;
        let decimals = data.decimals as i32;

        let (name, symbol) = match (&data.name, &data.symbol) {
            (Some(name), Some(symbol)) => (name.clone(), symbol.clone()),
            _ => {
                let uri = data.uri.as_ref().ok_or("missing jetton metadata uri")?;
                self.get_token_metadata_offchain(uri).await?
            }
        };

        Ok(Asset::new(AssetId::from_token(Chain::Ton, &token_id), name, symbol, decimals, AssetType::JETTON))
    }

    async fn get_token_metadata_offchain(&self, uri: &str) -> Result<(String, String), Box<dyn Error + Send + Sync>> {
        let metadata: JettonOffchainMetadata = self.client.get_url(uri).await?;
        Ok((metadata.name, metadata.symbol))
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
