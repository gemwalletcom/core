use alloy_sol_types::SolCall;
use num_bigint::BigUint;
use primitives::{Chain, SimulationResult};

use super::{approval_method::ApprovalMethod, approval_request::ApprovalRequest, approval_value::ApprovalValue};
use gem_evm::{
    contracts::{IERC20, IERC721, IERC1155},
    eip712::{EIP712Field, EIP712Message, EIP712TypedValue},
    ethereum_address_checksum,
};

pub fn simulate_eip712_message(chain: Chain, message: &EIP712Message) -> SimulationResult {
    match decode_eip712_approval(chain, message) {
        Some(approval) => approval.simulate(),
        None => SimulationResult::default(),
    }
}

pub fn simulate_evm_calldata(chain: Chain, calldata: &[u8], contract_address: &str) -> SimulationResult {
    match decode_evm_approval(chain, calldata, contract_address) {
        Some(approval) => approval.simulate(),
        None => SimulationResult::default(),
    }
}

pub(crate) fn decode_eip712_approval(chain: Chain, message: &EIP712Message) -> Option<ApprovalRequest> {
    let contract_address = message.domain.verifying_contract.clone()?;
    let method = ApprovalMethod::from_eip712(message)?;

    match method {
        ApprovalMethod::Permit => decode_permit_approval(chain, message, contract_address, method),
        ApprovalMethod::PermitSingle | ApprovalMethod::PermitBatch => decode_permit2_approval(chain, message, contract_address, method),
        ApprovalMethod::Approve | ApprovalMethod::SetApprovalForAll => None,
    }
}

pub(crate) fn decode_evm_approval(chain: Chain, calldata: &[u8], contract_address: &str) -> Option<ApprovalRequest> {
    if calldata.len() < 4 {
        return None;
    }

    if calldata.starts_with(&<IERC20::approveCall as SolCall>::SELECTOR)
        && let Ok(call) = <IERC20::approveCall as SolCall>::abi_decode(calldata)
    {
        return ApprovalRequest::erc20(chain, contract_address, format!("{:#x}", call.spender), call.value.to_string());
    }

    if calldata.starts_with(&<IERC721::setApprovalForAllCall as SolCall>::SELECTOR)
        && let Ok(call) = <IERC721::setApprovalForAllCall as SolCall>::abi_decode(calldata)
        && call.approved
    {
        return ApprovalRequest::nft_collection(chain, contract_address, format!("{:#x}", call.operator));
    }

    if calldata.starts_with(&<IERC1155::setApprovalForAllCall as SolCall>::SELECTOR)
        && let Ok(call) = <IERC1155::setApprovalForAllCall as SolCall>::abi_decode(calldata)
        && call.approved
    {
        return ApprovalRequest::nft_collection(chain, contract_address, format!("{:#x}", call.operator));
    }

    None
}

fn find_field_string(fields: &[EIP712Field], name: &str) -> Option<String> {
    fields.iter().find(|field| field.name == name).and_then(|field| match &field.value {
        EIP712TypedValue::Address { value } | EIP712TypedValue::Uint256 { value } | EIP712TypedValue::String { value } => Some(value.clone()),
        EIP712TypedValue::Struct { .. } | EIP712TypedValue::Int256 { .. } | EIP712TypedValue::Bool { .. } | EIP712TypedValue::Bytes { .. } | EIP712TypedValue::Array { .. } => None,
    })
}

fn find_field_struct<'a>(fields: &'a [EIP712Field], name: &str) -> Option<&'a [EIP712Field]> {
    fields.iter().find(|field| field.name == name).and_then(|field| match &field.value {
        EIP712TypedValue::Struct { fields } => Some(fields.as_slice()),
        EIP712TypedValue::Address { .. }
        | EIP712TypedValue::Uint256 { .. }
        | EIP712TypedValue::Int256 { .. }
        | EIP712TypedValue::String { .. }
        | EIP712TypedValue::Bool { .. }
        | EIP712TypedValue::Bytes { .. }
        | EIP712TypedValue::Array { .. } => None,
    })
}

