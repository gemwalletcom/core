use super::models::{CetusPool, Request, Response};
use crate::{
    network::{AlienProvider, AlienTarget},
    swapper::SwapperError,
};
use std::sync::Arc;

const CETUS_API_URL: &str = "https://api-sui.cetus.zone/v2";
const POOL_CACHE_TTL: u64 = 60 * 5; // 5 minutes

pub struct CetusClient {
    pub provider: Arc<dyn AlienProvider>,
}

impl CetusClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_pool_by_token(&self, token_a: &str, token_b: &str) -> Result<Vec<CetusPool>, SwapperError> {
        let request = Request {
            display_all_pools: true,
            has_mining: true,
            no_incentives: true,
            coin_type: format!("{token_a},{token_b}"),
        };
        let api = format!("{CETUS_API_URL}/sui/stats_pools");
        let query = serde_urlencoded::to_string(&request).unwrap();
        let url = format!("{api}?{query}");
        let mut target = AlienTarget::get(&url);
        target = target.set_cache_ttl(POOL_CACHE_TTL);

        let response = self.provider.request(target).await?;
        let response: Response = serde_json::from_slice(&response).map_err(|e| SwapperError::NetworkError(format!("Failed to parse json response: {e}")))?;

        if response.code != 200 {
            return Err(SwapperError::NetworkError(format!("API error: {}", response.msg)));
        }

        Ok(response.data.lp_list)
    }
}
