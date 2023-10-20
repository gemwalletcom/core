use crate::{
    asset_id::AssetId, transaction_direction::TransactionDirection,
    transaction_state::TransactionState, transaction_type::TransactionType, Chain,
    transaction_utxo::TransactionInput,
};
use chrono::offset::Utc;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct Transaction {
    pub id: String,
    pub hash: String,
    #[serde(rename = "assetId")]
    pub asset_id: AssetId,
    pub from: String,
    pub to: String,
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
    pub memo: Option<String>,
    pub direction: TransactionDirection,

    #[serde(rename = "utxoInputs")]
    pub utxo_inputs: Vec<TransactionInput>,
    #[serde(rename = "utxoOutputs")]
    pub utxo_outputs: Vec<TransactionInput>,

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
        block_number: String,
        sequence: String,
        fee: String,
        fee_asset_id: AssetId,
        value: String,
        memo: Option<String>,
        direction: TransactionDirection,
        created_at: DateTime<Utc>,
    ) -> Self {
        let id = Self::id_from(asset_id.clone().chain, hash.clone());
        Self {
            id,
            hash,
            asset_id,
            from: from_address,
            to: to_address,
            contract,
            transaction_type,
            state,
            block_number,
            sequence,
            fee,
            fee_asset_id,
            value,
            memo,
            direction,
            utxo_inputs: vec![],
            utxo_outputs: vec![],
            created_at,
        }
    }

    pub fn new_with_utxo(
        hash: String,
        asset_id: AssetId,
        from: String,
        to: String,
        contract: Option<String>,
        transaction_type: TransactionType,
        state: TransactionState,
        block_number: String,
        sequence: String,
        fee: String,
        fee_asset_id: AssetId,
        value: String,
        memo: Option<String>,
        direction: TransactionDirection,
        utxo_inputs: Vec<TransactionInput>,
        utxo_outputs: Vec<TransactionInput>,
        created_at: DateTime<Utc>,
    ) -> Self {
        let id = Self::id_from(asset_id.clone().chain, hash.clone());
        Self {
            id,
            hash,
            asset_id,
            from,
            to,
            contract,
            transaction_type,
            state,
            block_number,
            sequence,
            fee,
            fee_asset_id,
            value,
            memo,
            direction,
            utxo_inputs,
            utxo_outputs,
            created_at,
        }
    }

    pub fn id_from(chain: Chain, hash: String) -> String {
        format!("{}_{}", chain.as_str(), hash)
    }

    pub fn is_utxo_tx(&self) -> bool {
        self.utxo_inputs.len() > 0 && self.utxo_outputs.len() > 0
    }

    pub fn input_addresses(&self) -> Vec<String> {
        self.utxo_inputs
            .iter()
            .map(|x| x.addresses.clone())
            .flatten()
            .collect()
    }

    pub fn output_addresses(&self) -> Vec<String> {
        self.utxo_outputs
            .iter()
            .map(|x| x.addresses.clone())
            .flatten()
            .collect()
    }

    pub fn addresses(&self) -> Vec<String> {
        // Append addresses from utxo inputs and outputs
        let mut array = vec![self.from.clone(), self.to.clone()];
        array.extend(self.input_addresses());
        array.extend(self.output_addresses());
        array.dedup();
        array
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct TransactionsFetchOption {
    pub wallet_index: i32,
    pub asset_id: Option<String>,
    pub from_timestamp: Option<u32>,
}
