extern crate rocket;

use std::{str::FromStr, sync::Arc};

use rocket::{serde::json::Json, tokio::sync::Mutex, State};

use cacher::CacherClient;
use defi::DefiProviderFactory;
use primitives::{Chain, DeFiPortfolio, DeFiPosition, DeFiPositionFilters, DeFiPositionType, DeFiPositionsRequest};

pub struct DeFiClient {
    provider_factory: DefiProviderFactory,
    cache: Arc<Mutex<CacherClient>>,
}

impl DeFiClient {
    pub fn new(cache: CacherClient, debank_api_key: String) -> Self {
        let provider_factory = DefiProviderFactory::new().with_debank(debank_api_key);

        Self {
            provider_factory,
            cache: Arc::new(Mutex::new(cache)),
        }
    }

    pub async fn get_portfolio(&mut self, address: &str, chains: Option<Vec<String>>) -> Result<DeFiPortfolio, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("defi:portfolio:{address}:{chains:?}");

        if let Ok(cached) = self.cache.lock().await.get_value::<DeFiPortfolio>(&cache_key).await {
            return Ok(cached);
        }

        let parsed_chains = chains
            .map(|chain_strs| chain_strs.iter().filter_map(|s| Chain::from_str(s).ok()).collect())
            .unwrap_or_default();
        let providers_to_use = self.provider_factory.get_all_providers();
        if providers_to_use.is_empty() {
            return Err("No valid providers available".into());
        }

        let provider = &providers_to_use[0];
        let portfolio = provider
            .get_portfolio(address, parsed_chains)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Cache the result for 5 minutes
        let _ = self
            .cache
            .lock()
            .await
            .set_value_with_ttl(&cache_key, serde_json::to_string(&portfolio)?, 300)
            .await;

        Ok(portfolio)
    }

    pub async fn get_positions(&mut self, request: DeFiPositionsRequest) -> Result<Vec<DeFiPosition>, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("defi:positions:{}:{:?}", request.address, request.filters);

        if let Ok(cached) = self.cache.lock().await.get_value::<Vec<DeFiPosition>>(&cache_key).await {
            return Ok(cached);
        }

        let providers = self.provider_factory.get_all_providers();
        if providers.is_empty() {
            return Err("No providers available".into());
        }

        let provider = &providers[0];
        let positions = provider
            .get_positions(&request.address, request.filters)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Cache the result for 5 minutes
        let _ = self
            .cache
            .lock()
            .await
            .set_value_with_ttl(&cache_key, serde_json::to_string(&positions)?, 300)
            .await;

        Ok(positions)
    }
}

// API route handlers

#[get("/defi/portfolio/<address>?<chains>")]
pub async fn get_portfolio(address: &str, chains: Option<String>, client: &State<Mutex<DeFiClient>>) -> Json<Result<DeFiPortfolio, String>> {
    let chains_vec = chains.map(|s| s.split(',').map(|s| s.trim().to_string()).collect());

    match client.lock().await.get_portfolio(address, chains_vec).await {
        Ok(portfolio) => Json(Ok(portfolio)),
        Err(e) => Json(Err(e.to_string())),
    }
}

#[get("/defi/positions/<address>?<position_type>&<chains>")]
pub async fn get_positions(
    address: &str,
    position_type: Option<String>,
    chains: Option<String>,
    client: &State<Mutex<DeFiClient>>,
) -> Json<Result<Vec<DeFiPosition>, String>> {
    // Parse filters
    let position_types = position_type.map(|s| s.split(',').filter_map(|s| DeFiPositionType::from_str(s.trim()).ok()).collect());

    let chains_vec = chains
        .map(|s| s.split(',').filter_map(|s| Chain::from_str(s.trim()).ok()).collect::<Vec<Chain>>())
        .unwrap_or_default();

    let filters = if position_types.is_some() || !chains_vec.is_empty() {
        Some(DeFiPositionFilters {
            position_types,
            chains: chains_vec,
            protocols: None,
            has_debt: None,
            has_rewards: None,
        })
    } else {
        None
    };

    let request = DeFiPositionsRequest {
        address: address.to_string(),
        filters,
    };

    match client.lock().await.get_positions(request).await {
        Ok(positions) => Json(Ok(positions)),
        Err(e) => Json(Err(e.to_string())),
    }
}
