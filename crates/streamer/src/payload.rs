use primitives::{AssetAddress, AssetId, Chain, GorushNotification, Transaction};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsPayload {
    pub chain: Chain,
    pub blocks: Vec<i64>,
    pub transactions: Vec<Transaction>,
}

impl fmt::Display for TransactionsPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "chain: {}, blocks: {}, transactions: {}",
            self.chain.as_ref(),
            self.blocks.len(),
            self.transactions.len()
        )
    }
}

impl TransactionsPayload {
    pub fn new(chain: Chain, blocks: Vec<i64>, transactions: Vec<Transaction>) -> Self {
        Self { chain, blocks, transactions }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsPayload {
    pub notifications: Vec<GorushNotification>,
}

impl fmt::Display for NotificationsPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "notifications: {}", self.notifications.len())
    }
}

impl NotificationsPayload {
    pub fn new(notifications: Vec<GorushNotification>) -> Self {
        Self { notifications }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchAssetsPayload {
    pub asset_id: AssetId,
}

impl fmt::Display for FetchAssetsPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chain: {}, token_id: {:?}", self.asset_id.chain.as_ref(), self.asset_id.token_id)
    }
}

impl FetchAssetsPayload {
    pub fn new(asset_id: AssetId) -> Self {
        Self { asset_id }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchBlocksPayload {
    pub chain: Chain,
    pub blocks: Vec<i64>,
}

impl fmt::Display for FetchBlocksPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chain: {}, blocks: {:?}", self.chain.as_ref(), self.blocks)
    }
}

impl FetchBlocksPayload {
    pub fn new(chain: Chain, blocks: Vec<i64>) -> Self {
        Self { chain, blocks }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchNFTCollectionPayload {
    pub chain: Chain,
    pub collection_id: String,
}

impl FetchNFTCollectionPayload {
    pub fn new(chain: Chain, collection_id: String) -> Self {
        Self { chain, collection_id }
    }
}

impl fmt::Display for FetchNFTCollectionPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "chain: {}, collection_id: {}", self.chain.as_ref(), self.collection_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchNFTCollectionAssetPayload {
    pub chain: Chain,
    pub collection_id: String,
    pub asset_id: String,
}

impl FetchNFTCollectionAssetPayload {
    pub fn new(chain: Chain, collection_id: String, asset_id: String) -> Self {
        Self {
            chain,
            collection_id,
            asset_id,
        }
    }
}

impl fmt::Display for FetchNFTCollectionAssetPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "chain: {}, collection_id: {}, asset_id: {}",
            self.chain.as_ref(),
            self.collection_id,
            self.asset_id
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressAssetsPayload {
    pub values: Vec<AssetAddress>,
}

impl AddressAssetsPayload {
    pub fn new(values: Vec<AssetAddress>) -> Self {
        Self { values }
    }
}

impl fmt::Display for AddressAssetsPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "addresses: {}", self.values.len())
    }
}
