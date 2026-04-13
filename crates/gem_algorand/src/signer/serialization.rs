use crate::address::AlgorandAddress;
use crate::models::signing::{AlgorandTransaction, Operation};
use primitives::{Address, SignerError};

const SIGNED_TX_FIELDS: u8 = 2;

const FIXMAP_PREFIX: u8 = 0x80;
const FIXSTR_PREFIX: u8 = 0xa0;
const FIXSTR_MAX_LEN: usize = 0x20;
const STR8_PREFIX: u8 = 0xd9;
const UINT8_PREFIX: u8 = 0xcc;
const UINT16_PREFIX: u8 = 0xcd;
const UINT32_PREFIX: u8 = 0xce;
const UINT64_PREFIX: u8 = 0xcf;
const BIN8_PREFIX: u8 = 0xc4;
const BIN16_PREFIX: u8 = 0xc5;

/// Encode an unsigned Algorand transaction as canonical MessagePack (keys in lexicographic order).
pub(crate) fn encode_transaction(tx: &AlgorandTransaction) -> Result<Vec<u8>, SignerError> {
    let mut data = Vec::new();

    let mut size = tx.operation.size();
    if !tx.note.is_empty() {
        size += 1;
    }

    data.push(FIXMAP_PREFIX | size);

    if let Some(amount) = tx.operation.payment_amount() {
        encode_string("amt", &mut data);
        encode_uint(amount, &mut data);
    }

    match &tx.operation {
        Operation::Payment { destination, .. } => encode_payment(tx, destination, &mut data),
        Operation::AssetTransfer { destination, amount, asset_id } => encode_asset_transfer(tx, destination, *amount, *asset_id, &mut data),
        Operation::AssetOptIn { asset_id } => encode_asset_opt_in(tx, *asset_id, &mut data),
    }

    Ok(data)
}

fn encode_payment(tx: &AlgorandTransaction, destination: &AlgorandAddress, data: &mut Vec<u8>) {
    encode_common_fields(tx, data);
    encode_string("rcv", data);
    encode_address(destination, data);
    encode_sender_and_type(tx, data);
}

fn encode_asset_transfer(tx: &AlgorandTransaction, destination: &AlgorandAddress, amount: u64, asset_id: u64, data: &mut Vec<u8>) {
    encode_string("aamt", data);
    encode_uint(amount, data);
    encode_string("arcv", data);
    encode_address(destination, data);
    encode_common_fields(tx, data);
    encode_sender_and_type(tx, data);
    encode_string("xaid", data);
    encode_uint(asset_id, data);
}

fn encode_asset_opt_in(tx: &AlgorandTransaction, asset_id: u64, data: &mut Vec<u8>) {
    encode_string("arcv", data);
    encode_address(&tx.sender, data);
    encode_common_fields(tx, data);
    encode_sender_and_type(tx, data);
    encode_string("xaid", data);
    encode_uint(asset_id, data);
}

fn encode_sender_and_type(tx: &AlgorandTransaction, data: &mut Vec<u8>) {
    encode_string("snd", data);
    encode_address(&tx.sender, data);
    encode_string("type", data);
    encode_string(tx.operation.tx_type(), data);
}

pub(crate) fn encode_signed_transaction(encoded_tx: &[u8], signature: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();
    data.push(FIXMAP_PREFIX | SIGNED_TX_FIELDS);
    encode_string("sig", &mut data);
    encode_bytes(signature, &mut data);
    encode_string("txn", &mut data);
    data.extend_from_slice(encoded_tx);
    data
}

fn encode_common_fields(tx: &AlgorandTransaction, data: &mut Vec<u8>) {
    encode_string("fee", data);
    encode_uint(tx.fee, data);
    encode_string("fv", data);
    encode_uint(tx.first_round, data);
    encode_string("gen", data);
    encode_string(&tx.genesis_id, data);
    encode_string("gh", data);
    encode_bytes(&tx.genesis_hash, data);
    encode_string("lv", data);
    encode_uint(tx.last_round, data);
    if !tx.note.is_empty() {
        encode_string("note", data);
        encode_bytes(&tx.note, data);
    }
}

fn encode_address(address: &AlgorandAddress, data: &mut Vec<u8>) {
    encode_bytes(address.as_bytes(), data);
}

fn encode_string(value: &str, data: &mut Vec<u8>) {
    let len = value.len();
    if len < FIXSTR_MAX_LEN {
        data.push(FIXSTR_PREFIX | len as u8);
    } else {
        data.push(STR8_PREFIX);
        data.push(len as u8);
    }
    data.extend_from_slice(value.as_bytes());
}

fn encode_uint(value: u64, data: &mut Vec<u8>) {
    match value {
        0..0x80 => data.push(value as u8),
        0x80..0x100 => {
            data.push(UINT8_PREFIX);
            data.push(value as u8);
        }
        0x100..0x1_0000 => {
            data.push(UINT16_PREFIX);
            data.extend_from_slice(&(value as u16).to_be_bytes());
        }
        0x1_0000..0x1_0000_0000 => {
            data.push(UINT32_PREFIX);
            data.extend_from_slice(&(value as u32).to_be_bytes());
        }
        _ => {
            data.push(UINT64_PREFIX);
            data.extend_from_slice(&value.to_be_bytes());
        }
    }
}

fn encode_bytes(bytes: &[u8], data: &mut Vec<u8>) {
    let len = bytes.len();
    if len < 0x100 {
        data.push(BIN8_PREFIX);
        data.push(len as u8);
    } else {
        data.push(BIN16_PREFIX);
        data.extend_from_slice(&(len as u16).to_be_bytes());
    }
    data.extend_from_slice(bytes);
}
