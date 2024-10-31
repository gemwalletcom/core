use primitives::{AssetId, Chain};

#[allow(unused)]
pub struct ThorchainSwapClient {
    api_url: String,
    client: reqwest::Client,
}

const NATIVE_ADDRESS_DOGE: &str = "DOGE.DOGE";
const NATIVE_ADDRESS_RUNE: &str = "THOR.RUNE";
const NATIVE_ADDRESS_COSMOS: &str = "GAIA.ATOM";
const NATIVE_BITCOIN: &str = "BTC.BTC";
const NATIVE_LITECOIN: &str = "LTC.LTC";
const NATIVE_BSC_BNB: &str = "BSC.BNB";

impl ThorchainSwapClient {
    pub fn new(api_url: String) -> Self {
        let client = reqwest::Client::builder().build().unwrap();

        Self {
            client,
            api_url,
        }
    }

    pub fn get_asset(&self, asset_id: AssetId) -> Option<String> {
        match asset_id.chain {
            Chain::Thorchain => Some(NATIVE_ADDRESS_RUNE.into()),
            Chain::Doge => Some(NATIVE_ADDRESS_DOGE.into()),
            Chain::Cosmos => Some(NATIVE_ADDRESS_COSMOS.into()),
            Chain::Bitcoin => Some(NATIVE_BITCOIN.into()),
            Chain::Litecoin => Some(NATIVE_LITECOIN.into()),
            Chain::SmartChain => Some(NATIVE_BSC_BNB.into()),
            _ => None,
        }
    }
    //
    // pub async fn get_quote(&self, quote: SwapQuoteProtocolRequest) -> Result<SwapQuote, SwapError> {
    //     let from_asset = self.get_asset(quote.from_asset.clone())?;
    //     let to_asset = self.get_asset(quote.to_asset.clone())?;
    //
    //     let request = QuoteRequest {
    //         from_asset,
    //         to_asset,
    //         amount: quote.amount.clone(),
    //         destination: quote.destination_address.clone(),
    //         affiliate: self.fee_referral_address.clone(),
    //         affiliate_bps: (self.fee * 100.0) as i64,
    //     };
    //     let quote_swap = self.get_swap_quote(request).await?;
    //
    //     let data = if quote.include_data {
    //         let data = SwapQuoteData {
    //             to: quote_swap.inbound_address.unwrap_or_default(),
    //             value: quote.amount.clone(),
    //             data: quote_swap.memo,
    //         };
    //         Some(data)
    //     } else {
    //         None
    //     };
    //
    //     let quote = SwapQuote {
    //         chain_type: quote.from_asset.clone().chain.chain_type(),
    //         from_amount: quote.amount.clone(),
    //         to_amount: quote_swap.expected_amount_out.to_string(),
    //         fee_percent: self.fee as f32,
    //         provider: PROVIDER_NAME.into(),
    //         data,
    //         approval: None,
    //     };
    //     Ok(quote)
    // }
    //
    // pub async fn get_swap_quote(&self, request: QuoteRequest) -> Result<QuoteResponse, SwapError> {
    //     let url = format!("{}/thorchain/quote/swap", self.api_url);
    //     Ok(self
    //         .client
    //         .get(&url)
    //         .query(&request)
    //         .send()
    //         .await?
    //         .json::<QuoteResponse>()
    //         .await?)
    // }
}
