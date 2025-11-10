use coingecko::CoinGeckoClient;
use pricer::PriceClient;
use std::error::Error;
use storage::models::Chart;

pub struct ChartsUpdater {
    coin_gecko_client: CoinGeckoClient,

    prices_client: PriceClient,
}

impl ChartsUpdater {
    pub fn new(prices_client: PriceClient, coin_gecko_client: CoinGeckoClient) -> Self {
        Self {
            coin_gecko_client,
            prices_client,
        }
    }

    #[allow(unused)]
    pub async fn update_charts_all(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let coin_list = self.coin_gecko_client.get_all_coin_markets(None, 250, 10).await?;

        for coin_id in coin_list.clone() {
            match self.coin_gecko_client.get_market_chart(coin_id.id.as_str(), "daily", "max").await {
                Ok(prices) => {
                    let charts = prices
                        .prices
                        .clone()
                        .into_iter()
                        .map(|x| Chart {
                            coin_id: coin_id.id.clone(),
                            price: x[1],
                            created_at: chrono::DateTime::from_timestamp((x[0] / 1_000_f64) as i64, 0).unwrap().naive_utc(),
                        })
                        .filter(|x| x.price > 0.0)
                        .collect::<Vec<Chart>>();

                    self.prices_client.add_charts(charts).await?;

                    println!("update charts {}", coin_id.id.clone());

                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                }
                Err(err) => {
                    println!("update charts error: {err}");
                    continue;
                }
            }
        }
        Ok(coin_list.len())
    }

    pub async fn aggregate_hourly_charts(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.prices_client.aggregate_hourly_charts().await
    }

    pub async fn aggregate_daily_charts(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.prices_client.aggregate_daily_charts().await
    }

    pub async fn cleanup_charts_data(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.prices_client.cleanup_charts_data().await
    }
}
