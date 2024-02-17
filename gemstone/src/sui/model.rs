use bcs;
use blake2::{digest::consts::U32, Blake2b, Digest};
use std::str::FromStr;
type Blake2b256 = Blake2b<U32>;
use sui_types::{
    base_types::{ObjectID, ObjectRef, SequenceNumber},
    digests::ObjectDigest,
    transaction::TransactionData,
};

#[derive(uniffi::Record, Clone)]
pub struct SuiCoin {
    pub coin_type: String,
    pub balance: u64,
    pub object_ref: SuiObjectRef,
}

#[derive(uniffi::Record, Clone)]
pub struct SuiObjectRef {
    pub object_id: String,
    pub digest: String,
    pub version: u64,
}

impl SuiObjectRef {
    pub fn to_tuple(&self) -> ObjectRef {
        (
            ObjectID::from_hex_literal(&self.object_id).unwrap(),
            SequenceNumber::from_u64(self.version),
            ObjectDigest::from_str(&self.digest).unwrap(),
        )
    }
}

#[derive(uniffi::Record)]
pub struct SuiGas {
    pub budget: u64,
    pub price: u64,
}

#[derive(uniffi::Record)]
pub struct SuiStakeInput {
    pub sender: String,
    pub validator: String,
    pub stake_amount: u64,
    pub gas: SuiGas,
    pub coins: Vec<SuiCoin>,
}

#[derive(uniffi::Record)]
pub struct SuiUnstakeInput {
    pub sender: String,
    pub staked_sui: SuiObjectRef,
    pub gas: SuiGas,
    pub gas_coin: SuiCoin,
}

#[derive(uniffi::Record)]
pub struct SuiTransferInput {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub coins: Vec<SuiCoin>,
    pub send_max: bool,
    pub gas: SuiGas,
}

#[derive(uniffi::Record)]
pub struct SuiTokenTransferInput {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub tokens: Vec<SuiCoin>,
    pub gas: SuiGas,
    pub gas_coin: SuiCoin,
}

#[derive(uniffi::Record)]
pub struct SuiTxOutput {
    pub tx_data: Vec<u8>,
    pub hash: Vec<u8>,
}

impl SuiTxOutput {
    pub fn from_tx_data(tx_data: &TransactionData) -> Result<Self, anyhow::Error> {
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
