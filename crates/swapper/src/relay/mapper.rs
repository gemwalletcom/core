use primitives::{TransactionSwapMetadata, swap::ApprovalData};

use super::{
    DEFAULT_SWAP_GAS_LIMIT,
    asset::map_currency_to_asset_id,
    chain::RelayChain,
    model::{RelayQuoteResponse, RelayRequest, StepData, gas_fee_amount},
};
use crate::{SwapResult, SwapperError, SwapperProvider, SwapperQuoteData};

pub fn map_quote_data(quote_response: &RelayQuoteResponse, from_value: &str, approval: Option<ApprovalData>) -> Result<SwapperQuoteData, SwapperError> {
    let step_data = quote_response.step_data().ok_or(SwapperError::InvalidRoute)?;

    match step_data {
        StepData::Bitcoin(btc) => {
            let gas_limit = gas_fee_amount(&quote_response.fees);
            Ok(SwapperQuoteData::new_contract(String::new(), from_value.to_string(), btc.psbt.clone(), None, gas_limit))
        }
        StepData::Evm(evm) => {
            let gas_limit = approval.as_ref().map(|_| DEFAULT_SWAP_GAS_LIMIT.to_string());
            let call_data = evm.data.clone().unwrap_or_default();
            Ok(SwapperQuoteData::new_contract(evm.to.clone(), evm.value.clone(), call_data, approval, gas_limit))
        }
    }
}

