use super::{
    model::{ChainflipAsset, DcaParameters, DepositAddressResponse, RefundParameters},
    ChainflipEnvironment, ChainflipIngressEgress, VaultSwapExtras, VaultSwapResponse,
};
use crate::{
    network::{AlienClient, AlienProvider, JsonRpcClient},
    swapper::SwapperError,
};
use serde_json::json;
use std::sync::Arc;

const CHAINFLIP_BROKER_URL: &str = "https://chainflip-broker.io";
const CHAINFLIP_BROKER_KEY: &str = "ed08651813cc4d4798bf9b953b5d33fb";

#[derive(Debug)]
pub struct BrokerClient {
    client: JsonRpcClient<AlienClient>,
}

impl BrokerClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        let endpoint = format!("{CHAINFLIP_BROKER_URL}/rpc/{CHAINFLIP_BROKER_KEY}");
        let alien_client = AlienClient::new(endpoint.clone(), provider);
        Self {
            client: JsonRpcClient::new(endpoint, alien_client),
        }
    }

    pub async fn get_swap_limits(&self) -> Result<ChainflipIngressEgress, SwapperError> {
        self.client
            .call_method_with_param("cf_environment", json!([]), Some(60 * 60 * 24 * 30))
            .await
            .map_err(SwapperError::from)
            .map(|x| x.take().map_err(SwapperError::from))?
            .map(|x: ChainflipEnvironment| x.ingress_egress)
    }

    pub async fn get_deposit_address(
        &self,
        src_asset: ChainflipAsset,
        dst_asset: ChainflipAsset,
        dst_address: String,
        broker_commission_bps: u32,
        boost_fee: Option<u32>,
        refund_params: Option<RefundParameters>,
        dca_params: Option<DcaParameters>,
    ) -> Result<DepositAddressResponse, SwapperError> {
        let params = json!([
            src_asset,
            dst_asset,
            dst_address,
            broker_commission_bps,
            null,      // channel_metadata
            boost_fee, // boost_fee
            [],        // affiliate_fees
            refund_params,
            dca_params,
        ]);

        self.client
            .call_method_with_param("broker_request_swap_deposit_address", params, None)
            .await
            .map_err(SwapperError::from)
            .map(|x| x.take().map_err(SwapperError::from))?
    }

    pub async fn encode_vault_swap(
        &self,
        source_asset: ChainflipAsset,
        destination_asset: ChainflipAsset,
        destination_address: String,
        broker_commission: u32,
        boost_fee: Option<u32>,
        extra_params: VaultSwapExtras,
        dca_params: Option<DcaParameters>,
    ) -> Result<VaultSwapResponse, SwapperError> {
        let extra_params: serde_json::Value = match extra_params {
            VaultSwapExtras::Evm(evm) => serde_json::to_value(evm).unwrap(),
            VaultSwapExtras::Bitcoin(btc) => serde_json::to_value(btc).unwrap(),
            VaultSwapExtras::Solana(solana) => serde_json::to_value(solana).unwrap(),
            VaultSwapExtras::None => serde_json::json!(null),
        };

        let params = json!([
            source_asset,
            destination_asset,
            destination_address,
            broker_commission,
            extra_params,
            null,      // channel_metadata
            boost_fee, // boost_fee
            [],        // affiliate_fees
            dca_params,
        ]);
        self.client
            .call_method_with_param("broker_request_swap_parameter_encoding", params, None)
            .await
            .map_err(SwapperError::from)
            .map(|x| x.take().map_err(SwapperError::from))?
    }
}
