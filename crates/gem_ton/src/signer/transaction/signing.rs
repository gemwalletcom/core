use std::time::{SystemTime, UNIX_EPOCH};

use num_bigint::BigUint;
use primitives::{FeeOption, SignerError, SignerInput, WCTonSendTransaction};
use signer::Ed25519KeyPair;

use super::{
    message::{DEFAULT_SEND_MODE, InternalMessage, TRANSFER_ALL_TON_MODE, build_internal_message},
    request::{JettonTransferRequest, TransferPayload, TransferRequest},
    wallet::{WalletV4R2, build_signed_message},
};
use crate::address::Address;
use crate::signer::cells::{BagOfCells, CellArc};

const STATE_INIT_EXPIRE_AT: u32 = u32::MAX;
const EXTERNAL_EXPIRE_WINDOW_SECS: u64 = 600;

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
    let request = WCTonSendTransaction::from_bytes(data)?;
    validate_from(request.r#from.as_deref(), &input.sender_address)?;

    let requests = request
        .messages
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
    sign_requests(requests, input.metadata.get_sequence()?, private_key, expire_at.or(request.valid_until))
}

pub(crate) fn sign_requests(requests: Vec<TransferRequest>, sequence: u64, private_key: &[u8], expire_at: Option<u32>) -> Result<String, SignerError> {
    let sequence = u32::try_from(sequence).map_err(|_| SignerError::invalid_input("TON sequence does not fit in u32"))?;
    let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
    let wallet = WalletV4R2::new(key_pair.public_key_bytes)?;
    let expire_at = resolve_expire_at(sequence, expire_at)?;

    let internal_messages: Vec<InternalMessage> = requests.iter().map(build_internal_message).collect::<Result<_, _>>()?;
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

fn validate_from(from: Option<&str>, sender_address: &str) -> Result<(), SignerError> {
    let Some(from) = from.filter(|from| !from.is_empty()) else {
        return Ok(());
    };

    let from = parse_address(from)?;
    let sender = parse_address(sender_address)?;
    if from != sender {
        return Err(SignerError::invalid_input("TON from does not match sender address"));
    }
    Ok(())
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
    let expire_at = now
        .checked_add(EXTERNAL_EXPIRE_WINDOW_SECS)
        .ok_or_else(|| SignerError::invalid_input("TON expire time overflow"))?;
    u32::try_from(expire_at).map_err(|_| SignerError::invalid_input("TON expire time does not fit in u32"))
}
