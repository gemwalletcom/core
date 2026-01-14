use hex::encode;
use num_traits::ToPrimitive;
use primitives::{ChainSigner, SignerError, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata};
use serde_json::{Value, from_str};
use std::str::from_utf8;

use super::{
    AccountAddress, EntryFunction, EntryFunctionPayload, build_raw_transaction, build_submit_transaction, expiration_timestamp_secs,
    sign_message as sign_aptos_message, sign_raw_transaction,
};
use super::abi::{PANORA_ROUTER_ENTRY_PARAMS, PANORA_ROUTER_FUNCTION, PANORA_ROUTER_MODULE};

const APTOS_CHAIN_ID: u8 = 1;
const DEFAULT_MAX_GAS_AMOUNT: u64 = 1500;
const APTOS_TRANSFER_FUNCTION: &str = "0x1::aptos_account::transfer";
const APTOS_TRANSFER_COINS_FUNCTION: &str = "0x1::aptos_account::transfer_coins";
const FUNGIBLE_TRANSFER_FUNCTION: &str = "0x1::primary_fungible_store::transfer";
const OBJECT_CORE_TYPE: &str = "0x1::object::ObjectCore";

const DELEGATION_POOL_ADD_STAKE: &str = "0x1::delegation_pool::add_stake";
const DELEGATION_POOL_UNLOCK: &str = "0x1::delegation_pool::unlock";
const DELEGATION_POOL_WITHDRAW: &str = "0x1::delegation_pool::withdraw";

const STAKE_ENTRY_PARAMS: [&str; 2] = ["address", "u64"];

#[derive(Default)]
pub struct AptosChainSigner;

impl ChainSigner for AptosChainSigner {
    fn sign_transfer(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        let payload = EntryFunctionPayload {
            payload_type: "entry_function_payload".to_string(),
            function: APTOS_TRANSFER_FUNCTION.to_string(),
            type_arguments: Vec::new(),
            arguments: vec![
                Value::String(input.destination_address.clone()),
                Value::String(input.value.clone()),
            ],
        };

        self.sign_payload(payload, Some(&STAKE_ENTRY_PARAMS), input, private_key, DEFAULT_MAX_GAS_AMOUNT)
    }

    fn sign_token_transfer(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        let asset = input.input_type.get_asset();
        let token_id = asset
            .token_id
            .as_ref()
            .ok_or_else(|| SignerError::InvalidInput("Missing Aptos token id".to_string()))?;

        if token_id.contains("::") {
            let payload = EntryFunctionPayload {
                payload_type: "entry_function_payload".to_string(),
                function: APTOS_TRANSFER_COINS_FUNCTION.to_string(),
                type_arguments: vec![token_id.to_string()],
                arguments: vec![
                    Value::String(input.destination_address.clone()),
                    Value::String(input.value.clone()),
                ],
            };
            return self.sign_payload(payload, Some(&STAKE_ENTRY_PARAMS), input, private_key, DEFAULT_MAX_GAS_AMOUNT);
        }

        let payload = EntryFunctionPayload {
            payload_type: "entry_function_payload".to_string(),
            function: FUNGIBLE_TRANSFER_FUNCTION.to_string(),
            type_arguments: vec![OBJECT_CORE_TYPE.to_string()],
            arguments: vec![
                Value::String(token_id.to_string()),
                Value::String(input.destination_address.clone()),
                Value::String(input.value.clone()),
            ],
        };

        self.sign_payload(
            payload,
            Some(&["address", "address", "u64"]),
            input,
            private_key,
            DEFAULT_MAX_GAS_AMOUNT,
        )
    }

    fn sign_swap(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = match &input.input_type {
            TransactionInputType::Swap(_, _, data) => data,
            _ => return Err(SignerError::InvalidInput("Expected Aptos swap input".to_string())),
        };

        let payload_str = swap_data.data.data.as_str();
        let payload: EntryFunctionPayload = from_str(payload_str)?;
        let abi = resolve_payload_abi(&payload);
        let entry_function = payload.to_entry_function(abi)?;
        let max_gas_amount = resolve_max_gas_amount(input);

        let signed = self.sign_entry_function(payload, entry_function, input, private_key, max_gas_amount)?;
        Ok(vec![signed])
    }

    fn sign_stake(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let payload_str = match &input.metadata {
            TransactionLoadMetadata::Aptos { data: Some(data), .. } => data,
            _ => return Err(SignerError::InvalidInput("Missing Aptos stake payload".to_string())),
        };

        let payload: EntryFunctionPayload = from_str(payload_str)?;
        let abi = resolve_payload_abi(&payload);
        let entry_function = payload.to_entry_function(abi)?;

        let signed = self.sign_entry_function(payload, entry_function, input, private_key, DEFAULT_MAX_GAS_AMOUNT)?;
        Ok(vec![signed])
    }

    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        let (signature, _) = sign_aptos_message(message, private_key)?;
        Ok(format!("0x{}", encode(signature)))
    }

    fn sign_data(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        let (payload, max_gas_amount) = resolve_generic_payload(input)?;
        let abi = resolve_payload_abi(&payload);
        let entry_function = payload.to_entry_function(abi)?;

        self.sign_entry_function(payload, entry_function, input, private_key, max_gas_amount)
    }
}

