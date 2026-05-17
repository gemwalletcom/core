use primitives::{SignerError, TransactionLoadInput, TransactionLoadMetadata};
use signer::Ed25519KeyPair;

use crate::address::PolkadotAddress;

const BALANCES_MODULE_INDEX: u8 = 0x0a;
const TRANSFER_ALLOW_DEATH_METHOD_INDEX: u8 = 0x00;
const SIGNATURE_TYPE_ED25519: u8 = 0x00;
const EXTRINSIC_VERSION_SIGNED: u8 = 0x84;
const MULTI_ADDRESS_ID: u8 = 0x00;
const CHARGE_ASSET_TRANSACTION_PAYMENT_NONE: u8 = 0x00;
const CHECK_METADATA_MIN_SPEC_VERSION: u32 = 1_002_005;
const MULTI_ADDRESS_MIN_SPEC_VERSION: u32 = 28;
const MORTAL_ERA_MIN_PERIOD: u64 = 4;
const MORTAL_ERA_MAX_PERIOD: u64 = 1 << 16;
const MAX_UNHASHED_PAYLOAD_SIZE: usize = 256;

pub(crate) fn fee_estimation_transaction(input: &TransactionLoadInput) -> Result<String, SignerError> {
    let fee_estimation_private_key = rand::random::<[u8; 32]>();
    fee_estimation_transaction_with_private_key(input, &fee_estimation_private_key)
}

fn fee_estimation_transaction_with_private_key(input: &TransactionLoadInput, private_key: &[u8; 32]) -> Result<String, SignerError> {
    let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
    let transaction = NativeTransferTransaction::from_input(input, key_pair.public_key_bytes)?;
    let signing_payload = transaction.signing_payload();
    let signature = key_pair.sign(signing_payload.as_ref());
    Ok(transaction.encode_hex(&signature))
}

pub(crate) struct NativeTransferTransaction {
    call: Vec<u8>,
    extra: Vec<u8>,
    signer_account_id: [u8; 32],
    uses_multi_address: bool,
    signing_payload: NativeTransferSigningPayload,
}

impl NativeTransferTransaction {
    pub(crate) fn from_input(input: &TransactionLoadInput, signer_account_id: [u8; 32]) -> Result<Self, SignerError> {
        let parameters = NativeTransferParameters::from_input(input)?;
        let destination = PolkadotAddress::parse(&input.destination_address)?;
        let value = input.value.parse::<u128>().map_err(SignerError::from_display)?;
        let uses_multi_address = parameters.uses_multi_address();

        Ok(Self {
            call: Self::encode_call(destination.account_id(), value, uses_multi_address),
            extra: parameters.encode_extra(),
            signer_account_id,
            uses_multi_address,
            signing_payload: parameters.signing_payload,
        })
    }

    pub(crate) fn encode_hex(&self, signature: &[u8; 64]) -> String {
        format!("0x{}", hex::encode(self.encode(signature)))
    }

    pub(crate) fn signing_payload(&self) -> SigningPayload {
        self.signing_payload.encode(self)
    }

    fn encode(&self, signature: &[u8; 64]) -> Vec<u8> {
        let mut body = Vec::new();
        body.push(EXTRINSIC_VERSION_SIGNED);
        encode_multi_address(&self.signer_account_id, self.uses_multi_address, &mut body);
        body.push(SIGNATURE_TYPE_ED25519);
        body.extend_from_slice(signature);
        body.extend_from_slice(&self.extra);
        body.extend_from_slice(&self.call);

        let mut encoded = Vec::new();
        encode_compact_integer(body.len() as u128, &mut encoded);
        encoded.extend_from_slice(&body);
        encoded
    }

    fn encode_call(destination: &[u8], value: u128, uses_multi_address: bool) -> Vec<u8> {
        let mut call = Vec::new();
        call.push(BALANCES_MODULE_INDEX);
        call.push(TRANSFER_ALLOW_DEATH_METHOD_INDEX);
        encode_multi_address(destination, uses_multi_address, &mut call);
        encode_compact_integer(value, &mut call);
        call
    }
}

struct NativeTransferParameters {
    sequence: u32,
    spec_version: u32,
    era: MortalEra,
    signing_payload: NativeTransferSigningPayload,
}

