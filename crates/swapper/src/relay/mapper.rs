use primitives::{TransactionSwapMetadata, swap::ApprovalData};

use super::{
    DEFAULT_GAS_LIMIT,
    asset::relay_currency_to_asset_id,
    chain::RelayChain,
    model::{RelayRequest, Step, StepData},
};
use crate::{SwapResult, SwapperError, SwapperProvider, SwapperQuoteData};

pub const STEP_SWAP: &str = "swap";
pub const STEP_DEPOSIT: &str = "deposit";
pub const STEP_APPROVE: &str = "approve";

pub fn get_step_data(steps: &[Step]) -> Result<&StepData, SwapperError> {
    let tx_step = steps
        .iter()
        .find(|s| s.id == STEP_SWAP || s.id == STEP_DEPOSIT)
        .or_else(|| steps.iter().find(|s| s.kind == "transaction" && s.id != STEP_APPROVE))
        .or_else(|| steps.iter().find(|s| s.items.as_ref().is_some_and(|i| !i.is_empty())))
        .ok_or(SwapperError::InvalidRoute)?;
    tx_step
        .items
        .as_ref()
        .and_then(|items| items.first())
        .and_then(|item| item.data.as_ref())
        .ok_or(SwapperError::InvalidRoute)
}

pub fn map_quote_data(chain: &RelayChain, steps: &[Step], value: &str, approval: Option<ApprovalData>) -> Result<SwapperQuoteData, SwapperError> {
    let step_data = get_step_data(steps)?;

    let (to, tx_value, data, gas_limit) = match chain {
        RelayChain::Bitcoin => {
            let psbt = step_data.psbt.as_ref().ok_or(SwapperError::InvalidRoute)?;
            (String::new(), value.to_string(), psbt.clone(), None)
        }
        _ if chain.is_evm() => {
            let to = step_data.to.clone().unwrap_or_default();
            let data = step_data.data.clone().unwrap_or_default();
            let gas_limit = approval.as_ref().map(|_| DEFAULT_GAS_LIMIT.to_string());
            (to, step_data.value.clone(), data, gas_limit)
        }
        _ => return Err(SwapperError::NotSupportedChain),
    };

    Ok(SwapperQuoteData::new_contract(to, tx_value, data, approval, gas_limit))
}

pub fn map_swap_result(request: &RelayRequest) -> SwapResult {
    let metadata = request.metadata.as_ref().and_then(|m| {
        let currency_in = m.currency_in.as_ref()?;
        let currency_out = m.currency_out.as_ref()?;
        let from_chain = RelayChain::from_chain_id(currency_in.chain_id)?.to_chain();
        let to_chain = RelayChain::from_chain_id(currency_out.chain_id)?.to_chain();
        Some(TransactionSwapMetadata {
            from_asset: relay_currency_to_asset_id(from_chain, &currency_in.currency),
            from_value: currency_in.amount.clone().unwrap_or_default(),
            to_asset: relay_currency_to_asset_id(to_chain, &currency_out.currency),
            to_value: currency_out.amount.clone().unwrap_or_default(),
            provider: Some(SwapperProvider::Relay.as_ref().to_string()),
        })
    });

    SwapResult {
        status: request.status.clone().into_swap_status(),
        metadata,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::relay::model::{RelayCurrencyDetail, RelayRequestMetadata, RelayStatus, StepData, StepItem};
    use primitives::{AssetId, Chain, swap::SwapStatus};

    fn create_transaction_step(to: &str, value: &str, data: &str) -> Step {
        Step {
            id: "swap".to_string(),
            kind: "transaction".to_string(),
            items: Some(vec![StepItem {
                data: Some(StepData {
                    to: Some(to.to_string()),
                    data: Some(data.to_string()),
                    value: value.to_string(),
                    instructions: None,
                    address_lookup_table_addresses: None,
                    psbt: None,
                }),
            }]),
        }
    }

    fn create_bitcoin_step(psbt: &str) -> Step {
        Step {
            id: "deposit".to_string(),
            kind: "transaction".to_string(),
            items: Some(vec![StepItem {
                data: Some(StepData {
                    to: None,
                    data: None,
                    value: String::new(),
                    instructions: None,
                    address_lookup_table_addresses: None,
                    psbt: Some(psbt.to_string()),
                }),
            }]),
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

    fn create_relay_request(status: RelayStatus, metadata: Option<RelayRequestMetadata>) -> RelayRequest {
        RelayRequest { status, metadata }
    }

    fn create_currency_detail(currency: &str, chain_id: u64, amount: &str) -> RelayCurrencyDetail {
        RelayCurrencyDetail {
            currency: currency.to_string(),
            chain_id,
            amount: Some(amount.to_string()),
        }
    }

    #[test]
    fn test_map_swap_result_evm_to_evm() {
        let request = create_relay_request(
            RelayStatus::Success,
            Some(RelayRequestMetadata {
                currency_in: Some(create_currency_detail("0x0000000000000000000000000000000000000000", 1, "1000000000000000000")),
                currency_out: Some(create_currency_detail("0x0000000000000000000000000000000000000000", 8453, "999000000000000000")),
            }),
        );

        let result = map_swap_result(&request);

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(metadata.from_value, "1000000000000000000");
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Base));
        assert_eq!(metadata.to_value, "999000000000000000");
        assert_eq!(metadata.provider, Some("relay".to_string()));
    }

    #[test]
    fn test_map_swap_result_evm_token_to_btc() {
        let usdt_address = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
        let request = create_relay_request(
            RelayStatus::Completed,
            Some(RelayRequestMetadata {
                currency_in: Some(create_currency_detail(usdt_address, 1, "10000000")),
                currency_out: Some(create_currency_detail("bc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqmql8k8", 8253038, "50000")),
            }),
        );

        let result = map_swap_result(&request);

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_token(Chain::Ethereum, usdt_address));
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Bitcoin));
    }

    #[test]
    fn test_map_swap_result_pending() {
        let request = create_relay_request(RelayStatus::Pending, None);

        let result = map_swap_result(&request);

        assert_eq!(result.status, SwapStatus::Pending);
        assert!(result.metadata.is_none());
    }

    #[test]
    fn test_map_swap_result_failed() {
        let request = create_relay_request(RelayStatus::Failed, None);

        let result = map_swap_result(&request);

        assert_eq!(result.status, SwapStatus::Failed);
        assert!(result.metadata.is_none());
    }
}