impl AptosChainSigner {
    fn sign_payload(
        &self,
        payload: EntryFunctionPayload,
        abi: Option<&[&str]>,
        input: &TransactionLoadInput,
        private_key: &[u8],
        max_gas_amount: u64,
    ) -> Result<String, SignerError> {
        let entry_function = payload.to_entry_function(abi)?;
        self.sign_entry_function(payload, entry_function, input, private_key, max_gas_amount)
    }

    fn sign_entry_function(
        &self,
        payload: EntryFunctionPayload,
        entry_function: EntryFunction,
        input: &TransactionLoadInput,
        private_key: &[u8],
        max_gas_amount: u64,
    ) -> Result<String, SignerError> {
        let sender = AccountAddress::from_hex(&input.sender_address)?;
        let sequence = sequence_from_metadata(&input.metadata)?;
        let gas_unit_price = gas_unit_price(input)?;
        let expiration = expiration_timestamp_secs()?;

        let raw_tx = build_raw_transaction(
            sender,
            sequence,
            entry_function,
            max_gas_amount,
            gas_unit_price,
            expiration,
            APTOS_CHAIN_ID,
        );
        let (signature, public_key) = sign_raw_transaction(&raw_tx, private_key)?;

        build_submit_transaction(raw_tx, &payload, signature, public_key)
    }
}

fn resolve_payload_abi(payload: &EntryFunctionPayload) -> Option<&'static [&'static str]> {
    if is_panora_router(&payload.function) {
        return Some(&PANORA_ROUTER_ENTRY_PARAMS);
    }

    match payload.function.as_str() {
        DELEGATION_POOL_ADD_STAKE | DELEGATION_POOL_UNLOCK | DELEGATION_POOL_WITHDRAW => Some(&STAKE_ENTRY_PARAMS),
        _ => None,
    }
}

fn resolve_generic_payload(input: &TransactionLoadInput) -> Result<(EntryFunctionPayload, u64), SignerError> {
    let (data, gas_limit) = match &input.input_type {
        TransactionInputType::Generic(_, _, extra) => (extra.data.as_ref(), extra.gas_limit.as_ref()),
        _ => return Err(SignerError::InvalidInput("Expected Aptos generic input".to_string())),
    };

    let payload_str = if let Some(bytes) = data {
        if bytes.is_empty() {
            return Err(SignerError::InvalidInput("Missing Aptos payload data".to_string()));
        }
        from_utf8(bytes).map_err(|_| SignerError::InvalidInput("Aptos payload must be valid UTF-8".to_string()))?.to_string()
    } else if let TransactionLoadMetadata::Aptos { data: Some(payload), .. } = &input.metadata {
        payload.clone()
    } else {
        return Err(SignerError::InvalidInput("Missing Aptos payload data".to_string()));
    };

    let max_gas_amount = match gas_limit {
        Some(limit) => limit
            .to_u64()
            .ok_or_else(|| SignerError::InvalidInput("Invalid Aptos gas limit".to_string()))?,
        None => DEFAULT_MAX_GAS_AMOUNT,
    };

    let payload: EntryFunctionPayload = from_str(&payload_str)?;
    Ok((payload, max_gas_amount))
}

fn is_panora_router(function_id: &str) -> bool {
    let mut parts = function_id.split("::");
    let _address = parts.next();
    let module = parts.next();
    let function = parts.next();

    matches!(module, Some(PANORA_ROUTER_MODULE)) && matches!(function, Some(PANORA_ROUTER_FUNCTION))
}

fn sequence_from_metadata(metadata: &TransactionLoadMetadata) -> Result<u64, SignerError> {
    match metadata {
        TransactionLoadMetadata::Aptos { sequence, .. } => Ok(*sequence),
        _ => Err(SignerError::InvalidInput("Missing Aptos sequence".to_string())),
    }
}

fn gas_unit_price(input: &TransactionLoadInput) -> Result<u64, SignerError> {
    let gas_price = input.gas_price.gas_price();
    gas_price
        .to_string()
        .parse::<u64>()
        .map_err(|_| SignerError::InvalidInput("Invalid Aptos gas price".to_string()))
}

fn resolve_max_gas_amount(input: &TransactionLoadInput) -> u64 {
    if let TransactionInputType::Swap(_, _, swap_data) = &input.input_type
        && let Some(limit) = &swap_data.data.gas_limit
            && let Ok(parsed) = limit.parse::<u64>() {
                return parsed;
            }
    DEFAULT_MAX_GAS_AMOUNT
}