impl NativeTransferParameters {
    fn from_input(input: &TransactionLoadInput) -> Result<Self, SignerError> {
        let TransactionLoadMetadata::Polkadot {
            sequence,
            genesis_hash,
            block_hash,
            block_number,
            spec_version,
            transaction_version,
            period,
        } = &input.metadata
        else {
            return SignerError::invalid_input_err("missing Polkadot metadata");
        };

        let parameters = Self {
            sequence: to_u32(*sequence, "Polkadot sequence")?,
            spec_version: to_u32(*spec_version, "Polkadot spec version")?,
            era: MortalEra::new(*period, *block_number)?,
            signing_payload: NativeTransferSigningPayload::new(genesis_hash, block_hash, *spec_version, *transaction_version)?,
        };

        Ok(parameters)
    }

    fn uses_multi_address(&self) -> bool {
        self.spec_version >= MULTI_ADDRESS_MIN_SPEC_VERSION
    }

    fn encode_extra(&self) -> Vec<u8> {
        let mut extra = Vec::new();
        self.era.encode(&mut extra);
        encode_compact_integer(self.sequence.into(), &mut extra);
        encode_compact_integer(0, &mut extra);
        extra.push(CHARGE_ASSET_TRANSACTION_PAYMENT_NONE);
        if checks_metadata(self.spec_version) {
            extra.push(0);
        }
        extra
    }
}

fn checks_metadata(spec_version: u32) -> bool {
    spec_version >= CHECK_METADATA_MIN_SPEC_VERSION
}

pub(crate) enum SigningPayload {
    Raw(Vec<u8>),
    Blake2b256([u8; 32]),
}

impl SigningPayload {
    fn new(payload: Vec<u8>) -> Self {
        if payload.len() > MAX_UNHASHED_PAYLOAD_SIZE {
            Self::Blake2b256(gem_hash::blake2::blake2b_256(&payload))
        } else {
            Self::Raw(payload)
        }
    }
}

impl AsRef<[u8]> for SigningPayload {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Raw(payload) => payload,
            Self::Blake2b256(payload_hash) => payload_hash,
        }
    }
}

struct NativeTransferSigningPayload {
    spec_version: u32,
    transaction_version: u32,
    genesis_hash: [u8; 32],
    block_hash: [u8; 32],
}

impl NativeTransferSigningPayload {
    fn new(genesis_hash: &str, block_hash: &str, spec_version: u64, transaction_version: u64) -> Result<Self, SignerError> {
        Ok(Self {
            spec_version: to_u32(spec_version, "Polkadot spec version")?,
            transaction_version: to_u32(transaction_version, "Polkadot transaction version")?,
            genesis_hash: decode_hash(genesis_hash, "Polkadot genesis hash")?,
            block_hash: decode_hash(block_hash, "Polkadot block hash")?,
        })
    }

    fn encode(&self, transaction: &NativeTransferTransaction) -> SigningPayload {
        let include_metadata_hash = checks_metadata(self.spec_version);
        let mut payload = Vec::with_capacity(transaction.call.len() + transaction.extra.len() + 72 + usize::from(include_metadata_hash));
        payload.extend_from_slice(&transaction.call);
        payload.extend_from_slice(&transaction.extra);
        payload.extend_from_slice(&self.spec_version.to_le_bytes());
        payload.extend_from_slice(&self.transaction_version.to_le_bytes());
        payload.extend_from_slice(&self.genesis_hash);
        payload.extend_from_slice(&self.block_hash);
        if include_metadata_hash {
            payload.push(0);
        }

        SigningPayload::new(payload)
    }
}

#[derive(Clone, Copy)]
struct MortalEra {
    period: u64,
    phase: u64,
}

impl MortalEra {
    fn new(period: u64, block_number: u64) -> Result<Self, SignerError> {
        if period < MORTAL_ERA_MIN_PERIOD {
            return SignerError::invalid_input_err("Polkadot mortal era period must be at least 4");
        }

        let period = period
            .checked_next_power_of_two()
            .unwrap_or(MORTAL_ERA_MAX_PERIOD)
            .clamp(MORTAL_ERA_MIN_PERIOD, MORTAL_ERA_MAX_PERIOD);
        let quantize_factor = (period >> 12).max(1);
        let phase = block_number % period / quantize_factor * quantize_factor;
        Ok(Self { period, phase })
    }

    fn encode(&self, output: &mut Vec<u8>) {
        let quantize_factor = (self.period >> 12).max(1);
        let encoded = (self.period.trailing_zeros() - 1).clamp(1, 15) as u16 | ((self.phase / quantize_factor) << 4) as u16;
        output.extend_from_slice(&encoded.to_le_bytes());
    }
}

