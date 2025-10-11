use super::models::{CetusPool, Request, Response};
use crate::{SwapperError, alien::X_CACHE_TTL};
use gem_client::Client;
use std::collections::HashMap;

pub const CETUS_API_URL: &str = "https://api-sui.cetus.zone/v2";
const POOL_CACHE_TTL: u64 = 60 * 5; // 5 minutes

#[derive(Clone, Debug)]
pub struct CetusClient<C>
where
    C: Client + Clone,
{
    client: C,
}

impl<C> CetusClient<C>
where
    C: Client + Clone,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_pool_by_token(&self, token_a: &str, token_b: &str) -> Result<Vec<CetusPool>, SwapperError> {
        let request = Request {
            display_all_pools: true,
            has_mining: true,
            no_incentives: true,
            coin_type: format!("{token_a},{token_b}"),
        };
        let query = serde_urlencoded::to_string(&request).unwrap();
        let path = format!("/sui/stats_pools?{query}");
        let headers = Some(HashMap::from([(X_CACHE_TTL.to_string(), POOL_CACHE_TTL.to_string())]));

        let response: Response = self.client.get_with_headers(&path, headers).await.map_err(SwapperError::from)?;

        if response.code != 200 {
            return Err(SwapperError::NetworkError(format!("API error: {}", response.msg)));
        }

        Ok(response.data.lp_list)
    }
}
