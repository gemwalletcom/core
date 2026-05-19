use num_bigint::BigUint;
use primitives::SignerError;

use super::request::{JettonTransferRequest, NftTransferRequest, TransferPayload, TransferRequest};
use crate::{
    constants::{JETTON_TRANSFER_OPCODE, NFT_TRANSFER_OPCODE},
    tvm::{Cell, CellArc, CellBuilder},
};

pub(crate) const DEFAULT_SEND_MODE: u8 = 0b11;
pub(super) const TRANSFER_ALL_TON_MODE: u8 = DEFAULT_SEND_MODE | 0b1000_0000;

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

    builder.store_bit(true)?.store_reference(&payload)?;

    Ok(InternalMessage {
        mode: request.mode,
        message: builder.build()?,
    })
}

fn build_payload(request: &TransferRequest) -> Result<CellArc, SignerError> {
    match &request.payload {
        Some(TransferPayload::Jetton(jetton)) => build_jetton_payload(jetton),
        Some(TransferPayload::Nft(nft)) => build_nft_payload(nft),
        Some(TransferPayload::Custom(payload)) => Ok(payload.clone()),
        None => match &request.comment {
            Some(comment) => build_comment_payload(comment),
            None => Ok(CellBuilder::new().build()?.into_arc()),
        },
    }
}

fn build_comment_payload(comment: &str) -> Result<CellArc, SignerError> {
    let mut builder = CellBuilder::new();
    builder.store_u32(32, 0)?.store_string_snake(comment)?;
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

    builder.store_maybe_reference(request.custom_payload.as_ref())?;
    let forward_payload = request.comment.as_deref().map(build_comment_payload).transpose()?;
    builder.store_coins(&request.forward_ton_amount)?.store_maybe_reference(forward_payload.as_ref())?;

    Ok(builder.build()?.into_arc())
}

fn build_nft_payload(request: &NftTransferRequest) -> Result<CellArc, SignerError> {
    let mut builder = CellBuilder::new();
    builder
        .store_u32(32, NFT_TRANSFER_OPCODE)?
        .store_u64(64, request.query_id)?
        .store_address(&request.new_owner)?
        .store_address(&request.response_destination)?
        .store_maybe_reference(None)?;

    let forward_payload = request.comment.as_deref().map(build_comment_payload).transpose()?;
    builder.store_coins(&request.forward_amount)?.store_maybe_reference(forward_payload.as_ref())?;
    Ok(builder.build()?.into_arc())
}
