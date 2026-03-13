use gem_wallet_connect::{
    SignDigestType as WcSignDigestType, SignMessageValidation, WCEthereumTransactionData as WcEthereumTransactionData, WalletConnectRequestHandler,
    WalletConnectTransaction as WcWalletConnectTransaction, WalletConnectTransactionType as WcWalletConnectTransactionType, decode_sign_message, validate_send_transaction,
    validate_sign_message,
};
use primitives::{Chain, SimulationResult, SimulationSeverity, SimulationWarning, SimulationWarningType, hex};

use crate::message::sign_type::{SignDigestType, SignMessage};

use super::WalletConnectTransactionType;

pub fn decode_message(chain: Chain, sign_type: SignDigestType, data: String) -> SignMessage {
    let sign_type: WcSignDigestType = sign_type.into();
    let result = decode_sign_message(chain, sign_type, data);

    SignMessage {
        chain: result.chain,
        sign_type: result.sign_type.into(),
        data: result.data,
    }
}

pub fn simulate_sign_message(chain: Chain, sign_type: SignDigestType, data: String, session_domain: String) -> SimulationResult {
    let sign_type: WcSignDigestType = sign_type.into();
    let validation_warnings = sign_message_validation_warnings(chain, &sign_type, &data, &session_domain);

    let simulation = match sign_type {
        WcSignDigestType::Eip712 => parse_eip712_message(&data)
            .map(|message| ::simulation::evm::simulate_eip712_message(chain, &message))
            .unwrap_or_default(),
        _ => SimulationResult::default(),
    };

    simulation.prepend_warnings(validation_warnings)
}

pub fn simulate_send_transaction(chain: Chain, transaction_type: WalletConnectTransactionType, data: String) -> SimulationResult {
    let transaction_type: WcWalletConnectTransactionType = transaction_type.into();
    let validation_warnings = send_transaction_validation_warnings(&transaction_type, &data);

    let simulation = match transaction_type {
        WcWalletConnectTransactionType::Ethereum => simulate_ethereum_transaction(chain, &data).unwrap_or_default(),
        _ => SimulationResult::default(),
    };

    simulation.prepend_warnings(validation_warnings)
}

pub(super) fn parse_eip712_message(data: &str) -> Option<gem_evm::eip712::EIP712Message> {
    serde_json::from_str(data).ok().and_then(|value| gem_evm::eip712::parse_eip712_json(&value).ok())
}

pub(super) fn sign_message_validation_warnings(chain: Chain, sign_type: &WcSignDigestType, data: &str, session_domain: &str) -> Vec<SimulationWarning> {
    let input = SignMessageValidation {
        chain,
        sign_type,
        data,
        session_domain,
    };

    validate_sign_message(&input).err().into_iter().map(|error| validation_warning(&error)).collect()
}

pub(super) fn send_transaction_validation_warnings(transaction_type: &WcWalletConnectTransactionType, data: &str) -> Vec<SimulationWarning> {
    validate_send_transaction(transaction_type, data)
        .err()
        .into_iter()
        .map(|error| validation_warning(&error))
        .collect()
}

pub(super) fn simulate_ethereum_transaction(chain: Chain, data: &str) -> Option<SimulationResult> {
    let (transaction, bytes) = decode_ethereum_calldata(data)?;

    Some(::simulation::evm::simulate_evm_calldata(chain, &bytes, &transaction.to))
}

fn decode_ethereum_transaction_data(data: &str) -> Result<WcEthereumTransactionData, String> {
    let transaction = WalletConnectRequestHandler::decode_send_transaction(WcWalletConnectTransactionType::Ethereum, data.to_string())?;
    match transaction {
        WcWalletConnectTransaction::Ethereum { data } => Ok(data),
        _ => Err("Invalid Ethereum transaction".to_string()),
    }
}

pub(super) fn decode_ethereum_calldata(data: &str) -> Option<(WcEthereumTransactionData, Vec<u8>)> {
    let transaction = decode_ethereum_transaction_data(data).ok()?;
    let calldata = transaction.data.as_deref()?;
    let bytes = hex::decode_hex(calldata).ok()?;
    Some((transaction, bytes))
}

pub(super) fn spender_verification_warning(simulation: SimulationResult) -> SimulationResult {
    if simulation.requires_spender_verification() {
        return simulation.prepend_warnings(vec![validation_warning("Unable to verify spender is a contract")]);
    }

    simulation
}

pub(super) fn validation_warning(error: &str) -> SimulationWarning {
    SimulationWarning::new(SimulationSeverity::Critical, SimulationWarningType::ValidationError, Some(error.to_string()))
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use primitives::{SimulationWarning, SimulationWarningType};

    use super::{SimulationResult, SimulationSeverity, spender_verification_warning};

    #[test]
    fn spender_verification_failure_blocks_approval_simulation() {
        let simulation = SimulationResult::new(
            vec![SimulationWarning::new(
                SimulationSeverity::Warning,
                SimulationWarningType::PermitApproval {
                    asset_id: "ethereum_0x123".into(),
                    value: Some(BigInt::from(100)),
                },
                None,
            )],
            vec![],
        );

        let result = spender_verification_warning(simulation);

        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].warning, SimulationWarningType::ValidationError);
        assert_eq!(result.warnings[0].message.as_deref(), Some("Unable to verify spender is a contract"));
    }

    #[test]
    fn spender_verification_failure_does_not_block_non_approval_simulation() {
        let simulation = SimulationResult::default();

        let result = spender_verification_warning(simulation);

        assert!(result.warnings.is_empty());
    }
}
