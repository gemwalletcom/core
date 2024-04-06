use crate::api::SkipApi;
use crate::models::route::{
    RouteRequest, RouteResponse, RouteWithDataRequest, RouteWithDataResponse,
};
use async_trait::async_trait;
use primitives::{Chain, SwapQuote, SwapQuoteData, SwapQuoteProtocolRequest};
use reqwest_enum::provider::{JsonProviderType, Provider};
use std::collections::HashMap;
use swap_provider::{SwapError, SwapProvider, DEFAULT_SWAP_SLIPPAGE};

pub struct SkipProvider {
    provider: Provider<SkipApi>,
    client_id: String,
    fee_bps: u32,
    _fee_address: String, // FIXME need to convert based on dex, e.g. for osmosis, use osmo1
}

impl SkipProvider {
    pub fn new(client_id: String, fee_bps: u32, fee_address: String) -> Self {
        Self {
            provider: Provider::<SkipApi>::default(),
            client_id,
            fee_bps,
            _fee_address: fee_address,
        }
    }
}

#[async_trait]
impl SwapProvider for SkipProvider {
    fn provider(&self) -> primitives::SwapProvider {
        "Skip".into()
    }

    fn supported_chains(&self) -> Vec<primitives::Chain> {
        vec![
            Chain::Cosmos,
            Chain::Osmosis,
            Chain::Celestia,
            Chain::Injective,
            Chain::Sei,
            Chain::Noble,
        ]
    }

    async fn get_quote(
        &self,
        request: primitives::SwapQuoteProtocolRequest,
    ) -> Result<primitives::SwapQuote, SwapError> {
        // FIXME add affiliate, chain_ids_to_addresses, fee address
        if request.include_data {
            let skip_request: RouteWithDataRequest = RouteWithDataRequest::from(
                &request,
                DEFAULT_SWAP_SLIPPAGE.to_string(),
                self.client_id.clone(),
            );
            let response: RouteWithDataResponse = self
                .provider
                .request_json(SkipApi::MsgsDirect(skip_request))
                .await?;
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

        let response: RouteResponse = self
            .provider
            .request_json(SkipApi::Route(skip_request))
            .await?;
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
