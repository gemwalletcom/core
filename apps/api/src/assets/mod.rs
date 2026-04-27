pub mod cilent;
mod filter;
mod model;

use crate::params::{AssetIdParam, SearchQueryParam};
use crate::responders::{ApiError, ApiResponse};
pub use cilent::{AssetsClient, SearchClient};
pub use model::SearchRequest;
use pricer::PriceClient;
use primitives::{AssetBasic, AssetFull, AssetId, DEFAULT_FIAT_CURRENCY, SearchResponse};
use rocket::{State, get, post, serde::json::Json, tokio::sync::Mutex};

#[get("/assets/<asset_id>?<currency>")]
pub async fn get_asset(
    asset_id: AssetIdParam,
    currency: Option<&str>,
    client: &State<Mutex<AssetsClient>>,
    price_client: &State<Mutex<PriceClient>>,
) -> Result<ApiResponse<AssetFull>, ApiError> {
    let asset = client.lock().await.get_asset_full(&asset_id.0)?;
    let currency = currency.unwrap_or(DEFAULT_FIAT_CURRENCY);
    let rate = price_client.lock().await.get_fiat_rate(currency)?.rate;
    Ok(asset.with_rate(rate).into())
}

#[post("/assets?<currency>", format = "json", data = "<asset_ids>")]
pub async fn get_assets(
    asset_ids: Json<Vec<AssetId>>,
    currency: Option<&str>,
    client: &State<Mutex<AssetsClient>>,
    price_client: &State<Mutex<PriceClient>>,
) -> Result<ApiResponse<Vec<AssetBasic>>, ApiError> {
    let currency = currency.unwrap_or(DEFAULT_FIAT_CURRENCY);
    let rate = price_client.lock().await.get_fiat_rate(currency)?.rate;

    Ok(client.lock().await.get_assets(asset_ids.0, rate)?.into())
}

#[get("/assets/search?<query>&<chains>&<tags>&<limit>&<offset>")]
pub async fn get_assets_search(
    query: SearchQueryParam,
    chains: Option<&str>,
    tags: Option<&str>,
    limit: Option<usize>,
    offset: Option<usize>,
    client: &State<Mutex<SearchClient>>,
) -> Result<ApiResponse<Vec<AssetBasic>>, ApiError> {
    let request = SearchRequest::new(&query.0, chains, tags, limit, offset);
    Ok(client.lock().await.get_assets_search(&request).await?.into())
}

#[get("/search?<query>&<chains>&<tags>&<limit>&<offset>")]
pub async fn get_search(
    query: SearchQueryParam,
    chains: Option<&str>,
    tags: Option<&str>,
    limit: Option<usize>,
    offset: Option<usize>,
    client: &State<Mutex<SearchClient>>,
) -> Result<ApiResponse<SearchResponse>, ApiError> {
    let request = SearchRequest::new(&query.0, chains, tags, limit, offset);

    let search_client = client.lock().await;
    let assets = search_client.get_assets_search(&request).await?;
    let perpetuals = search_client.get_perpetuals_search(&request).await?;
    let nfts = search_client.get_nfts_search(&request).await?;

    Ok(SearchResponse { assets, perpetuals, nfts }.into())
}
