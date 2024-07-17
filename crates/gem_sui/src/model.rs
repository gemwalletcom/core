use anyhow::Error;
use bcs;
use blake2::{digest::consts::U32, Blake2b, Digest};
use std::str::FromStr;
use sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber},
    digests::ObjectDigest,
    transaction::TransactionData,
};
type Blake2b256 = Blake2b<U32>;

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
        (
            ObjectID::from_hex_literal(&self.object_id).unwrap(),
            SequenceNumber::from_u64(self.version),
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
    pub fn from_tx_data(tx_data: &TransactionData) -> Result<Self, Error> {
        let data = bcs::to_bytes(&tx_data)?;
        // manually build IntentMessage::new(Intent::sui_transaction(), tx_data.clone());
        let mut message = vec![0x0u8, 0x0, 0x0];
        message.append(&mut data.clone());
        let mut hasher = Blake2b256::new();
        hasher.update(&message);

        Ok(Self {
            tx_data: data,
            hash: hasher.finalize().to_vec(),
        })
    }
}
