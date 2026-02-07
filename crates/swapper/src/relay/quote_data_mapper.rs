use primitives::swap::ApprovalData;

use super::{
    DEFAULT_GAS_LIMIT,
    chain::RelayChain,
    model::{Step, StepData},
};
use crate::{SwapperError, SwapperQuoteData};

fn get_step_data(steps: &[Step]) -> Result<&StepData, SwapperError> {
    let tx_step = steps
        .iter()
        .find(|s| s.id == "swap" || s.id == "deposit" || s.kind == "transaction")
        .or_else(|| steps.iter().find(|s| !s.items.is_empty()))
        .ok_or(SwapperError::InvalidRoute)?;
    tx_step.items.first().and_then(|item| item.data.as_ref()).ok_or(SwapperError::InvalidRoute)
}

pub fn map_quote_data(chain: &RelayChain, steps: &[Step], value: &str, approval: Option<ApprovalData>) -> Result<SwapperQuoteData, SwapperError> {
    let step_data = get_step_data(steps)?;

    let (to, tx_value, data, gas_limit) = match chain {
        RelayChain::Bitcoin => {
            let psbt = step_data.psbt.as_ref().ok_or(SwapperError::InvalidRoute)?;
            (String::new(), value.to_string(), psbt.clone(), None)
        }
        RelayChain::Solana => {
            let data = step_data
                .instructions
                .as_ref()
                .map(|i| serde_json::to_string(i).unwrap_or_default())
                .unwrap_or_else(|| step_data.data.clone());
            (step_data.to.clone(), step_data.value.clone(), data, None)
        }
        _ if chain.is_evm() => {
            let gas_limit = approval.as_ref().map(|_| DEFAULT_GAS_LIMIT.to_string());
            (step_data.to.clone(), step_data.value.clone(), step_data.data.clone(), gas_limit)
        }
        _ => return Err(SwapperError::NotSupportedChain),
    };

    Ok(SwapperQuoteData::new_contract(to, tx_value, data, approval, gas_limit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::model::{StepData, StepItem};

    fn create_transaction_step(to: &str, value: &str, data: &str) -> Step {
        Step {
            id: "swap".to_string(),
            kind: "transaction".to_string(),
            items: vec![StepItem {
                data: Some(StepData {
                    to: to.to_string(),
                    data: data.to_string(),
                    value: value.to_string(),
                    instructions: None,
                    psbt: None,
                }),
            }],
        }
    }

    fn create_bitcoin_step(psbt: &str) -> Step {
        Step {
            id: "deposit".to_string(),
            kind: "transaction".to_string(),
            items: vec![StepItem {
                data: Some(StepData {
                    to: String::new(),
                    data: String::new(),
                    value: String::new(),
                    instructions: None,
                    psbt: Some(psbt.to_string()),
                }),
            }],
        }
    }

    #[test]
    fn test_map_evm_quote_data() {
        let steps = vec![create_transaction_step("0xrouter", "1000000000000000000", "0xabcdef")];

        let result = map_quote_data(&RelayChain::Ethereum, &steps, "1000000000000000000", None).unwrap();

        assert_eq!(result.to, "0xrouter");
        assert_eq!(result.value, "1000000000000000000");
        assert_eq!(result.data, "0xabcdef");
        assert!(result.approval.is_none());
        assert!(result.gas_limit.is_none());
    }

    #[test]
    fn test_map_evm_quote_data_with_approval() {
        let steps = vec![create_transaction_step("0xrouter", "0", "0xabcdef")];
        let approval = ApprovalData {
            token: "0xtoken".to_string(),
            spender: "0xrouter".to_string(),
            value: "1000".to_string(),
        };

        let result = map_quote_data(&RelayChain::Ethereum, &steps, "1000000000000000000", Some(approval.clone())).unwrap();

        assert_eq!(result.to, "0xrouter");
        assert_eq!(result.approval, Some(approval));
        assert_eq!(result.gas_limit, Some(DEFAULT_GAS_LIMIT.to_string()));
    }

    #[test]
    fn test_map_solana_quote_data() {
        let steps = vec![create_transaction_step("SolanaProgramAddress", "0", "base64txdata")];

        let result = map_quote_data(&RelayChain::Solana, &steps, "1000000000", None).unwrap();

        assert_eq!(result.to, "SolanaProgramAddress");
        assert_eq!(result.value, "0");
        assert_eq!(result.data, "base64txdata");
        assert!(result.approval.is_none());
        assert!(result.gas_limit.is_none());
    }

    #[test]
    fn test_map_bitcoin_quote_data() {
        let psbt = "70736274ff0100abcdef";
        let steps = vec![create_bitcoin_step(psbt)];

        let result = map_quote_data(&RelayChain::Bitcoin, &steps, "2000000", None).unwrap();

        assert_eq!(result.to, "");
        assert_eq!(result.value, "2000000");
        assert_eq!(result.data, psbt);
        assert!(result.approval.is_none());
        assert!(result.gas_limit.is_none());
    }
}
