use std::time::{SystemTime, UNIX_EPOCH};

use num_bigint::BigUint;
use primitives::{FeeOption, SignerError, SignerInput, WCTonMessage};
use signer::Ed25519KeyPair;

use super::request::{JettonTransferRequest, TransferPayload, TransferRequest};
use crate::address::Address;
use crate::signer::cells::{BagOfCells, Cell, CellArc, CellBuilder};

const BASE_WORKCHAIN: i32 = 0;
const DEFAULT_WALLET_ID: i32 = 0x29a9a317;
pub(super) const DEFAULT_SEND_MODE: u8 = 0b11;
const TRANSFER_ALL_TON_MODE: u8 = DEFAULT_SEND_MODE | 0b1000_0000;
const JETTON_TRANSFER_OPCODE: u32 = 0x0f8a7ea5;
const STATE_INIT_EXPIRE_AT: u32 = u32::MAX;
const EXTERNAL_EXPIRE_WINDOW_SECS: u64 = 600;
const WALLET_V4R2_CODE_BOC: &str = include_str!("wallet_v4r2_code.boc.b64");

pub(crate) fn sign_transfer(input: &SignerInput, private_key: &[u8], expire_at: Option<u32>) -> Result<String, SignerError> {
    let request = TransferRequest {
        destination: parse_address(&input.destination_address)?,
        value: parse_biguint(&input.value)?,
        mode: if input.is_max_value { TRANSFER_ALL_TON_MODE } else { DEFAULT_SEND_MODE },
        bounceable: false,
        comment: input.memo.clone(),
        payload: None,
        state_init: None,
    };
    sign_requests(vec![request], input.metadata.get_sequence()?, private_key, expire_at)
}

pub(crate) fn sign_token_transfer(input: &SignerInput, private_key: &[u8], expire_at: Option<u32>) -> Result<String, SignerError> {
    let sender_token_address = input
        .metadata
        .get_sender_token_address()?
        .ok_or_else(|| SignerError::invalid_input("missing sender token address"))?;

    let request = TransferRequest {
        destination: parse_address(&sender_token_address)?,
        value: token_account_creation_fee(input)?,
        mode: DEFAULT_SEND_MODE,
        bounceable: true,
        comment: input.memo.clone(),
        payload: Some(TransferPayload::Jetton(JettonTransferRequest {
            query_id: 0,
            value: parse_biguint(&input.value)?,
            destination: parse_address(&input.destination_address)?,
            response_address: parse_address(&input.sender_address)?,
            custom_payload: None,
            forward_ton_amount: BigUint::from(1u8),
            comment: input.memo.clone(),
        })),
        state_init: None,
    };
    sign_requests(vec![request], input.metadata.get_sequence()?, private_key, expire_at)
}

pub(crate) fn sign_swap(input: &SignerInput, private_key: &[u8], expire_at: Option<u32>) -> Result<Vec<String>, SignerError> {
    let swap_data = input.input_type.get_swap_data()?;
    if swap_data.data.approval.is_some() {
        return Err(SignerError::invalid_input("TON swap approvals are not supported"));
    }

    let request = build_custom_payload_transfer(
        &swap_data.data.to,
        &swap_data.data.value,
        input.memo.clone(),
        parse_optional_boc_root(&swap_data.data.data)?,
        true,
        None,
    )?;
    Ok(vec![sign_requests(vec![request], input.metadata.get_sequence()?, private_key, expire_at)?])
}

