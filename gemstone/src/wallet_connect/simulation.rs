use gem_wallet_connect::{
    SignDigestType as WcSignDigestType, SignMessageValidation, WCEthereumTransactionData as WcEthereumTransactionData, WalletConnectRequestHandler,
    WalletConnectTransaction as WcWalletConnectTransaction, WalletConnectTransactionType as WcWalletConnectTransactionType, decode_sign_message, validate_send_transaction,
    validate_sign_message,
};
use primitives::{Chain, SimulationSeverity, SimulationWarning, SimulationWarningType, hex};

use crate::message::sign_type::{SignDigestType, SignMessage};

pub fn decode_message(chain: Chain, sign_type: SignDigestType, data: String) -> SignMessage {
    let sign_type: WcSignDigestType = sign_type.into();
    let result = decode_sign_message(chain, sign_type, data);

    SignMessage {
        chain: result.chain,
        sign_type: result.sign_type.into(),
        data: result.data,
    }
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

pub(super) fn validation_warning(error: &str) -> SimulationWarning {
    SimulationWarning::new(SimulationSeverity::Critical, SimulationWarningType::ValidationError, Some(error.to_string()))
}
