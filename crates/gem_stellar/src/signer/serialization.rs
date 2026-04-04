use crate::address::Base32Address;
use crate::models::signing::{Memo, Operation, StellarAssetCode, StellarAssetData, StellarTransaction};

const ASSET_TYPE_NATIVE: u32 = 0;
const ASSET_TYPE_ALPHANUM4: u32 = 1;
const ASSET_TYPE_ALPHANUM12: u32 = 2;
const MEMO_NONE: u32 = 0;
const MEMO_TEXT: u32 = 1;
const MEMO_ID: u32 = 2;

pub(crate) fn encode_transaction(transaction: &StellarTransaction) -> Vec<u8> {
    let mut data = Vec::new();
    encode_address(&mut data, &transaction.account);
    write_u32_be(&mut data, transaction.fee);
    write_u64_be(&mut data, transaction.sequence);
    encode_time_bounds(&mut data, transaction);
    encode_memo(&mut data, &transaction.memo);
    write_u32_be(&mut data, 1);
    write_u32_be(&mut data, 0);
    write_u32_be(&mut data, transaction.operation.operation_type());

    match &transaction.operation {
        Operation::CreateAccount { destination, amount } => {
            encode_address(&mut data, destination);
            write_u64_be(&mut data, *amount);
        }
        Operation::Payment { destination, asset, amount } => {
            encode_address(&mut data, destination);
            encode_asset(&mut data, asset.as_ref());
            write_u64_be(&mut data, *amount);
        }
        Operation::ChangeTrust { asset, .. } => {
            encode_asset(&mut data, Some(asset));
            write_u64_be(&mut data, i64::MAX as u64);
        }
    }

    write_u32_be(&mut data, 0);
    data
}

fn encode_time_bounds(data: &mut Vec<u8>, transaction: &StellarTransaction) {
    let valid_before = match &transaction.operation {
        Operation::ChangeTrust { valid_before, .. } => *valid_before,
        Operation::CreateAccount { .. } | Operation::Payment { .. } => transaction.time_bounds,
    };

    if let Some(to) = valid_before.filter(|value| *value > 0) {
        write_u32_be(data, 1);
        write_u64_be(data, 0);
        write_u64_be(data, to);
    } else {
        write_u32_be(data, 0);
    }
}

fn encode_memo(data: &mut Vec<u8>, memo: &Memo) {
    match memo {
        Memo::None => write_u32_be(data, MEMO_NONE),
        Memo::Text(text) => {
            write_u32_be(data, MEMO_TEXT);
            write_u32_be(data, text.len() as u32);
            data.extend_from_slice(text.as_bytes());
            pad_to_four_bytes(data);
        }
        Memo::Id(id) => {
            write_u32_be(data, MEMO_ID);
            write_u64_be(data, *id);
        }
    }
}

fn encode_asset(data: &mut Vec<u8>, asset: Option<&StellarAssetData>) {
    match asset {
        Some(asset) => {
            match &asset.code {
                StellarAssetCode::Alphanum4(code) => {
                    write_u32_be(data, ASSET_TYPE_ALPHANUM4);
                    data.extend_from_slice(code);
                }
                StellarAssetCode::Alphanum12(code) => {
                    write_u32_be(data, ASSET_TYPE_ALPHANUM12);
                    data.extend_from_slice(code);
                }
            }
            encode_address(data, &asset.issuer);
        }
        None => write_u32_be(data, ASSET_TYPE_NATIVE),
    }
}

fn encode_address(data: &mut Vec<u8>, address: &Base32Address) {
    data.extend_from_slice(&0u32.to_be_bytes());
    data.extend_from_slice(address.payload());
}

fn write_u32_be(data: &mut Vec<u8>, value: u32) {
    data.extend_from_slice(&value.to_be_bytes());
}

fn write_u64_be(data: &mut Vec<u8>, value: u64) {
    data.extend_from_slice(&value.to_be_bytes());
}

fn pad_to_four_bytes(data: &mut Vec<u8>) {
    let padding = (4 - (data.len() % 4)) % 4;
    data.extend(std::iter::repeat_n(0, padding));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::parse_address;

    #[test]
    fn encode_asset_supports_alphanum4_and_alphanum12() {
        let issuer = parse_address("GA6HCMBLTZS5VYYBCATRBRZ3BZJMAFUDKYYF6AH6MVCMGWMRDNSWJPIH").unwrap();
        let alphanum4 = StellarAssetData::new("GA6HCMBLTZS5VYYBCATRBRZ3BZJMAFUDKYYF6AH6MVCMGWMRDNSWJPIH", "MOBI").unwrap();
        let alphanum12 = StellarAssetData::new("GA6HCMBLTZS5VYYBCATRBRZ3BZJMAFUDKYYF6AH6MVCMGWMRDNSWJPIH", "USDC12345678").unwrap();

        let mut native_bytes = Vec::new();
        encode_asset(&mut native_bytes, None);
        assert_eq!(native_bytes, ASSET_TYPE_NATIVE.to_be_bytes());

        let mut alphanum4_bytes = Vec::new();
        encode_asset(&mut alphanum4_bytes, Some(&alphanum4));
        assert_eq!(&alphanum4_bytes[..4], &ASSET_TYPE_ALPHANUM4.to_be_bytes());
        assert_eq!(&alphanum4_bytes[4..8], b"MOBI");
        assert_eq!(&alphanum4_bytes[8..12], &[0, 0, 0, 0]);
        assert_eq!(&alphanum4_bytes[12..44], issuer.payload());

        let mut alphanum12_bytes = Vec::new();
        encode_asset(&mut alphanum12_bytes, Some(&alphanum12));
        assert_eq!(&alphanum12_bytes[..4], &ASSET_TYPE_ALPHANUM12.to_be_bytes());
        assert_eq!(&alphanum12_bytes[4..16], b"USDC12345678");
        assert_eq!(&alphanum12_bytes[16..20], &[0, 0, 0, 0]);
        assert_eq!(&alphanum12_bytes[20..52], issuer.payload());
    }
}
