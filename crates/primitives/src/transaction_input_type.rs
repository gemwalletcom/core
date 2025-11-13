use crate::stake_type::StakeType;
use crate::swap::{ApprovalData, SwapData};
use crate::transaction_fee::TransactionFee;
use crate::transaction_load_metadata::TransactionLoadMetadata;
use crate::{
    Asset, GasPriceType, PerpetualType, TransactionPreloadInput, TransactionType, TransferDataExtra, WalletConnectionSessionAppMetadata, nft::NFTAsset,
    perpetual::AccountDataType,
};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[allow(clippy::large_enum_variant)]
pub enum TransactionInputType {
    Transfer(Asset),
    Deposit(Asset),
    Swap(Asset, Asset, SwapData),
    Stake(Asset, StakeType),
    TokenApprove(Asset, ApprovalData),
    Generic(Asset, WalletConnectionSessionAppMetadata, TransferDataExtra),
    TransferNft(Asset, NFTAsset),
    Account(Asset, AccountDataType),
    Perpetual(Asset, PerpetualType),
}

impl TransactionInputType {
    pub fn get_asset(&self) -> &Asset {
        match self {
            TransactionInputType::Transfer(asset) => asset,
            TransactionInputType::Deposit(asset) => asset,
            TransactionInputType::Swap(asset, _, _) => asset,
            TransactionInputType::Stake(asset, _) => asset,
            TransactionInputType::TokenApprove(asset, _) => asset,
            TransactionInputType::Generic(asset, _, _) => asset,
            TransactionInputType::TransferNft(asset, _) => asset,
            TransactionInputType::Account(asset, _) => asset,
            TransactionInputType::Perpetual(asset, _) => asset,
        }
    }

    pub fn get_recipient_asset(&self) -> &Asset {
        match self {
            TransactionInputType::Transfer(asset) => asset,
            TransactionInputType::Deposit(asset) => asset,
            TransactionInputType::Swap(_, asset, _) => asset,
            TransactionInputType::Stake(asset, _) => asset,
            TransactionInputType::TokenApprove(asset, _) => asset,
            TransactionInputType::Generic(asset, _, _) => asset,
            TransactionInputType::TransferNft(asset, _) => asset,
            TransactionInputType::Account(asset, _) => asset,
            TransactionInputType::Perpetual(asset, _) => asset,
        }
    }

    pub fn transaction_type(&self) -> TransactionType {
        match self {
            TransactionInputType::Transfer(_) | TransactionInputType::Deposit(_) => TransactionType::Transfer,
            TransactionInputType::Swap(_, _, _) => TransactionType::Swap,
            TransactionInputType::Stake(_, stake_type) => match stake_type {
                StakeType::Stake(_) => TransactionType::StakeDelegate,
                StakeType::Unstake(_) => TransactionType::StakeUndelegate,
                StakeType::Redelegate(_) => TransactionType::StakeRedelegate,
                StakeType::Rewards(_) => TransactionType::StakeRewards,
                StakeType::Withdraw(_) => TransactionType::StakeWithdraw,
                StakeType::Freeze(_) => TransactionType::StakeFreeze,
            },
            TransactionInputType::TokenApprove(_, _) => TransactionType::TokenApproval,
            TransactionInputType::Generic(_, _, _) => TransactionType::SmartContractCall,
            TransactionInputType::TransferNft(_, _) => TransactionType::TransferNFT,
            TransactionInputType::Account(_, _) => TransactionType::AssetActivation,
            TransactionInputType::Perpetual(_, perpetual_type) => match perpetual_type {
                PerpetualType::Open(_) | PerpetualType::Increase(_) => TransactionType::PerpetualOpenPosition,
                PerpetualType::Close(_) | PerpetualType::Reduce(_) => TransactionType::PerpetualClosePosition,
                PerpetualType::Modify(_) => TransactionType::PerpetualModifyPosition,
            },
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Asset, DelegationValidator, PerpetualConfirmData, PerpetualDirection};

    #[test]
    fn transaction_types() {
        assert_eq!(TransactionInputType::Transfer(Asset::mock()).transaction_type(), TransactionType::Transfer);
        assert_eq!(
            TransactionInputType::Stake(Asset::mock(), StakeType::Stake(DelegationValidator::mock())).transaction_type(),
            TransactionType::StakeDelegate
        );
        assert_eq!(
            TransactionInputType::Perpetual(Asset::mock(), PerpetualType::Open(PerpetualConfirmData::mock(PerpetualDirection::Long, 0))).transaction_type(),
            TransactionType::PerpetualOpenPosition
        );
    }
}
