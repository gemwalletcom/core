use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use num_bigint::BigUint;
use primitives::{FeeOption, SignerError, SignerInput, WCTonSendTransaction};
use signer::Ed25519KeyPair;

use super::{
    message::{InternalMessage, build_internal_message},
    request::{JettonTransferRequest, TransferRequest},
    wallet::{WalletV4R2, build_signed_message},
};
use crate::address::Address;
use crate::signer::cells::BagOfCells;

const STATE_INIT_EXPIRE_AT: u32 = u32::MAX;
const EXTERNAL_EXPIRE_WINDOW_SECS: u64 = 600;

pub(crate) fn sign_transfer(input: &SignerInput, private_key: &[u8], expire_at: Option<u32>) -> Result<String, SignerError> {
    let request = TransferRequest::new_transfer(&input.destination_address, &input.value, input.is_max_value, input.memo.clone())?;
    sign_requests(vec![request], input.metadata.get_sequence()?, private_key, expire_at)
}

pub(crate) fn sign_token_transfer(input: &SignerInput, private_key: &[u8], expire_at: Option<u32>) -> Result<String, SignerError> {
    let sender_token_address = input
        .metadata
        .get_sender_token_address()?
        .ok_or_else(|| SignerError::invalid_input("missing sender token address"))?;

    let jetton = JettonTransferRequest {
        query_id: 0,
        value: BigUint::from_str(&input.value)?,
        destination: Address::parse(&input.destination_address)?,
        response_address: Address::parse(&input.sender_address)?,
        custom_payload: None,
        forward_ton_amount: BigUint::from(1u8),
        comment: input.memo.clone(),
    };
    let request = TransferRequest::new_jetton_transfer(&sender_token_address, token_account_creation_fee(input)?, jetton)?;
    sign_requests(vec![request], input.metadata.get_sequence()?, private_key, expire_at)
}

pub(crate) fn sign_swap(input: &SignerInput, private_key: &[u8], expire_at: Option<u32>) -> Result<Vec<String>, SignerError> {
    let swap_data = input.input_type.get_swap_data()?;
    let request = TransferRequest::new_with_payload(
        &swap_data.data.to,
        &swap_data.data.value,
        input.memo.clone(),
        BagOfCells::parse_optional_base64_root(&swap_data.data.data)?,
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
            TransferRequest::new_with_payload(
                &message.address,
                &message.amount,
                None,
                BagOfCells::parse_optional_base64_root(message.payload.as_deref().unwrap_or(""))?,
                true,
                BagOfCells::parse_optional_base64_root(message.state_init.as_deref().unwrap_or(""))?,
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
    let signature = key_pair.sign(&external_body.hash);
    let signed_body = build_signed_message(&signature, &external_body)?;
    let signed_transaction = wallet.build_transaction(sequence == 0, signed_body)?;

    BagOfCells::from_root(signed_transaction).to_base64(true)
}

fn validate_from(from: Option<&str>, sender_address: &str) -> Result<(), SignerError> {
    let Some(from) = from.filter(|from| !from.is_empty()) else {
        return Ok(());
    };

    let from = Address::parse(from)?;
    let sender = Address::parse(sender_address)?;
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

fn resolve_expire_at(sequence: u32, expire_at: Option<u32>) -> Result<u32, SignerError> {
    match (sequence, expire_at) {
        (0, _) => Ok(STATE_INIT_EXPIRE_AT),
        (_, Some(value)) => Ok(value),
        (_, None) => {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(SignerError::from_display)?.as_secs();
            u32::try_from(now + EXTERNAL_EXPIRE_WINDOW_SECS).map_err(|_| SignerError::invalid_input("TON expire time does not fit in u32"))
        }
    }
}
