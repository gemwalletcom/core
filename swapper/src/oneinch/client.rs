use primitives::{SwapQuote, SwapQuoteProtocolRequest};

use super::model::{QuoteRequest, Allowance, SwapResult, SwapResultTransaction};

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
            amount: quote.from_amount,
            slippage: 1.0,
            disable_estimate: true,
            fee: self.fee,
            referrer: self.fee_referral_address.clone(),
        };
        let swap_quote = self.get_swap_quote(quote_request, network_id).await?;
        
        if src.clone() != NATIVE_ADDRESS {
            let allowance = self.get_approve_allowance(network_id, src.as_str(), quote.wallet_address.as_str()).await?;
            if allowance.allowance == "0" {
                let approve = self.get_approve_transaction(network_id, src.as_str()).await?;
                let quote = SwapQuote {data: swap_quote.tx.get_data(), approval: Some(approve.get_data())};
                return Ok(quote)
            }
        }
        let quote = SwapQuote {data: swap_quote.tx.get_data(), approval: None};
        return Ok(quote)
    }

    pub async fn get_swap_quote(&self, request: QuoteRequest, network_id: &str) -> Result<SwapResult, Box<dyn std::error::Error + Send + Sync>>   {
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

    pub async fn get_approve_allowance(&self, network_id: &str, token_address: &str, wallet_address: &str) -> Result<Allowance, Box<dyn std::error::Error + Send + Sync>> {
        let params = serde_urlencoded::to_string(&[("tokenAddress", token_address), ("walletAddress", wallet_address)])?;
        let url = format!("{}/swap/{}/{}/approve/allowance?{}", self.api_url, self.version, network_id, params);
        let allowance = self.client
            .get(&url)
            .bearer_auth(self.api_key.as_str())
            .send()
            .await?
            .json::<Allowance>()
            .await?;
        Ok(allowance)
    }

    pub async fn get_approve_transaction(&self, network_id: &str, token_address: &str) -> Result<SwapResultTransaction, Box<dyn std::error::Error + Send + Sync>> {
        let params = serde_urlencoded::to_string(&[("tokenAddress", token_address)])?;
        let url = format!("{}/swap/{}/{}/approve/transaction?{}", self.api_url, self.version, network_id, params);
        let allowance = self.client
            .get(&url)
            .bearer_auth(self.api_key.as_str())
            .send()
            .await?
            .json::<SwapResultTransaction>()
            .await?;
        Ok(allowance)
    }
}