use super::model::{QuoteDataRequest, QuoteDataResponse, QuoteRequest, QuoteResponse};
use blockchain::solana::WSOL_TOKEN_ADDRESS;
use primitives::{ChainType, SwapProvider, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest};

const PROGRAM_ADDRESS: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
const JUPITER: &str = "Jupiter";

pub struct JupiterClient {
    api_url: String,
    fee: f64,
    fee_referral_key: String,
    client: reqwest::Client,
}

impl JupiterClient {
    pub fn new(api_url: String, fee: f64, fee_referral_key: String) -> Self {
        let client = reqwest::Client::builder().build().unwrap();

        Self {
            client,
            api_url,
            fee,
            fee_referral_key,
        }
    }

    pub fn provider(&self) -> SwapProvider {
        SwapProvider {
            name: String::from(JUPITER),
        }
    }

    pub async fn get_quote(
        &self,
        quote: SwapQuoteProtocolRequest,
    ) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        let input_mint = if quote.from_asset.clone().is_native() {
            WSOL_TOKEN_ADDRESS.to_string()
        } else {
            quote.from_asset.clone().token_id.unwrap()
        };
        let output_mint = if quote.to_asset.clone().is_native() {
            WSOL_TOKEN_ADDRESS.to_string()
        } else {
            quote.to_asset.clone().token_id.unwrap()
        };

        let quote_request: QuoteRequest = QuoteRequest {
            input_mint,
            output_mint: output_mint.clone(),
            amount: quote.amount.clone(),
            platform_fee_bps: (self.fee * 100.0) as i32,
            slippage_bps: 100, // 1%
            only_direct_routes: true,
        };
        let swap_quote = self.get_swap_quote(quote_request).await?;
        let data = if quote.include_data {
            let fee_account =
                super::referral::get_referral_account(&self.fee_referral_key, &output_mint);
            let data = self
                .get_data(quote.clone(), swap_quote.clone(), fee_account)
                .await?;
            Some(data)
        } else {
            None
        };

        let quote = SwapQuote {
            chain_type: ChainType::Solana,
            from_amount: quote.amount.clone(),
            to_amount: swap_quote.out_amount.clone(),
            fee_percent: self.fee as f32,
            provider: self.provider(),
            data,
        };
        Ok(quote)
    }

    pub async fn get_data(
        &self,
        quote: SwapQuoteProtocolRequest,
        quote_response: QuoteResponse,
        fee_account: String,
    ) -> Result<SwapQuoteData, Box<dyn std::error::Error + Send + Sync>> {
        let request = QuoteDataRequest {
            user_public_key: quote.wallet_address,
            fee_account,
            quote_response,
            compute_unit_price_micro_lamports: "auto".into(),
        };
        let quote_data = self.get_swap_quote_data(request).await?;
        let data = SwapQuoteData {
            to: PROGRAM_ADDRESS.to_string(),
            value: "".to_string(),
            data: quote_data.swap_transaction,
        };
        Ok(data)
    }

    pub async fn get_swap_quote(
        &self,
        request: QuoteRequest,
    ) -> Result<QuoteResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/v6/quote", self.api_url);
        Ok(self
            .client
            .get(&url)
            .query(&request)
            .send()
            .await?
            .json::<QuoteResponse>()
            .await?)
    }

    pub async fn get_swap_quote_data(
        &self,
        request: QuoteDataRequest,
    ) -> Result<QuoteDataResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/v6/swap", self.api_url);
        let response = self.client.post(&url).json(&request).send().await?;
        Ok(response.json().await?)
    }
}
