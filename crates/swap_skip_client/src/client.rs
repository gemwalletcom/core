use crate::models::{
    msgs::{MsgRequest, MsgResponse},
    route::{RouteRequest, RouteResponse},
};
use primitives::{Chain, SwapProvider, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest};
pub struct SkipApi {
    client: reqwest::Client,
    base_url: String,
    client_id: String,
    fee_bps: u32,
}

impl SkipApi {
    pub fn new(base_url: String, client_id: String, fee_bps: u32) -> Self {
        let client = reqwest::Client::builder().build().unwrap();
        Self {
            client,
            base_url,
            client_id,
            fee_bps,
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

    pub async fn get_msgs(
        &self,
        request: MsgRequest,
    ) -> Result<MsgResponse, Box<dyn std::error::Error + Send + Sync>> {
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
        let mut skip_request: RouteRequest = request.clone().into();
        skip_request.cumulative_affiliate_fee_bps = self.fee_bps.to_string();
        skip_request.client_id = self.client_id.clone();

        let response = self.get_route(skip_request).await?;
        let mut quote = SwapQuote {
            chain_type: request.from_asset.chain.chain_type(),
            from_amount: response.amount_in.clone().to_string(),
            to_amount: response.amount_out.to_string(),
            fee_percent: self.fee_bps as f32 / 100.0,
            provider: self.provider(),
            data: None,
        };

        if request.include_data {
            let msg_request = self.msg_request_from_quote_response(&request, &response);
            let msg_response = self.get_msgs(msg_request).await?;
            let msgs = msg_response.msgs;
            quote.data = Some(SwapQuoteData {
                to: request.destination_address.clone(),
                value: String::from("0"),
                data: serde_json::to_string(&msgs).unwrap(),
            });
        }

        Ok(quote)
    }

    fn msg_request_from_quote_response(
        &self,
        request: &SwapQuoteProtocolRequest,
        response: &RouteResponse,
    ) -> MsgRequest {
        MsgRequest {
            source_asset_denom: response.source_asset_denom.clone(),
            source_asset_chain_id: response.source_asset_chain_id.clone(),
            dest_asset_denom: response.dest_asset_denom.clone(),
            dest_asset_chain_id: response.dest_asset_chain_id.clone(),
            amount_in: response.amount_in.clone().to_string(),
            amount_out: response.amount_out.to_string(),
            estimated_amount_out: response.estimated_amount_out.to_string(),
            operations: response.operations.clone(),
            address_list: vec![
                request.wallet_address.clone(),
                request.destination_address.clone(),
            ],
            // TODO: add slippage to SwapQuoteProtocolRequest
            slippage_tolerance_percent: "1".to_string(),
            client_id: self.client_id.clone(),
        }
    }
}

impl From<SwapQuoteProtocolRequest> for RouteRequest {
    fn from(request: SwapQuoteProtocolRequest) -> Self {
        RouteRequest {
            amount_in: request.amount.to_string(),
            source_asset_denom: request.from_asset.to_string(),
            source_asset_chain_id: request.from_asset.chain.network_id().to_string(),
            dest_asset_denom: request.to_asset.to_string(),
            dest_asset_chain_id: request.to_asset.chain.network_id().to_string(),
            cumulative_affiliate_fee_bps: String::from("0"),
            allow_multi_tx: false,
            client_id: String::new(),
        }
    }
}