pub(crate) fn sign_data(input: &SignerInput, private_key: &[u8], expire_at: Option<u32>) -> Result<String, SignerError> {
    let extra = input.input_type.get_generic_data()?;
    let data = extra.data.as_ref().ok_or_else(|| SignerError::invalid_input("missing TON messages"))?;
    let messages: Vec<WCTonMessage> = serde_json::from_slice(data)?;

    let requests = messages
        .into_iter()
        .map(|message| {
            build_custom_payload_transfer(
                &message.address,
                &message.amount,
                None,
                parse_optional_boc_root(message.payload.as_deref().unwrap_or(""))?,
                true,
                parse_optional_boc_root(message.state_init.as_deref().unwrap_or(""))?,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    sign_requests(requests, input.metadata.get_sequence()?, private_key, expire_at)
}

pub(crate) fn sign_requests(requests: Vec<TransferRequest>, sequence: u64, private_key: &[u8], expire_at: Option<u32>) -> Result<String, SignerError> {
    let sequence = u32::try_from(sequence).map_err(|_| SignerError::invalid_input("TON sequence does not fit in u32"))?;
    let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
    let wallet = WalletV4R2::new(key_pair.public_key_bytes)?;
    let expire_at = resolve_expire_at(sequence, expire_at)?;

    let internal_messages = requests.iter().map(build_internal_message).collect::<Result<Vec<_>, _>>()?;
    let external_body = wallet.build_external_body(expire_at, sequence, &internal_messages)?;
    let signature = key_pair.sign(&external_body.cell_hash());
    let signed_body = build_signed_message(&signature, &external_body)?;
    let signed_transaction = wallet.build_transaction(sequence == 0, signed_body)?;

    BagOfCells::from_root(signed_transaction).to_base64(true)
}

pub(super) fn parse_address(value: &str) -> Result<Address, SignerError> {
    Address::from_base64_url(value)
        .or_else(|_| Address::from_hex_str(value))
        .map_err(|error| SignerError::invalid_input(error.to_string()))
}

fn parse_biguint(value: &str) -> Result<BigUint, SignerError> {
    BigUint::parse_bytes(value.as_bytes(), 10).ok_or_else(|| SignerError::invalid_input("invalid TON amount"))
}

fn parse_boc_root(value: &str) -> Result<CellArc, SignerError> {
    Ok(BagOfCells::parse_base64(value)?.single_root()?.clone())
}

fn parse_optional_boc_root(value: &str) -> Result<Option<CellArc>, SignerError> {
    if value.is_empty() { Ok(None) } else { parse_boc_root(value).map(Some) }
}

fn token_account_creation_fee(input: &SignerInput) -> Result<BigUint, SignerError> {
    let Some(value) = input.fee.options.get(&FeeOption::TokenAccountCreation) else {
        return Ok(BigUint::ZERO);
    };
    value.to_biguint().ok_or_else(|| SignerError::invalid_input("invalid TON amount"))
}

fn build_custom_payload_transfer(
    destination: &str,
    amount: &str,
    comment: Option<String>,
    payload: Option<CellArc>,
    bounceable: bool,
    state_init: Option<CellArc>,
) -> Result<TransferRequest, SignerError> {
    Ok(TransferRequest {
        destination: parse_address(destination)?,
        value: parse_biguint(amount)?,
        mode: DEFAULT_SEND_MODE,
        bounceable,
        comment,
        payload: payload.map(TransferPayload::Custom),
        state_init,
    })
}

fn resolve_expire_at(sequence: u32, expire_at: Option<u32>) -> Result<u32, SignerError> {
    if sequence == 0 {
        return Ok(STATE_INIT_EXPIRE_AT);
    }
    if let Some(expire_at) = expire_at {
        return Ok(expire_at);
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(SignerError::from_display)?.as_secs();
    let expire_at = now.checked_add(EXTERNAL_EXPIRE_WINDOW_SECS).ok_or_else(|| SignerError::invalid_input("TON expire time overflow"))?;
    u32::try_from(expire_at).map_err(|_| SignerError::invalid_input("TON expire time does not fit in u32"))
}

fn build_internal_message(request: &TransferRequest) -> Result<InternalMessage, SignerError> {
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

struct InternalMessage {
    mode: u8,
    message: Cell,
}

#[derive(Clone)]
struct StateInit {
    code: Option<CellArc>,
    data: Option<CellArc>,
}

impl StateInit {
    fn to_cell(&self) -> Result<Cell, SignerError> {
        let mut builder = CellBuilder::new();
        builder
            .store_bit(false)?
            .store_bit(false)?
            .store_bit(self.code.is_some())?
            .store_bit(self.data.is_some())?
            .store_bit(false)?;
        if let Some(code) = &self.code {
            builder.store_reference(code)?;
        }
        if let Some(data) = &self.data {
            builder.store_reference(data)?;
        }
        builder.build()
    }
}

pub(super) struct WalletV4R2 {
    public_key: [u8; 32],
    address: Address,
}

impl WalletV4R2 {
    pub(super) fn new(public_key: [u8; 32]) -> Result<Self, SignerError> {
        let state_init = wallet_state_init(&public_key)?;
        Ok(Self {
            public_key,
            address: Address::new(BASE_WORKCHAIN, state_init.to_cell()?.cell_hash()),
        })
    }

    #[cfg(test)]
    pub(super) fn address(&self) -> &Address {
        &self.address
    }

    fn build_external_body(&self, expire_at: u32, sequence: u32, messages: &[InternalMessage]) -> Result<Cell, SignerError> {
        let mut builder = CellBuilder::new();
        builder
            .store_i32(32, DEFAULT_WALLET_ID)?
            .store_u32(32, expire_at)?
            .store_u32(32, sequence)?
            .store_u8(8, 0)?;
        for message in messages {
            builder.store_u8(8, message.mode)?.store_child(message.message.clone())?;
        }
        builder.build()
    }

    fn build_transaction(&self, include_state_init: bool, signed_body: Cell) -> Result<Cell, SignerError> {
        let mut builder = CellBuilder::new();
        builder
            .store_u8(2, 0b10)?
            .store_null_address()?
            .store_address(&self.address)?
            .store_coins(&BigUint::from(0u8))?;

        if include_state_init {
            builder.store_bit(true)?.store_bit(true)?.store_child(wallet_state_init(&self.public_key)?.to_cell()?)?;
        } else {
            builder.store_bit(false)?;
        }
        builder.store_bit(true)?.store_child(signed_body)?;
        builder.build()
    }
}

fn wallet_state_init(public_key: &[u8; 32]) -> Result<StateInit, SignerError> {
    let mut data = CellBuilder::new();
    data.store_u32(32, 0)?.store_i32(32, DEFAULT_WALLET_ID)?.store_slice(public_key)?.store_bit(false)?;

    Ok(StateInit {
        code: Some(parse_boc_root(WALLET_V4R2_CODE_BOC)?),
        data: Some(data.build()?.into_arc()),
    })
}

fn build_signed_message(signature: &[u8; 64], external_body: &Cell) -> Result<Cell, SignerError> {
    let mut builder = CellBuilder::new();
    builder.store_slice(signature)?.store_cell(external_body)?;
    builder.build()
}
