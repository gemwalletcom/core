use crate::{
    network::{AlienHttpMethod, AlienProvider, AlienTarget},
    swapper::SwapperError,
};
use gem_aptos::model::Resource;
use num_bigint::BigUint;
use std::{str::FromStr, sync::Arc};

use super::model::{TokenPairReserve, PANCAKE_SWAP_APTOS_ADDRESS};

#[derive(Debug)]
pub struct PancakeSwapAptosClient {
    provider: Arc<dyn AlienProvider>,
}

impl PancakeSwapAptosClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    fn calculate_swap_output(reserve_in: BigUint, reserve_out: BigUint, amount_in: BigUint, fee_bps: u32) -> BigUint {
        // Constants for basis points calculation
        let bps_base = BigUint::from(10_000u32); // 10,000 bps = 100%

        // Effective input after fee deduction
        let effective_fee = bps_base.clone() - BigUint::from(fee_bps);
        let effective_amount_in = &amount_in * effective_fee / &bps_base;

        // Calculate numerator and denominator
        let numerator = &reserve_out * &effective_amount_in;
        let denominator = &reserve_in + &effective_amount_in;

        // Final output
        numerator / denominator
    }

    fn sort_assets<T: Ord>(&self, asset1: T, asset2: T) -> (T, T) {
        if asset1 <= asset2 {
            (asset1, asset2)
        } else {
            (asset2, asset1)
        }
    }

    pub async fn get_quote(&self, endpoint: &str, from_asset: &str, to_asset: &str, value: &str, slippage_bps: u32) -> Result<String, SwapperError> {
        let (asset1, asset2) = self.sort_assets(from_asset, to_asset);
        let address = PANCAKE_SWAP_APTOS_ADDRESS;
        let resource = format!("{}::swap::TokenPairReserve<{}, {}>", address, asset1, asset2);
        let path = format!("/v1/accounts/{}/resource/{}", address, resource);
        let url = format!("{}{}", endpoint, path);

        let target = AlienTarget {
            url,
            method: AlienHttpMethod::Get,
            headers: None,
            body: None,
        };

        let data = self.provider.request(target).await.map_err(SwapperError::from)?;

        let result: Resource<TokenPairReserve> = serde_json::from_slice(&data).map_err(|e| SwapperError::NetworkError(e.to_string()))?;

        let reserve_x = BigUint::from_str(result.data.reserve_x.as_str()).unwrap_or_default();
        let reserve_y = BigUint::from_str(result.data.reserve_y.as_str()).unwrap_or_default();

        let reserve_in = if asset1 == from_asset { reserve_x.clone() } else { reserve_y.clone() };
        let reserve_out = if asset1 == from_asset { reserve_y.clone() } else { reserve_x.clone() };
        let amount_in = BigUint::from_str(value).unwrap_or_default();

        let value = Self::calculate_swap_output(reserve_in, reserve_out, amount_in, slippage_bps);

        Ok(value.to_string())
    }
}
