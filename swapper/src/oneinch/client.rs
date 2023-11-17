use primitives::{SwapQuote, SwapQuoteProtocolRequest, ChainType};

use super::model::{QuoteRequest, SwapResult};

pub struct OneInchClient {
    api_url: String,
    api_key: String,
    fee: f64,
    fee_referral_address: String,
    version: String,
    client: reqwest::Client,
}

const NATIVE_ADDRESS: &str = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

impl OneInchClient {
    pub fn new(api_url: String, api_key: String, fee: f64, fee_referral_address: String) -> Self {
        let client = reqwest::Client::builder()
            .build().unwrap();

        Self {
            client,
            api_url,
            api_key,
            fee,
            fee_referral_address,
            version: "v5.2".to_string(),
        }
    }

    pub async fn get_quote(&self, quote: SwapQuoteProtocolRequest) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        let network_id = quote.from_asset.chain.network_id();
        let src = if quote.from_asset.clone().is_native() { NATIVE_ADDRESS.to_string() } else { quote.from_asset.clone().token_id.unwrap() };
        let dst = if quote.to_asset.clone().is_native() { NATIVE_ADDRESS.to_string() } else { quote.to_asset.clone().token_id.unwrap() };
        let quote_request = QuoteRequest{
            src: src.clone(),
            dst,
            from: quote.wallet_address.clone(),
            amount: quote.amount,
            slippage: 1.0,
            disable_estimate: false,
            fee: self.fee,
            referrer: self.fee_referral_address.clone(),
        };

        let swap_quote = if quote.include_data {
            self.get_swap_quote_data(quote_request, network_id).await?
        } else {
            self.get_swap_quote(quote_request, network_id).await?
        };
        let data = if let Some(value) = swap_quote.tx { Some(value.get_data()) } else { None };

        let quote = SwapQuote {
            chain_type: ChainType::Ethereum, 
            to_amount: swap_quote.to_amount,
            fee_percent: self.fee as i32,
            data,
        };
        return Ok(quote)
    }

    pub async fn get_swap_quote(&self, request: QuoteRequest, network_id: &str) -> Result<SwapResult, Box<dyn std::error::Error + Send + Sync>>   {
        let params = serde_urlencoded::to_string(&request)?;
        let url = format!("{}/swap/{}/{}/quote?{}", self.api_url, self.version, network_id, params);
        return Ok(self.client
            .get(&url)
            .bearer_auth(self.api_key.as_str())
            .send()
            .await?
            .json::<SwapResult>()
            .await?);
    }

    pub async fn get_swap_quote_data(&self, request: QuoteRequest, network_id: &str) -> Result<SwapResult, Box<dyn std::error::Error + Send + Sync>>   {
        let params = serde_urlencoded::to_string(&request)?;
        let url = format!("{}/swap/{}/{}/swap?{}", self.api_url, self.version, network_id, params);
        return Ok(self.client
            .get(&url)
            .bearer_auth(self.api_key.as_str())
            .send()
            .await?
            .json::<SwapResult>()
            .await?);
    }
}