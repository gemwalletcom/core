use anyhow::Error;
use base64::{engine::general_purpose, Engine as _};
use bcs;
use sui_transaction_builder::unresolved::Input;
use sui_types::Transaction;

#[derive(Debug, PartialEq, Clone)]
pub struct Coin {
    pub coin_type: String,
    pub balance: u64,
    pub object: Object,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    pub object_id: String,
    pub digest: String,
    pub version: u64,
}

impl Object {
    pub fn to_input(&self) -> Input {
        Input::owned(self.object_id.parse().unwrap(), self.version, self.digest.parse().unwrap())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Gas {
    pub budget: u64,
    pub price: u64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StakeInput {
    pub sender: String,
    pub validator: String,
    pub stake_amount: u64,
    pub gas: Gas,
    pub coins: Vec<Coin>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnstakeInput {
    pub sender: String,
    pub staked_sui: Object,
    pub gas: Gas,
    pub gas_coin: Coin,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransferInput {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub coins: Vec<Coin>,
    pub send_max: bool,
    pub gas: Gas,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TokenTransferInput {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub tokens: Vec<Coin>,
    pub gas: Gas,
    pub gas_coin: Coin,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TxOutput {
    pub tx_data: Vec<u8>,
    pub hash: Vec<u8>,
}

impl TxOutput {
    pub fn from_tx(tx_data: &Transaction) -> Result<Self, Error> {
        let digest = tx_data.signing_digest();
        let tx_data = bcs::to_bytes(tx_data)?;
        Ok(Self {
            tx_data,
            hash: digest.to_vec(),
        })
    }

    pub fn base64_encoded(&self) -> String {
        general_purpose::STANDARD.encode(self.tx_data.clone())
    }
}
