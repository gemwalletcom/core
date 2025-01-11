extern crate rocket;
use std::{error::Error, vec};

use primitives::{Asset, AssetBasic, AssetFull, Chain};
use search_index::{AssetDocument, SearchIndexClient, ASSETS_INDEX_NAME};
use settings_chain::ChainProviders;
use storage::DatabaseClient;

pub struct AssetsClient {
    database: DatabaseClient,
}

impl AssetsClient {
    pub async fn new(database_url: &str) -> Self {
        let database = DatabaseClient::new(database_url);
        Self { database }
    }

    pub fn add_asset(&mut self, asset: Asset) -> Result<usize, Box<dyn Error>> {
        let _ = self.database.add_assets(vec![storage::models::Asset::from_primitive(asset.clone())])?;
        self.database.set_swap_enabled(vec![asset.id.to_string()], asset.id.chain.is_swap_supported())?;
        self.database.update_asset_rank(&asset.id.to_string(), 15)?;
        Ok(1)
    }

    #[allow(unused)]
    pub fn get_asset(&mut self, asset_id: &str) -> Result<Asset, Box<dyn Error>> {
        Ok(self.database.get_asset(asset_id)?.as_primitive())
    }

    pub fn get_assets_list(&mut self) -> Result<Vec<AssetBasic>, Box<dyn Error>> {
        let assets = self.database.get_assets_list()?.into_iter().map(|x| x.as_basic_primitive()).collect();
        Ok(assets)
    }

    pub fn get_assets(&mut self, asset_ids: Vec<String>) -> Result<Vec<AssetBasic>, Box<dyn Error>> {
        let assets = self.database.get_assets(asset_ids)?.into_iter().map(|x| x.as_basic_primitive()).collect();
        Ok(assets)
    }

    pub fn get_asset_full(&mut self, asset_id: &str) -> Result<AssetFull, Box<dyn Error>> {
        let asset = self.database.get_asset(asset_id)?;
        let links = self.database.get_asset_links(asset_id)?.into_iter().map(|link| link.as_primitive()).collect();

        Ok(AssetFull {
            asset: asset.as_primitive(),
            properties: asset.as_property_primitive(),
            links,
            score: asset.as_score_primitive(),
        })
    }

    pub fn get_assets_ids_by_device_id(&mut self, device_id: &str, wallet_index: i32, from_timestamp: Option<u32>) -> Result<Vec<String>, Box<dyn Error>> {
        let subscriptions = self.database.get_subscriptions_by_device_id_wallet_index(device_id, wallet_index)?;

        let addresses = subscriptions.clone().into_iter().map(|x| x.address).collect();
        let chains = subscriptions.clone().into_iter().map(|x| x.chain).collect::<Vec<_>>();

        let assets_ids = self.database.get_assets_ids_by_device_id(addresses, chains, from_timestamp)?;
        Ok(assets_ids)
    }
}

pub struct AssetsSearchClient {
    client: SearchIndexClient,
}

impl AssetsSearchClient {
    pub async fn new(client: &SearchIndexClient) -> Self {
        Self { client: client.clone() }
    }

    pub async fn get_assets_search(
        &mut self,
        query: &str,
        chains: Vec<String>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<primitives::AssetBasic>, Box<dyn Error + Send + Sync>> {
        let mut filters = vec![];
        filters.push("score.rank > 0".to_string());
        //filters.push("properties.isEnabled = true".to_string()); // Does not work, why?

        if !chains.is_empty() {
            let chain_filter = "asset.id.chain IN [\"".to_string() + &chains.join(",") + "\"]";
            filters.push(chain_filter);
        }
        let filter = &filters.join(" AND ");

        let assets: Vec<AssetDocument> = self.client.search(ASSETS_INDEX_NAME, query, filter, [].as_ref(), limit, offset).await?;

        Ok(assets
            .into_iter()
            .map(|x| AssetBasic {
                asset: x.asset,
                properties: x.properties,
                score: x.score,
            })
            .collect())
    }
}

pub struct AssetsChainProvider {
    providers: ChainProviders,
}

impl AssetsChainProvider {
    pub fn new(providers: ChainProviders) -> Self {
        Self { providers }
    }

    pub async fn get_token_data(&self, chain: Chain, token_id: String) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>> {
        self.providers.get_token_data(chain, token_id).await
    }
}
