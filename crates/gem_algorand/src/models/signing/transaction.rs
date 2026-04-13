use super::Operation;
use crate::address::AlgorandAddress;
use gem_encoding::decode_base64;
use num_traits::ToPrimitive;
use primitives::{Address, SignerError, SignerInput};
use signer::InvalidInput;

const TRANSACTION_VALIDITY_ROUNDS: u64 = 1000;

pub struct AlgorandTransaction {
    pub sender: AlgorandAddress,
    pub fee: u64,
    pub first_round: u64,
    pub last_round: u64,
    pub genesis_id: String,
    pub genesis_hash: Vec<u8>,
    pub note: Vec<u8>,
    pub operation: Operation,
}

impl AlgorandTransaction {
    pub fn transfer(input: &SignerInput) -> Result<Self, SignerError> {
        Self::from_input(
            input,
            Operation::Payment {
                destination: AlgorandAddress::from_str(&input.destination_address).invalid_input("invalid Algorand address")?,
                amount: input.value.parse::<u64>().invalid_input("invalid Algorand amount")?,
            },
        )
    }

    pub fn token_transfer(input: &SignerInput) -> Result<Self, SignerError> {
        Self::from_input(
            input,
            Operation::AssetTransfer {
                destination: AlgorandAddress::from_str(&input.destination_address).invalid_input("invalid Algorand address")?,
                amount: input.value.parse::<u64>().invalid_input("invalid Algorand amount")?,
                asset_id: get_asset_id(input)?,
            },
        )
    }

    pub fn account_action(input: &SignerInput) -> Result<Self, SignerError> {
        Self::from_input(input, Operation::AssetOptIn { asset_id: get_asset_id(input)? })
    }

    fn from_input(input: &SignerInput, operation: Operation) -> Result<Self, SignerError> {
        let fee = input.fee.fee.to_u64().invalid_input("invalid transaction fee")?;
        let first_round = input.metadata.get_sequence().map_err(SignerError::from_display)?;

        Ok(Self {
            sender: AlgorandAddress::from_str(&input.sender_address).invalid_input("invalid Algorand address")?,
            fee,
            first_round,
            last_round: first_round + TRANSACTION_VALIDITY_ROUNDS,
            genesis_id: input.metadata.get_chain_id().map_err(SignerError::from_display)?,
            genesis_hash: decode_base64(&input.metadata.get_block_hash().map_err(SignerError::from_display)?).invalid_input("invalid Algorand genesis hash")?,
            note: input.memo.clone().unwrap_or_default().into_bytes(),
            operation,
        })
    }
}

fn get_asset_id(input: &SignerInput) -> Result<u64, SignerError> {
    input.input_type.get_asset().id.get_token_id()?.parse::<u64>().invalid_input("invalid Algorand asset id")
}
