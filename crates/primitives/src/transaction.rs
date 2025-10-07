use crate::{
    AddressName, AssetAddress, Chain, TransactionSwapMetadata, asset_id::AssetId, transaction_direction::TransactionDirection,
    transaction_state::TransactionState, transaction_type::TransactionType, transaction_utxo::TransactionUtxoInput,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, vec};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable")]
pub struct TransactionsFetchOption {
    pub wallet_index: i32,
    pub asset_id: Option<String>,
    pub from_timestamp: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionsResponse {
    pub transactions: Vec<Transaction>,
    pub address_names: Vec<AddressName>,
}

impl TransactionsResponse {
    pub fn new(transactions: Vec<Transaction>, address_names: Vec<AddressName>) -> Self {
        Self { transactions, address_names }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Sendable, Equatable")]
pub struct Transaction {
    pub id: String,
    pub hash: String,
    #[serde(rename = "assetId")]
    pub asset_id: AssetId,
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract: Option<String>,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub state: TransactionState,
    #[serde(rename = "blockNumber")]
    pub block_number: String,
    pub sequence: String,
    pub fee: String,
    #[serde(rename = "feeAssetId")]
    pub fee_asset_id: AssetId,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub direction: TransactionDirection,
    #[serde(rename = "utxoInputs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utxo_inputs: Option<Vec<TransactionUtxoInput>>,
    #[serde(rename = "utxoOutputs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utxo_outputs: Option<Vec<TransactionUtxoInput>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    pub fn new(
        hash: String,
        asset_id: AssetId,
        from_address: String,
        to_address: String,
        contract: Option<String>,
        transaction_type: TransactionType,
        state: TransactionState,
        fee: String,
        fee_asset_id: AssetId,
        value: String,
        memo: Option<String>,
        metadata: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Self::id_from(asset_id.chain, hash.clone()),
            hash,
            asset_id,
            from: from_address,
            to: to_address,
            contract,
            transaction_type,
            state,
            block_number: "0".to_string(),
            sequence: "0".to_string(),
            fee,
            fee_asset_id,
            value,
            memo,
            direction: TransactionDirection::SelfTransfer,
            utxo_inputs: vec![].into(),
            utxo_outputs: vec![].into(),
            metadata,
            created_at,
        }
    }

    pub fn new_with_utxo(
        hash: String,
        asset_id: AssetId,
        transaction_type: TransactionType,
        state: TransactionState,
        fee: String,
        fee_asset_id: AssetId,
        value: String,
        memo: Option<String>,
        utxo_inputs: Option<Vec<TransactionUtxoInput>>,
        utxo_outputs: Option<Vec<TransactionUtxoInput>>,
        metadata: Option<serde_json::Value>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Self::id_from(asset_id.chain, hash.clone()),
            hash,
            asset_id,
            from: "".to_string(),
            to: "".to_string(),
            contract: None,
            transaction_type,
            state,
            block_number: "0".to_string(),
            sequence: "0".to_string(),
            fee,
            fee_asset_id,
            value,
            memo,
            direction: TransactionDirection::SelfTransfer,
            utxo_inputs: utxo_inputs.unwrap_or_default().into(),
            utxo_outputs: utxo_outputs.unwrap_or_default().into(),
            metadata,
            created_at,
        }
    }

    pub fn id_from(chain: Chain, hash: String) -> String {
        format!("{}_{}", chain.as_ref(), hash)
    }

    pub fn is_sent(&self, address: String) -> bool {
        self.input_addresses().contains(&address) || self.from == address
    }

    pub fn is_utxo_tx(&self) -> bool {
        !self.utxo_inputs.clone().unwrap_or_default().is_empty() && !self.utxo_outputs.clone().unwrap_or_default().is_empty()
    }

    pub fn input_addresses(&self) -> Vec<String> {
        self.utxo_inputs.clone().unwrap_or_default().iter().map(|x| x.address.clone()).collect()
    }

    pub fn output_addresses(&self) -> Vec<String> {
        self.utxo_outputs.clone().unwrap_or_default().iter().map(|x| x.address.clone()).collect()
    }

    pub fn addresses(&self) -> Vec<String> {
        // Append addresses from utxo inputs and outputs
        let mut array = vec![self.from.clone(), self.to.clone()];
        array.extend(self.input_addresses());
        array.extend(self.output_addresses());
        array.dedup();
        array.into_iter().filter(|x| !x.is_empty()).collect()
    }

