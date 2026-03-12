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

fn find_field_u64(fields: &[EIP712Field], name: &str) -> Option<u64> {
    find_field_string(fields, name)?.parse().ok()
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
    let warning_expiration = batch_warning_expiration(&details, &message.message);
    let mut total_value = BigUint::ZERO;

    for detail in details {
        let raw_value = find_field_string(detail, "amount").or_else(|| find_field_string(detail, "value"))?;
        match ApprovalValue::from_raw(&raw_value)? {
            ApprovalValue::Unlimited => {
                return ApprovalRequest::permit_batch(chain, contract_address, spender_address, ApprovalValue::Unlimited, token_address, warning_expiration);
            }
            ApprovalValue::Exact(value) => {
                total_value += value;
            }
        }
    }

    ApprovalRequest::permit_batch(
        chain,
        contract_address,
        spender_address,
        ApprovalValue::Exact(total_value),
        token_address,
        warning_expiration,
    )
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

fn batch_warning_expiration(details: &[&[EIP712Field]], message_fields: &[EIP712Field]) -> Option<u64> {
    details
        .iter()
        .filter_map(|detail| find_field_u64(detail, "expiration"))
        .chain(find_field_u64(message_fields, "sigDeadline").into_iter().chain(find_field_u64(message_fields, "deadline")))
        .max()
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{U256, address};
    use alloy_sol_types::SolCall;
    use gem_evm::eip712::parse_eip712_json;
    use primitives::{Chain, SimulationPayloadFieldKind, SimulationResult, SimulationWarningType};
    use serde_json::Value;

    use super::{decode_eip712_approval, simulate_eip712_message, simulate_evm_calldata};

    fn warning(result: &SimulationResult) -> &SimulationWarningType {
        assert_eq!(result.warnings.len(), 1);
        &result.warnings[0].warning
    }

    fn warning_messages(result: &SimulationResult) -> Vec<Option<&str>> {
        result.warnings.iter().map(|warning| warning.message.as_deref()).collect()
    }

    fn is_permit_warning(warning: &SimulationWarningType) -> bool {
        match warning {
            SimulationWarningType::PermitApproval { .. } | SimulationWarningType::PermitBatchApproval { .. } => true,
            SimulationWarningType::TokenApproval { .. }
            | SimulationWarningType::SuspiciousSpender
            | SimulationWarningType::ExternallyOwnedSpender
            | SimulationWarningType::NftCollectionApproval { .. }
            | SimulationWarningType::ValidationError => false,
        }
    }

    fn is_unlimited_permit_warning(warning: &SimulationWarningType) -> bool {
        match warning {
            SimulationWarningType::PermitApproval { value: None, .. } | SimulationWarningType::PermitBatchApproval { value: None } => true,
            SimulationWarningType::PermitApproval { value: Some(_), .. }
            | SimulationWarningType::PermitBatchApproval { value: Some(_) }
            | SimulationWarningType::TokenApproval { .. }
            | SimulationWarningType::SuspiciousSpender
            | SimulationWarningType::ExternallyOwnedSpender
            | SimulationWarningType::NftCollectionApproval { .. }
            | SimulationWarningType::ValidationError => false,
        }
    }

    fn is_token_warning(warning: &SimulationWarningType) -> bool {
        match warning {
            SimulationWarningType::TokenApproval { .. } => true,
            SimulationWarningType::PermitApproval { .. }
            | SimulationWarningType::PermitBatchApproval { .. }
            | SimulationWarningType::SuspiciousSpender
            | SimulationWarningType::ExternallyOwnedSpender
            | SimulationWarningType::NftCollectionApproval { .. }
            | SimulationWarningType::ValidationError => false,
        }
    }

    fn is_nft_warning(warning: &SimulationWarningType) -> bool {
        match warning {
            SimulationWarningType::NftCollectionApproval { .. } => true,
            SimulationWarningType::TokenApproval { .. }
            | SimulationWarningType::PermitApproval { .. }
            | SimulationWarningType::PermitBatchApproval { .. }
            | SimulationWarningType::SuspiciousSpender
            | SimulationWarningType::ExternallyOwnedSpender
            | SimulationWarningType::ValidationError => false,
        }
    }

    #[test]
    fn eip712_permit_simulation_result_contains_payload_and_warnings() {
        let json: Value = serde_json::from_str(include_str!("../../../gem_evm/testdata/1inch_permit.json")).unwrap();
        let message = parse_eip712_json(&json).unwrap();
        let result = simulate_eip712_message(Chain::Ethereum, &message);

        assert!(is_permit_warning(warning(&result)));
        assert_eq!(result.payload[0].kind, SimulationPayloadFieldKind::Contract);
        assert_eq!(result.payload[1].value, "Permit");
        assert_eq!(result.payload[2].kind, SimulationPayloadFieldKind::Spender);
        assert_eq!(result.payload[3].value, "Unlimited");
        assert_eq!(result.payload[4].kind, SimulationPayloadFieldKind::Custom);
        assert_eq!(result.payload[4].label.as_deref(), Some("expiration"));
        assert_eq!(
            result.header.as_ref().map(|header| header.asset_id.clone()),
            Some("ethereum_0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into())
        );
        assert_eq!(result.header.as_ref().map(|header| header.value.as_str()), Some("Unlimited"));
    }

    #[test]
    fn eip712_permit_with_excessive_expiration_adds_warning_message() {
        let json: Value = serde_json::from_str(include_str!("../../testdata/permit_excessive_expiration.json")).unwrap();
        let message = parse_eip712_json(&json).unwrap();
        let result = simulate_eip712_message(Chain::Ethereum, &message);

        assert_eq!(result.warnings.len(), 2);
        assert!(is_permit_warning(&result.warnings[0].warning));
        assert_eq!(warning_messages(&result), vec![None, Some("Excessive expiration")]);
    }

    #[test]
    fn eip712_permit2_simulation_result_contains_payload_and_warnings() {
        let json: Value = serde_json::from_str(include_str!("../../../gem_evm/testdata/uniswap_permit2.json")).unwrap();
        let message = parse_eip712_json(&json).unwrap();
        let result = simulate_eip712_message(Chain::Ethereum, &message);

        assert!(is_unlimited_permit_warning(warning(&result)));
        assert_eq!(result.payload[0].kind, SimulationPayloadFieldKind::Contract);
        assert_eq!(result.payload[1].value, "Permit Single");
        assert_eq!(result.payload[2].kind, SimulationPayloadFieldKind::Token);
        assert_eq!(result.payload[3].kind, SimulationPayloadFieldKind::Spender);
        assert_eq!(result.payload[4].kind, SimulationPayloadFieldKind::Value);
        assert_eq!(result.payload[4].value, "Unlimited");
        assert_eq!(result.payload[5].label.as_deref(), Some("expiration"));
    }

    #[test]
    fn eip712_permit2_batch_sums_finite_values() {
        let json: Value = serde_json::from_str(include_str!("../../testdata/permit_batch_multiple_tokens.json")).unwrap();
        let message = parse_eip712_json(&json).unwrap();
        let result = simulate_eip712_message(Chain::Ethereum, &message);

        assert!(is_permit_warning(warning(&result)));
        assert_eq!(result.payload[1].value, "Permit Batch");
        assert!(result.payload.iter().all(|field| match field.kind {
            SimulationPayloadFieldKind::Token => false,
            SimulationPayloadFieldKind::Contract
            | SimulationPayloadFieldKind::Method
            | SimulationPayloadFieldKind::Spender
            | SimulationPayloadFieldKind::Value
            | SimulationPayloadFieldKind::Custom => true,
        }));
        assert_eq!(result.header, None);
    }

    #[test]
    fn eip712_permit2_batch_without_single_token_does_not_claim_warning_asset() {
        let json: Value = serde_json::from_str(include_str!("../../testdata/permit_batch_multiple_tokens.json")).unwrap();
        let message = parse_eip712_json(&json).unwrap();
        let result = simulate_eip712_message(Chain::Ethereum, &message);

        assert_eq!(
            warning(&result),
            &SimulationWarningType::PermitBatchApproval {
                value: Some(3000000000000000000_u128.into()),
            }
        );
    }

    #[test]
    fn eip712_permit2_batch_preserves_message_spender_address() {
        let json: Value = serde_json::from_str(include_str!("../../testdata/permit_batch_single_token.json")).unwrap();
        let message = parse_eip712_json(&json).unwrap();
        let approval = decode_eip712_approval(Chain::Ethereum, &message).unwrap();

        assert_eq!(approval.spender_address, "0x3333333333333333333333333333333333333333");
        assert_ne!(approval.spender_address, "0x000000000022D473030F116dDEE9F6B43aC78BA3");
    }

    #[test]
    fn eip712_permit2_batch_shows_token_when_all_items_share_token() {
        let json: Value = serde_json::from_str(include_str!("../../testdata/permit_batch_shared_token.json")).unwrap();
        let message = parse_eip712_json(&json).unwrap();
        let result = simulate_eip712_message(Chain::Ethereum, &message);

        assert_eq!(result.payload[2].kind, SimulationPayloadFieldKind::Token);
        assert_eq!(result.payload[2].value, "0x1111111111111111111111111111111111111111");
        assert_eq!(result.payload[3].kind, SimulationPayloadFieldKind::Spender);
    }

    #[test]
    fn eip712_permit2_batch_with_excessive_expiration_adds_warning_message() {
        let json: Value = serde_json::from_str(include_str!("../../testdata/permit_batch_excessive_expiration.json")).unwrap();
        let message = parse_eip712_json(&json).unwrap();
        let result = simulate_eip712_message(Chain::Ethereum, &message);

        assert_eq!(warning_messages(&result), vec![None, Some("Excessive expiration")]);
    }

    #[test]
    fn erc20_approve_simulation_result_contains_payload_and_warnings() {
        let spender = address!("1111111111111111111111111111111111111111");
        let value = U256::MAX;
        let calldata = gem_evm::contracts::IERC20::approveCall { spender, value }.abi_encode();

        let result = simulate_evm_calldata(Chain::Ethereum, &calldata, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");

        assert!(is_token_warning(warning(&result)));
        assert_eq!(result.payload[0].kind, SimulationPayloadFieldKind::Contract);
        assert_eq!(result.payload[1].value, "Approve");
        assert_eq!(result.payload[2].kind, SimulationPayloadFieldKind::Spender);
        assert_eq!(result.payload[3].kind, SimulationPayloadFieldKind::Value);
        assert_eq!(result.payload[3].value, "Unlimited");
    }

    #[test]
    fn erc20_approve_requires_matching_selector() {
        let spender = address!("1111111111111111111111111111111111111111");
        let value = U256::MAX;
        let mut calldata = gem_evm::contracts::IERC20::approveCall { spender, value }.abi_encode();
        calldata[..4].copy_from_slice(&[0_u8; 4]);

        let result = simulate_evm_calldata(Chain::Ethereum, &calldata, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");

        assert_eq!(result, SimulationResult::default());
    }

    #[test]
    fn erc721_set_approval_for_all_simulation_result_contains_payload_and_warnings() {
        let operator = address!("1111111111111111111111111111111111111111");
        let calldata = gem_evm::contracts::IERC721::setApprovalForAllCall { operator, approved: true }.abi_encode();

        let result = simulate_evm_calldata(Chain::Ethereum, &calldata, "0x57f1887a8BF19b14fC0dF6Fd9B2acc9Af147eA85");

        assert!(is_nft_warning(warning(&result)));
        assert_eq!(result.payload[0].kind, SimulationPayloadFieldKind::Contract);
        assert_eq!(result.payload[1].value, "Set Approval For All");
        assert_eq!(result.payload[2].kind, SimulationPayloadFieldKind::Spender);
    }

    #[test]
    fn erc721_set_approval_for_all_false_returns_empty_result() {
        let operator = address!("1111111111111111111111111111111111111111");
        let calldata = gem_evm::contracts::IERC721::setApprovalForAllCall { operator, approved: false }.abi_encode();

        let result = simulate_evm_calldata(Chain::Ethereum, &calldata, "0x57f1887a8BF19b14fC0dF6Fd9B2acc9Af147eA85");

        assert_eq!(result, SimulationResult::default());
    }

    #[test]
    fn erc1155_set_approval_for_all_simulation_result_contains_payload_and_warnings() {
        let operator = address!("1111111111111111111111111111111111111111");
        let calldata = gem_evm::contracts::IERC1155::setApprovalForAllCall { operator, approved: true }.abi_encode();

        let result = simulate_evm_calldata(Chain::Ethereum, &calldata, "0x495f947276749Ce646f68AC8c248420045cb7b5e");

        assert!(is_nft_warning(warning(&result)));
        assert_eq!(result.payload[0].kind, SimulationPayloadFieldKind::Contract);
        assert_eq!(result.payload[1].value, "Set Approval For All");
        assert_eq!(result.payload[2].kind, SimulationPayloadFieldKind::Spender);
    }
}
