use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainAddressStatus, ChainPerpetual};
use gem_client::Client;
use primitives::{
    perpetual::{PerpetualData, PerpetualPositionsSummary},
    ChartCandleStick, ChartPeriod,
};

use crate::{
    provider::perpetual_mapper::{map_candlesticks, map_perpetuals_data, map_positions},
    rpc::client::HyperCoreClient,
};

#[async_trait]
impl<C: Client> ChainPerpetual for HyperCoreClient<C> {
    async fn get_positions(&self, address: String) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        let positions = self.get_clearinghouse_state(&address).await?;
        Ok(map_positions(positions, address))
    }

    async fn get_perpetuals_data(&self) -> Result<Vec<PerpetualData>, Box<dyn Error + Sync + Send>> {
        let metadata = self.get_metadata().await?;
        Ok(map_perpetuals_data(metadata))
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
            ChartPeriod::Hour => end_time - 60 * 60 * 1000,
            ChartPeriod::Day => end_time - 24 * 60 * 60 * 1000,
            ChartPeriod::Week => end_time - 7 * 24 * 60 * 60 * 1000,
            ChartPeriod::Month => end_time - 30 * 24 * 60 * 60 * 1000,
            ChartPeriod::Year => end_time - 365 * 24 * 60 * 60 * 1000,
            ChartPeriod::All => 0,
        };

        let candlesticks = self.get_candlesticks(&symbol, interval, start_time, end_time).await?;
        Ok(map_candlesticks(candlesticks))
    }
}

#[async_trait]
impl<C: Client> ChainAddressStatus for HyperCoreClient<C> {}
