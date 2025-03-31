use std::sync::Arc;

use super::models::{Quote, QuoteResult, QuoteUrlParams};
use crate::network::{AlienProvider, AlienTarget};
use crate::swapper::SwapperError;

pub const MAYAN_API_URL: &str = "https://price-api.mayan.finance/v3";

#[derive(Debug)]
pub struct MayanRelayer {
    url: String,
    provider: Arc<dyn AlienProvider>,
}

impl MayanRelayer {
    pub fn new(url: String, provider: Arc<dyn AlienProvider>) -> Self {
        Self { url, provider }
    }

    pub fn default_relayer(provider: Arc<dyn AlienProvider>) -> Self {
        Self::new(MAYAN_API_URL.to_string(), provider)
    }

    pub async fn get_quote(&self, params: QuoteUrlParams) -> Result<Vec<Quote>, SwapperError> {
        let query = serde_urlencoded::to_string(&params).map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;

        let url = format!("{}/quote?{}", self.url, query);
        let target = AlienTarget::get(&url);

        let data = self
            .provider
            .request(target)
            .await
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;

        let result = serde_json::from_slice::<QuoteResult>(&data).map_err(|e| SwapperError::ComputeQuoteError { msg: e.to_string() })?;

        match result {
            QuoteResult::Success(response) => Ok(response.quotes),
            QuoteResult::Error(error) => match error.code.as_str() {
                "QUOTE_NOT_FOUND" => Err(SwapperError::NoQuoteAvailable),
                _ => Err(SwapperError::ComputeQuoteError { msg: error.msg }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{MAYAN_FORWARDER_CONTRACT, MAYAN_PROGRAM_ID};
    use crate::swapper::mayan::{
        models::{QuoteOptions, QuoteResponse, QuoteUrlParams},
        Token,
    };
    use primitives::Chain;

    #[test]
    fn test_quote_url_params_from_params() {
        let options = QuoteOptions::default();
        let params = QuoteUrlParams::new(
            "100".to_string(),
            "ETH".to_string(),
            Chain::Ethereum,
            "USDC".to_string(),
            Chain::Solana,
            &options,
            Some("50".to_string()),
            Some("referrer".to_string()),
            Some(10),
        );

        assert_eq!(params.amount_in64, "100");
        assert_eq!(params.from_token, "ETH");
        assert_eq!(params.from_chain, "ethereum");
        assert_eq!(params.to_token, "USDC");
        assert_eq!(params.to_chain, "solana");
        assert_eq!(params.solana_program, MAYAN_PROGRAM_ID.to_string());
        assert_eq!(params.forwarder_address, MAYAN_FORWARDER_CONTRACT.to_string());
        assert_eq!(params.slippage_bps, Some("50".to_string()));
        assert_eq!(params.referrer, Some("referrer".to_string()));
        assert_eq!(params.referrer_bps, Some(10));
        assert!(params.swift);
        assert!(!params.fast_mctp);
        assert!(!params.only_direct);
    }

    #[test]
    fn test_smart_chain_conversion() {
        let options = QuoteOptions::default();
        let params = QuoteUrlParams::new(
            "100".to_string(),
            "BNB".to_string(),
            Chain::SmartChain,
            "USDC".to_string(),
            Chain::Solana,
            &options,
            None,
            None,
            None,
        );

        assert_eq!(params.from_chain, "bsc");
    }

    #[test]
    fn test_quote_deserialization() {
        let data = include_str!("test/eth_usd_to_sol.json");
        let result = serde_json::from_str::<QuoteResponse>(data).unwrap();
        let quote = result.quotes.first().unwrap();

        assert_eq!(quote.r#type, "SWIFT");
        assert_eq!(quote.swift_input_decimals, 6);
        assert_eq!(quote.swift_input_contract, Some("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()));
    }

    #[test]
    fn test_quote_deserialization_2() {
        let data = include_str!("test/eth_link_to_sol.json");
        let result = serde_json::from_str::<QuoteResponse>(data).unwrap();
        let quote = result.quotes.first().unwrap();

        assert_eq!(quote.evm_swap_router_address.clone().unwrap(), "0x111111125421ca6dc452d289314280a0f8842a65");

        assert!(quote.evm_swap_router_calldata.is_some());
        assert!(quote.min_middle_amount.is_some());
    }

    #[test]
    fn test_quote_deserialization_3() {
        let data = include_str!("test/sol_to_eth.json");
        let result = serde_json::from_str::<QuoteResponse>(data).unwrap();
        let quote = result.quotes.first().unwrap();

        assert_eq!(quote.swift_mayan_contract.clone().unwrap(), "BLZRi6frs4X4DNLw56V4EXai1b6QVESN1BhHBTYM9VcY");
        assert_eq!(quote.swift_input_contract.clone().unwrap(), "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs");
        assert_eq!(quote.swift_input_decimals, 8);
        assert_eq!(quote.from_token.contract, "So11111111111111111111111111111111111111112");
        assert_eq!(quote.from_token.decimals, 9);
    }

    #[test]
    fn test_token_deserialization() {
        let data = include_str!("test/quote_token_response.json");

        let token: Token = serde_json::from_str(data).expect("Failed to deserialize Token");
        assert_eq!(token.name, "ETH");
        assert!(token.verified);
        assert_eq!(token.w_chain_id, 30);
        assert_eq!(token.wrapped_address.unwrap(), "0x4200000000000000000000000000000000000006");
    }
}
