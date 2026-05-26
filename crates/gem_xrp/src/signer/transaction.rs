use gem_hash::sha2::sha512_half;
use primitives::SignerError;

use crate::address::XrpAddress;
use crate::signer::amount::XrpAmount;

const SIGNING_PREFIX: [u8; 4] = [0x53, 0x54, 0x58, 0x00];
const TRANSACTION_TYPE_PAYMENT: u16 = 0;
const TRANSACTION_TYPE_TRUST_SET: u16 = 20;

const TYPE_UINT16: u8 = 1;
const TYPE_UINT32: u8 = 2;
const TYPE_AMOUNT: u8 = 6;
const TYPE_VL: u8 = 7;
const TYPE_ACCOUNT_ID: u8 = 8;
const TYPE_ST_OBJECT: u8 = 14;
const TYPE_ST_ARRAY: u8 = 15;

const FIELD_TRANSACTION_TYPE: u8 = 2;
const FIELD_FLAGS: u8 = 2;
const FIELD_SEQUENCE: u8 = 4;
const FIELD_DESTINATION_TAG: u8 = 14;
const FIELD_LAST_LEDGER_SEQUENCE: u8 = 27;
const FIELD_AMOUNT: u8 = 1;
const FIELD_LIMIT_AMOUNT: u8 = 3;
const FIELD_FEE: u8 = 8;
const FIELD_SIGNING_PUB_KEY: u8 = 3;
const FIELD_TXN_SIGNATURE: u8 = 4;
const FIELD_MEMO_DATA: u8 = 13;
const FIELD_ACCOUNT: u8 = 1;
const FIELD_DESTINATION: u8 = 3;
const FIELD_MEMO: u8 = 10;
const FIELD_MEMOS: u8 = 9;

