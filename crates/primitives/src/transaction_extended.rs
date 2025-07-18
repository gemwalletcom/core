use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use crate::{Transaction, Asset, Price, AssetPrice, AddressName};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct TransactionExtended {
    pub transaction: Transaction,
    pub asset: Asset,
    #[serde(rename = "feeAsset")]
    pub feeAsset: Asset,
    pub price: Option<Price>,
    #[serde(rename = "feePrice")]
    pub fee_price: Option<Price>,
    pub assets: Vec<Asset>,
    pub prices: Vec<AssetPrice>,
    #[serde(rename = "fromAddress")]
    pub from_address: Option<AddressName>,
    #[serde(rename = "toAddress")]
    pub to_address: Option<AddressName>,
}
