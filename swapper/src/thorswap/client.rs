use primitives::{
    AssetId, Chain, SwapProvider, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest,
};

use super::model::{QuoteRequest, QuoteResponse};

pub struct ThorchainSwapClient {
    api_url: String,
    fee: f64,
    fee_referral_address: String,
    client: reqwest::Client,
}

const NATIVE_ADDRESS_DOGE: &str = "DOGE.DOGE";
const NATIVE_ADDRESS_RUNE: &str = "THOR.RUNE";
const NATIVE_ADDRESS_COSMOS: &str = "GAIA.ATOM";
const NATIVE_BITCOIN: &str = "BTC.BTC";
const NATIVE_LITECOIN: &str = "LTC.LTC";
const NATIVE_BSC_BNB: &str = "BSC.BNB";

impl ThorchainSwapClient {
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
            name: "Thorchain".to_string(),
        }
    }

    pub fn get_asset(
        &self,
        asset_id: AssetId,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if !asset_id.is_native() {
            return Err("not native asset".into());
        }
        match asset_id.chain {
            Chain::Thorchain => Ok(NATIVE_ADDRESS_RUNE.into()),
            Chain::Doge => Ok(NATIVE_ADDRESS_DOGE.into()),
            Chain::Cosmos => Ok(NATIVE_ADDRESS_COSMOS.into()),
            Chain::Bitcoin => Ok(NATIVE_BITCOIN.into()),
            Chain::Litecoin => Ok(NATIVE_LITECOIN.into()),
            Chain::SmartChain => Ok(NATIVE_BSC_BNB.into()),
            _ => Err(format!("asset {} not supported", asset_id.to_string()).into()),
        }
    }

    pub async fn get_quote(
        &self,
        quote: SwapQuoteProtocolRequest,
    ) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        let from_asset = self.get_asset(quote.from_asset.clone())?;
        let to_asset = self.get_asset(quote.to_asset.clone())?;

        let request = QuoteRequest {
            from_asset,
            to_asset,
            amount: quote.amount.clone(),
            destination: quote.destination_address.clone(),
            affiliate: self.fee_referral_address.clone(),
            affiliate_bps: (self.fee * 100.0) as i64,
        };
        let quote_swap = self.get_swap_quote(request).await?;

        let data = if quote.include_data {
            let data = SwapQuoteData {
                to: quote_swap.inbound_address.unwrap_or_default(),
                value: quote.amount.clone(),
                gas_limit: 0,
                data: quote_swap.memo,
            };
            Some(data)
        } else {
            None
        };

        let quote = SwapQuote {
            chain_type: quote.from_asset.clone().chain.chain_type(),
            from_amount: quote.amount.clone(),
            to_amount: quote_swap.expected_amount_out.to_string(),
            fee_percent: self.fee as f32,
            provider: self.provider(),
            data,
        };
        Ok(quote)
    }

    pub async fn get_swap_quote(
        &self,
        request: QuoteRequest,
    ) -> Result<QuoteResponse, Box<dyn std::error::Error + Send + Sync>> {
        let params = serde_urlencoded::to_string(&request)?;
        let url = format!("{}/thorchain/quote/swap?{}", self.api_url, params);
        Ok(self
            .client
            .get(&url)
            .send()
            .await?
            .json::<QuoteResponse>()
            .await?)
    }
}
