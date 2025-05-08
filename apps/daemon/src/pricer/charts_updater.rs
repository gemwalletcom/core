use coingecko::CoinGeckoClient;
use pricer::{ChartClient, PriceClient};
use std::error::Error;
use storage::models::CreateChart;

pub struct ChartsUpdater {
    coin_gecko_client: CoinGeckoClient,
    charts_client: ChartClient,
    prices_client: PriceClient,
}

impl ChartsUpdater {
    pub fn new(charts_client: ChartClient, prices_client: PriceClient, coin_gecko_client: CoinGeckoClient) -> Self {
        Self {
            coin_gecko_client,
            prices_client,
            charts_client,
        }
    }

    #[allow(unused)]
    pub async fn update_charts_all(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let coin_list = self.coin_gecko_client.get_all_coin_markets(250, 10).await?;

        for coin_id in coin_list.clone() {
            match self.coin_gecko_client.get_market_chart(coin_id.id.as_str()).await {
                Ok(prices) => {
                    let charts = prices
                        .prices
                        .clone()
                        .into_iter()
                        .map(|x| CreateChart {
                            coin_id: coin_id.id.clone(),
                            price: x[1] as f32,
                            ts: (x[0] / 1_000_f64) as u32,
                        })
                        .filter(|x| x.ts > 0 && x.price > 0.0)
                        .collect::<Vec<CreateChart>>();

                    match self.charts_client.set_charts(charts).await {
                        Ok(_) => {
                            println!("set charts {}", coin_id.id.clone());
                        }
                        Err(err) => {
                            println!("set charts error: {}", err);
                        }
                    };

                    println!("update charts {}", coin_id.id.clone());

                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
                Err(err) => {
                    println!("update charts error: {}", err);
                    continue;
                }
            }
        }
        Ok(coin_list.len())
    }

    pub async fn update_charts(&mut self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let prices = self.prices_client.get_prices()?;
        let charts = prices
            .clone()
            .into_iter()
            .map(|x| x.as_chart())
            .filter(|x| x.ts > 0 && x.price > 0.0)
            .collect::<Vec<CreateChart>>();

        self.charts_client.set_charts(charts).await
    }
}
