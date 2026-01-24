use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainAddressStatus, ChainPerpetual};
use futures::try_join;
use gem_client::Client;
use primitives::{
    ChartPeriod,
    chart::ChartCandleStick,
    perpetual::{PerpetualData, PerpetualPositionsSummary},
    portfolio::PerpetualPortfolio,
};

use crate::{
    provider::perpetual_mapper::{map_candlesticks, map_perpetual_portfolio, map_perpetuals_data, map_positions},
    rpc::client::HyperCoreClient,
};

#[async_trait]
impl<C: Client> ChainPerpetual for HyperCoreClient<C> {
    async fn get_positions(&self, address: String) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        let (positions, orders) = try_join!(self.get_clearinghouse_state(&address), self.get_open_orders(&address))?;
        Ok(map_positions(positions, address, &orders))
    }

    async fn get_perpetuals_data(&self) -> Result<Vec<PerpetualData>, Box<dyn Error + Sync + Send>> {
        let metadata = self.get_metadata().await?;
        Ok(map_perpetuals_data(metadata))
    }

    async fn get_perpetual_candlesticks(&self, symbol: String, period: ChartPeriod) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Sync + Send>> {
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

    async fn get_perpetual_portfolio(&self, address: String) -> Result<PerpetualPortfolio, Box<dyn Error + Sync + Send>> {
        let (response, positions) = try_join!(self.get_perpetual_portfolio(&address), self.get_clearinghouse_state(&address))?;
        Ok(map_perpetual_portfolio(response, &positions))
    }
}

#[async_trait]
impl<C: Client> ChainAddressStatus for HyperCoreClient<C> {}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod integration_tests {
    use crate::provider::testkit::{TEST_ADDRESS, create_hypercore_test_client};
    use chain_traits::ChainPerpetual;
    use primitives::ChartPeriod;

    #[tokio::test]
    async fn test_hypercore_get_perpetual_portfolio() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let portfolio = ChainPerpetual::get_perpetual_portfolio(&client, TEST_ADDRESS.to_string()).await?;

        println!("Perpetual portfolio day: {:?}", portfolio.day.is_some());

        assert!(portfolio.day.is_some() || portfolio.week.is_some() || portfolio.month.is_some() || portfolio.all_time.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_hypercore_get_perpetual_candlesticks() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        let candlesticks = client.get_perpetual_candlesticks("BTC".to_string(), ChartPeriod::Day).await?;

        println!("Perpetual candlesticks count: {:?}", candlesticks.len());

        assert!(!candlesticks.is_empty());
        Ok(())
    }
}
