use num_bigint::BigUint;

use super::super::cells::CellArc;
use crate::address::Address;

pub(crate) struct TransferRequest {
    pub destination: Address,
    pub value: BigUint,
    pub mode: u8,
    pub bounceable: bool,
    pub comment: Option<String>,
    pub payload: Option<TransferPayload>,
    pub state_init: Option<CellArc>,
}

pub(crate) enum TransferPayload {
    Jetton(JettonTransferRequest),
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
