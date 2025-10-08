use crate::SwapperError;
use gem_aptos::models::Resource;
use gem_client::{Client, ClientError};
use num_bigint::BigUint;
use std::fmt::Debug;
use std::str::FromStr;

use super::model::{PANCAKE_SWAP_APTOS_ADDRESS, TokenPairReserve};

#[derive(Clone, Debug)]
pub struct PancakeSwapAptosClient<C>
where
    C: Client + Clone + Debug,
{
    client: C,
}

impl<C> PancakeSwapAptosClient<C>
where
    C: Client + Clone + Debug,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_quote(&self, from_asset: &str, to_asset: &str, value: &str, slippage_bps: u32) -> Result<String, SwapperError> {
        let (asset1, asset2) = sort_assets(from_asset, to_asset);
        let address = PANCAKE_SWAP_APTOS_ADDRESS;
        let resource = format!("{address}::swap::TokenPairReserve<{asset1}, {asset2}>");
        let path = format!("/v1/accounts/{address}/resource/{resource}");

        let reserve: Resource<TokenPairReserve> = self.client.get(&path).await.map_err(map_client_error)?;

        let reserve_x = BigUint::from_str(reserve.data.reserve_x.as_str()).unwrap_or_default();
        let reserve_y = BigUint::from_str(reserve.data.reserve_y.as_str()).unwrap_or_default();

        let reserve_in = if asset1 == from_asset { reserve_x.clone() } else { reserve_y.clone() };
        let reserve_out = if asset1 == from_asset { reserve_y.clone() } else { reserve_x.clone() };
        let amount_in = BigUint::from_str(value).unwrap_or_default();

        let output = calculate_swap_output(reserve_in, reserve_out, amount_in, slippage_bps);

        Ok(output.to_string())
    }
}

fn calculate_swap_output(reserve_in: BigUint, reserve_out: BigUint, amount_in: BigUint, fee_bps: u32) -> BigUint {
    let bps_base = BigUint::from(10_000u32);
    let effective_fee = &bps_base - BigUint::from(fee_bps);
    let effective_amount_in = &amount_in * effective_fee / &bps_base;
    let numerator = &reserve_out * &effective_amount_in;
    let denominator = &reserve_in + &effective_amount_in;
    numerator / denominator
}

fn sort_assets<T: Ord + Clone>(asset1: T, asset2: T) -> (T, T) {
    if asset1 <= asset2 { (asset1, asset2) } else { (asset2, asset1) }
}

fn map_client_error(err: ClientError) -> SwapperError {
    SwapperError::from(err)
}