fn find_field_struct_array<'a>(fields: &'a [EIP712Field], name: &str) -> Option<Vec<&'a [EIP712Field]>> {
    fields.iter().find(|field| field.name == name).and_then(|field| match &field.value {
        EIP712TypedValue::Array { items } => Some(
            items
                .iter()
                .filter_map(|item| match item {
                    EIP712TypedValue::Struct { fields } => Some(fields.as_slice()),
                    EIP712TypedValue::Address { .. }
                    | EIP712TypedValue::Uint256 { .. }
                    | EIP712TypedValue::Int256 { .. }
                    | EIP712TypedValue::String { .. }
                    | EIP712TypedValue::Bool { .. }
                    | EIP712TypedValue::Bytes { .. }
                    | EIP712TypedValue::Array { .. } => None,
                })
                .collect(),
        ),
        EIP712TypedValue::Address { .. }
        | EIP712TypedValue::Uint256 { .. }
        | EIP712TypedValue::Struct { .. }
        | EIP712TypedValue::Int256 { .. }
        | EIP712TypedValue::String { .. }
        | EIP712TypedValue::Bool { .. }
        | EIP712TypedValue::Bytes { .. } => None,
    })
}

fn decode_permit_approval(chain: Chain, message: &EIP712Message, contract_address: String, method: ApprovalMethod) -> Option<ApprovalRequest> {
    ApprovalRequest::permit(
        chain,
        contract_address,
        find_field_string(&message.message, "spender")?,
        find_field_string(&message.message, "value")?,
        find_field_string(&message.message, "deadline"),
        None,
        method,
    )
}

fn decode_permit2_approval(chain: Chain, message: &EIP712Message, contract_address: String, method: ApprovalMethod) -> Option<ApprovalRequest> {
    if method == ApprovalMethod::PermitBatch {
        return decode_permit2_batch_approval(chain, message, contract_address);
    }

    let details = find_permit_details(&message.message)?;
    let approval_value = find_field_string(details, "amount").or_else(|| find_field_string(details, "value"))?;
    let expiration = find_field_string(details, "expiration")
        .or_else(|| find_field_string(&message.message, "sigDeadline"))
        .or_else(|| find_field_string(&message.message, "deadline"));

    ApprovalRequest::permit(
        chain,
        contract_address,
        find_field_string(&message.message, "spender")?,
        approval_value,
        expiration,
        find_field_string(details, "token"),
        method,
    )
}

fn decode_permit2_batch_approval(chain: Chain, message: &EIP712Message, contract_address: String) -> Option<ApprovalRequest> {
    let details = find_field_struct_array(&message.message, "details")?;
    if details.is_empty() {
        return None;
    }

    let spender_address = find_field_string(&message.message, "spender")?;
    let token_address = single_token_address(&details);
    let mut total_value = BigUint::ZERO;

    for detail in details {
        let raw_value = find_field_string(detail, "amount").or_else(|| find_field_string(detail, "value"))?;
        match ApprovalValue::from_raw(&raw_value)? {
            ApprovalValue::Unlimited => {
                return ApprovalRequest::permit_batch(chain, contract_address, spender_address, ApprovalValue::Unlimited, token_address);
            }
            ApprovalValue::Exact(value) => {
                total_value += value;
            }
        }
    }

    ApprovalRequest::permit_batch(chain, contract_address, spender_address, ApprovalValue::Exact(total_value), token_address)
}

fn find_permit_details(fields: &[EIP712Field]) -> Option<&[EIP712Field]> {
    if let Some(details) = find_field_struct(fields, "details") {
        return Some(details);
    }

    if let Some(details) = find_field_struct(fields, "permitted") {
        return Some(details);
    }

    fields.iter().find_map(|field| match &field.value {
        EIP712TypedValue::Struct { fields } => {
            let has_value = find_field_string(fields, "amount").is_some() || find_field_string(fields, "value").is_some();
            if has_value {
                return Some(fields.as_slice());
            }
            None
        }
        EIP712TypedValue::Address { .. }
        | EIP712TypedValue::Uint256 { .. }
        | EIP712TypedValue::Int256 { .. }
        | EIP712TypedValue::String { .. }
        | EIP712TypedValue::Bool { .. }
        | EIP712TypedValue::Bytes { .. }
        | EIP712TypedValue::Array { .. } => None,
    })
}

fn single_token_address(details: &[&[EIP712Field]]) -> Option<String> {
    let mut token_address: Option<String> = None;

    for detail in details {
        let next_token_address = find_field_string(detail, "token")?;
        let next_token_address = ethereum_address_checksum(&next_token_address).ok()?;

        match token_address.as_ref() {
            Some(current_token_address) if current_token_address != &next_token_address => return None,
            Some(_) => {}
            None => token_address = Some(next_token_address),
        }
    }

    token_address
}
