use crate::SignerError;
use crate::contract_call_data::ContractCallData;
use crate::earn_type::EarnType;
use crate::stake_type::StakeType;
use crate::swap::{ApprovalData, SwapData};
use crate::transaction_fee::TransactionFee;
use crate::transaction_load_metadata::TransactionLoadMetadata;
use crate::{
    Asset, GasPriceType, PerpetualType, TransactionPreloadInput, TransactionType, TransferDataExtra, WalletConnectionSessionAppMetadata, nft::NFTAsset, perpetual::AccountDataType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
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
    Earn(Asset, EarnType, ContractCallData),
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
            TransactionInputType::Earn(asset, _, _) => asset,
        }
    }

    pub fn get_swap_data(&self) -> Result<&SwapData, &'static str> {
        match self {
            TransactionInputType::Swap(_, _, swap_data) => Ok(swap_data),
            _ => Err("expected swap transaction"),
        }
    }

    pub fn get_generic_data(&self) -> Result<&TransferDataExtra, &'static str> {
        match self {
            TransactionInputType::Generic(_, _, extra) => Ok(extra),
            _ => Err("expected generic transaction"),
        }
    }

    pub fn get_approval_data(&self) -> Result<&ApprovalData, &'static str> {
        match self {
            TransactionInputType::TokenApprove(_, approval) => Ok(approval),
            _ => Err("expected token approval transaction"),
        }
    }

    pub fn get_nft_asset(&self) -> Result<&NFTAsset, &'static str> {
        match self {
            TransactionInputType::TransferNft(_, nft) => Ok(nft),
            _ => Err("expected NFT transfer transaction"),
        }
    }

    pub fn get_earn_data(&self) -> Result<&ContractCallData, &'static str> {
        match self {
            TransactionInputType::Earn(_, _, data) => Ok(data),
            _ => Err("expected earn transaction"),
        }
    }

    pub fn get_stake_type(&self) -> Result<&StakeType, &'static str> {
        match self {
            TransactionInputType::Stake(_, stake_type) => Ok(stake_type),
            _ => Err("expected stake transaction"),
        }
    }

    pub fn get_perpetual_type(&self) -> Result<&PerpetualType, &'static str> {
        match self {
            TransactionInputType::Perpetual(_, perpetual_type) => Ok(perpetual_type),
            _ => Err("expected perpetual transaction"),
        }
    }

    pub fn swap_to_address(&self) -> Option<&str> {
        match self {
            TransactionInputType::Swap(_, _, swap_data) => Some(&swap_data.data.to),
            _ => None,
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
            TransactionInputType::Earn(asset, _, _) => asset,
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
                StakeType::Unfreeze(_) => TransactionType::StakeUnfreeze,
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
            TransactionInputType::Earn(_, earn_type, _) => match earn_type {
                EarnType::Deposit(_) => TransactionType::EarnDeposit,
                EarnType::Withdraw(_) => TransactionType::EarnWithdraw,
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
        TransactionFee {
            fee: self.gas_price.total_fee(),
            gas_price_type: self.gas_price.clone(),
            gas_limit: 0.into(),
            options: HashMap::new(),
        }
    }
}

impl TransactionLoadInput {
    pub fn get_data_extra(&self) -> Result<&TransferDataExtra, &'static str> {
        self.input_type.get_generic_data()
    }

    pub fn to_preload_input(&self) -> TransactionPreloadInput {
        TransactionPreloadInput {
            input_type: self.input_type.clone(),
            sender_address: self.sender_address.clone(),
            destination_address: self.destination_address.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerInput {
    pub input: TransactionLoadInput,
    pub fee: TransactionFee,
}

impl SignerInput {
    pub fn new(input: TransactionLoadInput, fee: TransactionFee) -> Self {
        Self { input, fee }
    }

    pub fn get_token_id(&self) -> Result<&str, SignerError> {
        self.input_type.get_asset().id.get_token_id().map(|id| id.as_str())
    }

    pub fn get_sub_token_parts(&self) -> Result<(String, String), SignerError> {
        self.input_type.get_asset().id.split_sub_token_parts()
    }

    pub fn get_fee_u32(&self) -> Result<u32, SignerError> {
        self.fee
            .fee
            .to_string()
            .parse::<u32>()
            .map_err(|_| SignerError::invalid_input("invalid transaction fee"))
    }
}

impl Deref for SignerInput {
    type Target = TransactionLoadInput;

    fn deref(&self) -> &Self::Target {
        &self.input
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
    use crate::{Asset, AssetType, Chain, DelegationValidator, PerpetualConfirmData, PerpetualDirection, Resource};

    #[test]
    fn transaction_types() {
        assert_eq!(TransactionInputType::Transfer(Asset::mock()).transaction_type(), TransactionType::Transfer);
        assert_eq!(
            TransactionInputType::Stake(Asset::mock(), StakeType::Stake(DelegationValidator::mock())).transaction_type(),
            TransactionType::StakeDelegate
        );
        assert_eq!(
            TransactionInputType::Stake(Asset::mock(), StakeType::Freeze(Resource::Bandwidth)).transaction_type(),
            TransactionType::StakeFreeze
        );
        assert_eq!(
            TransactionInputType::Stake(Asset::mock(), StakeType::Unfreeze(Resource::Bandwidth)).transaction_type(),
            TransactionType::StakeUnfreeze
        );
        assert_eq!(
            TransactionInputType::Perpetual(Asset::mock(), PerpetualType::Open(PerpetualConfirmData::mock(PerpetualDirection::Long, 0, None, None))).transaction_type(),
            TransactionType::PerpetualOpenPosition
        );
    }

    #[test]
    fn transaction_input_accessors() {
        let stake_type = StakeType::Freeze(Resource::Bandwidth);
        let stake_input = TransactionInputType::Stake(Asset::mock(), stake_type);
        match stake_input.get_stake_type().unwrap() {
            StakeType::Freeze(resource) => assert_eq!(resource, &Resource::Bandwidth),
            StakeType::Stake(_) | StakeType::Unstake(_) | StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) | StakeType::Unfreeze(_) => {
                panic!("expected freeze stake type")
            }
        }

        let perpetual_type = PerpetualType::Open(PerpetualConfirmData::mock(PerpetualDirection::Long, 11, None, None));
        let perpetual_input = TransactionInputType::Perpetual(Asset::mock(), perpetual_type);
        match perpetual_input.get_perpetual_type().unwrap() {
            PerpetualType::Open(data) => assert_eq!(data.asset_index, 11),
            PerpetualType::Close(_) | PerpetualType::Modify(_) | PerpetualType::Increase(_) | PerpetualType::Reduce(_) => panic!("expected open perpetual type"),
        }

        assert_eq!(TransactionInputType::Transfer(Asset::mock()).get_stake_type().unwrap_err(), "expected stake transaction");
        assert_eq!(
            TransactionInputType::Transfer(Asset::mock()).get_perpetual_type().unwrap_err(),
            "expected perpetual transaction"
        );
    }
}
