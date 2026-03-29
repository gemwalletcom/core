use hex::encode;
use primitives::{ChainSigner, SignerError, SignerInput, TransactionInputType, TransactionLoadMetadata};
use serde_json::{Value, from_str};
use std::str::from_utf8;

use super::abi::{PANORA_ROUTER_ENTRY_PARAMS, PANORA_ROUTER_FUNCTION, PANORA_ROUTER_MODULE};
use super::{
    AccountAddress, EntryFunction, EntryFunctionPayload, build_raw_transaction, build_submit_transaction_bcs, expiration_timestamp_secs, sign_message as sign_aptos_message,
    sign_raw_transaction,
};
use crate::token_id::is_fungible_asset_token_id;
use crate::{APTOS_TRANSFER_FUNCTION, DELEGATION_POOL_ADD_STAKE_FUNCTION, DELEGATION_POOL_UNLOCK_FUNCTION, DELEGATION_POOL_WITHDRAW_FUNCTION, ENTRY_FUNCTION_PAYLOAD_TYPE};

const APTOS_CHAIN_ID: u8 = 1;
const FUNGIBLE_TRANSFER_FUNCTION: &str = "0x1::primary_fungible_store::transfer";
const OBJECT_CORE_TYPE: &str = "0x1::object::ObjectCore";

const STAKE_ENTRY_PARAMS: [&str; 2] = ["address", "u64"];
const FUNGIBLE_TRANSFER_ENTRY_PARAMS: [&str; 3] = ["address", "address", "u64"];

#[derive(Default)]
pub struct AptosChainSigner;

impl ChainSigner for AptosChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let payload = EntryFunctionPayload {
            payload_type: ENTRY_FUNCTION_PAYLOAD_TYPE.to_string(),
            function: APTOS_TRANSFER_FUNCTION.to_string(),
            type_arguments: Vec::new(),
            arguments: vec![Value::String(input.destination_address.clone()), Value::String(input.value.clone())],
        };

        let gas_limit = input.fee.gas_limit()?;
        self.sign_payload(payload, Some(&STAKE_ENTRY_PARAMS), input, private_key, gas_limit)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let gas_limit = input.fee.gas_limit()?;
        let (payload, abi) = token_transfer_payload(input)?;
        self.sign_payload(payload, Some(abi), input, private_key, gas_limit)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = match &input.input_type {
            TransactionInputType::Swap(_, _, data) => data,
            _ => return Err(SignerError::InvalidInput("Expected Aptos swap input".to_string())),
        };

        let payload: EntryFunctionPayload = from_str(swap_data.data.data.as_str())?;
        let (payload, abi) = prepare_payload(payload)?;
        let entry_function = payload.to_entry_function(abi)?;

        let signed = self.sign_entry_function(entry_function, input, private_key, input.fee.gas_limit()?)?;
        Ok(vec![signed])
    }

    fn sign_stake(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let data = match &input.metadata {
            TransactionLoadMetadata::Aptos { data: Some(data), .. } => data,
            _ => return Err(SignerError::InvalidInput("Missing Aptos stake payload".to_string())),
        };

        let payload: EntryFunctionPayload = from_str(data)?;
        let (payload, abi) = prepare_payload(payload)?;
        let entry_function = payload.to_entry_function(abi)?;

        let signed = self.sign_entry_function(entry_function, input, private_key, input.fee.gas_limit()?)?;
        Ok(vec![signed])
    }

    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        let (signature, _) = sign_aptos_message(message, private_key)?;
        Ok(format!("0x{}", encode(signature)))
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let (payload, gas_limit) = get_generic_payload(input)?;
        let (payload, abi) = prepare_payload(payload)?;
        let entry_function = payload.to_entry_function(abi)?;

        self.sign_entry_function(entry_function, input, private_key, gas_limit)
    }
}

impl AptosChainSigner {
    fn sign_payload(&self, payload: EntryFunctionPayload, abi: Option<&[&str]>, input: &SignerInput, private_key: &[u8], gas_limit: u64) -> Result<String, SignerError> {
        let entry_function = payload.to_entry_function(abi)?;
        self.sign_entry_function(entry_function, input, private_key, gas_limit)
    }

    fn sign_entry_function(&self, entry_function: EntryFunction, input: &SignerInput, private_key: &[u8], gas_limit: u64) -> Result<String, SignerError> {
        let sender = AccountAddress::from_hex(&input.sender_address)?;
        let sequence = sequence_from_metadata(&input.metadata)?;
        let gas_unit_price = input.fee.gas_price_u64()?;
        let expiration = expiration_timestamp_secs()?;

        let raw_tx = build_raw_transaction(sender, sequence, entry_function, gas_limit, gas_unit_price, expiration, APTOS_CHAIN_ID);
        let (signature, public_key) = sign_raw_transaction(&raw_tx, private_key)?;

        build_submit_transaction_bcs(raw_tx, signature, public_key)
    }
}

fn get_payload_abi(payload: &EntryFunctionPayload) -> Option<&'static [&'static str]> {
    match payload.function.as_str() {
        DELEGATION_POOL_ADD_STAKE_FUNCTION | DELEGATION_POOL_UNLOCK_FUNCTION | DELEGATION_POOL_WITHDRAW_FUNCTION => Some(&STAKE_ENTRY_PARAMS),
        _ => None,
    }
}

