use anyhow::Error;
use base64::{engine::general_purpose, Engine as _};
use bcs;
use std::str::FromStr;
use sui_types::{ObjectDigest, ObjectId as ObjectID, ObjectReference as ObjectRef, Transaction};

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
    pub fn to_ref(&self) -> ObjectRef {
        ObjectRef::new(
            ObjectID::from_str(&self.object_id).unwrap(),
            self.version,
            ObjectDigest::from_str(&self.digest).unwrap(),
        )
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
    pub fn from_tx_data(tx_data: &Transaction) -> Result<Self, Error> {
        let digest = tx_data.signing_digest();

        Ok(Self {
            tx_data: bcs::to_bytes(tx_data)?,
            hash: digest.to_vec(),
        })
    }

    pub fn base64_encoded(&self) -> String {
        general_purpose::STANDARD.encode(self.tx_data.clone())
    }
}