    // addresses - is a list of user addresses
    pub fn finalize(&self, addresses: Vec<String>) -> Self {
        let chain = self.asset_id.chain;
        if !chain.is_utxo() {
            return self.clone();
        }

        let inputs_addresses = self.input_addresses();
        let outputs_addresses = self.output_addresses();

        // skip if addresses is empty or coinbase or op_return only
        if addresses.is_empty() || inputs_addresses.is_empty() || outputs_addresses.is_empty() {
            return self.clone();
        }

        // set doesn't keep order
        let user_set: HashSet<String> = HashSet::from_iter(addresses.clone());
        let input_set = HashSet::from_iter(inputs_addresses);
        let output_set = HashSet::from_iter(outputs_addresses.clone());

        // unrelated tx, return self
        if user_set.is_disjoint(&input_set) && user_set.is_disjoint(&output_set) {
            return self.clone();
        }

        let mut direction: TransactionDirection;
        if user_set.intersection(&input_set).count() > 0 {
            direction = TransactionDirection::Outgoing;
            if user_set.is_superset(&output_set) {
                direction = TransactionDirection::SelfTransfer;
            }
        } else {
            direction = TransactionDirection::Incoming;
        }

        // from is always picked from first
        let from = self.utxo_inputs.clone().unwrap_or_default().first().unwrap().address.clone();
        let to: String;
        let value: String;

        match direction {
            TransactionDirection::Incoming => {
                let addrs: Vec<String> = outputs_addresses.clone().into_iter().filter(|x| user_set.contains(x)).collect();
                to = addrs.first().unwrap().clone();
                value = Self::utxo_calculate_value(&self.utxo_outputs.clone().unwrap_or_default(), addresses).to_string();
            }
            TransactionDirection::Outgoing => {
                let filtered: Vec<String> = outputs_addresses.clone().into_iter().filter(|x| !user_set.contains(x)).collect();
                to = filtered.first().unwrap().clone();
                let vals: Vec<TransactionUtxoInput> = self
                    .utxo_outputs
                    .clone()
                    .unwrap_or_default()
                    .clone()
                    .into_iter()
                    .filter(|x| x.address == to)
                    .collect();
                value = vals.first().unwrap().value.clone();
            }
            TransactionDirection::SelfTransfer => {
                to = self.utxo_outputs.clone().unwrap_or_default().first().unwrap().address.clone();
                value = Self::utxo_calculate_value(&self.utxo_outputs.clone().unwrap_or_default(), addresses).to_string()
            }
        };
        Transaction {
            id: Self::id_from(self.asset_id.chain, self.hash.clone()),
            hash: self.hash.clone(),
            asset_id: self.asset_id.clone(),
            from,
            to,
            contract: self.contract.clone(),
            transaction_type: self.transaction_type.clone(),
            state: self.state.clone(),
            block_number: self.block_number.clone(),
            sequence: self.sequence.clone(),
            fee: self.fee.clone(),
            fee_asset_id: self.fee_asset_id.clone(),
            value: value.to_string(),
            memo: self.memo.clone(),
            direction,
            utxo_inputs: self.utxo_inputs.clone(),
            utxo_outputs: self.utxo_outputs.clone(),
            metadata: self.metadata.clone(),
            created_at: self.created_at,
        }
    }

    fn utxo_calculate_value(values: &[TransactionUtxoInput], addresses: Vec<String>) -> i64 {
        let values = values
            .to_owned()
            .clone()
            .into_iter()
            .filter(|x| addresses.contains(&x.address))
            .collect::<Vec<TransactionUtxoInput>>();

        values.clone().into_iter().map(|x| x.value.parse::<i64>().unwrap()).sum::<i64>()
    }

    pub fn asset_ids(&self) -> Vec<AssetId> {
        match self.transaction_type {
            TransactionType::Transfer
            | TransactionType::TokenApproval
            | TransactionType::StakeDelegate
            | TransactionType::StakeUndelegate
            | TransactionType::StakeRewards
            | TransactionType::StakeRedelegate
            | TransactionType::StakeWithdraw
            | TransactionType::StakeFreeze
            | TransactionType::StakeUnfreeze
            | TransactionType::AssetActivation
            | TransactionType::TransferNFT
            | TransactionType::SmartContractCall
            | TransactionType::PerpetualOpenPosition
            | TransactionType::PerpetualClosePosition => vec![self.asset_id.clone()],
            TransactionType::Swap => self
                .metadata
                .clone()
                .and_then(|metadata| serde_json::from_value::<TransactionSwapMetadata>(metadata).ok())
                .map(|metadata| vec![metadata.from_asset, metadata.to_asset])
                .unwrap_or_default(),
        }
    }

    pub fn assets_addresses(&self) -> Vec<AssetAddress> {
        match self.transaction_type {
            TransactionType::Transfer
            | TransactionType::TokenApproval
            | TransactionType::StakeDelegate
            | TransactionType::StakeUndelegate
            | TransactionType::StakeRewards
            | TransactionType::StakeRedelegate
            | TransactionType::StakeWithdraw
            | TransactionType::StakeFreeze
            | TransactionType::StakeUnfreeze
            | TransactionType::AssetActivation
            | TransactionType::TransferNFT
            | TransactionType::SmartContractCall
            | TransactionType::PerpetualOpenPosition
            | TransactionType::PerpetualClosePosition => vec![AssetAddress::new(self.asset_id.clone(), self.to.clone(), None)],
            TransactionType::Swap => self
                .metadata
                .clone()
                .and_then(|metadata| serde_json::from_value::<TransactionSwapMetadata>(metadata).ok())
                .map(|metadata| {
                    vec![
                        AssetAddress::new(metadata.from_asset.clone(), self.from.clone(), None),
                        AssetAddress::new(metadata.to_asset.clone(), self.to.clone(), None),
                    ]
                })
                .unwrap_or_default(),
        }
        .into_iter()
        .collect()
    }
}
