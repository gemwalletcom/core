use crate::address::{Base32Address, parse_address};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use primitives::{SignerError, SignerInput};
const ALGORAND_TX_TYPE_PAYMENT: &str = "pay";
const ALGORAND_TX_TYPE_ASSET_TRANSFER: &str = "axfer";

pub(crate) enum Operation {
    Payment { destination: Base32Address, amount: u64 },
    AssetTransfer { destination: Base32Address, amount: u64, asset_id: u64 },
    AssetOptIn { asset_id: u64 },
}

impl Operation {
    pub(crate) fn tx_type(&self) -> &'static str {
        match self {
            Self::Payment { .. } => ALGORAND_TX_TYPE_PAYMENT,
            Self::AssetTransfer { .. } | Self::AssetOptIn { .. } => ALGORAND_TX_TYPE_ASSET_TRANSFER,
        }
    }
}

pub(crate) struct AlgorandTransaction {
    pub(crate) sender: Base32Address,
    pub(crate) fee: u64,
    pub(crate) first_round: u64,
    pub(crate) last_round: u64,
    pub(crate) genesis_id: String,
    pub(crate) genesis_hash: Vec<u8>,
    pub(crate) note: Vec<u8>,
    pub(crate) operation: Operation,
}

impl AlgorandTransaction {
    pub(crate) fn transfer(input: &SignerInput) -> Result<Self, SignerError> {
        Self::from_input(
            input,
            Operation::Payment {
                destination: parse_address(&input.destination_address)?,
                amount: input.value.parse::<u64>().map_err(|_| SignerError::invalid_input("invalid Algorand amount"))?,
            },
        )
    }

    pub(crate) fn token_transfer(input: &SignerInput) -> Result<Self, SignerError> {
        let asset_id = input.get_token_id()?;

        Self::from_input(
            input,
            Operation::AssetTransfer {
                destination: parse_address(&input.destination_address)?,
                amount: input.value.parse::<u64>().map_err(|_| SignerError::invalid_input("invalid Algorand amount"))?,
                asset_id: asset_id.parse::<u64>().map_err(|_| SignerError::invalid_input("invalid Algorand asset id"))?,
            },
        )
    }

    pub(crate) fn account_action(input: &SignerInput) -> Result<Self, SignerError> {
        let asset_id = input.get_token_id()?;

        Self::from_input(
            input,
            Operation::AssetOptIn {
                asset_id: asset_id.parse::<u64>().map_err(|_| SignerError::invalid_input("invalid Algorand asset id"))?,
            },
        )
    }

    fn from_input(input: &SignerInput, operation: Operation) -> Result<Self, SignerError> {
        let fee = input.fee.fee.to_string().parse::<u64>().map_err(|_| SignerError::invalid_input("invalid Algorand fee"))?;
        let first_round = input.metadata.get_sequence().map_err(SignerError::from_display)?;

        Ok(Self {
            sender: parse_address(&input.sender_address)?,
            fee,
            first_round,
            last_round: first_round + 1000,
            genesis_id: input.metadata.get_chain_id().map_err(SignerError::from_display)?,
            genesis_hash: STANDARD
                .decode(input.metadata.get_block_hash().map_err(SignerError::from_display)?)
                .map_err(|e| SignerError::invalid_input(format!("invalid Algorand genesis hash: {e}")))?,
            note: input.memo.clone().unwrap_or_default().into_bytes(),
            operation,
        })
    }
}
