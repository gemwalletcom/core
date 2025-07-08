extern crate rocket;

use std::{str::FromStr, sync::Arc};

use rocket::{serde::json::Json, tokio::sync::Mutex, State};

use cacher::CacherClient;
use defi::DefiProviderFactory;
use primitives::{Chain, DeFiPortfolio};

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
}

// API route handlers

#[get("/defi/portfolio/<address>?<chains>")]
pub async fn get_portfolio(address: &str, chains: Option<String>, client: &State<Mutex<DeFiClient>>) -> Result<Json<DeFiPortfolio>, String> {
    let chains_vec = chains.map(|s| s.split(',').map(|s| s.trim().to_string()).collect());

    match client.lock().await.get_portfolio(address, chains_vec).await {
        Ok(portfolio) => Ok(Json(portfolio)),
        Err(e) => Err(e.to_string()),
    }
}
