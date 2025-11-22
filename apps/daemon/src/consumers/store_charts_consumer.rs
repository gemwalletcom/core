use async_trait::async_trait;
use pricer::PriceClient;
use std::error::Error;
use storage::models::Chart;
use streamer::{ChartsPayload, consumer::MessageConsumer};

pub struct StoreChartsConsumer {
    pub price_client: PriceClient,
}

impl StoreChartsConsumer {
    pub fn new(price_client: PriceClient) -> Self {
        Self { price_client }
    }
}

#[async_trait]
impl MessageConsumer<ChartsPayload, usize> for StoreChartsConsumer {
    async fn should_process(&self, _payload: ChartsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: ChartsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        println!("StoreChartsConsumer received {} charts", payload.charts.len());
        let charts: Vec<Chart> = payload.charts.iter().map(|c| Chart::from_chart_data(c.clone())).collect();
        self.price_client.add_charts(charts).await?;
        Ok(payload.charts.len())
    }
}
