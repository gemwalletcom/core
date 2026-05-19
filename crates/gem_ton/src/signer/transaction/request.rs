use std::str::FromStr;

use num_bigint::BigUint;
use primitives::SignerError;

use super::message::{DEFAULT_SEND_MODE, TRANSFER_ALL_TON_MODE};
use crate::{address::Address, tvm::CellArc};

pub(crate) struct TransferRequest {
    pub destination: Address,
    pub value: BigUint,
    pub mode: u8,
    pub bounceable: bool,
    pub comment: Option<String>,
    pub payload: Option<TransferPayload>,
    pub state_init: Option<CellArc>,
}

impl TransferRequest {
    pub(crate) fn new_transfer(destination: &str, value: &str, is_max: bool, comment: Option<String>) -> Result<Self, SignerError> {
        Ok(Self {
            destination: Address::parse(destination)?,
            value: BigUint::from_str(value)?,
            mode: if is_max { TRANSFER_ALL_TON_MODE } else { DEFAULT_SEND_MODE },
            bounceable: false,
            comment,
            payload: None,
            state_init: None,
        })
    }

    pub(crate) fn new_contract_transfer(destination: &str, attached_amount: BigUint, payload: TransferPayload) -> Result<Self, SignerError> {
        Ok(Self {
            destination: Address::parse(destination)?,
            value: attached_amount,
            mode: DEFAULT_SEND_MODE,
            bounceable: true,
            comment: None,
            payload: Some(payload),
            state_init: None,
        })
    }

    pub(crate) fn new_with_payload(
        destination: &str,
        amount: &str,
        comment: Option<String>,
        payload: Option<CellArc>,
        bounceable: bool,
        state_init: Option<CellArc>,
    ) -> Result<Self, SignerError> {
        Ok(Self {
            destination: Address::parse(destination)?,
            value: BigUint::from_str(amount)?,
            mode: DEFAULT_SEND_MODE,
            bounceable,
            comment,
            payload: payload.map(TransferPayload::Custom),
            state_init,
        })
    }
}

pub(crate) enum TransferPayload {
    Jetton(JettonTransferRequest),
    Nft(NftTransferRequest),
    Custom(CellArc),
}

pub(crate) struct JettonTransferRequest {
    pub query_id: u64,
    pub value: BigUint,
    pub destination: Address,
    pub response_address: Address,
    pub custom_payload: Option<CellArc>,
    pub forward_ton_amount: BigUint,
    pub comment: Option<String>,
}

pub(crate) struct NftTransferRequest {
    pub query_id: u64,
    pub new_owner: Address,
    pub response_destination: Address,
    pub forward_amount: BigUint,
    pub comment: Option<String>,
}
