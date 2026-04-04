use crate::address::{Base32Address, parse_address};
use primitives::{SignerError, SignerInput};

#[derive(Clone)]
pub(crate) enum Memo {
    None,
    Text(String),
    #[cfg_attr(not(test), allow(dead_code))]
    Id(u64),
}

#[derive(Clone)]
pub(crate) enum Operation {
    CreateAccount {
        destination: Base32Address,
        amount: u64,
    },
    Payment {
        destination: Base32Address,
        asset: Option<StellarAssetData>,
        amount: u64,
    },
    ChangeTrust {
        asset: StellarAssetData,
        valid_before: Option<u64>,
    },
}

impl Operation {
    pub(crate) fn operation_type(&self) -> u32 {
        match self {
            Self::CreateAccount { .. } => 0,
            Self::Payment { .. } => 1,
            Self::ChangeTrust { .. } => 6,
        }
    }
}

#[derive(Clone)]
pub(crate) enum StellarAssetCode {
    Alphanum4([u8; 4]),
    Alphanum12([u8; 12]),
}

#[derive(Clone)]
pub(crate) struct StellarAssetData {
    pub(crate) issuer: Base32Address,
    pub(crate) code: StellarAssetCode,
}

#[derive(Clone)]
pub(crate) struct StellarTransaction {
    pub(crate) account: Base32Address,
    pub(crate) fee: u32,
    pub(crate) sequence: u64,
    pub(crate) memo: Memo,
    pub(crate) time_bounds: Option<u64>,
    pub(crate) operation: Operation,
}

impl StellarAssetData {
    pub(crate) fn new(issuer: &str, code: &str) -> Result<Self, SignerError> {
        let code = match code.len() {
            1..=4 => {
                let mut asset_code = [0u8; 4];
                asset_code[..code.len()].copy_from_slice(code.as_bytes());
                StellarAssetCode::Alphanum4(asset_code)
            }
            5..=12 => {
                let mut asset_code = [0u8; 12];
                asset_code[..code.len()].copy_from_slice(code.as_bytes());
                StellarAssetCode::Alphanum12(asset_code)
            }
            _ => return Err(SignerError::invalid_input("Stellar asset code must fit alphanum4 or alphanum12")),
        };

        Ok(Self {
            issuer: parse_address(issuer)?,
            code,
        })
    }
}

impl StellarTransaction {
    pub(crate) fn transfer(input: &SignerInput) -> Result<Self, SignerError> {
        let amount = input.get_value_u64("invalid Stellar amount")?;
        let destination = parse_address(&input.destination_address)?;
        let is_destination_address_exist = input.metadata.get_is_destination_address_exist().map_err(SignerError::from_display)?;

        let operation = if is_destination_address_exist {
            Operation::Payment { destination, asset: None, amount }
        } else {
            Operation::CreateAccount { destination, amount }
        };

        Self::from_public_input(input, input.get_fee_u32()?, operation)
    }

    pub(crate) fn token_transfer(input: &SignerInput) -> Result<Self, SignerError> {
        let asset = StellarAssetData::from_input(input)?;
        let amount = input.get_value_u64("invalid Stellar amount")?;
        let operation = Operation::Payment {
            destination: parse_address(&input.destination_address)?,
            asset: Some(asset),
            amount,
        };

        Self::from_public_input(input, input.get_fee_u32()?, operation)
    }

    pub(crate) fn account_action(input: &SignerInput) -> Result<Self, SignerError> {
        let operation = Operation::ChangeTrust {
            asset: StellarAssetData::from_input(input)?,
            valid_before: None,
        };

        Self::from_public_input(input, input.get_fee_u32()?, operation)
    }

    fn from_public_input(input: &SignerInput, fee: u32, operation: Operation) -> Result<Self, SignerError> {
        Ok(Self {
            account: parse_address(&input.sender_address)?,
            fee,
            sequence: input.metadata.get_sequence().map_err(SignerError::from_display)?,
            memo: input.memo.clone().map(Memo::Text).unwrap_or(Memo::None),
            time_bounds: None,
            operation,
        })
    }
}

impl StellarAssetData {
    fn from_input(input: &SignerInput) -> Result<Self, SignerError> {
        let (issuer, code) = input.get_sub_token_parts()?;
        Self::new(&issuer, &code)
    }
}