fn prepare_payload(payload: EntryFunctionPayload) -> Result<(EntryFunctionPayload, Option<&'static [&'static str]>), SignerError> {
    if is_panora_router(&payload.function) {
        return prepare_panora_payload(payload);
    }

    let abi = get_payload_abi(&payload);
    Ok((payload, abi))
}

fn prepare_panora_payload(mut payload: EntryFunctionPayload) -> Result<(EntryFunctionPayload, Option<&'static [&'static str]>), SignerError> {
    let expected = PANORA_ROUTER_ENTRY_PARAMS.len();
    let before_len = payload.arguments.len();
    if before_len + 1 == expected {
        payload.arguments.insert(0, Value::Null);
    }
    if payload.arguments.len() != expected {
        return Err(SignerError::InvalidInput("Aptos ABI length does not match arguments".to_string()));
    }

    Ok((payload, Some(&PANORA_ROUTER_ENTRY_PARAMS)))
}

fn get_generic_payload(input: &SignerInput) -> Result<(EntryFunctionPayload, u64), SignerError> {
    let data = match &input.input_type {
        TransactionInputType::Generic(_, _, extra) => extra.data.as_ref(),
        _ => return Err(SignerError::InvalidInput("Expected Aptos generic input".to_string())),
    };

    let json = if let Some(bytes) = data {
        if bytes.is_empty() {
            return Err(SignerError::InvalidInput("Missing Aptos payload data".to_string()));
        }
        from_utf8(bytes)
            .map_err(|_| SignerError::InvalidInput("Aptos payload must be valid UTF-8".to_string()))?
            .to_string()
    } else if let TransactionLoadMetadata::Aptos { data: Some(json), .. } = &input.metadata {
        json.clone()
    } else {
        return Err(SignerError::InvalidInput("Missing Aptos payload data".to_string()));
    };

    let payload: EntryFunctionPayload = from_str(&json)?;
    Ok((payload, input.fee.gas_limit()?))
}

fn is_panora_router(function_id: &str) -> bool {
    let mut parts = function_id.split("::");
    let _address = parts.next();
    let module = parts.next();
    let function = parts.next();

    module == Some(PANORA_ROUTER_MODULE) && function == Some(PANORA_ROUTER_FUNCTION)
}

fn sequence_from_metadata(metadata: &TransactionLoadMetadata) -> Result<u64, SignerError> {
    match metadata {
        TransactionLoadMetadata::Aptos { sequence, .. } => Ok(*sequence),
        _ => Err(SignerError::InvalidInput("Missing Aptos sequence".to_string())),
    }
}

fn token_transfer_payload(input: &SignerInput) -> Result<(EntryFunctionPayload, &'static [&'static str]), SignerError> {
    let asset = input.input_type.get_asset();
    let token_id = asset.token_id.as_ref().ok_or_else(|| SignerError::invalid_input("Missing Aptos token id"))?;
    if !is_fungible_asset_token_id(token_id) {
        return Err(SignerError::invalid_input("Invalid Aptos token ID format"));
    }

    Ok((
        EntryFunctionPayload {
            payload_type: ENTRY_FUNCTION_PAYLOAD_TYPE.to_string(),
            function: FUNGIBLE_TRANSFER_FUNCTION.to_string(),
            type_arguments: vec![OBJECT_CORE_TYPE.to_string()],
            arguments: vec![
                Value::String(token_id.to_string()),
                Value::String(input.destination_address.clone()),
                Value::String(input.value.clone()),
            ],
        },
        &FUNGIBLE_TRANSFER_ENTRY_PARAMS,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{GasPriceType, SignerInput, TransactionFee, TransactionLoadInput};

    #[test]
    fn gas_limit_uses_fee_model() {
        let input = TransactionLoadInput::mock_aptos_token_transfer("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b");
        let input = SignerInput::new(
            input,
            TransactionFee::new_gas_price_type(GasPriceType::regular(1u64), 42u64.into(), 42u64.into(), Default::default()),
        );

        assert_eq!(input.fee.gas_limit().unwrap(), 42);
    }

    #[test]
    fn token_transfer_payload_uses_fungible_asset_transfer() {
        let input = TransactionLoadInput::mock_aptos_token_transfer("0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b");
        let fee = input.default_fee();
        let input = SignerInput::new(input, fee);
        let (payload, abi) = token_transfer_payload(&input).unwrap();

        assert_eq!(payload.function, FUNGIBLE_TRANSFER_FUNCTION);
        assert_eq!(payload.type_arguments, vec![OBJECT_CORE_TYPE.to_string()]);
        assert_eq!(abi, &FUNGIBLE_TRANSFER_ENTRY_PARAMS);
    }

    #[test]
    fn token_transfer_payload_rejects_invalid_token_id() {
        let input = TransactionLoadInput::mock_aptos_token_transfer("invalid");
        let fee = input.default_fee();
        let input = SignerInput::new(input, fee);
        let err = token_transfer_payload(&input).unwrap_err();

        match err {
            SignerError::InvalidInput(message) => assert_eq!(message, "Invalid Aptos token ID format"),
            SignerError::SigningError(message) => panic!("unexpected signing error: {message}"),
        }
    }
}
