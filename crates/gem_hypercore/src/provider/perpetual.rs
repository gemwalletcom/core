use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainPerpetual;
use gem_client::Client;
use primitives::{
    perpetual::{PerpetualData, PerpetualPositionsSummary},
    ChartCandleStick, ChartPeriod,
};

use crate::rpc::client::HyperCoreClient;

#[async_trait]
impl<C: Client> ChainPerpetual for HyperCoreClient<C> {
    async fn get_positions(&self, address: String) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        Ok(self.get_clearinghouse_state(&address).await?.into())
    }

    async fn get_perpetuals_data(&self) -> Result<Vec<PerpetualData>, Box<dyn Error + Sync + Send>> {
        Ok(self.get_metadata().await?.into())
    }

    async fn get_candlesticks(&self, symbol: String, period: ChartPeriod) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Sync + Send>> {
        let interval = match period {
            ChartPeriod::Hour => "1m",
            ChartPeriod::Day => "30m",
            ChartPeriod::Week => "4h",
            ChartPeriod::Month => "12h",
            ChartPeriod::Year => "1w",
            ChartPeriod::All => "1M",
        };

        let end_time = chrono::Utc::now().timestamp() * 1000;
        let start_time = match period {
            ChartPeriod::Hour => end_time - 24 * 60 * 60 * 1000,           // 24 hours
            ChartPeriod::Day => end_time - 30 * 24 * 60 * 60 * 1000,       // 30 days
            ChartPeriod::Week => end_time - 7 * 24 * 60 * 60 * 1000,       // 7 days
            ChartPeriod::Month => end_time - 365 * 24 * 60 * 60 * 1000,    // 1 year
            ChartPeriod::Year => end_time - 5 * 365 * 24 * 60 * 60 * 1000, // 5 years
            ChartPeriod::All => 0,                                         // All time
        };

        let candlesticks = self.get_candlesticks(&symbol, interval, start_time, end_time).await?;
        Ok(candlesticks.into_iter().map(|c| c.into()).collect())
    }
}
