use crate::stake_type::StakeType;
use crate::swap::ApprovalData;
use crate::transaction_fee::TransactionFee;
use crate::transaction_load_metadata::TransactionLoadMetadata;
use crate::{Asset, GasPriceType, PerpetualType, TransactionPreloadInput, TransferDataExtra, WalletConnectionSessionAppMetadata};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[allow(clippy::large_enum_variant)]
pub enum TransactionInputType {
    Transfer(Asset),
    Deposit(Asset),
    Swap(Asset, Asset),
    Stake(Asset, StakeType),
    TokenApprove(Asset, ApprovalData),
    Generic(Asset, WalletConnectionSessionAppMetadata, TransferDataExtra),
    Perpetual(Asset, PerpetualType),
}

impl TransactionInputType {
    pub fn get_asset(&self) -> &Asset {
        match self {
            TransactionInputType::Transfer(asset) => asset,
            TransactionInputType::Deposit(asset) => asset,
            TransactionInputType::Swap(asset, _) => asset,
            TransactionInputType::Stake(asset, _) => asset,
            TransactionInputType::TokenApprove(asset, _) => asset,
            TransactionInputType::Generic(asset, _, _) => asset,
            TransactionInputType::Perpetual(asset, _) => asset,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLoadInput {
    pub input_type: TransactionInputType,
    pub sender_address: String,
    pub destination_address: String,
    pub value: String,
    pub gas_price: GasPriceType,
    pub memo: Option<String>,
    pub is_max_value: bool,
    pub metadata: TransactionLoadMetadata,
}

impl TransactionLoadInput {
    pub fn default_fee(&self) -> TransactionFee {
        TransactionFee::new_from_fee(self.gas_price.total_fee())
    }
}

impl TransactionLoadInput {
    pub fn to_preload_input(&self) -> TransactionPreloadInput {
        TransactionPreloadInput {
            input_type: self.input_type.clone(),
            sender_address: self.sender_address.clone(),
            destination_address: self.destination_address.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLoadData {
    pub fee: TransactionFee,
    pub metadata: TransactionLoadMetadata,
}

impl TransactionLoadData {
    pub fn new_from(&self, fee: TransactionFee) -> Self {
        Self {
            fee,
            metadata: self.metadata.clone(),
        }
    }
}
