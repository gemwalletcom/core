use crate::models::route::{
    RouteRequest, RouteResponse, RouteWithDataRequest, RouteWithDataResponse,
};
use primitives::{Chain, SwapProvider, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest};
use std::collections::HashMap;
pub struct SkipApi {
    client: reqwest::Client,
    base_url: String,
    client_id: String,
    fee_bps: u32,
    fee_address: String, // FIXME need to convert based on dex, e.g. for osmosis, use osmo1
}

impl SkipApi {
    pub fn new(base_url: String, client_id: String, fee_bps: u32, fee_address: String) -> Self {
        let client = reqwest::Client::builder().build().unwrap();
        Self {
            client,
            base_url,
            client_id,
            fee_bps,
            fee_address,
        }
    }

    pub async fn get_route(
        &self,
        request: RouteRequest,
    ) -> Result<RouteResponse, Box<dyn std::error::Error + Send + Sync>> {
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

    pub async fn get_msgs_direct(
        &self,
        request: RouteWithDataRequest,
    ) -> Result<RouteWithDataResponse, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/v2/fungible/msgs_direct", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json::<RouteWithDataResponse>()
            .await?;
        Ok(response)
    }
}

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
            Chain::Noble,
        ]
    }

    pub async fn get_quote(
        &self,
        request: SwapQuoteProtocolRequest,
    ) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        // FIXME add affiliate, chain_ids_to_addresses, fee address
        if request.include_data {
            let skip_request: RouteWithDataRequest =
                RouteWithDataRequest::from(&request, "1".to_string(), self.client_id.clone());
            let response = self.get_msgs_direct(skip_request).await?;
            let quote = SwapQuote {
                chain_type: request.from_asset.chain.chain_type(),
                from_amount: response.route.amount_in.clone().to_string(),
                to_amount: response.route.amount_out.to_string(),
                fee_percent: self.fee_bps as f32 / 100.0,
                provider: self.provider(),
                data: Some(SwapQuoteData {
                    to: request.destination_address.clone(),
                    value: String::from("0"),
                    data: serde_json::to_string(&response.txs).unwrap(),
                }),
            };
            return Ok(quote);
        }

        let mut skip_request =
            RouteRequest::from(&request, self.fee_bps.to_string(), self.client_id.clone());
        skip_request.cumulative_affiliate_fee_bps = self.fee_bps.to_string();
        skip_request.client_id = self.client_id.clone();

        let response = self.get_route(skip_request).await?;
        let quote = SwapQuote {
            chain_type: request.clone().from_asset.chain.chain_type(),
            from_amount: response.amount_in.clone().to_string(),
            to_amount: response.amount_out.to_string(),
            fee_percent: self.fee_bps as f32 / 100.0,
            provider: self.provider(),
            data: None,
        };
        Ok(quote)
    }
}

impl RouteRequest {
    fn from(request: &SwapQuoteProtocolRequest, fee_bps: String, client_id: String) -> Self {
        RouteRequest {
            amount_in: request.amount.to_string(),
            source_asset_denom: request.from_asset.to_string(),
            source_asset_chain_id: request.from_asset.chain.network_id().to_string(),
            dest_asset_denom: request.to_asset.to_string(),
            dest_asset_chain_id: request.to_asset.chain.network_id().to_string(),
            cumulative_affiliate_fee_bps: fee_bps,
            allow_multi_tx: false,
            client_id,
        }
    }
}

impl RouteWithDataRequest {
    fn from(
        request: &SwapQuoteProtocolRequest,
        slippage_tolerance_percent: String,
        client_id: String,
    ) -> Self {
        RouteWithDataRequest {
            amount_in: request.amount.to_string(),
            source_asset_denom: request.from_asset.to_string(),
            source_asset_chain_id: request.from_asset.chain.network_id().to_string(),
            dest_asset_denom: request.to_asset.to_string(),
            dest_asset_chain_id: request.to_asset.chain.network_id().to_string(),
            allow_multi_tx: false,
            slippage_tolerance_percent,
            affiliate: vec![],
            client_id,
            chain_ids_to_addresses: HashMap::new(),
        }
    }
}
