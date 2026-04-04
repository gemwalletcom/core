use crate::address::Base32Address;
use crate::models::signing::{AlgorandTransaction, Operation};
use primitives::SignerError;

const PAYMENT_FIELD_COUNT: u8 = 9;
const PAYMENT_ZERO_AMOUNT_FIELD_COUNT: u8 = 8;
const ASSET_TRANSFER_FIELD_COUNT: u8 = 10;
const ASSET_OPT_IN_FIELD_COUNT: u8 = 9;
const SIGNED_TRANSACTION_FIELD_COUNT: u8 = 2;
const OPTIONAL_NOTE_FIELD_COUNT: u8 = 1;

const FIXMAP_PREFIX: u8 = 0x80;
const FIXMAP_MAX_ENTRIES: u8 = 0x10;
const FIXSTR_PREFIX: u8 = 0xa0;
const FIXSTR_MAX_LEN: usize = 0x20;
const STR8_PREFIX: u8 = 0xd9;
const STR16_PREFIX: u8 = 0xda;
const STR32_PREFIX: u8 = 0xdb;
const STR8_MAX_LEN: usize = 0x100;
const STR16_MAX_LEN: usize = 0x1_0000;
const UINT8_PREFIX: u8 = 0xcc;
const UINT16_PREFIX: u8 = 0xcd;
const UINT32_PREFIX: u8 = 0xce;
const UINT64_PREFIX: u8 = 0xcf;
const FIXUINT_MAX: u64 = 0x80;
const UINT8_MAX: u64 = 0x100;
const UINT16_MAX: u64 = 0x1_0000;
const UINT32_MAX: u64 = 0x1_0000_0000;
const BIN8_PREFIX: u8 = 0xc4;
const BIN16_PREFIX: u8 = 0xc5;
const BIN32_PREFIX: u8 = 0xc6;
const BIN8_MAX_LEN: usize = 0x100;
const BIN16_MAX_LEN: usize = 0x1_0000;

pub(crate) fn encode_transaction(transaction: &AlgorandTransaction) -> Result<Vec<u8>, SignerError> {
    let mut data = Vec::new();
    let mut size = match transaction.operation {
        Operation::Payment { amount, .. } => {
            if amount == 0 {
                PAYMENT_ZERO_AMOUNT_FIELD_COUNT
            } else {
                PAYMENT_FIELD_COUNT
            }
        }
        Operation::AssetTransfer { .. } => ASSET_TRANSFER_FIELD_COUNT,
        Operation::AssetOptIn { .. } => ASSET_OPT_IN_FIELD_COUNT,
    };
    if !transaction.note.is_empty() {
        size += OPTIONAL_NOTE_FIELD_COUNT;
    }

    encode_map_len(size, &mut data)?;

    // Algorand signed transactions use canonical msgpack, so keys must be encoded
    // in lexicographic order. Keep the write order in this function aligned with key order.
    match transaction.operation {
        Operation::Payment { amount, .. } if amount > 0 => {
            encode_string("amt", &mut data);
            encode_number(amount, &mut data);
        }
        _ => {}
    }

    match &transaction.operation {
        Operation::AssetTransfer { destination, amount, asset_id } => {
            encode_string("aamt", &mut data);
            encode_number(*amount, &mut data);
            encode_string("arcv", &mut data);
            encode_address(destination, &mut data);
            encode_common_fields(transaction, &mut data);
            encode_string("snd", &mut data);
            encode_address(&transaction.sender, &mut data);
            encode_string("type", &mut data);
            encode_string(transaction.operation.tx_type(), &mut data);
            encode_string("xaid", &mut data);
            encode_number(*asset_id, &mut data);
        }
        Operation::AssetOptIn { asset_id } => {
            encode_string("arcv", &mut data);
            encode_address(&transaction.sender, &mut data);
            encode_common_fields(transaction, &mut data);
            encode_string("snd", &mut data);
            encode_address(&transaction.sender, &mut data);
            encode_string("type", &mut data);
            encode_string(transaction.operation.tx_type(), &mut data);
            encode_string("xaid", &mut data);
            encode_number(*asset_id, &mut data);
        }
        Operation::Payment { destination, amount: _ } => {
            encode_common_fields(transaction, &mut data);
            encode_string("rcv", &mut data);
            encode_address(destination, &mut data);
            encode_string("snd", &mut data);
            encode_address(&transaction.sender, &mut data);
            encode_string("type", &mut data);
            encode_string(transaction.operation.tx_type(), &mut data);
        }
    }

    Ok(data)
}

pub(crate) fn encode_signed_transaction(transaction: &[u8], signature: &[u8]) -> Result<Vec<u8>, SignerError> {
    let mut data = Vec::new();
    encode_map_len(SIGNED_TRANSACTION_FIELD_COUNT, &mut data)?;
    encode_string("sig", &mut data);
    encode_bytes(signature, &mut data);
    encode_string("txn", &mut data);
    data.extend_from_slice(transaction);
    Ok(data)
}

fn encode_common_fields(transaction: &AlgorandTransaction, data: &mut Vec<u8>) {
    encode_string("fee", data);
    encode_number(transaction.fee, data);
    encode_string("fv", data);
    encode_number(transaction.first_round, data);
    encode_string("gen", data);
    encode_string(&transaction.genesis_id, data);
    encode_string("gh", data);
    encode_bytes(&transaction.genesis_hash, data);
    encode_string("lv", data);
    encode_number(transaction.last_round, data);
    if !transaction.note.is_empty() {
        encode_string("note", data);
        encode_bytes(&transaction.note, data);
    }
}

fn encode_address(address: &Base32Address, data: &mut Vec<u8>) {
    encode_bytes(address.payload(), data);
}

fn encode_map_len(size: u8, data: &mut Vec<u8>) -> Result<(), SignerError> {
    if size >= FIXMAP_MAX_ENTRIES {
        return Err(SignerError::invalid_input("Algorand map too large"));
    }
    data.push(FIXMAP_PREFIX + size);
    Ok(())
}

fn encode_string(value: &str, data: &mut Vec<u8>) {
    let bytes = value.as_bytes();
    match bytes.len() {
        0..FIXSTR_MAX_LEN => data.push(FIXSTR_PREFIX + bytes.len() as u8),
        FIXSTR_MAX_LEN..STR8_MAX_LEN => {
            data.push(STR8_PREFIX);
            data.push(bytes.len() as u8);
        }
        STR8_MAX_LEN..STR16_MAX_LEN => {
            data.push(STR16_PREFIX);
            data.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        }
        _ => {
            data.push(STR32_PREFIX);
            data.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        }
    }
    data.extend_from_slice(bytes);
}

fn encode_number(value: u64, data: &mut Vec<u8>) {
    match value {
        0..FIXUINT_MAX => data.push(value as u8),
        FIXUINT_MAX..UINT8_MAX => {
            data.push(UINT8_PREFIX);
            data.push(value as u8);
        }
        UINT8_MAX..UINT16_MAX => {
            data.push(UINT16_PREFIX);
            data.extend_from_slice(&(value as u16).to_be_bytes());
        }
        UINT16_MAX..UINT32_MAX => {
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
    match bytes.len() {
        0..BIN8_MAX_LEN => {
            data.push(BIN8_PREFIX);
            data.push(bytes.len() as u8);
        }
        BIN8_MAX_LEN..BIN16_MAX_LEN => {
            data.push(BIN16_PREFIX);
            data.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
        }
        _ => {
            data.push(BIN32_PREFIX);
            data.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        }
    }
    data.extend_from_slice(bytes);
}
