use super::asset::StellarAssetData;
use super::operation::{Memo, Operation};
use crate::address::StellarAddress;
use num_traits::ToPrimitive;
use primitives::{Address, SignerError, SignerInput};
use signer::InvalidInput;

const MEMO_TEXT_MAX_BYTES: usize = 28;

#[derive(Clone)]
pub struct StellarTransaction {
    pub account: StellarAddress,
    pub fee: u32,
    pub sequence: u64,
    pub memo: Memo,
    pub time_bounds: Option<u64>,
    pub operation: Operation,
}

impl StellarTransaction {
    pub fn transfer(input: &SignerInput) -> Result<Self, SignerError> {
        let amount = input.value.parse::<u64>().invalid_input("invalid Stellar amount")?;
        let destination = StellarAddress::from_str(&input.destination_address).invalid_input("invalid Stellar address")?;
        let is_destination_exist = input.metadata.get_is_destination_address_exist().map_err(SignerError::from_display)?;

        let operation = if is_destination_exist {
            Operation::Payment { destination, asset: None, amount }
        } else {
            Operation::CreateAccount { destination, amount }
        };

        Self::build(input, fee_u32(input)?, operation)
    }

    pub fn token_transfer(input: &SignerInput) -> Result<Self, SignerError> {
        if !input.metadata.get_is_destination_address_exist().map_err(SignerError::from_display)? {
            return SignerError::invalid_input_err("Stellar destination account not found for token transfer");
        }

        let amount = input.value.parse::<u64>().invalid_input("invalid Stellar amount")?;
        let operation = Operation::Payment {
            destination: StellarAddress::from_str(&input.destination_address).invalid_input("invalid Stellar address")?,
            asset: Some(StellarAssetData::from_input(input)?),
            amount,
        };

        Self::build(input, fee_u32(input)?, operation)
    }

    pub fn account_action(input: &SignerInput) -> Result<Self, SignerError> {
        let operation = Operation::ChangeTrust {
            asset: StellarAssetData::from_input(input)?,
            valid_before: None,
        };

        Self::build(input, fee_u32(input)?, operation)
    }

    fn build(input: &SignerInput, fee: u32, operation: Operation) -> Result<Self, SignerError> {
        Ok(Self {
            account: StellarAddress::from_str(&input.sender_address).invalid_input("invalid Stellar address")?,
            fee,
            sequence: input.metadata.get_sequence().map_err(SignerError::from_display)?,
            memo: memo(input.memo.as_deref())?,
            time_bounds: None,
            operation,
        })
    }
}

fn fee_u32(input: &SignerInput) -> Result<u32, SignerError> {
    input.fee.fee.to_u32().invalid_input("invalid transaction fee")
}

fn memo(value: Option<&str>) -> Result<Memo, SignerError> {
    match value {
        Some(text) if text.len() > MEMO_TEXT_MAX_BYTES => SignerError::invalid_input_err("Stellar memo text must be at most 28 bytes"),
        Some(text) => Ok(Memo::Text(text.to_string())),
        None => Ok(Memo::None),
    }
}
