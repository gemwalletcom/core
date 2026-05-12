use std::error::Error;

use primitives::{Asset, AssetId, AssetType, chain::Chain};
use serde::Serialize;
use serde_json;

use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual, ChainStaking, ChainTraits};
use gem_client::{Client, ClientExt, build_path_with_query};

use crate::models::{
    ApiResult, BroadcastTransaction, Chainhead, JettonInfo, JettonOffchainMetadata, JettonWalletsResponse, NftCollectionsResponse, NftItemsResponse, SimpleJettonBalance,
    TraceByAddressQuery, TraceByBlockQuery, TraceByMessageQuery, TraceByTransactionQuery, TraceResponse, WalletInfo,
};

const TONCENTER_V3_BLOCK_LIMIT: usize = 100;
const TONCENTER_SORT_DESC: &str = "desc";
const TONCENTER_SORT_ASC: &str = "asc";

#[derive(Debug)]
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

    pub async fn get_traces_by_message(&self, hash: String) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let query = TraceByMessageQuery {
            msg_hash: hash,
            include_actions: true,
        };
        self.get_traces(query).await
    }

    pub async fn get_traces_by_transaction(&self, hash: String) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let query = TraceByTransactionQuery {
            tx_hash: hash,
            include_actions: true,
        };
        self.get_traces(query).await
    }

    pub async fn get_traces_by_hash(&self, hash: String) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let traces = self.get_traces_by_message(hash.clone()).await?;
        if traces.traces.is_empty() {
            self.get_traces_by_transaction(hash).await
        } else {
            Ok(traces)
        }
    }

    pub async fn get_traces_by_masterchain_block(&self, block: u64) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let query = TraceByBlockQuery {
            mc_seqno: block,
            include_actions: true,
            limit: TONCENTER_V3_BLOCK_LIMIT,
            offset: 0,
            sort: TONCENTER_SORT_ASC,
        };
        self.get_traces(query).await
    }

    pub async fn get_traces_by_address(&self, address: String, limit: usize) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let query = TraceByAddressQuery {
            account: address,
            include_actions: true,
            limit,
            offset: 0,
            sort: TONCENTER_SORT_DESC,
        };
        self.get_traces(query).await
    }

    async fn get_traces<T: Serialize>(&self, query: T) -> Result<TraceResponse, Box<dyn Error + Send + Sync>> {
        let path = build_path_with_query("/api/v3/traces", &query)?;
        Ok(self.client.get(&path).await?)
    }

    pub async fn get_jetton_wallets(&self, address: String) -> Result<JettonWalletsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/jetton/wallets?owner_address={}&limit=100&offset=0", address)).await?)
    }

    pub async fn get_nft_items_by_owner(&self, owner_address: &str) -> Result<NftItemsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/nft/items?owner_address={}&limit=1000&offset=0", owner_address)).await?)
    }

    pub async fn get_nft_item(&self, address: &str) -> Result<NftItemsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/nft/items?address={}", address)).await?)
    }

    pub async fn get_nft_collection(&self, collection_address: &str) -> Result<NftCollectionsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v3/nft/collections?collection_address={}", collection_address)).await?)
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
