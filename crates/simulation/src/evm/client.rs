use std::error::Error;

use gem_client::Client;
use gem_evm::{eip712::EIP712Message, rpc::EthereumClient};
use primitives::hex;
use primitives::{Chain, SimulationResult, SimulationSeverity, SimulationWarning, SimulationWarningType};

use super::{
    approval_request::ApprovalRequest,
    decode::{decode_eip712_approval, decode_evm_approval},
};

pub struct SimulationClient<'a, C: Client + Clone> {
    ethereum_client: &'a EthereumClient<C>,
}

impl<'a, C: Client + Clone> SimulationClient<'a, C> {
    pub fn new(ethereum_client: &'a EthereumClient<C>) -> Self {
        Self { ethereum_client }
    }

    pub async fn simulate_eip712_message(&self, chain: Chain, message: &EIP712Message) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        match decode_eip712_approval(chain, message) {
            Some(approval) => self.simulate_approval(approval).await,
            None => Ok(SimulationResult::default()),
        }
    }

    pub async fn simulate_evm_calldata(&self, chain: Chain, calldata: &[u8], contract_address: &str) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        match decode_evm_approval(chain, calldata, contract_address) {
            Some(approval) => self.simulate_approval(approval).await,
            None => Ok(SimulationResult::default()),
        }
    }

    async fn simulate_approval(&self, approval: ApprovalRequest) -> Result<SimulationResult, Box<dyn Error + Send + Sync>> {
        let warnings = self.approval_warnings(&approval).await?.into_iter().chain(approval.expiration_warning()).collect();
        Ok(approval.build_simulation_result(warnings))
    }

    async fn approval_warnings(&self, approval: &ApprovalRequest) -> Result<Vec<SimulationWarning>, Box<dyn Error + Send + Sync>> {
        if self.spender_is_externally_owned(&approval.spender_address).await? {
            return Ok(vec![SimulationWarning::new(
                SimulationSeverity::Critical,
                SimulationWarningType::ExternallyOwnedSpender,
                None,
            )]);
        }

        Ok(vec![approval.primary_warning()])
    }

    async fn spender_is_externally_owned(&self, spender_address: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let code = self.ethereum_client.get_code(spender_address).await?;
        let bytecode = hex::decode_hex(&code)?;
        Ok(bytecode.is_empty() || bytecode.iter().all(|byte| *byte == 0))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use alloy_primitives::{U256, address};
    use alloy_sol_types::SolCall;
    use gem_evm::eip712::parse_eip712_json;
    use gem_evm::rpc::EthereumClient;
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::{Chain, EVMChain, SimulationSeverity, SimulationWarning, SimulationWarningType, asset_constants::ETHEREUM_USDC_TOKEN_ID};
    use serde_json::Value;

    use super::SimulationClient;

    #[tokio::test]
    async fn eip712_permit_with_externally_owned_spender_adds_critical_warning() -> Result<(), Box<dyn Error + Send + Sync>> {
        let json: Value = serde_json::from_str(include_str!("../../../gem_evm/testdata/1inch_permit.json"))?;
        let message = parse_eip712_json(&json)?;
        let client = ethereum_client("0x");

        let result = SimulationClient::new(&client).simulate_eip712_message(Chain::Ethereum, &message).await?;

        assert_eq!(result.warnings.len(), 1);
        assert_eq!(
            result.warnings.first(),
            Some(&SimulationWarning {
                severity: SimulationSeverity::Critical,
                warning: SimulationWarningType::ExternallyOwnedSpender,
                message: None,
            })
        );

        Ok(())
    }

    #[tokio::test]
    async fn erc20_approve_with_contract_spender_does_not_add_externally_owned_warning() -> Result<(), Box<dyn Error + Send + Sync>> {
        let calldata = gem_evm::contracts::IERC20::approveCall {
            spender: address!("3fC91A3afd70395Cd496C647d5a6CC9D4B2b7FAD"),
            value: U256::from(1000u64),
        }
        .abi_encode();

        let client = ethereum_client("0x1234");
        let result = SimulationClient::new(&client)
            .simulate_evm_calldata(Chain::Ethereum, &calldata, ETHEREUM_USDC_TOKEN_ID)
            .await?;

        assert_eq!(result.warnings.len(), 1);
        assert_ne!(result.warnings[0].warning, SimulationWarningType::ExternallyOwnedSpender);

        Ok(())
    }

    #[tokio::test]
    async fn invalid_spender_code_response_returns_error() {
        let json: Value = serde_json::from_str(include_str!("../../../gem_evm/testdata/1inch_permit.json")).unwrap();
        let message = parse_eip712_json(&json).unwrap();
        let client = ethereum_client("0xzz");

        let result = SimulationClient::new(&client).simulate_eip712_message(Chain::Ethereum, &message).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn zero_filled_spender_code_is_treated_as_externally_owned() -> Result<(), Box<dyn Error + Send + Sync>> {
        let json: Value = serde_json::from_str(include_str!("../../../gem_evm/testdata/1inch_permit.json"))?;
        let message = parse_eip712_json(&json)?;
        let client = ethereum_client("0x00");

        let result = SimulationClient::new(&client).simulate_eip712_message(Chain::Ethereum, &message).await?;

        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].warning, SimulationWarningType::ExternallyOwnedSpender);

        Ok(())
    }

    #[tokio::test]
    async fn eip712_permit_with_excessive_expiration_keeps_warning_with_client() -> Result<(), Box<dyn Error + Send + Sync>> {
        let json: Value = serde_json::from_str(include_str!("../../testdata/permit_excessive_expiration.json"))?;
        let message = parse_eip712_json(&json)?;
        let client = ethereum_client("0x1234");

        let result = SimulationClient::new(&client).simulate_eip712_message(Chain::Ethereum, &message).await?;

        assert_eq!(result.warnings.len(), 2);
        assert_eq!(result.warnings[1].warning, SimulationWarningType::ValidationError);
        assert_eq!(result.warnings[1].message.as_deref(), Some("Excessive expiration"));

        Ok(())
    }

    #[tokio::test]
    async fn eip712_permit_batch_with_externally_owned_spender_adds_critical_warning() -> Result<(), Box<dyn Error + Send + Sync>> {
        let json: Value = serde_json::from_str(include_str!("../../testdata/permit_batch_multiple_tokens.json"))?;
        let message = parse_eip712_json(&json)?;
        let client = ethereum_client("0x");

        let result = SimulationClient::new(&client).simulate_eip712_message(Chain::Ethereum, &message).await?;

        assert_eq!(result.warnings.len(), 1);
        assert_eq!(
            result.warnings.first(),
            Some(&SimulationWarning {
                severity: SimulationSeverity::Critical,
                warning: SimulationWarningType::ExternallyOwnedSpender,
                message: None,
            })
        );

        Ok(())
    }
    fn ethereum_client(code: &str) -> EthereumClient<gem_client::testkit::MockClient> {
        let code = code.to_string();
        let client = mock_jsonrpc_client(move |method, _| match method {
            "eth_getCode" => Ok(Value::from(code.clone())),
            _ => Ok(Value::Null),
        });
        EthereumClient::new(client, EVMChain::Ethereum)
    }
}
