use crate::models::{
    msgs::{MsgRequest, MsgResponse},
    route::{RouteRequest, RouteResponse},
};
use primitives::{Chain, SwapProvider, SwapQuote, SwapQuoteProtocolRequest};
pub struct SkipApi {
    client: reqwest::Client,
    base_url: String,
    client_id: String,
}

impl SkipApi {
    pub fn new(base_url: String, client_id: String) -> Self {
        let client = reqwest::Client::builder().build().unwrap();
        Self {
            client,
            base_url,
            client_id,
        }
    }

    pub async fn get_route(
        &self,
        request: RouteRequest,
    ) -> Result<RouteResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/v2/fungible/route", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<RouteResponse>()
            .await?;
        Ok(response)
    }

    pub async fn get_msgs(
        &self,
        request: MsgRequest,
    ) -> Result<MsgResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/v2/fungible/msgs", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<MsgResponse>()
            .await?;
        Ok(response)
    }
}

// Fixme add SwapProvider trait
impl SkipApi {
    pub fn provider(&self) -> SwapProvider {
        SwapProvider {
            name: "Skip".to_string(),
        }
    }

    pub fn chains(&self) -> Vec<Chain> {
        vec![
            Chain::Cosmos,
            Chain::Osmosis,
            Chain::Celestia,
            Chain::Injective,
            Chain::Sei,
            // Chain::Noble
        ]
    }

    pub async fn get_quote(
        &self,
        request: SwapQuoteProtocolRequest,
    ) -> Result<SwapQuote, Box<dyn std::error::Error>> {
        // FIXME add From/Into methods
        let skip_request = RouteRequest {
            amount_in: request.amount.to_string(),
            source_asset_denom: request.from_asset.to_string(),
            source_asset_chain_id: request.from_asset.chain.network_id().to_string(),
            dest_asset_denom: request.to_asset.to_string(),
            dest_asset_chain_id: request.to_asset.chain.network_id().to_string(),
            cumulative_affiliate_fee_bps: "3".to_string(),
            allow_multi_tx: false,
            client_id: self.client_id.clone(),
        };
        let response = self.get_route(skip_request).await?;
        if request.include_data {
            // FIXME call get_msgs
        }
        let quote = SwapQuote {
            chain_type: request.from_asset.chain.chain_type(),
            from_amount: request.amount.to_string(),
            to_amount: response.amount_out.to_string(),
            fee_percent: 0.0,
            provider: self.provider(),
            data: None,
        };
        Ok(quote)
    }
}
