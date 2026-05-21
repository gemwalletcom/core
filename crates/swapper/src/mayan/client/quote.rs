use super::MayanClient;
use crate::{
    SwapperError,
    mayan::{
        constants::{MAYAN_FORWARDER, MAYAN_PROGRAM_ID, SDK_VERSION},
        model::{ErrorResponse, MayanQuote, QuoteParams, QuoteResponse},
    },
};
use gem_client::{Client, ClientError, ClientExt};
use serde::Serialize;
use std::fmt::Debug;

const QUOTE_DEFAULTS: [(&str, &str); 12] = [
    ("wormhole", "false"),
    ("swift", "true"),
    ("mctp", "true"),
    ("shuttle", "false"),
    ("fastMctp", "true"),
    ("gasless", "false"),
    ("onlyDirect", "false"),
    ("fullList", "false"),
    ("monoChain", "true"),
    ("solanaProgram", MAYAN_PROGRAM_ID),
    ("forwarderAddress", MAYAN_FORWARDER),
    ("slippageBps", "auto"),
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuoteDynamicQuery {
    amount_in64: String,
    from_token: String,
    from_chain: String,
    to_token: String,
    to_chain: String,
    referrer: String,
    referrer_bps: u32,
    sdk_version: &'static str,
}

impl From<QuoteParams> for QuoteDynamicQuery {
    fn from(params: QuoteParams) -> Self {
        Self {
            amount_in64: params.amount_in64,
            from_token: params.from_token,
            from_chain: params.from_chain,
            to_token: params.to_token,
            to_chain: params.to_chain,
            referrer: params.referrer,
            referrer_bps: params.referrer_bps,
            sdk_version: SDK_VERSION,
        }
    }
}

impl<C> MayanClient<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub async fn get_quotes(&self, params: QuoteParams, input_decimals: u32) -> Result<Vec<MayanQuote>, SwapperError> {
        let path = quote_path(params)?;
        let response = self.client.get::<QuoteResponse>(&path).await.map_err(|err| map_quote_error(err, input_decimals))?;
        Ok(response.quotes)
    }
}

fn quote_path(params: QuoteParams) -> Result<String, SwapperError> {
    let defaults = serde_urlencoded::to_string(QUOTE_DEFAULTS)?;
    let query = serde_urlencoded::to_string(QuoteDynamicQuery::from(params))?;
    Ok(format!("/quote?{defaults}&{query}"))
}

fn map_quote_error(err: ClientError, decimals: u32) -> SwapperError {
    match err {
        ClientError::Http { status, body } => {
            if let Ok(response) = serde_json::from_slice::<ErrorResponse>(&body) {
                return map_response_error(&response, decimals);
            }
            SwapperError::ComputeQuoteError(format!("HTTP error: status {}", status))
        }
        other => SwapperError::from(other),
    }
}

fn map_response_error(response: &ErrorResponse, decimals: u32) -> SwapperError {
    if response.is_input_amount_error() {
        return SwapperError::InputAmountError {
            min_amount: response.min_amount_in(decimals),
        };
    }

    SwapperError::compute_quote_error(response.message().unwrap_or("Unknown Mayan error"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mayan::model::ErrorData;

    #[test]
    fn test_quote_path() {
        let path = quote_path(QuoteParams {
            amount_in64: "1000000".to_string(),
            from_token: "0x0000000000000000000000000000000000000000".to_string(),
            from_chain: "ethereum".to_string(),
            to_token: "So11111111111111111111111111111111111111112".to_string(),
            to_chain: "solana".to_string(),
            referrer: "0x1111111111111111111111111111111111111111".to_string(),
            referrer_bps: 50,
        })
        .unwrap();

        assert_eq!(
            path,
            "/quote?wormhole=false&swift=true&mctp=true&shuttle=false&fastMctp=true&gasless=false&onlyDirect=false&fullList=false&monoChain=true&solanaProgram=FC4eXxkyrMPTjiYUpp4EAnkmwMbQyZ6NDCh1kfLn6vsf&forwarderAddress=0x337685fdaB40D39bd02028545a4FfA7D287cC3E2&slippageBps=auto&amountIn64=1000000&fromToken=0x0000000000000000000000000000000000000000&fromChain=ethereum&toToken=So11111111111111111111111111111111111111112&toChain=solana&referrer=0x1111111111111111111111111111111111111111&referrerBps=50&sdkVersion=14_1_0"
        );
    }

    #[test]
    fn test_map_response_error() {
        let response = ErrorResponse {
            msg: Some("Amount too small (min ~0.0004349 ETH)".to_string()),
            message: None,
            data: Some(ErrorData {
                min_amount_in: Some(serde_json::json!(0.0004349)),
            }),
        };
        assert_eq!(
            map_response_error(&response, 18),
            SwapperError::InputAmountError {
                min_amount: Some("434900000000000".to_string())
            }
        );

        let response = ErrorResponse {
            msg: Some("Amount too small (min ~1,234.5 USDC)".to_string()),
            message: None,
            data: None,
        };
        assert_eq!(
            map_response_error(&response, 6),
            SwapperError::InputAmountError {
                min_amount: Some("1234500000".to_string())
            }
        );

        let response = ErrorResponse {
            msg: Some("Amount too small".to_string()),
            message: None,
            data: None,
        };
        assert_eq!(map_response_error(&response, 6), SwapperError::InputAmountError { min_amount: None });

        let response = ErrorResponse {
            msg: None,
            message: Some("Route not found".to_string()),
            data: None,
        };
        assert_eq!(map_response_error(&response, 6), SwapperError::ComputeQuoteError("Route not found".to_string()));
    }
}
