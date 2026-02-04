use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainAddressStatus, ChainPerpetual};
use futures::{future::try_join_all, try_join};
use gem_client::Client;
use primitives::{
    ChartPeriod,
    chart::ChartCandleStick,
    perpetual::{PerpetualBalance, PerpetualData, PerpetualPositionsSummary},
    portfolio::PerpetualPortfolio,
};

use crate::{
    models::position::AssetPositions,
    provider::perpetual_mapper::{map_account_summary_aggregate, map_candlesticks, map_perpetual_portfolio, map_perpetuals_data, map_positions, merge_perpetual_portfolios},
    rpc::client::HyperCoreClient,
};

impl<C: Client> HyperCoreClient<C> {
    async fn fetch_positions_for_dex(&self, address: String, dex: Option<String>) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        let (positions, orders) = try_join!(self.get_clearinghouse_state_with_dex(&address, dex.clone()), self.get_open_orders_with_dex(&address, dex))?;
        Ok(map_positions(positions, address, &orders))
    }

    async fn fetch_portfolio_for_dex(&self, address: String, dex: Option<String>) -> Result<(PerpetualPortfolio, AssetPositions), Box<dyn Error + Sync + Send>> {
        let (response, positions) = try_join!(
            self.get_perpetual_portfolio_with_dex(&address, dex.clone()),
            self.get_clearinghouse_state_with_dex(&address, dex)
        )?;
        Ok((map_perpetual_portfolio(response, &positions), positions))
    }
}

#[async_trait]
impl<C: Client> ChainPerpetual for HyperCoreClient<C> {
    async fn get_positions(&self, address: String) -> Result<PerpetualPositionsSummary, Box<dyn Error + Sync + Send>> {
        let perp_dexs = self.get_perp_dexs().await;
        if let Ok(perp_dexs) = perp_dexs {
            let mut requests = Vec::new();
            for (index, entry) in perp_dexs.iter().enumerate() {
                if index == 0 {
                    requests.push(self.fetch_positions_for_dex(address.clone(), None));
                    continue;
                }

                let dex = entry
                    .as_ref()
                    .and_then(|dex| dex.name.as_ref())
                    .map(|name| name.to_string())
                    .filter(|name| !name.is_empty());

                if let Some(dex) = dex {
                    requests.push(self.fetch_positions_for_dex(address.clone(), Some(dex)));
                }
            }

            if !requests.is_empty() {
                let summaries = try_join_all(requests).await?;
                let mut positions = Vec::new();
                let mut balance = PerpetualBalance {
                    available: 0.0,
                    reserved: 0.0,
                    withdrawable: 0.0,
                };
                for summary in summaries {
                    positions.extend(summary.positions);
                    balance.available += summary.balance.available;
                    balance.reserved += summary.balance.reserved;
                    balance.withdrawable += summary.balance.withdrawable;
                }
                return Ok(PerpetualPositionsSummary { positions, balance });
            }
        }

        let (positions, orders) = try_join!(self.get_clearinghouse_state(&address), self.get_open_orders(&address))?;
        Ok(map_positions(positions, address, &orders))
    }

    async fn get_perpetuals_data(&self) -> Result<Vec<PerpetualData>, Box<dyn Error + Sync + Send>> {
        let perp_dexs = self.get_perp_dexs().await;
        if let Ok(perp_dexs) = perp_dexs {
            let mut requests = Vec::new();
            let mut dex_indexes = Vec::new();

            for (index, entry) in perp_dexs.iter().enumerate() {
                if index == 0 {
                    requests.push(self.get_metadata_with_dex(None));
                    dex_indexes.push(index as u32);
                    continue;
                }

                let name = entry.as_ref().and_then(|dex| dex.name.as_ref());
                if let Some(name) = name
                    && !name.is_empty()
                {
                    requests.push(self.get_metadata_with_dex(Some(name.to_string())));
                    dex_indexes.push(index as u32);
                }
            }

            if !requests.is_empty() {
                let metadata = try_join_all(requests).await?;
                let mut result = Vec::new();
                for (dex_index, meta) in dex_indexes.into_iter().zip(metadata) {
                    result.extend(map_perpetuals_data(meta, dex_index));
                }
                return Ok(result);
            }
        }

        let metadata = self.get_metadata().await?;
        Ok(map_perpetuals_data(metadata, 0))
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
        let perp_dexs = self.get_perp_dexs().await;
        if let Ok(perp_dexs) = perp_dexs {
            let mut requests = Vec::new();

            for (index, entry) in perp_dexs.iter().enumerate() {
                if index == 0 {
                    requests.push(self.fetch_portfolio_for_dex(address.clone(), None));
                    continue;
                }

                let dex = entry
                    .as_ref()
                    .and_then(|dex| dex.name.as_ref())
                    .map(|name| name.to_string())
                    .filter(|name| !name.is_empty());

                if let Some(dex) = dex {
                    requests.push(self.fetch_portfolio_for_dex(address.clone(), Some(dex)));
                }
            }

            if !requests.is_empty() {
                let results = try_join_all(requests).await?;
                let (portfolios, positions): (Vec<_>, Vec<_>) = results.into_iter().unzip();

                let account_summary = Some(map_account_summary_aggregate(&positions));
                return Ok(merge_perpetual_portfolios(portfolios, account_summary));
            }
        }

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