pub fn map_swap_result(request: &RelayRequest) -> SwapResult {
    let metadata = request.data.as_ref().and_then(|d| d.metadata.as_ref()).and_then(|m| {
        let currency_in = m.currency_in.as_ref()?;
        let currency_out = m.currency_out.as_ref()?;
        let from_chain = RelayChain::from_chain_id(currency_in.currency.chain_id)?.to_chain();
        let to_chain = RelayChain::from_chain_id(currency_out.currency.chain_id)?.to_chain();
        Some(TransactionSwapMetadata {
            from_asset: map_currency_to_asset_id(from_chain, &currency_in.currency.address),
            from_value: currency_in.amount.clone()?,
            to_asset: map_currency_to_asset_id(to_chain, &currency_out.currency.address),
            to_value: currency_out.amount.clone()?,
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
    use crate::relay::model::{
        CurrencyAmount, QuoteDetails, RelayCurrencyDetail, RelayFeeAmount, RelayFees, RelayQuoteResponse, RelayRequest, RelayRequestMetadata, RelayStatus, Step,
    };
    use primitives::{AssetId, Chain, swap::SwapStatus};

    #[test]
    fn test_map_evm_quote_data() {
        let quote_response = RelayQuoteResponse {
            steps: vec![Step::mock_transaction("swap", "0xrouter", "1000000000000000000", "0xabcdef")],
            details: QuoteDetails {
                currency_out: CurrencyAmount { amount: "0".to_string() },
                time_estimate: None,
                swap_impact: None,
            },
            fees: None,
        };

        let result = map_quote_data(&quote_response, "1000000000000000000", None).unwrap();

        assert_eq!(result.to, "0xrouter");
        assert_eq!(result.value, "1000000000000000000");
        assert_eq!(result.data, "0xabcdef");
        assert!(result.approval.is_none());
        assert!(result.limit.is_none());
    }

    #[test]
    fn test_map_evm_quote_data_with_approval() {
        let quote_response = RelayQuoteResponse {
            steps: vec![Step::mock_transaction("swap", "0xrouter", "0", "0xabcdef")],
            details: QuoteDetails {
                currency_out: CurrencyAmount { amount: "0".to_string() },
                time_estimate: None,
                swap_impact: None,
            },
            fees: None,
        };
        let approval = ApprovalData {
            token: "0xtoken".to_string(),
            spender: "0xrouter".to_string(),
            value: "1000".to_string(),
        };

        let result = map_quote_data(&quote_response, "1000000000000000000", Some(approval.clone())).unwrap();

        assert_eq!(result.to, "0xrouter");
        assert_eq!(result.approval, Some(approval));
        assert_eq!(result.limit, Some(DEFAULT_SWAP_GAS_LIMIT.to_string()));
    }

    #[test]
    fn test_map_bitcoin_quote_data() {
        let psbt = "70736274ff0100abcdef";
        let quote_response = RelayQuoteResponse {
            steps: vec![Step::mock_bitcoin(psbt)],
            details: QuoteDetails {
                currency_out: CurrencyAmount { amount: "0".to_string() },
                time_estimate: None,
                swap_impact: None,
            },
            fees: None,
        };

        let result = map_quote_data(&quote_response, "2000000", None).unwrap();

        assert_eq!(result.to, "");
        assert_eq!(result.value, "2000000");
        assert_eq!(result.data, psbt);
        assert!(result.approval.is_none());
        assert!(result.limit.is_none());
    }

    #[test]
    fn test_map_bitcoin_quote_data_with_gas_fee() {
        let psbt = "70736274ff0100abcdef";
        let quote_response = RelayQuoteResponse {
            steps: vec![Step::mock_bitcoin(psbt)],
            details: QuoteDetails {
                currency_out: CurrencyAmount { amount: "0".to_string() },
                time_estimate: None,
                swap_impact: None,
            },
            fees: Some(RelayFees {
                gas: Some(RelayFeeAmount {
                    amount: Some("15000".to_string()),
                }),
            }),
        };

        let result = map_quote_data(&quote_response, "2000000", None).unwrap();

        assert_eq!(result.data, psbt);
        assert_eq!(result.limit, Some("15000".to_string()));
    }

    #[test]
    fn test_map_swap_result_evm_to_evm() {
        let request = RelayRequest::mock(
            RelayStatus::Success,
            Some(RelayRequestMetadata {
                currency_in: Some(RelayCurrencyDetail::mock("0x0000000000000000000000000000000000000000", 1, "1000000000000000000")),
                currency_out: Some(RelayCurrencyDetail::mock("0x0000000000000000000000000000000000000000", 8453, "999000000000000000")),
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
    fn test_map_swap_result_evm_to_btc() {
        use super::super::chain::BITCOIN_CHAIN_ID;
        let usdt_address = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
        let request = RelayRequest::mock(
            RelayStatus::Completed,
            Some(RelayRequestMetadata {
                currency_in: Some(RelayCurrencyDetail::mock(usdt_address, 1, "10000000")),
                currency_out: Some(RelayCurrencyDetail::mock("bc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqmql8k8", BITCOIN_CHAIN_ID, "50000")),
            }),
        );

        let result = map_swap_result(&request);

        assert_eq!(result.status, SwapStatus::Completed);
        let metadata = result.metadata.unwrap();
        assert_eq!(metadata.from_asset, AssetId::from_token(Chain::Ethereum, usdt_address));
        assert_eq!(metadata.to_asset, AssetId::from_chain(Chain::Bitcoin));
    }

    #[test]
    fn test_map_swap_result_status() {
        let pending = map_swap_result(&RelayRequest::mock(RelayStatus::Pending, None));
        assert_eq!(pending.status, SwapStatus::Pending);
        assert!(pending.metadata.is_none());

        let failed = map_swap_result(&RelayRequest::mock(RelayStatus::Failed, None));
        assert_eq!(failed.status, SwapStatus::Failed);
        assert!(failed.metadata.is_none());
    }

    #[test]
    fn test_map_quote_data_without_step_data() {
        let quote_response = RelayQuoteResponse {
            steps: vec![Step::mock_empty("approve", "transaction")],
            details: QuoteDetails {
                currency_out: CurrencyAmount { amount: "0".to_string() },
                time_estimate: None,
                swap_impact: None,
            },
            fees: None,
        };

        assert!(map_quote_data(&quote_response, "0", None).is_err());
    }
}