fn encode_multi_address(address: &[u8], uses_multi_address: bool, output: &mut Vec<u8>) {
    if uses_multi_address {
        output.push(MULTI_ADDRESS_ID);
    }
    output.extend_from_slice(address);
}

fn encode_compact_integer(value: u128, output: &mut Vec<u8>) {
    if value <= 0b0011_1111 {
        output.push((value as u8) << 2);
    } else if value <= 0b0011_1111_1111_1111 {
        output.extend_from_slice(&(((value as u16) << 2) | 0b01).to_le_bytes());
    } else if value <= 0b0011_1111_1111_1111_1111_1111_1111_1111 {
        output.extend_from_slice(&(((value as u32) << 2) | 0b10).to_le_bytes());
    } else {
        let bytes = value.to_le_bytes();
        let bytes_needed = bytes.iter().rposition(|byte| *byte != 0).map(|index| index + 1).unwrap_or(1);
        output.push(0b11 | (((bytes_needed - 4) as u8) << 2));
        output.extend_from_slice(&bytes[..bytes_needed]);
    }
}

fn decode_hash(value: &str, name: &'static str) -> Result<[u8; 32], SignerError> {
    let value = value.strip_prefix("0x").unwrap_or(value);
    let bytes = hex::decode(value).map_err(SignerError::from_display)?;
    bytes.try_into().map_err(|_| SignerError::invalid_input(format!("{name} must be 32 bytes")))
}

fn to_u32(value: u64, name: &'static str) -> Result<u32, SignerError> {
    value.try_into().map_err(|_| SignerError::invalid_input(format!("{name} does not fit u32")))
}

#[cfg(test)]
mod tests {
    use primitives::{Asset, Chain, GasPriceType, TransactionInputType};

    use super::*;

    const ADDRESS: &str = "15e6w4u9nH4Tb9HdJco2Zua4y5DpHb1hHXBKBGkUrLMTpuXo";

    fn input() -> TransactionLoadInput {
        TransactionLoadInput {
            input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Polkadot)),
            sender_address: ADDRESS.to_string(),
            destination_address: ADDRESS.to_string(),
            value: "10000".to_string(),
            gas_price: GasPriceType::regular(10),
            memo: None,
            is_max_value: false,
            metadata: TransactionLoadMetadata::Polkadot {
                sequence: 0,
                genesis_hash: "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3".to_string(),
                block_hash: "0x6e3ffeaa3be9d19bd110e5b6e7cbbc92cceed0d2ec557276c296bf7970ace2e5".to_string(),
                block_number: 24_666_537,
                spec_version: 1_003_004,
                transaction_version: 26,
                period: 64,
            },
        }
    }

    #[test]
    fn test_encode_compact_integer() {
        let mut encoded = Vec::new();
        encode_compact_integer(10_000, &mut encoded);
        assert_eq!(hex::encode(encoded), "419c");
    }

    #[test]
    fn test_mortal_era() {
        let mut encoded = Vec::new();
        MortalEra::new(64, 24_666_537).unwrap().encode(&mut encoded);
        assert_eq!(hex::encode(encoded), "9502");
    }

    #[test]
    fn test_mortal_era_rejects_short_period() {
        let error = match MortalEra::new(0, 24_666_537) {
            Ok(_) => panic!("expected short mortal era period to fail"),
            Err(error) => error,
        };

        assert_eq!(error.to_string(), "Invalid input: Polkadot mortal era period must be at least 4");
    }

    #[test]
    fn test_signing_payload_hashes_large_payload() {
        let payload = vec![7u8; MAX_UNHASHED_PAYLOAD_SIZE + 1];
        let expected_hash = gem_hash::blake2::blake2b_256(&payload);

        assert_eq!(SigningPayload::new(payload).as_ref(), expected_hash);
    }

    #[test]
    fn test_fee_estimation_transaction() {
        assert_eq!(
            fee_estimation_transaction_with_private_key(&input(), &[1; 32]).unwrap(),
            concat!(
                "0x39028400",
                "8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c",
                "00",
                "a98ce781ec36fa2a7c83204c8c113f6a5fe482a49f35c4457a4132bc25cd06220",
                "9ed9de9f6eb4523b1b00105c75e4f75f3c1b43490d22bef7bebb633c1a5ce0b",
                "950200000000",
                "0a0000",
                "cd3cfbbaa8f217c2a29ceae4b4063b597b629861916bad98f9826e03d1ab120e",
                "419c"
            )
        );
    }
}
