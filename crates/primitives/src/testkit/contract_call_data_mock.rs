use super::signer_mock::TEST_EVM_RECIPIENT;
use crate::contract_call_data::ContractCallData;

impl ContractCallData {
    pub fn mock() -> Self {
        ContractCallData {
            contract_address: TEST_EVM_RECIPIENT.to_string(),
            call_data: "abcd".to_string(),
            approval: None,
            gas_limit: None,
        }
    }

    pub fn mock_with_call_data(call_data: &str) -> Self {
        ContractCallData {
            call_data: call_data.to_string(),
            ..Self::mock()
        }
    }
}
