use primitives::{ChainType, SwapProvider, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest};

use super::model::{QuoteDataRequest, QuoteDataResponse, QuoteRequest, QuoteResponse};

use blockchain::solana::spl_associated_token_account::get_associated_token_address;

const NATIVE_ADDRESS: &str = "So11111111111111111111111111111111111111112";
const REFERRAL_PROGRAM_ID: &str = "REFER4ZgmyYx9c6He5XfaTMiGfdLwRnkV4RPp9t9iF3";

pub struct JupiterClient {
    api_url: String,
    fee: f64,
    fee_referral_address: String,
    client: reqwest::Client,
}

impl JupiterClient {
    pub fn new(api_url: String, fee: f64, fee_referral_address: String) -> Self {
        let client = reqwest::Client::builder().build().unwrap();

        Self {
            client,
            api_url,
            fee,
            fee_referral_address,
        }
    }

    pub fn provider(&self) -> SwapProvider {
        SwapProvider {
            name: "Jupiter".to_string(),
        }
    }

    pub async fn get_quote(
        &self,
        quote: SwapQuoteProtocolRequest,
    ) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        let input_mint = if quote.from_asset.clone().is_native() {
            NATIVE_ADDRESS.to_string()
        } else {
            quote.from_asset.clone().token_id.unwrap()
        };
        let output_mint = if quote.to_asset.clone().is_native() {
            NATIVE_ADDRESS.to_string()
        } else {
            quote.to_asset.clone().token_id.unwrap()
        };

        let quote_request: QuoteRequest = QuoteRequest {
            input_mint: input_mint.clone(),
            output_mint,
            amount: quote.amount.clone(),
            platform_fee_bps: (self.fee * 100.0) as i32,
        };
        let swap_quote = self.get_swap_quote(quote_request).await?;
        let data = if quote.include_data {
            let fee_account = get_associated_token_address(
                REFERRAL_PROGRAM_ID,
                vec!["referral_ata"],
                &self.fee_referral_address.clone(),
                input_mint.as_str(),
            );

            println!(
                "&self.fee_referral_address.clone(): {}",
                &self.fee_referral_address.clone()
            );
            println!("fee_account: {}", fee_account);

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
        let data = SwapQuoteData::from_data(&quote_data.swap_transaction);
        Ok(data)
    }

    pub async fn get_swap_quote(
        &self,
        request: QuoteRequest,
    ) -> Result<QuoteResponse, Box<dyn std::error::Error + Send + Sync>> {
        let params = serde_urlencoded::to_string(&request)?;
        let url = format!("{}/v6/quote?{}", self.api_url, params);
        Ok(self
            .client
            .get(&url)
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
