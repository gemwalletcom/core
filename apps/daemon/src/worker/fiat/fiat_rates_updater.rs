use coingecko::CoinGeckoClient;
use pricer::PriceClient;
use std::error::Error;

pub struct FiatRatesUpdater {
    client: CoinGeckoClient,
    price_client: PriceClient,
}

impl FiatRatesUpdater {
    pub fn new(client: CoinGeckoClient, price_client: PriceClient) -> Self {
        Self { client, price_client }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let rates = self.client.get_fiat_rates().await?;
        self.price_client.set_fiat_rates(rates).await
    }
}
