use super::signer_mock::TEST_EVM_RECIPIENT;
use crate::{
    AssetId, Chain, SwapProvider, TransactionSwapMetadata,
    asset_constants::ETHEREUM_USDT_TOKEN_ID,
    swap::{ApprovalData, SwapData, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteDataType},
};

impl ApprovalData {
    pub fn mock() -> Self {
        ApprovalData {
            token: ETHEREUM_USDT_TOKEN_ID.to_string(),
            spender: TEST_EVM_RECIPIENT.to_string(),
            value: "0".to_string(),
            is_unlimited: false,
        }
    }
}

impl SwapData {
    pub fn mock() -> Self {
        SwapData {
            quote: SwapQuote::mock(),
            data: SwapQuoteData::mock(),
        }
    }

    pub fn mock_with_provider(provider: SwapProvider) -> Self {
        SwapData {
            quote: SwapQuote::mock_with_provider(provider),
            data: SwapQuoteData::mock(),
        }
    }

    pub fn mock_with_values(provider: SwapProvider, from_value: &str, to_value: &str) -> Self {
        SwapData {
            quote: SwapQuote::mock_with_values(provider, from_value, to_value),
            data: SwapQuoteData::mock(),
        }
    }
}

impl SwapQuote {
    pub fn mock() -> Self {
        SwapQuote {
            from_value: "1000000000".to_string(),
            to_value: "1000000".to_string(),
            provider_data: SwapProviderData::mock(),
            from_address: TEST_EVM_RECIPIENT.to_string(),
            to_address: TEST_EVM_RECIPIENT.to_string(),
            slippage_bps: 50,
            eta_in_seconds: Some(30),
            use_max_amount: None,
        }
    }

    pub fn mock_with_provider(provider: SwapProvider) -> Self {
        Self::mock_with_values(provider, "1000000000", "1000000")
    }

    pub fn mock_with_values(provider: SwapProvider, from_value: &str, to_value: &str) -> Self {
        SwapQuote {
            from_value: from_value.to_string(),
            to_value: to_value.to_string(),
            provider_data: SwapProviderData::mock_with_provider(provider),
            from_address: TEST_EVM_RECIPIENT.to_string(),
            to_address: TEST_EVM_RECIPIENT.to_string(),
            slippage_bps: 50,
            eta_in_seconds: Some(30),
            use_max_amount: None,
        }
    }
}

impl SwapQuoteData {
    pub fn mock() -> Self {
        SwapQuoteData {
            data_type: SwapQuoteDataType::Contract,
            to: TEST_EVM_RECIPIENT.to_string(),
            value: "0".to_string(),
            data: "0x".to_string(),
            memo: None,
            approval: None,
            gas_limit: Some("21000".to_string()),
        }
    }

    pub fn mock_with_gas_limit(gas_limit: Option<String>) -> Self {
        SwapQuoteData { gas_limit, ..Self::mock() }
    }
}

impl TransactionSwapMetadata {
    pub fn mock() -> Self {
        TransactionSwapMetadata {
            from_asset: AssetId::from_chain(Chain::Bitcoin),
            from_value: "100000".to_string(),
            to_asset: AssetId::from_chain(Chain::Ethereum),
            to_value: "2500".to_string(),
            provider: Some("thorchain".to_string()),
        }
    }
}

impl SwapProviderData {
    pub fn mock() -> Self {
        SwapProviderData {
            provider: SwapProvider::UniswapV3,
            name: "Uniswap V3".to_string(),
            protocol_name: "uniswap_v3".to_string(),
        }
    }

    pub fn mock_with_provider(provider: SwapProvider) -> Self {
        SwapProviderData {
            provider,
            name: provider.name().to_string(),
            protocol_name: provider.protocol_name().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_data_mock() {
        let swap_data = SwapData::mock();
        assert_eq!(swap_data.quote.from_value, "1000000000");
        assert_eq!(swap_data.quote.to_value, "1000000");
        assert_eq!(swap_data.quote.provider_data.provider, SwapProvider::UniswapV3);
    }

    #[test]
    fn test_swap_data_mock_with_provider() {
        let swap_data = SwapData::mock_with_provider(SwapProvider::Jupiter);
        assert_eq!(swap_data.quote.provider_data.provider, SwapProvider::Jupiter);
        assert_eq!(swap_data.quote.provider_data.name, "Jupiter");
    }
}
