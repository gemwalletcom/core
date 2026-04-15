use num_bigint::BigUint;
use primitives::SignerError;

use super::request::{JettonTransferRequest, TransferPayload, TransferRequest};
use crate::signer::cells::{Cell, CellArc, CellBuilder};

pub(super) const DEFAULT_SEND_MODE: u8 = 0b11;
pub(super) const TRANSFER_ALL_TON_MODE: u8 = DEFAULT_SEND_MODE | 0b1000_0000;
const JETTON_TRANSFER_OPCODE: u32 = 0x0f8a7ea5;

pub(super) struct InternalMessage {
    pub mode: u8,
    pub message: Cell,
}

pub(super) fn build_internal_message(request: &TransferRequest) -> Result<InternalMessage, SignerError> {
    let payload = build_payload(request)?;
    let zero = BigUint::from(0u8);

    let mut builder = CellBuilder::new();
    builder
        // int_msg_info$0 ihr_disabled:Bool bounce:Bool bounced:Bool
        .store_bit(false)?
        .store_bit(true)?
        .store_bit(request.bounceable)?
        .store_bit(false)?
        // src (addr_none) + dest
        .store_null_address()?
        .store_address(&request.destination)?
        // value, currency_collection (empty extra), ihr_fee, fwd_fee, created_lt, created_at
        .store_coins(&request.value)?
        .store_bit(false)?
        .store_coins(&zero)?
        .store_coins(&zero)?
        .store_u64(64, 0)?
        .store_u32(32, 0)?;

    match &request.state_init {
        Some(state_init) => {
            builder.store_bit(true)?.store_bit(true)?.store_reference(state_init)?;
        }
        None => {
            builder.store_bit(false)?;
        }
    }

    // body stored in its own cell reference
    builder.store_bit(true)?.store_reference(&payload)?;

    Ok(InternalMessage {
        mode: request.mode,
        message: builder.build()?,
    })
}

fn build_payload(request: &TransferRequest) -> Result<CellArc, SignerError> {
    match (&request.payload, &request.comment) {
        (Some(TransferPayload::Jetton(jetton)), _) => build_jetton_payload(jetton),
        (Some(TransferPayload::Custom(payload)), _) => Ok(payload.clone()),
        (None, Some(comment)) => build_comment_payload(comment),
        (None, None) => Ok(CellBuilder::new().build()?.into_arc()),
    }
}

fn build_comment_payload(comment: &str) -> Result<CellArc, SignerError> {
    let mut builder = CellBuilder::new();
    builder.store_u32(32, 0)?.store_string(comment)?;
    Ok(builder.build()?.into_arc())
}

fn build_jetton_payload(request: &JettonTransferRequest) -> Result<CellArc, SignerError> {
    let mut builder = CellBuilder::new();
    builder
        .store_u32(32, JETTON_TRANSFER_OPCODE)?
        .store_u64(64, request.query_id)?
        .store_coins(&request.value)?
        .store_address(&request.destination)?
        .store_address(&request.response_address)?;

    match &request.custom_payload {
        Some(custom_payload) => {
            builder.store_bit(true)?.store_reference(custom_payload)?;
        }
        None => {
            builder.store_bit(false)?;
        }
    }

    builder.store_coins(&request.forward_ton_amount)?.store_bit(false)?;

    if let Some(comment) = &request.comment {
        builder.store_u32(32, 0)?.store_string(comment)?;
    }
    Ok(builder.build()?.into_arc())
}
