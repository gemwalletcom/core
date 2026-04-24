use std::sync::Arc;

use rocket::{State, get};

use crate::params::{AddressParam, ChainParam, NftAssetIdParam, NftCollectionIdParam};
use crate::responders::{ApiError, ApiResponse};
use ::nft::NFTProviderClient;
use primitives::{NFTAsset, NFTCollection, NFTData};

#[get("/chain/nft/assets/<asset_id>")]
pub async fn get_nft_asset(asset_id: NftAssetIdParam, client: &State<Arc<NFTProviderClient>>) -> Result<ApiResponse<NFTAsset>, ApiError> {
    Ok(client.get_nft_asset(asset_id.0).await?.into())
}

#[get("/chain/nft/collections/<collection_id>")]
pub async fn get_nft_collection(collection_id: NftCollectionIdParam, client: &State<Arc<NFTProviderClient>>) -> Result<ApiResponse<NFTCollection>, ApiError> {
    Ok(client.get_nft_collection(collection_id.0).await?.into())
}

#[get("/chain/address/<chain>/<address>/nfts")]
pub async fn get_nfts(chain: ChainParam, address: AddressParam, client: &State<Arc<NFTProviderClient>>) -> Result<ApiResponse<Vec<NFTData>>, ApiError> {
    Ok(client.get_nft_data(chain.0, &address.0).await?.into())
}
