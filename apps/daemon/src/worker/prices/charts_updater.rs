use pricer::PriceClient;
use primitives::ChartTimeframe;
use std::error::Error;

#[derive(Clone)]
pub struct ChartsUpdater {
    prices_client: PriceClient,
}

impl ChartsUpdater {
    pub fn new(prices_client: PriceClient) -> Self {
        Self { prices_client }
    }

    pub async fn aggregate_charts(&self, timeframe: ChartTimeframe) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.prices_client.aggregate_charts(timeframe).await
    }

    pub async fn cleanup_charts(&self, timeframe: ChartTimeframe) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.prices_client.cleanup_charts(timeframe).await
    }
}
