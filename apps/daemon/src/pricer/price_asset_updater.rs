use chain_primitives::format_token_id;
use coingecko::{get_chain_for_coingecko_platform_id, get_coingecko_market_id_for_chain, Coin, CoinGeckoClient};
use pricer::PriceClient;
use primitives::Chain;
use storage::models::PriceAsset;

pub struct PriceAssetUpdater {
    coin_gecko_client: CoinGeckoClient,
    price_client: PriceClient,
}

impl PriceAssetUpdater {
    pub fn new(price_client: PriceClient, coin_gecko_client: CoinGeckoClient) -> Self {
        Self {
            coin_gecko_client,
            price_client,
        }
    }

    pub async fn update_prices_assets(&mut self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        // native assets
        let chains_assets = Chain::all()
            .into_iter()
            .map(|x| PriceAsset {
                asset_id: x.as_ref().to_string(),
                price_id: get_coingecko_market_id_for_chain(x).to_string(),
            })
            .collect::<Vec<_>>();
        self.price_client.set_prices_assets(chains_assets.clone())?;

        // assets
        let coin_list = self.coin_gecko_client.get_coin_list().await?;

        let assets = coin_list.into_iter().flat_map(|x| self.get_prices_assets_for_coin(x)).collect::<Vec<_>>();

        self.price_client.set_prices_assets(assets.clone())?;

        Ok(chains_assets.len() + assets.len())
    }

    pub fn get_prices_assets_for_coin(&mut self, coin: Coin) -> Vec<PriceAsset> {
        coin.platforms
            .clone()
            .into_iter()
            .flat_map(|(platform, token_id)| {
                let platform = get_chain_for_coingecko_platform_id(platform.as_str());
                if let Some(chain) = platform {
                    let token_id = token_id.unwrap_or_default();
                    if !token_id.is_empty()
                        && let Some(asset_id) = get_asset_id(chain, token_id) {
                            return Some(PriceAsset {
                                asset_id,
                                price_id: coin.id.clone(),
                            });
                        }
                }
                None
            })
            .collect::<Vec<_>>()
    }
}

fn get_asset_id(chain: Chain, token_id: String) -> Option<String> {
    if token_id.is_empty() {
        return Some(chain.as_ref().to_string());
    }
    let token_id = format_token_id(chain, token_id)?;
    format!("{}_{}", chain.as_ref(), token_id).into()
}
