use crate::address::{Base32Address, parse_address};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use primitives::{SignerError, SignerInput};

const TRANSACTION_VALIDITY_ROUNDS: u64 = 1000;

const ALGORAND_TX_TYPE_PAYMENT: &str = "pay";
const ALGORAND_TX_TYPE_ASSET_TRANSFER: &str = "axfer";

pub enum Operation {
    Payment { destination: Base32Address, amount: u64 },
    AssetTransfer { destination: Base32Address, amount: u64, asset_id: u64 },
    AssetOptIn { asset_id: u64 },
}

impl Operation {
    pub fn tx_type(&self) -> &'static str {
        match self {
            Self::Payment { .. } => ALGORAND_TX_TYPE_PAYMENT,
            Self::AssetTransfer { .. } | Self::AssetOptIn { .. } => ALGORAND_TX_TYPE_ASSET_TRANSFER,
        }
    }
}

pub struct AlgorandTransaction {
    pub sender: Base32Address,
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
                destination: parse_address(&input.destination_address)?,
                amount: input.get_value_u64("invalid Algorand amount")?,
            },
        )
    }

    pub fn token_transfer(input: &SignerInput) -> Result<Self, SignerError> {
        Self::from_input(
            input,
            Operation::AssetTransfer {
                destination: parse_address(&input.destination_address)?,
                amount: input.get_value_u64("invalid Algorand amount")?,
                asset_id: input.get_token_id_u64("invalid Algorand asset id")?,
            },
        )
    }

    pub fn account_action(input: &SignerInput) -> Result<Self, SignerError> {
        Self::from_input(
            input,
            Operation::AssetOptIn {
                asset_id: input.get_token_id_u64("invalid Algorand asset id")?,
            },
        )
    }

    fn from_input(input: &SignerInput, operation: Operation) -> Result<Self, SignerError> {
        let fee = input.get_fee_u64()?;
        let first_round = input.metadata.get_sequence().map_err(SignerError::from_display)?;

        Ok(Self {
            sender: parse_address(&input.sender_address)?,
            fee,
            first_round,
            last_round: first_round + TRANSACTION_VALIDITY_ROUNDS,
            genesis_id: input.metadata.get_chain_id().map_err(SignerError::from_display)?,
            genesis_hash: STANDARD
                .decode(input.metadata.get_block_hash().map_err(SignerError::from_display)?)
                .map_err(|e| SignerError::invalid_input(format!("invalid Algorand genesis hash: {e}")))?,
            note: input.memo.clone().unwrap_or_default().into_bytes(),
            operation,
        })
    }
}
