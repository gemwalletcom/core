use super::{
    model::{ChainflipAsset, DcaParameters, DepositAddressResponse, RefundParameters, VaultSwapExtraParams, VaultSwapResponse},
    ChainflipEnvironment,
};
use crate::{
    network::{
        jsonrpc::{jsonrpc_call_with_endpoint, jsonrpc_call_with_endpoint_cache},
        AlienProvider,
    },
    swapper::SwapperError,
};
use serde_json::json;
use std::sync::Arc;

const CHAINFLIP_BROKER_URL: &str = "https://chainflip-broker.io";
const CHAINFLIP_BROKER_KEY: &str = "ed08651813cc4d4798bf9b953b5d33fb";

#[derive(Debug)]
pub struct BrokerClient {
    provider: Arc<dyn AlienProvider>,
}

impl BrokerClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub fn get_endpoint(&self) -> String {
        format!("{}/rpc/{}", CHAINFLIP_BROKER_URL, CHAINFLIP_BROKER_KEY)
    }

    pub async fn get_swap_limits(&self) -> Result<ChainflipEnvironment, SwapperError> {
        jsonrpc_call_with_endpoint_cache(
            self.provider.clone(),
            &self.get_endpoint(),
            "cf_environment",
            json!([]),
            Some(60 * 60 * 24 * 30),
        )
        .await
        .map_err(SwapperError::from)
    }

    pub async fn get_deposit_address(
        &self,
        src_asset: ChainflipAsset,
        dst_asset: ChainflipAsset,
        dst_address: String,
        broker_commission_bps: u32,
        refund_params: Option<RefundParameters>,
        dca_params: Option<DcaParameters>,
    ) -> Result<DepositAddressResponse, SwapperError> {
        let params = json!([
            src_asset,
            dst_asset,
            dst_address,
            broker_commission_bps,
            null, // channel_metadata
            null, // boost_fee
            [],   // affiliate_fees
            refund_params,
            dca_params,
        ]);

        jsonrpc_call_with_endpoint(self.provider.clone(), &self.get_endpoint(), "broker_request_swap_deposit_address", params)
            .await
            .map_err(SwapperError::from)
    }

    pub async fn encode_vault_swap(
        &self,
        source_asset: ChainflipAsset,
        destination_asset: ChainflipAsset,
        destination_address: String,
        broker_commission: u32,
        extra_params: Option<VaultSwapExtraParams>,
        dca_params: Option<DcaParameters>,
    ) -> Result<VaultSwapResponse, SwapperError> {
        let params = json!([
            source_asset,
            destination_asset,
            destination_address,
            broker_commission,
            extra_params,
            null, // channel_metadata
            null, // boost_fee
            [],   // affiliate_fees
            dca_params,
        ]);
        jsonrpc_call_with_endpoint(self.provider.clone(), &self.get_endpoint(), "broker_request_swap_parameter_encoding", params)
            .await
            .map_err(SwapperError::from)
    }
}
