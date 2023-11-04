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
        from: Option<String>,
        to: Option<String>,
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
            from: from.unwrap_or_default(),
            to: to.unwrap_or_default(),
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
            .map(|x| x.address.clone())
            .collect()
    }

    pub fn output_addresses(&self) -> Vec<String> {
        self.utxo_outputs
            .iter()
            .map(|x| x.address.clone())
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

    // addresses - is a list of user addresses
    pub fn finalize(&self, addresses: Vec<String>) -> Self {
        let chain = self.asset_id.chain.clone();
        if !chain.is_utxo() {
            return self.clone()
        }
        let inputs: Option<Vec<TransactionInput>> = self.utxo_inputs.clone().into();
        let outputs: Option<Vec<TransactionInput>> = self.utxo_outputs.clone().into();

        let inputs_values = inputs.clone().unwrap_or_default();
        let inputs_addresses = inputs_values.clone().into_iter().map(|x| x.address).collect::<Vec<String>>();
        let outputs_values = outputs.clone().unwrap_or_default();
        let outputs_addresses = outputs_values.clone().into_iter().map(|x| x.address).collect::<Vec<String>>();

        let direction = if !addresses.clone().into_iter().filter(|x| inputs_addresses.contains(x)).collect::<Vec<String>>().is_empty()  {
            TransactionDirection::Outgoing
        } else {
            TransactionDirection::Incoming
        };
        let from: String = match direction {
            TransactionDirection::Incoming => {
                inputs_values.first().unwrap().address.clone()
            },
            TransactionDirection::Outgoing | TransactionDirection::SelfTransfer => { 
                outputs_values.first().unwrap().address.clone()
            },
        };
        let to = match direction {
            TransactionDirection::Incoming => {
                outputs_values.first().unwrap().address.clone()
            },
            TransactionDirection::Outgoing | TransactionDirection::SelfTransfer => { 
                inputs_values.first().unwrap().address.clone()
            },
        };

        let value: i64 = match direction {
            TransactionDirection::Incoming => {
                Self::utxo_calculate_value(outputs_values.clone(), addresses)
            },
            TransactionDirection::Outgoing | TransactionDirection::SelfTransfer => { 
                Self::utxo_calculate_value(inputs_values.clone(), addresses)
            },
        };

        return Transaction { 
            id: self.id.clone(), 
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
            created_at: self.created_at
         }
    }

    fn utxo_calculate_value(values: Vec<TransactionInput>, addresses: Vec<String>) -> i64 {
        let values = values.clone().into_iter().filter(|x| 
            addresses.contains(&x.address)
        ).collect::<Vec<TransactionInput>>();
        
        return values.clone().into_iter().map(|x| x.value.parse::<i64>().unwrap()).sum::<i64>();
    }


}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct TransactionsFetchOption {
    pub wallet_index: i32,
    pub asset_id: Option<String>,
    pub from_timestamp: Option<u32>,
}
