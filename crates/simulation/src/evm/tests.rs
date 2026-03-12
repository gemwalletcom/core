use alloy_primitives::{U256, address};
use alloy_sol_types::SolCall;
use gem_evm::eip712::parse_eip712_json;
use primitives::{Chain, SimulationPayloadFieldKind, SimulationResult, SimulationWarningType};
use serde_json::Value;

use super::{simulate_eip712_message, simulate_evm_calldata};

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
    let json: Value = serde_json::json!({
        "types": {
            "EIP712Domain": [
                { "name": "name", "type": "string" },
                { "name": "version", "type": "string" },
                { "name": "chainId", "type": "uint256" },
                { "name": "verifyingContract", "type": "address" }
            ],
            "Permit": [
                { "name": "owner", "type": "address" },
                { "name": "spender", "type": "address" },
                { "name": "value", "type": "uint256" },
                { "name": "nonce", "type": "uint256" },
                { "name": "deadline", "type": "uint256" }
            ]
        },
        "primaryType": "Permit",
        "domain": {
            "name": "USD Coin",
            "version": "2",
            "chainId": "1",
            "verifyingContract": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        },
        "message": {
            "owner": "0x1111111111111111111111111111111111111111",
            "spender": "0x2222222222222222222222222222222222222222",
            "value": "1000",
            "nonce": "0",
            "deadline": "9999999999"
        }
    });
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
    let json: Value = serde_json::json!({
        "types": {
            "EIP712Domain": [
                { "name": "name", "type": "string" },
                { "name": "chainId", "type": "uint256" },
                { "name": "verifyingContract", "type": "address" }
            ],
            "PermitBatch": [
                { "name": "details", "type": "PermitDetails[]" },
                { "name": "spender", "type": "address" },
                { "name": "sigDeadline", "type": "uint256" }
            ],
            "PermitDetails": [
                { "name": "token", "type": "address" },
                { "name": "amount", "type": "uint160" },
                { "name": "expiration", "type": "uint48" },
                { "name": "nonce", "type": "uint48" }
            ]
        },
        "primaryType": "PermitBatch",
        "domain": {
            "name": "Permit2",
            "chainId": "1",
            "verifyingContract": "0x000000000022D473030F116dDEE9F6B43aC78BA3"
        },
        "message": {
            "details": [
                {
                    "token": "0x1111111111111111111111111111111111111111",
                    "amount": "1000000000000000000",
                    "expiration": "1712600000",
                    "nonce": "0"
                },
                {
                    "token": "0x2222222222222222222222222222222222222222",
                    "amount": "2000000000000000000",
                    "expiration": "1712600001",
                    "nonce": "1"
                }
            ],
            "spender": "0x3333333333333333333333333333333333333333",
            "sigDeadline": "1712600500"
        }
    });
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
    let json: Value = serde_json::json!({
        "types": {
            "EIP712Domain": [
                { "name": "name", "type": "string" },
                { "name": "chainId", "type": "uint256" },
                { "name": "verifyingContract", "type": "address" }
            ],
            "PermitBatch": [
                { "name": "details", "type": "PermitDetails[]" },
                { "name": "spender", "type": "address" },
                { "name": "sigDeadline", "type": "uint256" }
            ],
            "PermitDetails": [
                { "name": "token", "type": "address" },
                { "name": "amount", "type": "uint160" },
                { "name": "expiration", "type": "uint48" },
                { "name": "nonce", "type": "uint48" }
            ]
        },
        "primaryType": "PermitBatch",
        "domain": {
            "name": "Permit2",
            "chainId": "1",
            "verifyingContract": "0x000000000022D473030F116dDEE9F6B43aC78BA3"
        },
        "message": {
            "details": [
                {
                    "token": "0x1111111111111111111111111111111111111111",
                    "amount": "1000000000000000000",
                    "expiration": "1712600000",
                    "nonce": "0"
                },
                {
                    "token": "0x2222222222222222222222222222222222222222",
                    "amount": "2000000000000000000",
                    "expiration": "1712600001",
                    "nonce": "1"
                }
            ],
            "spender": "0x3333333333333333333333333333333333333333",
            "sigDeadline": "1712600500"
        }
    });
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
fn eip712_permit2_batch_shows_token_when_all_items_share_token() {
    let json: Value = serde_json::json!({
        "types": {
            "EIP712Domain": [
                { "name": "name", "type": "string" },
                { "name": "chainId", "type": "uint256" },
                { "name": "verifyingContract", "type": "address" }
            ],
            "PermitBatch": [
                { "name": "details", "type": "PermitDetails[]" },
                { "name": "spender", "type": "address" },
                { "name": "sigDeadline", "type": "uint256" }
            ],
            "PermitDetails": [
                { "name": "token", "type": "address" },
                { "name": "amount", "type": "uint160" },
                { "name": "expiration", "type": "uint48" },
                { "name": "nonce", "type": "uint48" }
            ]
        },
        "primaryType": "PermitBatch",
        "domain": {
            "name": "Permit2",
            "chainId": "1",
            "verifyingContract": "0x000000000022D473030F116dDEE9F6B43aC78BA3"
        },
        "message": {
            "details": [
                {
                    "token": "0x1111111111111111111111111111111111111111",
                    "amount": "1000000000000000000",
                    "expiration": "1712600000",
                    "nonce": "0"
                },
                {
                    "token": "0x1111111111111111111111111111111111111111",
                    "amount": "2000000000000000000",
                    "expiration": "1712600001",
                    "nonce": "1"
                }
            ],
            "spender": "0x3333333333333333333333333333333333333333",
            "sigDeadline": "1712600500"
        }
    });
    let message = parse_eip712_json(&json).unwrap();
    let result = simulate_eip712_message(Chain::Ethereum, &message);

    assert_eq!(result.payload[2].kind, SimulationPayloadFieldKind::Token);
    assert_eq!(result.payload[2].value, "0x1111111111111111111111111111111111111111");
    assert_eq!(result.payload[3].kind, SimulationPayloadFieldKind::Spender);
}

#[test]
fn eip712_permit2_batch_with_excessive_expiration_adds_warning_message() {
    let json: Value = serde_json::json!({
        "types": {
            "EIP712Domain": [
                { "name": "name", "type": "string" },
                { "name": "chainId", "type": "uint256" },
                { "name": "verifyingContract", "type": "address" }
            ],
            "PermitBatch": [
                { "name": "details", "type": "PermitDetails[]" },
                { "name": "spender", "type": "address" },
                { "name": "sigDeadline", "type": "uint256" }
            ],
            "PermitDetails": [
                { "name": "token", "type": "address" },
                { "name": "amount", "type": "uint160" },
                { "name": "expiration", "type": "uint48" },
                { "name": "nonce", "type": "uint48" }
            ]
        },
        "primaryType": "PermitBatch",
        "domain": {
            "name": "Permit2",
            "chainId": "1",
            "verifyingContract": "0x000000000022D473030F116dDEE9F6B43aC78BA3"
        },
        "message": {
            "details": [
                {
                    "token": "0x1111111111111111111111111111111111111111",
                    "amount": "1000000000000000000",
                    "expiration": "9999999999",
                    "nonce": "0"
                },
                {
                    "token": "0x2222222222222222222222222222222222222222",
                    "amount": "2000000000000000000",
                    "expiration": "9999999998",
                    "nonce": "1"
                }
            ],
            "spender": "0x3333333333333333333333333333333333333333",
            "sigDeadline": "9999999997"
        }
    });
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