const OBJECT_END_MARKER: u8 = 0xe1;
const ARRAY_END_MARKER: u8 = 0xf1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct XrpTransaction {
    sequence: u32,
    last_ledger_sequence: u32,
    fee: u64,
    signing_pub_key: Vec<u8>,
    account: XrpAddress,
    operation: XrpOperation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum XrpOperation {
    Payment {
        amount: XrpAmount,
        destination: XrpAddress,
        memo: XrpPaymentMemo,
    },
    TrustSet {
        limit_amount: XrpAmount,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum XrpPaymentMemo {
    None,
    DestinationTag(u32),
    Memo(Vec<u8>),
}

impl XrpTransaction {
    pub(crate) fn new_payment(params: XrpTransactionParams, amount: XrpAmount, destination: &str, memo: XrpPaymentMemo) -> Result<Self, SignerError> {
        Ok(Self {
            sequence: params.sequence,
            last_ledger_sequence: params.last_ledger_sequence,
            fee: params.fee,
            signing_pub_key: params.signing_pub_key,
            account: params.account,
            operation: XrpOperation::Payment {
                amount,
                destination: XrpAddress::parse(destination)?,
                memo,
            },
        })
    }

    pub(crate) fn new_trust_set(params: XrpTransactionParams, limit_amount: XrpAmount) -> Self {
        Self {
            sequence: params.sequence,
            last_ledger_sequence: params.last_ledger_sequence,
            fee: params.fee,
            signing_pub_key: params.signing_pub_key,
            account: params.account,
            operation: XrpOperation::TrustSet { limit_amount },
        }
    }

    pub(crate) fn sign(&self, private_key: &[u8]) -> Result<String, SignerError> {
        let unsigned = self.encode(None)?;
        let mut preimage = Vec::with_capacity(SIGNING_PREFIX.len() + unsigned.len());
        preimage.extend_from_slice(&SIGNING_PREFIX);
        preimage.extend_from_slice(&unsigned);
        let digest = sha512_half(&preimage);
        let mut signature = ::signer::Signer::sign_digest(::signer::SignatureScheme::Secp256k1, &digest, private_key)?;
        if signature.len() < 64 {
            return Err(SignerError::signing_error("secp256k1 signature too short"));
        }
        signature.truncate(64);
        let der_signature = der_encode_signature(&signature)?;
        Ok(hex::encode(self.encode(Some(&der_signature))?))
    }

    fn encode(&self, signature: Option<&[u8]>) -> Result<Vec<u8>, SignerError> {
        let mut buffer = Vec::new();

        append_u16(&mut buffer, FIELD_TRANSACTION_TYPE, self.transaction_type());
        append_u32(&mut buffer, FIELD_FLAGS, 0);
        append_u32(&mut buffer, FIELD_SEQUENCE, self.sequence);

        if let XrpOperation::Payment {
            memo: XrpPaymentMemo::DestinationTag(destination_tag),
            ..
        } = &self.operation
        {
            append_u32(&mut buffer, FIELD_DESTINATION_TAG, *destination_tag);
        }

        append_u32(&mut buffer, FIELD_LAST_LEDGER_SEQUENCE, self.last_ledger_sequence);

        match &self.operation {
            XrpOperation::Payment { amount, .. } => append_amount(&mut buffer, FIELD_AMOUNT, amount)?,
            XrpOperation::TrustSet { limit_amount } => append_amount(&mut buffer, FIELD_LIMIT_AMOUNT, limit_amount)?,
        }

        append_amount(&mut buffer, FIELD_FEE, &XrpAmount::Native(self.fee))?;
        append_blob(&mut buffer, FIELD_SIGNING_PUB_KEY, &self.signing_pub_key)?;

        if let Some(signature) = signature {
            append_blob(&mut buffer, FIELD_TXN_SIGNATURE, signature)?;
        }

        append_address(&mut buffer, FIELD_ACCOUNT, &self.account)?;

        if let XrpOperation::Payment { destination, memo, .. } = &self.operation {
            append_address(&mut buffer, FIELD_DESTINATION, destination)?;
            append_memo(&mut buffer, memo)?;
        }

        Ok(buffer)
    }

    fn transaction_type(&self) -> u16 {
        match &self.operation {
            XrpOperation::Payment { .. } => TRANSACTION_TYPE_PAYMENT,
            XrpOperation::TrustSet { .. } => TRANSACTION_TYPE_TRUST_SET,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct XrpTransactionParams {
    pub(crate) account: XrpAddress,
    pub(crate) fee: u64,
    pub(crate) sequence: u32,
    pub(crate) last_ledger_sequence: u32,
    pub(crate) signing_pub_key: Vec<u8>,
}

fn append_u16(buffer: &mut Vec<u8>, field: u8, value: u16) {
    append_header(buffer, TYPE_UINT16, field);
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn append_u32(buffer: &mut Vec<u8>, field: u8, value: u32) {
    append_header(buffer, TYPE_UINT32, field);
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn append_amount(buffer: &mut Vec<u8>, field: u8, amount: &XrpAmount) -> Result<(), SignerError> {
    append_header(buffer, TYPE_AMOUNT, field);
    amount.encode(buffer)
}

fn append_blob(buffer: &mut Vec<u8>, field: u8, value: &[u8]) -> Result<(), SignerError> {
    append_header(buffer, TYPE_VL, field);
    append_variable_length(buffer, value.len())?;
    buffer.extend_from_slice(value);
    Ok(())
}

fn append_address(buffer: &mut Vec<u8>, field: u8, value: &XrpAddress) -> Result<(), SignerError> {
    append_header(buffer, TYPE_ACCOUNT_ID, field);
    append_variable_length(buffer, value.as_bytes().len())?;
    buffer.extend_from_slice(value.as_bytes());
    Ok(())
}

fn append_memo(buffer: &mut Vec<u8>, memo: &XrpPaymentMemo) -> Result<(), SignerError> {
    match memo {
        XrpPaymentMemo::Memo(memo) => {
            append_header(buffer, TYPE_ST_ARRAY, FIELD_MEMOS);
            append_header(buffer, TYPE_ST_OBJECT, FIELD_MEMO);
            append_blob(buffer, FIELD_MEMO_DATA, memo)?;
            buffer.push(OBJECT_END_MARKER);
            buffer.push(ARRAY_END_MARKER);
            Ok(())
        }
        XrpPaymentMemo::None | XrpPaymentMemo::DestinationTag(_) => Ok(()),
    }
}

fn append_header(buffer: &mut Vec<u8>, type_code: u8, field_code: u8) {
    match (type_code < 16, field_code < 16) {
        (true, true) => buffer.push((type_code << 4) | field_code),
        (true, false) => {
            buffer.push(type_code << 4);
            buffer.push(field_code);
        }
        (false, true) => {
            buffer.push(field_code);
            buffer.push(type_code);
        }
        (false, false) => {
            buffer.push(0);
            buffer.push(type_code);
            buffer.push(field_code);
        }
    }
}

fn append_variable_length(buffer: &mut Vec<u8>, length: usize) -> Result<(), SignerError> {
    if length <= 192 {
        buffer.push(length as u8);
        return Ok(());
    }

    if length < 12_481 {
        let length = length - 193;
        buffer.push(((length >> 8) + 193) as u8);
        buffer.push((length & 0xff) as u8);
        return Ok(());
    }

    if length <= 918_744 {
        let length = length - 12_481;
        buffer.push(((length >> 16) + 241) as u8);
        buffer.push(((length >> 8) & 0xff) as u8);
        buffer.push((length & 0xff) as u8);
        return Ok(());
    }

    Err(SignerError::invalid_input("XRP field is too large"))
}

fn der_encode_signature(signature: &[u8]) -> Result<Vec<u8>, SignerError> {
    if signature.len() != 64 {
        return Err(SignerError::signing_error("invalid secp256k1 signature length"));
    }

    let r = der_integer(&signature[..32]);
    let s = der_integer(&signature[32..]);
    let payload_len = r.len() + s.len();
    let mut der = Vec::with_capacity(payload_len + 2);
    der.push(0x30);
    der.push(payload_len.try_into().map_err(|_| SignerError::signing_error("signature is too large"))?);
    der.extend_from_slice(&r);
    der.extend_from_slice(&s);
    Ok(der)
}

fn der_integer(value: &[u8]) -> Vec<u8> {
    let mut bytes = value.iter().skip_while(|b| **b == 0).copied().collect::<Vec<_>>();
    if bytes.is_empty() {
        bytes.push(0);
    }
    if bytes[0] & 0x80 != 0 {
        bytes.insert(0, 0);
    }

    let mut encoded = Vec::with_capacity(bytes.len() + 2);
    encoded.push(0x02);
    encoded.push(bytes.len() as u8);
    encoded.extend_from_slice(&bytes);
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_length_encoding() {
        let mut bytes = Vec::new();
        append_variable_length(&mut bytes, 180).unwrap();
        assert_eq!(hex::encode(&bytes), "b4");

        let mut bytes = Vec::new();
        append_variable_length(&mut bytes, 1000).unwrap();
        assert_eq!(hex::encode(&bytes), "c427");

        let mut bytes = Vec::new();
        append_variable_length(&mut bytes, 12_580).unwrap();
        assert_eq!(hex::encode(&bytes), "f10063");
    }
}
