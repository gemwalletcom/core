use super::{
    ChainflipEnvironment, ChainflipIngressEgress, VaultSwapExtras, VaultSwapResponse,
    model::{ChainflipAsset, DcaParameters, DepositAddressResponse, RefundParameters},
};
use crate::swapper::SwapperError;
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use serde_json::{Value, json};
use std::fmt::Debug;

const RPC_PATH: &str = "/rpc";
const RPC_KEY: &str = "ed08651813cc4d4798bf9b953b5d33fb";

#[derive(Clone, Debug)]
pub struct BrokerClient<C>
where
    C: Client + Clone + Debug,
{
    client: JsonRpcClient<C>,
}

impl<C> BrokerClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: JsonRpcClient<C>) -> Self {
        Self { client }
    }

    pub async fn get_swap_limits(&self) -> Result<ChainflipIngressEgress, SwapperError> {
        let result = self
            .client
            .call_method_with_param("cf_environment", json!([]), Some(60 * 60 * 24 * 30))
            .await
            .map_err(SwapperError::from)?;

        let env: ChainflipEnvironment = result.take().map_err(SwapperError::from)?;
        Ok(env.ingress_egress)
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
            Value::Null,
            boost_fee,
            Vec::<Value>::new(),
            refund_params,
            dca_params,
        ]);

        let result = self
            .client
            .call_method_with_param("broker_request_swap_deposit_address", params, None)
            .await
            .map_err(SwapperError::from)?;

        result.take().map_err(SwapperError::from)
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
        let extra_params_json = match extra_params {
            VaultSwapExtras::Evm(evm) => serde_json::to_value(evm).unwrap(),
            VaultSwapExtras::Bitcoin(btc) => serde_json::to_value(btc).unwrap(),
            VaultSwapExtras::Solana(sol) => serde_json::to_value(sol).unwrap(),
            VaultSwapExtras::None => Value::Null,
        };

        let params = json!([
            source_asset,
            destination_asset,
            destination_address,
            broker_commission,
            extra_params_json,
            Value::Null,
            boost_fee,
            Vec::<Value>::new(),
            dca_params,
        ]);

        let result = self
            .client
            .call_method_with_param("broker_request_swap_parameter_encoding", params, None)
            .await
            .map_err(SwapperError::from)?;

        result.take().map_err(SwapperError::from)
    }
}

pub fn build_broker_path(base_url: &str) -> String {
    format!("{base_url}{RPC_PATH}/{RPC_KEY}")
}
